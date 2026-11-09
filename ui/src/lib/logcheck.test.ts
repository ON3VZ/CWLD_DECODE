import { describe, expect, it } from 'vitest';
import { checkExport, validGrid, validWwffRef } from './logcheck';
import { emptyLogQso } from './types';
import type { Activation, LogQso } from './types';

const q = (o: Partial<LogQso> = {}): LogQso => ({ ...emptyLogQso(), id: 1, call: 'DL1ABC', band: '40m', mode: 'CW', ts: 1000, rst_sent: '599', rst_rcvd: '599', my_call: 'ON3VZ', my_grid: 'JO20EE', ...o });
const codes = (l: ReturnType<typeof checkExport>) => l.map((i) => i.code);

describe('grid and reference shapes', () => {
  it('accepts six character grids only', () => {
    expect(validGrid('JO20QE')).toBe(true);
    expect(validGrid('jo20qe')).toBe(true);
    expect(validGrid('JO20')).toBe(false);
    expect(validGrid('ZZ99AA')).toBe(false);
  });
  it('accepts ONFF-0599 and refuses other shapes', () => {
    expect(validWwffRef('ONFF-0599')).toBe(true);
    expect(validWwffRef('ONFF-599')).toBe(false);
    expect(validWwffRef('onff-0599')).toBe(false);
  });
});

describe('export check', () => {
  it('refuses an empty export', () => {
    expect(codes(checkExport([], [], 2000))).toEqual(['empty']);
  });
  it('is clean for a complete home QSO', () => {
    expect(checkExport([q()], [], 2000)).toEqual([]);
  });
  it('a missing grid is only a warning at home but an error when portable', () => {
    const home = checkExport([q({ my_grid: '' })], [], 2000);
    expect(home[0]).toMatchObject({ code: 'no_grid', severity: 'warning' });
    const p = checkExport([q({ my_grid: '', station_call: 'ON3VZ/P' })], [], 2000);
    expect(p[0]).toMatchObject({ code: 'no_grid', severity: 'error' });
  });
  it('flags a bad reference and a QSO outside its activation', () => {
    const a: Activation = { id: 3, profile: 'p', grid: 'JO20QE', power: '5', wwff: 'ONFF-0599', pota: '', sota: '', note: '', started_ts: 500, ended_ts: 900 };
    const l = checkExport([q({ activation_id: 3, my_wwff: 'ONFF599', ts: 5000 })], [a], 6000);
    expect(codes(l)).toContain('bad_wwff_ref');
    expect(codes(l)).toContain('outside_activation');
  });
  it('warns about duplicates and mixed grids', () => {
    const l = checkExport([q({ id: 1 }), q({ id: 2, ts: 1100, my_grid: 'JO21AA' })], [], 2000);
    expect(codes(l)).toContain('dupe');
    expect(codes(l)).toContain('several_grids');
  });
});
