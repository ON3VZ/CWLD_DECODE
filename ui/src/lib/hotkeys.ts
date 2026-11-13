/**
 * Keys after N1MM Logger+ (0.20.4). This file only decides which key means what; the actions live in
 * `app.svelte.ts`. Keeping the decision here makes it testable, and the list of keys that existed
 * before is kept next to the new ones so a clash shows up in the tests.
 *
 *   Insert        send his callsign and your exchange (contest mode)
 *   .  (period)   send TU and log the QSO (contest mode), or just log (outside contest mode)
 *   PgUp / PgDn   speed 2 WPM faster or slower
 *   Ctrl+Up/Down  jump to the next spot above or below, on the current band
 *   Alt+Q         back to the frequency where you last called CQ
 *   Alt+F7        split on or off
 */
import type { ClusterSpot } from './types';
import { bandOf } from './adif';

export type HotkeyAction = 'send_call_exchange' | 'log_confirm' | 'speed_up' | 'speed_down' | 'spot_up' | 'spot_down' | 'cq_freq' | 'split_toggle';

/** N1MM changes the speed in steps of 2 WPM. */
export const SPEED_STEP = 2;
export const WPM_MIN = 5;
export const WPM_MAX = 60;

export interface KeyLike {
  key: string;
  code?: string;
  ctrlKey: boolean;
  altKey: boolean;
  shiftKey: boolean;
  metaKey: boolean;
  repeat?: boolean;
}

export interface KeyContext {
  /** Tag name of the element that has the focus (`INPUT`, `BUTTON`, ...), empty for none. */
  tag: string;
  /** Id of that element. The entry line of the contest mode has ids that start with `ce-`. */
  id: string;
}

/** The new keys, as shown to the operator. */
export const NEW_KEYS = ['Insert', '.', 'PageUp', 'PageDown', 'Ctrl+ArrowUp', 'Ctrl+ArrowDown', 'Alt+Q', 'Alt+F7'] as const;

/**
 * Keys that CWLD already used before 0.20.4 (see `onKey` in App.svelte), for the clash test.
 * Shift+click on the band scope (transmit frequency of a split) is a mouse action and does not clash.
 */
export const EXISTING_KEYS = [
  'Escape', 'Ctrl+Q', 'Alt+R', 'F1', 'F2', 'F3', 'F4', 'F5', 'F6', 'F7', 'F8', 'Ctrl+F6', 'Ctrl+F7', 'Ctrl+R', 'Ctrl+W', 'S', 'Enter', 'Tab', ' ',
] as const;

const TEXT_TAGS = new Set(['INPUT', 'TEXTAREA', 'SELECT']);

/** Which action a key press means, or null when it is not one of the new keys (or must be left alone). */
export function hotkeyAction(e: KeyLike, ctx: KeyContext): HotkeyAction | null {
  const mod = e.ctrlKey || e.altKey || e.metaKey;
  // Insert, the period and Alt+Q send or log something once: a held key must not repeat it.
  if (!mod && e.key === 'Insert') return e.repeat ? null : 'send_call_exchange';
  if (!mod && (e.key === '.' || e.code === 'NumpadDecimal')) {
    // In a text field the period is a character. Only the entry line of the contest mode may use it.
    const typing = TEXT_TAGS.has(ctx.tag) && !ctx.id.startsWith('ce-');
    return typing || e.repeat ? null : 'log_confirm';
  }
  if (!mod && !e.shiftKey && e.key === 'PageUp') return 'speed_up';
  if (!mod && !e.shiftKey && e.key === 'PageDown') return 'speed_down';
  if (e.ctrlKey && !e.altKey && !e.metaKey && !e.shiftKey && (e.key === 'ArrowUp' || e.key === 'ArrowDown')) {
    return e.repeat ? null : e.key === 'ArrowUp' ? 'spot_up' : 'spot_down';
  }
  if (e.altKey && !e.ctrlKey && !e.metaKey && !e.shiftKey) {
    // Key, not code: on an AZERTY keyboard the code of the Q key is another letter.
    if (e.key === 'q' || e.key === 'Q') return e.repeat ? null : 'cq_freq';
    if (e.key === 'F7') return e.repeat ? null : 'split_toggle';
  }
  return null;
}

export interface SpotJumpOptions {
  nowSec: number;
  /** Spots older than this are not offered (the same limit as the labels on the band scope). */
  maxAgeSec: number;
  /** Skip stations that are already worked on this band. */
  skipWorked?: boolean;
}

/** Spots within this distance of the current frequency count as "here", so the key moves on. */
export const SPOT_HERE_HZ = 100;

/**
 * The next spot above (`dir` 1) or below (`dir` -1) the current frequency, on the band of that
 * frequency. Worked stations are skipped by default (the colors of the scope: worked is grey).
 */
export function nextSpot(spots: ClusterSpot[], hz: number, dir: 1 | -1, o: SpotJumpOptions): ClusterSpot | null {
  const band = bandOf(hz);
  if (!band) return null;
  const list = spots
    .filter((s) => bandOf(s.freq_hz) === band && o.nowSec - s.ts <= o.maxAgeSec && !(o.skipWorked !== false && s.status === 'worked'))
    .sort((a, b) => a.freq_hz - b.freq_hz);
  if (dir === 1) return list.find((s) => s.freq_hz > hz + SPOT_HERE_HZ) ?? null;
  for (let i = list.length - 1; i >= 0; i--) if (list[i].freq_hz < hz - SPOT_HERE_HZ) return list[i];
  return null;
}

/** The speed after a step, kept inside the range the radio takes. */
export function stepSpeed(wpm: number, delta: number): number {
  return Math.min(WPM_MAX, Math.max(WPM_MIN, Math.round(wpm + delta)));
}
