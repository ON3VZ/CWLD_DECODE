<script lang="ts">
  import { onMount } from 'svelte';
  import { app } from '../lib/app.svelte';
  import { t } from '../lib/i18n.svelte';
  import type { ClusterSpot } from '../lib/types';

  let spectrum: HTMLCanvasElement;
  let fall: HTMLCanvasElement;
  let hoverHz = $state<number | null>(null);
  let hoverX = $state(0);
  let areaW = $state(600);
  /** Seconds since 1970, moved on every 15 s so old spots fade out without new data. */
  let nowSec = $state(Math.floor(Date.now() / 1000));

  const lo = $derived(app.scopeLo);
  const hi = $derived(app.scopeHi);
  const span = $derived(Math.max(1, hi - lo));
  const vfo = $derived(app.status.frequency_hz);
  const vfoX = $derived((vfo - lo) / span);
  /** A clear step for the frequency scale: about 6 to 10 labels over the width. */
  const ticks = $derived.by(() => {
    if (!app.scopeLive) return [] as { x: number; khz: string }[];
    const steps = [1000, 2000, 5000, 10_000, 20_000, 50_000, 100_000];
    const step = steps.find((s) => span / s <= 10) ?? 100_000;
    const out: { x: number; khz: string }[] = [];
    for (let f = Math.ceil(lo / step) * step; f <= hi; f += step) {
      out.push({ x: (f - lo) / span, khz: (f / 1000).toFixed(step < 10_000 ? 1 : 0) });
    }
    return out;
  });

  /** Waterfall colors: night blue for noise, through blue and cyan to amber and warm white for strong signals. */
  const STOPS: [number, number, number, number][] = [
    [0.0, 4, 7, 12],
    [0.12, 9, 22, 44],
    [0.35, 18, 82, 150],
    [0.58, 34, 170, 196],
    [0.8, 240, 196, 80],
    [1.0, 255, 246, 220],
  ];
  const LUT = new Uint8ClampedArray(256 * 3);
  for (let i = 0; i < 256; i++) {
    const x = i / 255;
    let k = 1;
    while (k < STOPS.length - 1 && x > STOPS[k][0]) k++;
    const [x0, r0, g0, b0] = STOPS[k - 1];
    const [x1, r1, g1, b1] = STOPS[k];
    const f = Math.max(0, Math.min(1, (x - x0) / (x1 - x0)));
    LUT[i * 3] = r0 + (r1 - r0) * f;
    LUT[i * 3 + 1] = g0 + (g1 - g0) * f;
    LUT[i * 3 + 2] = b0 + (b1 - b0) * f;
  }

  function freqAt(e: MouseEvent): number {
    const r = (e.currentTarget as HTMLElement).getBoundingClientRect();
    return lo + ((e.clientX - r.left) / r.width) * span;
  }

  function move(e: MouseEvent) {
    if (!app.scopeLive) return;
    hoverHz = freqAt(e);
    const r = (e.currentTarget as HTMLElement).getBoundingClientRect();
    hoverX = (e.clientX - r.left) / r.width;
  }

  function click(e: MouseEvent) {
    if (!app.scopeLive) return;
    app.tuneTo(freqAt(e));
  }

  /** Rows of history kept for the waterfall. */
  const HIST = 240;

  onMount(() => {
    const sctx = spectrum.getContext('2d');
    const fctx = fall.getContext('2d');
    if (!sctx || !fctx) return;
    // The history lives in its own small canvas (one pixel per scope point and per sweep) and is
    // stretched smoothly onto the screen, so the waterfall never looks blocky.
    const hist = document.createElement('canvas');
    hist.height = HIST;
    const hctx = hist.getContext('2d')!;
    let n = 0;
    let rows = 0;
    let disp = new Float32Array(0);
    let peak = new Float32Array(0);
    // The level range follows the band: the noise floor sits near the bottom, the strongest signal near the top.
    let floorS = 0;
    let topS = 0;
    let lastLo = 0;
    let lastHi = 0;
    let dirty = true;

    const fit = (c: HTMLCanvasElement) => {
      const dpr = Math.min(2, window.devicePixelRatio || 1);
      const w = Math.max(1, Math.round(c.clientWidth * dpr));
      const h = Math.max(1, Math.round(c.clientHeight * dpr));
      if (c.width !== w || c.height !== h) {
        c.width = w;
        c.height = h;
        dirty = true;
      }
      return dpr;
    };

    const norm = (v: number) => Math.max(0, Math.min(1, (v - (floorS - 3)) / Math.max(24, topS + 5 - (floorS - 3))));

    const take = (row: { data: number[]; lo_hz: number; hi_hz: number }) => {
      const d = row.data;
      if (d.length !== n) {
        n = d.length;
        hist.width = n;
        hctx.fillStyle = '#04070c';
        hctx.fillRect(0, 0, n, HIST);
        rows = 0;
        disp = new Float32Array(n);
        peak = new Float32Array(n);
        floorS = 0;
      }
      // Follow the radio when the scope window moves: shift the history sideways, or start again after a new span.
      const sp = row.hi_hz - row.lo_hz;
      if (lastHi > lastLo && (row.lo_hz !== lastLo || row.hi_hz !== lastHi)) {
        if (Math.abs(sp - (lastHi - lastLo)) < 1 && sp > 0) {
          const dx = Math.round(((lastLo - row.lo_hz) / sp) * n);
          if (dx !== 0 && Math.abs(dx) < n) {
            hctx.drawImage(hist, dx, 0);
            hctx.fillStyle = '#04070c';
            if (dx > 0) hctx.fillRect(0, 0, dx, HIST);
            else hctx.fillRect(n + dx, 0, -dx, HIST);
            const moved = new Float32Array(n);
            for (let i = 0; i < n; i++) moved[i] = i - dx >= 0 && i - dx < n ? peak[i - dx] : 0;
            peak = moved;
          } else if (Math.abs(dx) >= n) {
            hctx.fillRect(0, 0, n, HIST);
            peak.fill(0);
          }
        } else {
          hctx.fillStyle = '#04070c';
          hctx.fillRect(0, 0, n, HIST);
          rows = 0;
          peak.fill(0);
          floorS = 0;
        }
      }
      lastLo = row.lo_hz;
      lastHi = row.hi_hz;
      const sorted = Float32Array.from(d).sort();
      const fl = sorted[Math.floor(n * 0.3)];
      const tp = sorted[Math.min(n - 1, Math.floor(n * 0.998))];
      if (floorS === 0) {
        floorS = fl;
        topS = tp;
      } else {
        floorS += 0.08 * (fl - floorS);
        topS = Math.max(tp, topS - 0.25);
      }
      hctx.drawImage(hist, 0, 0, n, HIST - 1, 0, 1, n, HIST - 1);
      const img = hctx.createImageData(n, 1);
      for (let i = 0; i < n; i++) {
        const v = d[i];
        disp[i] += (v > disp[i] ? 0.65 : 0.22) * (v - disp[i]);
        peak[i] = Math.max(v, peak[i] - 0.6);
        const c = Math.round(norm(v) * 255) * 3;
        img.data[i * 4] = LUT[c];
        img.data[i * 4 + 1] = LUT[c + 1];
        img.data[i * 4 + 2] = LUT[c + 2];
        img.data[i * 4 + 3] = 255;
      }
      hctx.putImageData(img, 0, 0);
      rows = Math.min(HIST, rows + 1);
      dirty = true;
    };

    const curve = (ctx: CanvasRenderingContext2D, values: Float32Array, W: number, H: number, pad: number) => {
      const y = (i: number) => H - pad - norm(values[i]) * (H - 2 * pad);
      ctx.moveTo(0, y(0));
      for (let i = 1; i < n; i++) {
        const x0 = ((i - 1) / (n - 1)) * W;
        const x1 = (i / (n - 1)) * W;
        const xm = (x0 + x1) / 2;
        ctx.quadraticCurveTo(x0, y(i - 1), xm, (y(i - 1) + y(i)) / 2);
        if (i === n - 1) ctx.lineTo(x1, y(i));
      }
    };

    const paint = () => {
      const dpr = fit(spectrum);
      fit(fall);
      if (!dirty) return;
      dirty = false;
      const W = spectrum.width;
      const H = spectrum.height;
      sctx.clearRect(0, 0, W, H);
      // Quiet grid: the frequency ticks and three level lines.
      sctx.strokeStyle = 'rgba(140, 170, 200, 0.07)';
      sctx.lineWidth = 1;
      sctx.beginPath();
      for (const k of ticks) {
        const x = Math.round(k.x * W) + 0.5;
        sctx.moveTo(x, 0);
        sctx.lineTo(x, H);
      }
      for (let g = 1; g <= 3; g++) {
        const y = Math.round((g / 4) * H) + 0.5;
        sctx.moveTo(0, y);
        sctx.lineTo(W, y);
      }
      sctx.stroke();
      if (n > 1) {
        const pad = 3 * dpr;
        // Peak hold: where signals were a moment ago.
        sctx.beginPath();
        curve(sctx, peak, W, H, pad);
        sctx.strokeStyle = 'rgba(125, 211, 252, 0.22)';
        sctx.lineWidth = dpr;
        sctx.stroke();
        // The spectrum itself: a soft filled curve with a bright edge.
        const grad = sctx.createLinearGradient(0, 0, 0, H);
        grad.addColorStop(0, 'rgba(56, 189, 248, 0.55)');
        grad.addColorStop(0.6, 'rgba(56, 189, 248, 0.16)');
        grad.addColorStop(1, 'rgba(56, 189, 248, 0.02)');
        sctx.beginPath();
        curve(sctx, disp, W, H, pad);
        sctx.lineTo(W, H);
        sctx.lineTo(0, H);
        sctx.closePath();
        sctx.fillStyle = grad;
        sctx.fill();
        sctx.beginPath();
        curve(sctx, disp, W, H, pad);
        sctx.strokeStyle = '#8fdcff';
        sctx.lineWidth = 1.4 * dpr;
        sctx.lineJoin = 'round';
        sctx.shadowColor = 'rgba(56, 189, 248, 0.7)';
        sctx.shadowBlur = 6 * dpr;
        sctx.stroke();
        sctx.shadowBlur = 0;
      }
      // Waterfall: the newest sweeps on top, about two screen pixels per sweep, stretched smoothly.
      const FW = fall.width;
      const FH = fall.height;
      fctx.fillStyle = '#04070c';
      fctx.fillRect(0, 0, FW, FH);
      if (n > 1 && rows > 0) {
        const want = Math.max(8, Math.min(HIST, Math.round(fall.clientHeight / 2)));
        const use = Math.min(rows, want);
        fctx.imageSmoothingEnabled = true;
        fctx.imageSmoothingQuality = 'high';
        fctx.drawImage(hist, 0, 0, n, use, 0, 0, FW, (use / want) * FH);
      }
    };

    let raf = 0;
    const loop = () => {
      raf = requestAnimationFrame(loop);
      const fresh = app.scopeRows.splice(0);
      for (const row of fresh) take(row);
      paint();
    };
    raf = requestAnimationFrame(loop);
    const clock = setInterval(() => (nowSec = Math.floor(Date.now() / 1000)), 15_000);
    return () => {
      cancelAnimationFrame(raf);
      clearInterval(clock);
    };
  });
  /** The chosen size: one setting outside contest mode and one in contest mode. */
  const size = $derived((app.settings?.contest_mode ? app.settings?.scope_size_contest : app.settings?.scope_size) ?? 'medium');

  // ---- DX cluster labels ----
  const ROW_PX = 18;
  const PRIORITY = { new: 0, other_band: 1, worked: 2 } as const;
  const rowsMax = $derived(size === 'small' ? 1 : size === 'large' ? 4 : 3);
  const cluster = $derived(app.settings?.cluster);
  const showLabels = $derived(!!cluster?.show_labels && app.clusterStatus.state !== 'off');
  /** Spots in view with a row each. A crowded stretch shows the stations that are not in the log first. */
  const placed = $derived.by(() => {
    if (!app.scopeLive || !showLabels || !cluster) return { items: [] as Array<ClusterSpot & { x: number; row: number }>, hidden: 0 };
    const maxAge = Math.max(1, cluster.max_age_minutes) * 60;
    const inView = app.clusterSpots
      .filter((s) => s.freq_hz >= lo && s.freq_hz <= hi && nowSec - s.ts <= maxAge)
      .sort((a, b) => PRIORITY[a.status] - PRIORITY[b.status] || b.ts - a.ts);
    const rows: Array<Array<[number, number]>> = Array.from({ length: rowsMax }, () => []);
    const items: Array<ClusterSpot & { x: number; row: number }> = [];
    for (const s of inView) {
      const x = ((s.freq_hz - lo) / span) * areaW - 8;
      const end = x + s.call.length * 6.4 + 24;
      const row = rows.findIndex((r) => r.every(([a, b]) => end + 3 <= a || x >= b + 3));
      if (row < 0) continue;
      rows[row].push([x, end]);
      items.push({ ...s, x: (s.freq_hz - lo) / span, row });
    }
    return { items, hidden: inView.length - items.length };
  });

  /** The labels get their own lane above the spectrum, only as high as the rows in use. */
  const lanePx = $derived(showLabels && placed.items.length ? (Math.max(...placed.items.map((p) => p.row)) + 1) * ROW_PX + 6 : 0);

  function spotTitle(s: ClusterSpot): string {
    const mins = Math.max(0, Math.round((nowSec - s.ts) / 60));
    const extra = [s.snr !== null ? `${s.snr} dB` : '', s.wpm !== null ? `${s.wpm} WPM` : ''].filter(Boolean).join(', ');
    return t('spot.title', { call: s.call, khz: (s.freq_hz / 1000).toFixed(1), spotter: s.spotter, extra: extra ? ` (${extra})` : '', mins, status: t(`spot.status.${s.status}`) });
  }

  function spotClick(e: MouseEvent, hz: number) {
    e.stopPropagation();
    app.tuneTo(hz);
  }

  const cs = $derived(app.clusterStatus);
  const clusterText = $derived(
    cs.state === 'online'
      ? t('cluster.bar.online', { node: cs.node, n: placed.items.length + placed.hidden })
      : cs.state === 'connecting'
        ? t('cluster.bar.connecting', { node: cs.node })
        : cs.state === 'offline'
          ? t('cluster.bar.offline', { node: cs.node })
          : '',
  );
</script>

<section class="scope" aria-label={t('scope.title')}>
  <div
    class="area"
    class:small={size === 'small'}
    class:large={size === 'large'}
    class:live={app.scopeLive}
    style:--lane={`${lanePx}px`}
    role="presentation"
    onmousemove={move}
    onmouseleave={() => (hoverHz = null)}
    onclick={click}
    bind:clientWidth={areaW}
    title={app.scopeLive ? t('scope.click') : ''}>
    <div class="upper">
      <canvas class="spectrum" bind:this={spectrum}></canvas>
      {#if app.scopeLive}
        {#each placed.items as sp (`${sp.call}|${sp.band}|${sp.freq_hz}`)}
          <div class="pin {sp.status}" style:left={`${sp.x * 100}%`} style:top={`${3 + sp.row * ROW_PX + 15}px`}></div>
          <button class="spot {sp.status}" style:left={`${sp.x * 100}%`} style:top={`${3 + sp.row * ROW_PX}px`} title={spotTitle(sp)} onclick={(e) => spotClick(e, sp.freq_hz)} onmousemove={(e) => e.stopPropagation()}><i></i>{sp.call}</button>
        {/each}
      {/if}
    </div>
    <div class="scale">
      {#each ticks as k (k.khz)}
        <span class="tick mono" style:left={`${k.x * 100}%`}>{k.khz}</span>
      {/each}
    </div>
    <canvas class="fall" bind:this={fall}></canvas>
    {#if app.scopeLive}
      {#if vfoX >= 0 && vfoX <= 1}
        <div class="vfo" style:left={`${vfoX * 100}%`}></div>
      {/if}
      {#if hoverHz !== null}
        <div class="hover" style:left={`${hoverX * 100}%`}></div>
        <span class="hoverlabel mono" style:left={`${Math.min(88, Math.max(0, hoverX * 100 + 0.5))}%`}>{(hoverHz / 1000).toFixed(2)} kHz</span>
      {/if}
    {:else}
      <span class="off muted">{app.scopeOutOfRange ? t('scope.out_of_range') : t('scope.waiting')}</span>
    {/if}
  </div>
  <div class="bar">
    <span class="left muted">{t('scope.title')}{#if app.scopeLive}: {t('scope.range', { lo: (lo / 1000).toFixed(1), hi: (hi / 1000).toFixed(1) })}{/if}</span>
    {#if cs.state === 'off'}
      <button class="link" title={t('cluster.scope.connect.help')} onclick={() => app.clusterConnect()}>{t('cluster.scope.connect')}</button>
    {:else}
      <span class="cl muted" title={cs.state === 'offline' && cs.reason ? cs.reason : cs.software}><i class="dot {cs.state}"></i>{clusterText}{#if placed.hidden > 0} · {t('cluster.hidden', { n: placed.hidden })}{/if}</span>
      <button class="link" onclick={() => app.setClusterLabels(!cluster?.show_labels)}>{cluster?.show_labels ? t('cluster.labels.hide') : t('cluster.labels.show')}</button>
    {/if}
    <button class="link" onclick={() => app.setView('band_scope', false)}>{t('scope.hide')}</button>
  </div>
</section>

<style>
  .scope { background: #04070c; border-bottom: 1px solid var(--line); }
  /* From top to bottom: the label lane and the spectrum, the frequency scale, the waterfall. */
  .area { position: relative; height: 270px; overflow: hidden; display: grid; grid-template-rows: minmax(0, 6fr) 17px minmax(0, 5fr); }
  .area.small { height: 140px; grid-template-rows: minmax(0, 6fr) 15px minmax(0, 4fr); }
  .area.large { height: 380px; }
  .area.live { cursor: crosshair; }
  .upper { position: relative; min-height: 0; padding-top: var(--lane, 0px); background: linear-gradient(180deg, #070c14 0%, #050910 100%); }
  canvas { display: block; width: 100%; height: 100%; }
  .spectrum { min-height: 0; }
  .scale { position: relative; background: #0a1019; border-top: 1px solid rgba(140, 170, 200, 0.16); border-bottom: 1px solid rgba(140, 170, 200, 0.1); }
  .tick { position: absolute; top: 50%; transform: translate(-50%, -50%); font-size: 10px; color: rgba(190, 210, 230, 0.72); pointer-events: none; }
  .fall { min-height: 0; }
  .vfo { position: absolute; top: var(--lane, 0px); bottom: 0; width: 0; border-left: 1px solid rgba(255, 99, 99, 0.95); box-shadow: 0 0 8px 1px rgba(255, 99, 99, 0.45); pointer-events: none; z-index: 2; }
  .hover { position: absolute; top: var(--lane, 0px); bottom: 0; border-left: 1px dashed rgba(255, 255, 255, 0.5); pointer-events: none; z-index: 2; }
  .hoverlabel { position: absolute; top: calc(var(--lane, 0px) + 4px); padding: 1px 6px; border-radius: 6px; background: rgba(10, 16, 25, 0.9); border: 1px solid rgba(160, 185, 210, 0.3); font-size: 11px; color: #fff; pointer-events: none; z-index: 4; }
  .off { position: absolute; inset: 0; display: grid; place-items: center; font-size: 13px; padding: 0 16px; text-align: center; }
  .bar { display: flex; gap: 14px; align-items: center; padding: 3px 12px; font-size: 12px; border-top: 1px solid var(--line); }
  .bar .left { flex: 1; }
  .cl { display: inline-flex; align-items: center; gap: 5px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 55%; }
  .dot { width: 7px; height: 7px; border-radius: 50%; background: #888; display: inline-block; flex: none; }
  .dot.online { background: #4ade80; }
  .dot.connecting { background: #fbbf24; }
  .dot.offline { background: #f87171; }
  /* Spot labels: quiet tags. The color lives in the small dot only; worked stations fade. */
  .spot { position: absolute; z-index: 3; display: inline-flex; align-items: center; gap: 5px; height: 15px; padding: 0 7px 0 5px; border: 1px solid rgba(160, 185, 210, 0.22); border-radius: 8px; background: rgba(22, 30, 40, 0.92); color: #d5dee8; font: 500 10.5px/1 var(--mono, ui-monospace, monospace); letter-spacing: 0.02em; cursor: pointer; white-space: nowrap; transform: translateX(-8px); transition: background 0.12s, border-color 0.12s; }
  .spot i { width: 6px; height: 6px; border-radius: 50%; background: currentColor; flex: none; }
  .spot.new i { color: #5fc79a; }
  .spot.other_band i { color: #d9a84a; }
  .spot.worked i { color: #7d8894; }
  .spot.worked { color: #8b96a2; background: rgba(22, 30, 40, 0.65); border-color: rgba(160, 185, 210, 0.12); }
  .spot:hover { background: rgba(38, 50, 64, 0.98); border-color: rgba(160, 185, 210, 0.55); color: #fff; z-index: 5; }
  .pin { position: absolute; bottom: 0; width: 0; border-left: 1px solid rgba(160, 185, 210, 0.2); pointer-events: none; z-index: 1; }
  .pin.worked { border-left-color: rgba(160, 185, 210, 0.08); }
  .link { background: none; border: none; color: var(--accent); font-size: 12px; cursor: pointer; padding: 0; }
</style>
