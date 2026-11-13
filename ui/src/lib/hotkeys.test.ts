import { describe, expect, it } from 'vitest';
import { EXISTING_KEYS, NEW_KEYS, hotkeyAction, nextSpot, stepSpeed, type KeyLike } from './hotkeys';
import type { ClusterSpot } from './types';

/** `Ctrl+Alt+Shift+X` as a key event. */
function ev(spec: string, extra: Partial<KeyLike> = {}): KeyLike {
  const parts = spec === '+' ? ['+'] : spec.split('+');
  const key = parts.pop() || '+';
  return {
    key,
    ctrlKey: parts.includes('Ctrl'),
    altKey: parts.includes('Alt'),
    shiftKey: parts.includes('Shift'),
    metaKey: parts.includes('Meta'),
    ...extra,
  };
}
const none = { tag: 'BODY', id: '' };
const inInput = { tag: 'INPUT', id: 'draft' };

describe('which key means what', () => {
  it('knows the new keys', () => {
    expect(hotkeyAction(ev('Insert'), none)).toBe('send_call_exchange');
    expect(hotkeyAction(ev('.'), none)).toBe('log_confirm');
    expect(hotkeyAction(ev('PageUp'), none)).toBe('speed_up');
    expect(hotkeyAction(ev('PageDown'), none)).toBe('speed_down');
    expect(hotkeyAction(ev('Ctrl+ArrowUp'), none)).toBe('spot_up');
    expect(hotkeyAction(ev('Ctrl+ArrowDown'), none)).toBe('spot_down');
    expect(hotkeyAction(ev('Alt+q'), none)).toBe('cq_freq');
    expect(hotkeyAction(ev('Alt+Q'), none)).toBe('cq_freq');
    expect(hotkeyAction(ev('Alt+F7'), none)).toBe('split_toggle');
  });

  it('no new key is one that CWLD already used', () => {
    const fresh = new Set<string>(NEW_KEYS);
    for (const k of EXISTING_KEYS) expect(fresh.has(k), k).toBe(false);
    // And the other way round: none of the old keys starts a new action.
    for (const k of EXISTING_KEYS) {
      const spec = k === ' ' ? ' ' : k;
      expect(hotkeyAction(ev(spec), none), k).toBeNull();
    }
  });

  it('the period is a character in every text field except the entry line of the contest mode', () => {
    expect(hotkeyAction(ev('.'), inInput)).toBeNull();
    expect(hotkeyAction(ev('.'), { tag: 'TEXTAREA', id: '' })).toBeNull();
    expect(hotkeyAction(ev('.'), { tag: 'INPUT', id: 'ce-call' })).toBe('log_confirm');
    expect(hotkeyAction(ev('.'), { tag: 'INPUT', id: 'ce-nr' })).toBe('log_confirm');
    expect(hotkeyAction(ev('.'), { tag: 'BUTTON', id: '' })).toBe('log_confirm');
  });

  it('the period works with Shift (AZERTY) and on the numeric keypad', () => {
    expect(hotkeyAction(ev('Shift+.'), none)).toBe('log_confirm');
    expect(hotkeyAction({ ...ev(','), code: 'NumpadDecimal' }, none)).toBe('log_confirm');
    expect(hotkeyAction(ev(','), none)).toBeNull();
  });

  it('keys with the wrong modifiers are left alone', () => {
    expect(hotkeyAction(ev('Ctrl+Insert'), none)).toBeNull();
    expect(hotkeyAction(ev('Shift+PageUp'), none)).toBeNull();
    expect(hotkeyAction(ev('Ctrl+PageUp'), none)).toBeNull();
    expect(hotkeyAction(ev('ArrowUp'), none)).toBeNull();
    expect(hotkeyAction(ev('Ctrl+Shift+ArrowUp'), none)).toBeNull();
    expect(hotkeyAction(ev('Ctrl+Alt+q'), none)).toBeNull();
    expect(hotkeyAction(ev('Alt+Shift+F7'), none)).toBeNull();
    expect(hotkeyAction(ev('F7'), none)).toBeNull();
    expect(hotkeyAction(ev('Ctrl+.'), none)).toBeNull();
  });

  it('a held key does not send, log or jump again, but the speed keys repeat', () => {
    for (const k of ['Insert', '.', 'Ctrl+ArrowUp', 'Alt+q', 'Alt+F7']) expect(hotkeyAction(ev(k, { repeat: true }), none), k).toBeNull();
    expect(hotkeyAction(ev('PageUp', { repeat: true }), none)).toBe('speed_up');
  });
});

describe('speed', () => {
  it('moves in steps and stays inside 5 to 60 WPM', () => {
    expect(stepSpeed(24, 2)).toBe(26);
    expect(stepSpeed(24, -2)).toBe(22);
    expect(stepSpeed(59, 2)).toBe(60);
    expect(stepSpeed(60, 2)).toBe(60);
    expect(stepSpeed(6, -2)).toBe(5);
    expect(stepSpeed(5, -2)).toBe(5);
  });
});

describe('next spot', () => {
  const now = 1_800_000_000;
  const spot = (call: string, khz: number, extra: Partial<ClusterSpot> = {}): ClusterSpot => ({
    call, freq_hz: Math.round(khz * 1000), band: '', spotter: 'X', comment: '', snr: null, wpm: null, skimmer: false, ts: now - 60, hits: 1, status: 'new', ...extra,
  });
  const spots = [
    spot('AA1', 7010.0), spot('BB2', 7015.2), spot('CC3', 7030.0), spot('DD4', 7040.5),
    spot('WORKED', 7022.0, { status: 'worked' }),
    spot('OLD', 7025.0, { ts: now - 7200 }),
    spot('OTHER', 14025.0), spot('OB', 7020.0, { status: 'other_band' }),
  ];
  const o = { nowSec: now, maxAgeSec: 1800 };

  it('goes up and down on the current band', () => {
    expect(nextSpot(spots, 7_012_000, 1, o)?.call).toBe('BB2');
    expect(nextSpot(spots, 7_012_000, -1, o)?.call).toBe('AA1');
    expect(nextSpot(spots, 7_015_200, 1, o)?.call).toBe('OB');
    expect(nextSpot(spots, 7_020_000, 1, o)?.call).toBe('CC3');
  });
  it('skips worked stations, old spots and other bands', () => {
    const calls = new Set<string>();
    let hz = 7_000_000;
    for (let i = 0; i < 10; i++) {
      const s = nextSpot(spots, hz, 1, o);
      if (!s) break;
      calls.add(s.call);
      hz = s.freq_hz;
    }
    expect([...calls]).toEqual(['AA1', 'BB2', 'OB', 'CC3', 'DD4']);
  });
  it('worked stations come back when asked', () => {
    expect(nextSpot(spots, 7_021_000, 1, { ...o, skipWorked: false })?.call).toBe('WORKED');
  });
  it('a spot at the frequency you are on counts as here: the key moves on', () => {
    expect(nextSpot(spots, 7_015_200, 1, o)?.call).not.toBe('BB2');
    expect(nextSpot(spots, 7_015_250, -1, o)?.call).toBe('AA1');
  });
  it('nothing in that direction, or off the amateur bands', () => {
    expect(nextSpot(spots, 7_041_000, 1, o)).toBeNull();
    expect(nextSpot(spots, 7_009_000, -1, o)).toBeNull();
    expect(nextSpot(spots, 7_500_000, 1, o)).toBeNull();
  });
});
