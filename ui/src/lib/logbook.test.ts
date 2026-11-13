import { describe, expect, it } from 'vitest';
import { csvField, csvText, COLUMNS, DEFAULT_COLUMNS, bandOf, isSplit, parseKhz, parseUtc, rxHz, sortRows, summary } from './logbook';
import type { LogQso } from './types';

function q(id: number, call: string, ts: number, band: string, freq_hz = 7_030_000): LogQso {
  const blank = Object.fromEntries(['rst_sent', 'rst_rcvd', 'nr_rcvd', 'extra_sent', 'extra_rcvd', 'name', 'qth', 'comment', 'contest', 'my_call', 'station_call', 'operator', 'my_grid', 'tx_pwr', 'my_rig', 'my_antenna', 'my_wwff', 'my_pota', 'my_sota', 'their_prog', 'their_ref', 'exported_to'].map((k) => [k, '']));
  return { ...blank, id, call, ts, band, freq_hz, mode: 'CW', nr_sent: 0, session: 0, test: false, deleted: false, activation_id: 0 } as LogQso;
}

describe('logbook table', () => {
  const rows = [q(1, 'PA3ABC', 100, '20m', 14_030_000), q(2, 'DL1XYZ', 300, '40m'), q(3, 'PA3ABC', 200, '80m', 3_550_000)];

  it('sorts by time, call, band in band order and frequency', () => {
    expect(sortRows(rows, 'ts', false).map((r) => r.id)).toEqual([2, 3, 1]);
    expect(sortRows(rows, 'call', true).map((r) => r.id)).toEqual([2, 3, 1]);
    expect(sortRows(rows, 'band', true).map((r) => r.band)).toEqual(['80m', '40m', '20m']);
    expect(sortRows(rows, 'freq_hz', false).map((r) => r.id)).toEqual([1, 2, 3]);
    expect(rows.map((r) => r.id), 'the input stays as it was').toEqual([1, 2, 3]);
  });

  it('counts QSOs, different calls and bands', () => {
    expect(summary(rows)).toEqual({ qsos: 3, calls: 2, bands: 3 });
    expect(summary([])).toEqual({ qsos: 0, calls: 0, bands: 0 });
  });

  it('every default column exists and shows the details', () => {
    for (const k of DEFAULT_COLUMNS) expect(COLUMNS.some((c) => c.key === k), k).toBe(true);
    const col = (k: string) => COLUMNS.find((c) => c.key === k)!;
    const x = { ...q(9, 'ON4AA', Date.UTC(2026, 10, 12, 14, 5) / 1000, '40m', 7_012_100), their_prog: 'WWFF', their_ref: 'ONFF-0599', nr_sent: 12, extra_sent: 'WLD', exported_to: 'adif' };
    expect(col('date').value(x)).toBe('2026-11-12');
    expect(col('time').value(x)).toBe('14:05');
    expect(col('freq').value(x)).toBe('7012.1');
    expect(col('their_ref').value(x)).toBe('WWFF ONFF-0599');
    expect(col('exch_sent').value(x)).toBe('12 WLD');
    expect(col('exported').value(x)).toBe('ADIF');
  });
});

describe('logbook edit helpers', () => {
  it('finds the band like the core does', () => {
    expect(bandOf(14_025_000)).toBe('20m');
    expect(bandOf(7_030_000)).toBe('40m');
    expect(bandOf(9_000_000)).toBe('');
  });
  it('reads UTC date and time', () => {
    expect(parseUtc('2026-11-12', '14:05')).toBe(Date.UTC(2026, 10, 12, 14, 5) / 1000);
    expect(parseUtc('2026-11-12', '14:05:30')).toBe(Date.UTC(2026, 10, 12, 14, 5, 30) / 1000);
    expect(parseUtc('2026-13-12', '14:05')).toBeNaN();
    expect(parseUtc('2026-11-12', '25:00')).toBeNaN();
    expect(parseUtc('12-11-2026', '14:05')).toBeNaN();
  });
  it('reads kilohertz', () => {
    expect(parseKhz('14025')).toBe(14_025_000);
    expect(parseKhz('14025,5')).toBe(14_025_500);
    expect(parseKhz('')).toBe(0);
    expect(parseKhz('abc')).toBeNaN();
  });
});

describe('csv', () => {
  it('quotes only where needed', () => {
    expect(csvField('PA3ABC')).toBe('PA3ABC');
    expect(csvField('a;b')).toBe('"a;b"');
    expect(csvField('say "hi"')).toBe('"say ""hi"""');
    expect(csvField('two\nlines')).toBe('"two\nlines"');
  });
  it('defuses spreadsheet formulas but keeps plain numbers', () => {
    expect(csvField('=1+1')).toBe("'=1+1");
    expect(csvField('+32 3 123')).toBe("'+32 3 123");
    expect(csvField('-599')).toBe('-599');
  });
  it('writes the chosen columns in the logbook order, with a BOM and CRLF', () => {
    const a = { ...q(1, 'PA3ABC', Date.UTC(2026, 10, 12, 14, 5) / 1000, '20m', 14_025_000), name: 'Jan; Piet' };
    const text = csvText([a], ['name', 'call', 'freq', 'nope'], (k) => k.toUpperCase());
    expect(text).toBe('\uFEFFCALL;FREQ;NAME\r\nPA3ABC;14025.0;"Jan; Piet"\r\n');
  });
});

describe('transmit and receive frequency', () => {
  const col = (k: string) => COLUMNS.find((c) => c.key === k)!;

  it('are the same for a normal QSO', () => {
    const x = q(1, 'PA3ABC', 1000, '40m', 7_030_000);
    expect(isSplit(x)).toBe(false);
    expect(rxHz(x)).toBe(7_030_000);
    expect(col('freq').value(x)).toBe('7030.0');
    expect(col('freq_rx').value(x)).toBe('7030.0');
  });

  it('differ for a QSO in split', () => {
    const x = { ...q(2, '3B8XX', 1000, '40m', 7_026_000), freq_rx_hz: 7_025_000 };
    expect(isSplit(x)).toBe(true);
    expect(col('freq').value(x)).toBe('7026.0');
    expect(col('freq_rx').value(x)).toBe('7025.0');
  });

  it('both columns show by default', () => {
    expect(DEFAULT_COLUMNS).toContain('freq');
    expect(DEFAULT_COLUMNS).toContain('freq_rx');
  });
});
