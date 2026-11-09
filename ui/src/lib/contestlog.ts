import { activeProfile, recvApplies, usesSerial } from './contest';
import type { ContestSettings } from './types';

/** Run: you call CQ and stations answer. S&P (search and pounce): you answer other stations. */
export type RunMode = 'run' | 'sp';

export interface ContestKey {
  /** F1 to F8. */
  key: string;
  /** Translation key of the label. */
  label: string;
  template: string;
  /** Sending this message also logs the QSO (TU). */
  log?: boolean;
}

/** The exchange as sent in this contest: the active sent template. */
function exchange(c: ContestSettings | undefined): string {
  const p = activeProfile(c);
  if (!p || !p.sent.length) return '{CALL} {RST} {NR}';
  return (p.sent[c!.sent_index] ?? p.sent[0]).template;
}

/**
 * The messages behind F1 to F8, like N1MM Logger+: F1 CQ (or your call in S&P), F2 exchange,
 * F3 TU, F4 your call, F5 his call, F6 QSO B4, F7 NR?, F8 AGN.
 */
export function contestKeys(run: RunMode, c: ContestSettings | undefined): ContestKey[] {
  const p = activeProfile(c);
  const cq = (p?.calls[0] ?? 'CQ TEST') + ' {MYCALL}';
  return [
    { key: 'F1', label: run === 'run' ? 'ckey.cq' : 'ckey.mycall', template: run === 'run' ? cq : '{MYCALL}' },
    { key: 'F2', label: 'ckey.exchange', template: exchange(c) },
    { key: 'F3', label: 'ckey.tu', template: run === 'run' ? 'TU {MYCALL}' : 'TU', log: true },
    { key: 'F4', label: 'ckey.mycall', template: '{MYCALL}' },
    { key: 'F5', label: 'ckey.hiscall', template: '{CALL}' },
    { key: 'F6', label: 'ckey.b4', template: '{CALL} QSO B4' },
    { key: 'F7', label: 'ckey.nr', template: 'NR?' },
    { key: 'F8', label: 'ckey.agn', template: 'AGN' },
  ];
}

export interface EntryState {
  call: string;
  /** Our exchange (F2) was sent for this call. */
  exchangeSent: boolean;
  /** Our own call was sent (S&P). */
  mycallSent: boolean;
  /** Everything the profile asks for is filled in. */
  complete: boolean;
}

export interface EnterAction {
  /** Which function key's message to send, or null. */
  key: string | null;
  /** Log the QSO after sending. */
  log: boolean;
}

/**
 * What Enter does (N1MM calls it ESM, Enter Sends Message).
 * Run: no call sends CQ; a call sends the exchange; with the other exchange complete, TU and log;
 * without it after our exchange, AGN. S&P: a call sends your call; with the exchange complete,
 * your exchange and log.
 */
export function enterAction(run: RunMode, s: EntryState): EnterAction {
  const call = s.call.trim();
  if (run === 'run') {
    if (!call) return { key: 'F1', log: false };
    if (s.exchangeSent && s.complete) return { key: 'F3', log: true };
    if (s.exchangeSent) return { key: 'F8', log: false };
    return { key: 'F2', log: false };
  }
  if (!call) return { key: null, log: false };
  if (s.complete) return { key: 'F2', log: true };
  if (!s.mycallSent) return { key: 'F1', log: false };
  return { key: null, log: false };
}

/** True when all fields the profile asks for are filled in. */
export function entryComplete(c: ContestSettings | undefined, call: string, nr: string, extra: string): boolean {
  if (!call.trim()) return false;
  const p = activeProfile(c);
  if (!p) return true;
  // A contest that counts serial numbers expects one back (WPX, WAE, UBA DX and ON).
  if (usesSerial(c) && !nr.trim()) return false;
  if (p.recv && recvApplies(p.recv, call) && !extra.trim()) return false;
  return true;
}

/** QSOs per hour, counted over the last 10 and the last 60 minutes. */
export function rates(times: number[], nowSec: number): { rate10: number; rate60: number; last60: number } {
  const n10 = times.filter((t) => nowSec - t <= 600 && nowSec - t >= 0).length;
  const n60 = times.filter((t) => nowSec - t <= 3600 && nowSec - t >= 0).length;
  return { rate10: n10 * 6, rate60: n60, last60: n60 };
}

/** The next serial number after a logged QSO. */
export function nextSerial(current: number): number {
  return Math.min(9999, current + 1);
}

/** Looks like a signal report (599, 5NN, 579, 59) or a serial number: what the decoder may suggest. */
export interface Suggestion {
  rst?: string;
  nr?: string;
  extra?: string;
}

/**
 * Picks a report, a serial number and the extra field out of the other station's reply.
 * Only suggestions: the operator confirms them. Nothing is guessed when a part is missing.
 */
export function suggestFrom(text: string, c: ContestSettings | undefined, call: string): Suggestion {
  const words = text.toUpperCase().split(/\s+/).map((w) => w.replace(/[^A-Z0-9]/g, '')).filter(Boolean);
  const out: Suggestion = {};
  const p = activeProfile(c);
  const rstIdx = words.findIndex((w) => /^(5[1-9][1-9N]|[1-5][1-9][1-9])$/.test(w) || w === '5NN');
  if (rstIdx >= 0) out.rst = words[rstIdx];
  const rest = rstIdx >= 0 ? words.slice(rstIdx + 1) : words;
  const nrWord = usesSerial(c) ? rest.find((w) => /^[0-9TN]{1,4}$/.test(w) && /[0-9]/.test(w)) : undefined;
  if (nrWord) out.nr = nrWord.replace(/T/g, '0').replace(/N/g, '9');
  if (p?.recv && recvApplies(p.recv, call)) {
    const pick = rest.filter((w) => w !== nrWord && !/^(TU|DE|CQ|K|KN|AR|73)$/.test(w));
    const word = p.recv.kind === 'number' ? pick.find((w) => /^\d{1,3}$/.test(w)) : pick.find((w) => /^[A-Z0-9]{2,4}$/.test(w) && w !== call.toUpperCase());
    if (word) out.extra = word;
  }
  return out;
}
