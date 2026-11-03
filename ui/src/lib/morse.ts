const TABLE: Record<string, string> = {
  A: '.-', B: '-...', C: '-.-.', D: '-..', E: '.', F: '..-.', G: '--.', H: '....', I: '..',
  J: '.---', K: '-.-', L: '.-..', M: '--', N: '-.', O: '---', P: '.--.', Q: '--.-', R: '.-.',
  S: '...', T: '-', U: '..-', V: '...-', W: '.--', X: '-..-', Y: '-.--', Z: '--..',
  '0': '-----', '1': '.----', '2': '..---', '3': '...--', '4': '....-', '5': '.....',
  '6': '-....', '7': '--...', '8': '---..', '9': '----.',
  '.': '.-.-.-', ',': '--..--', '?': '..--..', '/': '-..-.', '=': '-...-', '+': '.-.-.',
};

export function pattern(ch: string): string | undefined {
  return TABLE[ch.toUpperCase()];
}

/** Duration in units (PARIS timing: dot 1, dash 3, gap in char 1, between chars 3, between words 7). */
export function durationUnits(text: string): number {
  let units = 0;
  const words = text.trim().split(/\s+/).filter(Boolean);
  words.forEach((w, wi) => {
    if (wi > 0) units += 7;
    const chars = [...w].map((c) => pattern(c)).filter((p): p is string => !!p);
    chars.forEach((p, ci) => {
      if (ci > 0) units += 3;
      [...p].forEach((el, ei) => {
        if (ei > 0) units += 1;
        units += el === '.' ? 1 : 3;
      });
    });
  });
  return units;
}

export function durationSeconds(text: string, wpm: number): number {
  return (durationUnits(text) * 1.2) / Math.max(1, wpm);
}

/** Key-down/up schedule in seconds, used for the practice sidetone. */
export function schedule(text: string, wpm: number): Array<{ on: number; off: number }> {
  const unit = 1.2 / Math.max(1, wpm);
  const out: Array<{ on: number; off: number }> = [];
  let t = 0;
  const words = text.trim().split(/\s+/).filter(Boolean);
  words.forEach((w, wi) => {
    if (wi > 0) t += 7 * unit;
    const chars = [...w].map((c) => pattern(c)).filter((p): p is string => !!p);
    chars.forEach((p, ci) => {
      if (ci > 0) t += 3 * unit;
      [...p].forEach((el, ei) => {
        if (ei > 0) t += unit;
        const len = (el === '.' ? 1 : 3) * unit;
        out.push({ on: t, off: t + len });
        t += len;
      });
    });
  });
  return out;
}

let ctx: AudioContext | null = null;

/** Plays a sidetone in the browser. Silent when audio is not available. Returns a cancel function. */
export function playSidetone(text: string, wpm: number, freq = 650): () => void {
  try {
    const AC = (globalThis as any).AudioContext ?? (globalThis as any).webkitAudioContext;
    if (!AC) return () => {};
    ctx ??= new AC() as AudioContext;
    const c = ctx!;
    const gain = c.createGain();
    gain.gain.value = 0;
    gain.connect(c.destination);
    const osc = c.createOscillator();
    osc.frequency.value = freq;
    osc.connect(gain);
    const start = c.currentTime + 0.05;
    const ramp = 0.004;
    for (const { on, off } of schedule(text, wpm)) {
      gain.gain.setTargetAtTime(0.25, start + on, ramp);
      gain.gain.setTargetAtTime(0, start + off, ramp);
    }
    osc.start(start);
    const total = durationSeconds(text, wpm);
    osc.stop(start + total + 0.2);
    return () => {
      try {
        gain.gain.cancelScheduledValues(c.currentTime);
        gain.gain.value = 0;
        osc.stop();
      } catch {
        /* already stopped */
      }
    };
  } catch {
    return () => {};
  }
}
