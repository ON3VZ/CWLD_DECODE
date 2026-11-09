import type { InputDevice, PortInfo, RadioKind } from './types';
import { durationSeconds } from './morse';

/** The wizard steps in order. */
export const STEPS = ['radio', 'rigset', 'audio', 'station'] as const;
export type WizardStep = (typeof STEPS)[number];

/**
 * The port to suggest. Exactly one port that looks like the chosen radio: take it.
 * The IC-7300 MK II shows two ports. The first one named like the CI-V port wins.
 * Otherwise nothing is chosen, the operator picks.
 */
export function pickPort(ports: PortInfo[], kind: RadioKind): string {
  if (kind === 'simulator') return '';
  const like = ports.filter((p) => p.hint === kind);
  if (like.length === 1) return like[0].name;
  if (like.length > 1) {
    const civ = like.find((p) => /\(CI-V\)|serial port a/i.test(p.description));
    return civ ? civ.name : '';
  }
  return '';
}

/** The sound input to suggest: the one that looks like the radio, else keep what is chosen. */
export function pickInput(inputs: InputDevice[], current: string): string {
  if (current && inputs.some((d) => d.name === current)) return current;
  const s = inputs.find((d) => d.suggested);
  return s ? s.name : current;
}

const CALL = /^(?:[A-Z]{1,2}|[0-9][A-Z])[0-9][A-Z]{1,4}(?:\/[A-Z0-9]{1,4})?$/;
const LOC = /^[A-R]{2}[0-9]{2}(?:[A-X]{2})?$/i;

export interface StationDraft {
  calls: string;
  locator: string;
}

/** Problems of the station step as translation keys, with the call that is wrong. Empty: fine. */
export function stationProblems(d: StationDraft): Array<{ key: string; call?: string }> {
  const out: Array<{ key: string; call?: string }> = [];
  const calls = d.calls.split(',').map((c) => c.trim().toUpperCase()).filter(Boolean);
  if (calls.length === 0) out.push({ key: 'wiz.err.nocall' });
  const bad = calls.find((c) => !CALL.test(c));
  if (bad) out.push({ key: 'settings.invalid.call', call: bad });
  if (d.locator && !LOC.test(d.locator)) out.push({ key: 'settings.invalid.locator' });
  return out;
}

export function splitCalls(text: string): string[] {
  return text.split(',').map((c) => c.trim().toUpperCase()).filter(Boolean);
}

/** The silent test: what CWLD would send, and how long it takes. Nothing is transmitted. */
export function silentTest(call: string, wpm = 18): { text: string; seconds: number } {
  const text = `CQ CQ DE ${call} ${call} K`;
  return { text, seconds: Math.round(durationSeconds(text, wpm) * 10) / 10 };
}
