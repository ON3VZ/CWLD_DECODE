/**
 * Split: reading where a station listens out of a spot comment ("UP 1", "QSX 7027.5") or out of
 * what the operator typed, and the checks before CWLD puts it on the radio.
 */
import { bandOf } from './adif';

export interface SplitHint {
  /** Where to transmit, in Hz (the low end when the station listens over a range). */
  txHz: number;
  /** The far end of the range in Hz when the comment gave one ("UP 1-2"), else `null`. */
  toHz: number | null;
  /** Short text for the label and the bar: "UP 1", "UP 1-2", "DWN 2", "QSX 7027.5". */
  label: string;
  /** True when the comment gave no number ("UP"): CWLD took `DEFAULT_OFFSET_KHZ`. */
  assumed: boolean;
}

/** "UP" without a number: one kilohertz, the usual meaning on CW. */
export const DEFAULT_OFFSET_KHZ = 1;
/** A larger offset than this is not a CW split but a typing error or something else. */
export const MAX_OFFSET_KHZ = 30;

const NUM = String.raw`\d+(?:[.,]\d+)?`;
const UP_RE = new RegExp(String.raw`(?:^|[^A-Z0-9])(UP|DWN|DN|DOWN)(?:\s*(${NUM})(?:\s*(?:-|TO)\s*(${NUM}))?)?(?![A-Z0-9])`, 'g');
const QSX_RE = new RegExp(String.raw`(?:^|[^A-Z0-9])QSX\s*:?\s*(${NUM})(?![A-Z0-9])`);

function num(s: string): number {
  return parseFloat(s.replace(',', '.'));
}

function trimKhz(khz: number): string {
  return String(Math.round(khz * 100) / 100);
}

function round10(hz: number): number {
  return Math.round(hz / 10) * 10;
}

/** An absolute frequency as people write it: 7027.5 (kHz), 7.0275 (MHz) or 27.5 (kHz within the same MHz). */
function absoluteHz(n: number, rxHz: number): number | null {
  if (!(n > 0)) return null;
  const rxMhz = Math.floor(rxHz / 1_000_000);
  const tries: number[] = [];
  if (n >= 1000) tries.push(n * 1000);
  else {
    if (Math.floor(n) === rxMhz && rxMhz > 0) tries.push(n * 1_000_000);
    tries.push(rxMhz * 1_000_000 + n * 1000);
  }
  for (const t of tries) {
    const hz = round10(t);
    if (Math.abs(hz - rxHz) <= MAX_OFFSET_KHZ * 1000 && hz !== rxHz) return hz;
  }
  return null;
}

function offsetHint(dir: 'UP' | 'DWN', a: number | null, b: number | null, rxHz: number): SplitHint | null {
  const sign = dir === 'UP' ? 1 : -1;
  const first = a ?? DEFAULT_OFFSET_KHZ;
  if (!(first > 0) || first > MAX_OFFSET_KHZ) return null;
  const far = b !== null && b > first && b <= MAX_OFFSET_KHZ ? b : null;
  return {
    txHz: round10(rxHz + sign * first * 1000),
    toHz: far !== null ? round10(rxHz + sign * far * 1000) : null,
    label: `${dir}${a !== null ? ` ${trimKhz(first)}` : ''}${far !== null ? `-${trimKhz(far)}` : ''}`,
    assumed: a === null,
  };
}

/**
 * Reads a split out of a spot comment. Understood: `UP`, `UP 1`, `UP1`, `UP 1.5`, `UP 1-2`,
 * `UP 1 TO 2`, `DWN 2`, `DN 2`, `DOWN 2`, `QSX 7027.5`, `QSX 7.0275`, `QSX 27.5`.
 * Anything else gives `null`.
 */
export function splitFromComment(comment: string, rxHz: number): SplitHint | null {
  if (!comment || !(rxHz > 0)) return null;
  const text = comment.toUpperCase();
  const q = QSX_RE.exec(text);
  if (q) {
    const hz = absoluteHz(num(q[1]), rxHz);
    if (hz !== null) return { txHz: hz, toHz: null, label: `QSX ${(hz / 1000).toFixed(1)}`, assumed: false };
  }
  UP_RE.lastIndex = 0;
  for (let m = UP_RE.exec(text); m; m = UP_RE.exec(text)) {
    const word = m[1];
    // "UP TO 5W", "UP 5W", "UP 20 WPM": not a split.
    const rest = text.slice(m.index + m[0].length);
    if (!m[2] && /^\s*(TO\b|\d)/.test(rest)) continue;
    if (m[2] && /^\s*(W|WATT|WATTS|WPM|DB|M|MTR|MIN|MINS|HRS|H|Z|UTC|EL|ELE)\b/.test(rest)) continue;
    const dir = word.startsWith('U') ? 'UP' : 'DWN';
    const h = offsetHint(dir, m[2] ? num(m[2]) : null, m[3] ? num(m[3]) : null, rxHz);
    if (h) return h;
  }
  return null;
}

/**
 * Reads what the operator typed in the split box. Everything a spot comment may hold, and
 * also a bare offset (`1`, `+1.5`, `-2`, in kHz) or a bare frequency (`7027.5`, `7.0275`).
 */
export function splitFromInput(input: string, rxHz: number): SplitHint | null {
  const text = input.trim().toUpperCase();
  if (!text || !(rxHz > 0)) return null;
  const m = new RegExp(String.raw`^([+-])?\s*(${NUM})(?:\s*KHZ)?$`).exec(text);
  if (m) {
    const n = num(m[2]);
    if (m[1] || n <= MAX_OFFSET_KHZ) return n > 0 ? offsetHint(m[1] === '-' ? 'DWN' : 'UP', n, null, rxHz) : null;
    const hz = absoluteHz(n, rxHz);
    return hz !== null ? { txHz: hz, toHz: null, label: offsetLabel(rxHz, hz), assumed: false } : null;
  }
  return splitFromComment(text, rxHz);
}

/** "UP 1" or "DWN 2.5" for a transmit frequency next to a receive frequency. */
export function offsetLabel(rxHz: number, txHz: number): string {
  const d = (txHz - rxHz) / 1000;
  return `${d >= 0 ? 'UP' : 'DWN'} ${trimKhz(Math.abs(d))}`;
}

/** The reason CWLD will not set this split, as a stable code, or `null` when it is fine. */
export function splitProblem(rxHz: number, txHz: number): 'split_other_band' | 'split_same_frequency' | null {
  const band = bandOf(rxHz);
  if (!band || bandOf(txHz) !== band) return 'split_other_band';
  if (Math.abs(txHz - rxHz) < 10) return 'split_same_frequency';
  return null;
}
