"""Trains the small key-down network of CWLD on synthetic CW (numpy only, CPU).

Input per 5 ms frame: the envelope around it (scaled by the recent signal level), the classic
log-likelihood ratio around it and the ratio of noise to signal level. Output: is the key down?

Data: cargo run --release -p cwld-dsp --example nndump -- train.bin 700 11
Run:  python3 tools/nn/train.py train.bin val.bin weights.json
"""
import json
import sys
import numpy as np

W = 24          # frames of context on each side (120 ms)
BACK = 400      # frames of history for the level (2 s)
STEP = 8        # the level is measured every STEP frames


def features(env, llr):
    n = len(env)
    scale = np.empty(n, np.float32)
    ratio = np.empty(n, np.float32)
    for i in range(0, n, STEP):
        a = max(0, i - BACK)
        b = min(n, i + W + 1)
        seg = env[a:b]
        hi = max(float(np.percentile(seg, 90)), 1e-6)
        lo = float(np.percentile(seg, 20))
        scale[i:i + STEP] = hi
        ratio[i:i + STEP] = lo / hi
    e = np.clip(env / scale, 0.0, 2.5)
    l = np.clip(llr / 6.0, -1.0, 1.0)
    ep = np.pad(e, W)
    lp = np.pad(l, W)
    we = np.lib.stride_tricks.sliding_window_view(ep, 2 * W + 1)
    wl = np.lib.stride_tricks.sliding_window_view(lp, 2 * W + 1)
    return np.concatenate([we, wl, ratio[:, None]], axis=1).astype(np.float32)


def align(env, key):
    """The frontend delays the signal: find the shift that lines the truth up with the envelope."""
    e = env - env.mean()
    k = key - key.mean()
    best, lag = -1e30, 0
    for d in range(0, 120):
        c = float(np.dot(e[d:], k[:len(k) - d])) if d else float(np.dot(e, k))
        if c > best:
            best, lag = c, d
    out = np.zeros_like(key)
    out[lag:] = key[:len(key) - lag] if lag else key
    return out


def load(path, every):
    raw = np.fromfile(path, np.float32).reshape(-1, 7)
    xs, ys, meta, cls = [], [], [], []
    for f in np.unique(raw[:, 0]):
        r = raw[raw[:, 0] == f]
        y = align(r[:, 1], r[:, 3])
        x = features(r[:, 1], r[:, 2])
        xs.append(x[::every])
        ys.append(y[::every])
        meta.append(r[::every, 4:7])
        cls.append(r[::every, 2])
    return np.concatenate(xs), np.concatenate(ys), np.concatenate(meta), np.concatenate(cls)


def forward(p, x):
    h1 = np.maximum(0, x @ p["w1"] + p["b1"])
    h2 = np.maximum(0, h1 @ p["w2"] + p["b2"])
    return h1, h2, (h2 @ p["w3"] + p["b3"])[:, 0]


def logloss(z, y):
    return float(np.mean(np.logaddexp(0, z) - y * z))


def report(name, z, y, meta):
    snr, qsb = meta[:, 0], meta[:, 2]
    rows = [("all", np.ones(len(y), bool)), ("snr < -3", snr < -3), ("-3..3", (snr >= -3) & (snr < 3)), ("3..10", (snr >= 3) & (snr < 10)), (">= 10", snr >= 10), ("qsb", qsb > 0.5), ("no qsb", qsb == 0)]
    out = []
    for label, m in rows:
        if m.sum() == 0:
            continue
        out.append(f"{label}: loss {logloss(z[m], y[m]):.3f} err {100 * np.mean((z[m] > 0) != (y[m] > 0.5)):.1f}%")
    print(f"{name:10s} " + " | ".join(out))


def main():
    train, val, out = sys.argv[1], sys.argv[2], sys.argv[3]
    hidden = (48, 24)
    x, y, _, _ = load(train, 2)
    xv, yv, mv, cv = load(val, 1)
    print("train", x.shape, "val", xv.shape, "key down share", round(float(y.mean()), 3))
    rng = np.random.default_rng(1)
    d = x.shape[1]
    p = {
        "w1": rng.normal(0, np.sqrt(2 / d), (d, hidden[0])).astype(np.float32), "b1": np.zeros(hidden[0], np.float32),
        "w2": rng.normal(0, np.sqrt(2 / hidden[0]), hidden).astype(np.float32), "b2": np.zeros(hidden[1], np.float32),
        "w3": rng.normal(0, np.sqrt(1 / hidden[1]), (hidden[1], 1)).astype(np.float32), "b3": np.zeros(1, np.float32),
    }
    m = {k: np.zeros_like(v) for k, v in p.items()}
    v = {k: np.zeros_like(v) for k, v in p.items()}
    step, bs = 0, 1024
    report("classic", cv, yv, mv)
    for epoch in range(8):
        lr = 2e-3 * (0.6 ** epoch)
        order = rng.permutation(len(x))
        for i in range(0, len(order) - bs + 1, bs):
            idx = order[i:i + bs]
            xb, yb = x[idx], y[idx]
            h1, h2, z = forward(p, xb)
            g = (1 / (1 + np.exp(-z)) - yb)[:, None] / bs
            grads = {"w3": h2.T @ g, "b3": g.sum(0)}
            g2 = (g @ p["w3"].T) * (h2 > 0)
            grads["w2"], grads["b2"] = h1.T @ g2, g2.sum(0)
            g1 = (g2 @ p["w2"].T) * (h1 > 0)
            grads["w1"], grads["b1"] = xb.T @ g1, g1.sum(0)
            step += 1
            for k in p:
                m[k] = 0.9 * m[k] + 0.1 * grads[k]
                v[k] = 0.999 * v[k] + 0.001 * grads[k] ** 2
                p[k] -= lr * (m[k] / (1 - 0.9 ** step)) / (np.sqrt(v[k] / (1 - 0.999 ** step)) + 1e-8)
        report(f"epoch {epoch + 1}", forward(p, xv)[2], yv, mv)
    prior = float(np.log(y.mean() / (1 - y.mean())))
    json.dump({"w": W, "back": BACK, "step": STEP, "prior_logit": prior, **{k: np.round(a, 5).tolist() for k, a in p.items()}}, open(out, "w"))
    print("saved", out, "parameters", sum(a.size for a in p.values()))


if __name__ == "__main__":
    main()
