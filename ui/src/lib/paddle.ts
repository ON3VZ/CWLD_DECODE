// Paddle trainer: what to key, how well it was keyed, and which characters need work.
//
// The timing comes from the key times of the sidetone (key down and up in milliseconds).
// With the electronic keyer of the radio, dots and dashes are made by the keyer and are always
// right; then only the spacing is yours: the gaps between characters and between words, and
// pauses inside a character (they split the character in two). With a straight key the
// element lengths count too.

import { pattern } from './morse';

/** One key run: down (true) or up, and its length in milliseconds. */
export type Run = [boolean, number];

export type PaddleExercise = 'chars' | 'weak' | 'words' | 'calls' | 'phrases';
export const PADDLE_EXERCISES: PaddleExercise[] = ['chars', 'weak', 'words', 'calls', 'phrases'];

export interface CharStat {
  ch: string;
  tries: number;
  errors: number;
}

export interface Timing {
  /** Number of elements (dots and dashes) measured. */
  elements: number;
  wpm: number | null;
  unitMs: number | null;
  /** Dash length over dot length, 3 is right. */
  dashDot: number | null;
  /** Gaps in units: inside a character (1), between characters (3), between words (7). */
  elementGap: number | null;
  charGap: number | null;
  wordGap: number | null;
  /** Spread of the dot and dash lengths (standard deviation over mean). */
  dotSpread: number | null;
  dashSpread: number | null;
  /** The elements are as even as a keyer makes them: only the spacing is the operator's. */
  keyer: boolean;
  /** 0 to 100. */
  score: number;
  /** Advice, as translation keys (`paddle.tip.*`). */
  tips: string[];
}

const median = (v: number[]) => {
  if (!v.length) return NaN;
  const s = [...v].sort((a, b) => a - b);
  const m = s.length >> 1;
  return s.length % 2 ? s[m] : (s[m - 1] + s[m]) / 2;
};
const spread = (v: number[]) => {
  if (v.length < 2) return null;
  const m = v.reduce((a, b) => a + b, 0) / v.length;
  const sd = Math.sqrt(v.reduce((a, b) => a + (b - m) ** 2, 0) / (v.length - 1));
  return m > 0 ? sd / m : null;
};

/** Splits lengths in a short and a long group (two-means on the logarithm). */
function twoGroups(v: number[]): { short: number[]; long: number[] } {
  if (v.length < 2) return { short: [...v], long: [] };
  const lg = v.map((x) => Math.log(x));
  let a = Math.min(...lg);
  let b = Math.max(...lg);
  if (b - a < Math.log(1.8)) return { short: [...v], long: [] };
  for (let i = 0; i < 20; i++) {
    const cut = (a + b) / 2;
    const s = lg.filter((x) => x <= cut);
    const l = lg.filter((x) => x > cut);
    if (!s.length || !l.length) break;
    a = s.reduce((p, q) => p + q, 0) / s.length;
    b = l.reduce((p, q) => p + q, 0) / l.length;
  }
  const cut = (a + b) / 2;
  return { short: v.filter((x) => Math.log(x) <= cut), long: v.filter((x) => Math.log(x) > cut) };
}

/** Gaps longer than this many units are a pause (thinking, the end), not spacing. */
const PAUSE_UNITS = 20;

/**
 * Measures the keying. `wordGaps`: how many word gaps the asked text has (spaces). When it is
 * known, the longest gaps are the word gaps and the others between characters, so a long pause
 * between characters is not taken for a word gap.
 */
export function analyseTiming(runs: Run[], wordGaps?: number): Timing {
  // Leading and trailing silence say nothing about the keying.
  let r = [...runs];
  while (r.length && !r[0][0]) r.shift();
  while (r.length && !r[r.length - 1][0]) r.pop();
  const downs = r.filter((x) => x[0]).map((x) => x[1]);
  const ups = r.filter((x) => !x[0]).map((x) => x[1]);
  const empty: Timing = { elements: downs.length, wpm: null, unitMs: null, dashDot: null, elementGap: null, charGap: null, wordGap: null, dotSpread: null, dashSpread: null, keyer: false, score: 0, tips: [] };
  if (downs.length < 4) return { ...empty, tips: ['paddle.tip.too_short'] };
  let { short: dots, long: dashes } = twoGroups(downs);
  // The shortest gaps are the gaps inside characters: one unit.
  const gapUnit = ups.length ? median(twoGroups(ups).short) : NaN;
  if (!dashes.length && Number.isFinite(gapUnit) && median(dots) > 2 * gapUnit) {
    // Only dashes were keyed (for example T, M, O or 0).
    dashes = dots;
    dots = [];
  }
  const unit = dots.length ? median(dots) : median(dashes) / 3;
  if (!Number.isFinite(unit) || unit <= 0) return empty;
  const inUnits = ups.map((g) => g / unit).filter((g) => g < PAUSE_UNITS);
  const el = inUnits.filter((g) => g < 2);
  let ch = inUnits.filter((g) => g >= 2 && g < 5);
  let wd = inUnits.filter((g) => g >= 5);
  if (wordGaps !== undefined) {
    const between = inUnits.filter((g) => g >= 2).sort((a, b) => b - a);
    wd = between.slice(0, wordGaps);
    ch = between.slice(wordGaps);
  }
  const t: Timing = {
    elements: downs.length,
    unitMs: unit,
    wpm: 1200 / unit,
    dashDot: dots.length && dashes.length ? median(dashes) / median(dots) : null,
    elementGap: el.length ? median(el) : null,
    charGap: ch.length ? median(ch) : null,
    wordGap: wd.length ? median(wd) : null,
    dotSpread: spread(dots),
    dashSpread: spread(dashes),
    keyer: false,
    score: 100,
    tips: [],
  };
  t.keyer = (t.dotSpread ?? 1) < 0.06 && (t.dashSpread ?? 0) < 0.06 && t.dashDot !== null && Math.abs(t.dashDot - 3) < 0.25;
  let pen = 0;
  if (t.dashDot !== null) {
    pen += Math.abs(Math.log(t.dashDot / 3)) * 60;
    if (t.dashDot < 2.6) t.tips.push('paddle.tip.dash_short');
    if (t.dashDot > 3.5) t.tips.push('paddle.tip.dash_long');
  }
  if (t.elementGap !== null) {
    pen += Math.abs(Math.log(t.elementGap)) * 30;
  }
  // A pause inside a character of about two units: the keyer then cuts the character in two.
  const hesitations = inUnits.filter((g) => g >= 1.7 && g < 2.4).length;
  if (hesitations) {
    pen += Math.min(20, hesitations * 5);
    t.tips.push('paddle.tip.hesitation');
  }
  if (t.charGap !== null) {
    if (t.charGap < 2.5) {
      pen += (2.5 - t.charGap) * 25;
      t.tips.push('paddle.tip.char_gap_short');
    }
    if (t.charGap > 4.5) {
      pen += (t.charGap - 4.5) * 8;
      t.tips.push('paddle.tip.char_gap_long');
    }
  }
  if (t.wordGap !== null) {
    if (t.wordGap < 5.5) {
      pen += (5.5 - t.wordGap) * 8;
      t.tips.push('paddle.tip.word_gap_short');
    }
    if (t.wordGap > 10) {
      pen += (t.wordGap - 10) * 3;
      t.tips.push('paddle.tip.word_gap_long');
    }
  }
  for (const s of [t.dotSpread, t.dashSpread]) if (s !== null) pen += Math.max(0, s - 0.1) * 100;
  if ((t.dotSpread ?? 0) > 0.2 || (t.dashSpread ?? 0) > 0.2) t.tips.push('paddle.tip.uneven');
  if (t.keyer) t.tips.push('paddle.tip.keyer');
  t.score = Math.round(Math.max(0, Math.min(100, 100 - pen)));
  if (!t.tips.length || (t.tips.length === 1 && t.keyer)) t.tips.push('paddle.tip.good');
  return t;
}

export interface Marked {
  ch: string;
  ok: boolean;
  /** What came out instead (empty: nothing). */
  got: string;
}

/** Compares what was asked with what the decoder heard, character by character. */
export function markTarget(target: string, got: string): Marked[] {
  const a = [...target.toUpperCase()];
  const b = [...got.toUpperCase().replace(/\s+/g, ' ').trim()];
  const d: number[][] = Array.from({ length: a.length + 1 }, (_, i) => Array.from({ length: b.length + 1 }, (_, j) => (i === 0 ? j : j === 0 ? i : 0)));
  for (let i = 1; i <= a.length; i++)
    for (let j = 1; j <= b.length; j++) d[i][j] = Math.min(d[i - 1][j - 1] + (a[i - 1] === b[j - 1] ? 0 : 1), d[i - 1][j] + 1, d[i][j - 1] + 1);
  const out: Marked[] = a.map((ch) => ({ ch, ok: false, got: '' }));
  let i = a.length;
  let j = b.length;
  const extra: string[] = [];
  while (i > 0 || j > 0) {
    if (i > 0 && j > 0 && d[i][j] === d[i - 1][j - 1] + (a[i - 1] === b[j - 1] ? 0 : 1)) {
      out[i - 1] = { ch: a[i - 1], ok: a[i - 1] === b[j - 1], got: b[j - 1] + extra.reverse().join('') };
      extra.length = 0;
      i--;
      j--;
    } else if (i > 0 && d[i][j] === d[i - 1][j] + 1) {
      out[i - 1] = { ch: a[i - 1], ok: false, got: '' };
      i--;
    } else {
      // An extra character: it belongs to the next target character to the left.
      extra.push(b[j - 1]);
      j--;
    }
  }
  // Extra characters keyed inside a character (a split) make that character wrong.
  for (const m of out) if (m.ok && m.got.length > 1) m.ok = false;
  return out;
}

/** Share of the characters (not spaces) keyed right, 0 to 1. */
export function accuracy(marks: Marked[]): number {
  const real = marks.filter((m) => m.ch !== ' ');
  return real.length ? real.filter((m) => m.ok).length / real.length : 0;
}

export function updateStats(stats: CharStat[], marks: Marked[]): CharStat[] {
  const out = stats.map((s) => ({ ...s }));
  for (const m of marks) {
    if (m.ch === ' ') continue;
    let s = out.find((x) => x.ch === m.ch);
    if (!s) {
      s = { ch: m.ch, tries: 0, errors: 0 };
      out.push(s);
    }
    s.tries++;
    if (!m.ok) s.errors++;
  }
  return out;
}

/** The characters with the most errors (at least three tries), worst first. */
export function weakest(stats: CharStat[], n = 6): CharStat[] {
  return stats
    .filter((s) => s.tries >= 3 && s.errors > 0)
    .sort((a, b) => b.errors / b.tries - a.errors / a.tries || b.tries - a.tries)
    .slice(0, n);
}

/** Order of the Koch method: also the set of characters the trainer asks. */
export const KOCH = 'KMRSUAPTLOWINJEF0YVG5/Q9ZH38B?427C1D6X';

const WORDS = ['CQ', 'DE', 'TNX', 'FER', 'CALL', 'UR', 'RST', 'NAME', 'QTH', 'RIG', 'ANT', 'WX', 'HR', 'ES', 'GL', 'GB', 'OM', 'YL', 'PSE', 'AGN', 'QRZ', 'QSL', 'QRS', 'QRM', 'QSB', 'HW', 'CPY', 'FB', 'BK', 'TU', '73', '599', '5NN', 'TEST', 'DX', 'SRI', 'WID', 'BEST', 'GOOD', 'SUN', 'RAIN', 'COLD', 'WARM', 'KEY', 'BUG', 'POWER', 'WATTS', 'DIPOLE', 'VERTICAL'];
const PHRASES = ['CQ CQ DE {MYCALL} {MYCALL} K', 'TNX FER CALL UR RST 599 599', 'NAME IS {NAME} QTH {QTH}', 'RIG IC7300 PWR 50W ANT DIPOLE', 'WX HR SUNNY TEMP 12 C', 'TNX FER QSO 73 ES GL DE {MYCALL} SK', 'QRZ DE {MYCALL} K', 'PSE QRS QRS', 'R R TU 5NN 14'];
const PREFIXES = ['ON', 'OT', 'OO', 'PA', 'PD', 'DL', 'DK', 'DJ', 'F', 'G', 'M', 'EA', 'I', 'IK', 'OK', 'SP', 'HA', 'YU', 'K', 'W', 'N', 'VE', 'JA', 'LZ', 'S5', '9A', 'OH', 'SM', 'LA', 'OZ'];

/** A small seeded random source, so tests and replays give the same targets. */
export function rng(seed: number) {
  let s = seed >>> 0 || 1;
  return () => {
    s ^= s << 13;
    s ^= s >>> 17;
    s ^= s << 5;
    return (s >>> 0) / 4294967296;
  };
}

function pickWeighted(chars: string[], weight: (c: string) => number, r: () => number): string {
  const w = chars.map(weight);
  let x = r() * w.reduce((a, b) => a + b, 0);
  for (let i = 0; i < chars.length; i++) {
    x -= w[i];
    if (x <= 0) return chars[i];
  }
  return chars[chars.length - 1];
}

export interface TargetContext {
  myCall: string;
  name: string;
  qth: string;
  stats: CharStat[];
  /** The characters in use (Koch). Empty or missing: all of them. */
  chars?: string;
}

/** The characters in use, in Koch order, only known ones. All of them when nothing valid is given. */
export function kochPool(chars?: string): string[] {
  const set = new Set([...(chars ?? '').toUpperCase()]);
  const pool = [...KOCH].filter((c) => set.has(c));
  return pool.length ? pool : [...KOCH];
}

/** The first `n` characters of the Koch order. */
export const kochFirst = (n: number): string => KOCH.slice(0, Math.max(2, Math.min(KOCH.length, Math.round(n))));

const fits = (text: string, pool: string[]) => [...text.replace(/ /g, '')].every((c) => pool.includes(c));

/** Words and abbreviations that only use the characters in use. */
export function wordPool(chars?: string): string[] {
  const pool = kochPool(chars);
  return WORDS.filter((w) => fits(w, pool));
}

/** Fewer words than this and the word exercises fall back to character groups. */
export const MIN_WORDS = 4;

/** The next text to key or copy. Characters that went wrong before and the newest one come back more often. */
export function nextTarget(ex: PaddleExercise, ctx: TargetContext, r: () => number): string {
  const all = kochPool(ctx.chars);
  const newest = all.length < KOCH.length ? all[all.length - 1] : '';
  const errRate = (c: string) => {
    const s = ctx.stats.find((x) => x.ch === c);
    return s && s.tries ? s.errors / s.tries : 0.2;
  };
  const group = (pool: string[], n: number) => Array.from({ length: n }, () => pickWeighted(pool, (c) => 1 + 4 * errRate(c) + (c === newest ? 1.5 : 0), r)).join('');
  const groups = (pool: string[]) => Array.from({ length: 4 }, () => group(pool, 5)).join(' ');
  switch (ex) {
    case 'chars':
      return groups(all);
    case 'weak': {
      const weak = weakest(ctx.stats).map((s) => s.ch).filter((c) => all.includes(c));
      return groups(weak.length >= 2 ? weak : all);
    }
    case 'words': {
      const words = wordPool(ctx.chars);
      if (words.length < MIN_WORDS) return groups(all);
      return Array.from({ length: 5 }, () => words[Math.floor(r() * words.length)]).join(' ');
    }
    case 'calls': {
      const letters = all.filter((c) => /[A-Z]/.test(c));
      const digits = all.filter((c) => /[0-9]/.test(c));
      // A callsign needs letters and a digit. Without them there is nothing real to ask.
      if (letters.length < 3 || !digits.length) return groups(all);
      const prefixes = PREFIXES.filter((p) => fits(p, all) && /[A-Z]/.test(p));
      return Array.from({ length: 3 }, () => {
        const p = prefixes.length ? prefixes[Math.floor(r() * prefixes.length)] : pickWeighted(letters, () => 1, r) + pickWeighted(letters, () => 1, r);
        const suffix = Array.from({ length: 1 + Math.floor(r() * 3) }, () => pickWeighted(letters, (c) => 1 + 4 * errRate(c) + (c === newest ? 1.5 : 0), r)).join('');
        return `${p}${pickWeighted(digits, () => 1, r)}${suffix}`;
      }).join(' ');
    }
    case 'phrases': {
      // Without own callsign or name the phrase uses neutral words: nothing is made up.
      const fill = (p: string) =>
        p
          .replaceAll('{MYCALL}', ctx.myCall || 'TEST')
          .replaceAll('{NAME}', (ctx.name || 'OP').toUpperCase())
          .replaceAll('{QTH}', (ctx.qth || 'HOME').toUpperCase())
          .replace(/[^A-Z0-9/?., ]/g, '');
      const ok = PHRASES.map(fill).filter((p) => fits(p, all));
      if (!ok.length) return groups(all);
      return ok[Math.floor(r() * ok.length)];
    }
  }
}

/** Whether an exercise can use real words, calls or phrases with these characters (else groups). */
export function exerciseFits(ex: PaddleExercise, chars?: string): boolean {
  const all = kochPool(chars);
  switch (ex) {
    case 'words':
      return wordPool(chars).length >= MIN_WORDS;
    case 'calls':
      return all.filter((c) => /[A-Z]/.test(c)).length >= 3 && all.some((c) => /[0-9]/.test(c));
    case 'phrases':
      return PHRASES.some((p) => fits(p.replace(/\{[A-Z]+\}/g, 'TEST'), all));
    default:
      return true;
  }
}

/**
 * The 90 percent rule of the Koch method: when the last `rounds` listening rounds with the
 * characters in use are all together at least `limit` right, the next character may be added.
 * Only rounds with the same number of characters count, so a new character starts fresh.
 */
export function kochReady(sessions: { accuracy: number; n: number; chars: number }[], n: number, limit = 0.9, rounds = 3): boolean {
  if (n >= KOCH.length) return false;
  const last = sessions.slice(-rounds);
  if (last.length < rounds || last.some((s) => s.n !== n)) return false;
  const total = last.reduce((a, s) => a + s.chars, 0);
  if (total < 30) return false;
  return last.reduce((a, s) => a + s.accuracy * s.chars, 0) / total >= limit;
}

/** One number for a session: mostly the right characters, partly the timing. */
export function sessionScore(acc: number, timing: Timing): number {
  return Math.round(Math.max(0, Math.min(100, acc * 70 + timing.score * 0.3)));
}

/** Character for each dot-dash pattern, built from the morse table. */
const FROM_PATTERN: Map<string, string> = (() => {
  const m = new Map<string, string>();
  for (const c of 'ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789/?=.,') {
    const p = pattern(c);
    if (p && !m.has(p)) m.set(p, c);
  }
  return m;
})();

/**
 * Reads the keyed text straight from the key times, with the operator's own speed taken from the
 * whole attempt. More reliable for scoring than the live decoder, which needs a few characters to
 * find the speed (the first letter was often wrong) and cannot look back.
 * Gaps under 2 units are inside a character, 2 to 5 between characters, longer between words.
 * A pattern that is no character gives `*`.
 */
export function runsToText(runs: Run[]): string {
  const r = [...runs];
  while (r.length && !r[0][0]) r.shift();
  while (r.length && !r[r.length - 1][0]) r.pop();
  const downs = r.filter((x) => x[0]).map((x) => x[1]);
  if (!downs.length) return '';
  const unit = analyseTiming(r).unitMs ?? (downs.length === 1 ? downs[0] : median(twoGroups(downs).short));
  if (!unit || !Number.isFinite(unit)) return '';
  let out = '';
  let cur = '';
  const close = () => {
    if (cur) out += FROM_PATTERN.get(cur) ?? '*';
    cur = '';
  };
  for (const [down, ms] of r) {
    const u = ms / unit;
    if (down) cur += u < 2 ? '.' : '-';
    else if (u >= 5) {
      close();
      out += ' ';
    } else if (u >= 2) close();
  }
  close();
  return out.replace(/ +/g, ' ').trim();
}
