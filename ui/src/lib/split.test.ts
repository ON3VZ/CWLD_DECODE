import { describe, expect, it } from 'vitest';
import { offsetLabel, splitFromComment, splitFromInput, splitProblem } from './split';

const RX = 7_025_000;

describe('split out of a spot comment', () => {
  const tx = (c: string, rx = RX) => splitFromComment(c, rx)?.txHz ?? null;
  const label = (c: string, rx = RX) => splitFromComment(c, rx)?.label ?? null;

  it('reads the examples of the plan', () => {
    expect(splitFromComment('UP 1', RX)).toEqual({ txHz: 7_026_000, toHz: null, label: 'UP 1', assumed: false });
    expect(splitFromComment('UP 1-2', RX)).toEqual({ txHz: 7_026_000, toHz: 7_027_000, label: 'UP 1-2', assumed: false });
    expect(splitFromComment('QSX 7027.5', RX)).toEqual({ txHz: 7_027_500, toHz: null, label: 'QSX 7027.5', assumed: false });
    expect(splitFromComment('DWN 2', RX)).toEqual({ txHz: 7_023_000, toHz: null, label: 'DWN 2', assumed: false });
  });

  it('reads the ways people write it', () => {
    expect(tx('up1')).toBe(7_026_000);
    expect(tx('CW UP 1.5 pse')).toBe(7_026_500);
    expect(tx('UP 1,5')).toBe(7_026_500);
    expect(tx('599 in ON, UP2')).toBe(7_027_000);
    expect(label('UP 2 TO 5')).toBe('UP 2-5');
    expect(tx('listening DOWN 3')).toBe(7_022_000);
    expect(tx('DN1')).toBe(7_024_000);
    expect(tx('QSX 7.0275')).toBe(7_027_500);
    expect(tx('QSX 27.5')).toBe(7_027_500);
    expect(tx('qsx: 7027')).toBe(7_027_000);
    expect(tx('QSX 14.030', 14_025_000)).toBe(14_030_000);
    expect(tx('QSX 7.5')).toBe(7_007_500);
  });

  it('takes one kilohertz for UP without a number, and says so', () => {
    expect(splitFromComment('TNX QSO UP', RX)).toEqual({ txHz: 7_026_000, toHz: null, label: 'UP', assumed: true });
    expect(splitFromComment('DWN', RX)).toEqual({ txHz: 7_024_000, toHz: null, label: 'DWN', assumed: true });
  });

  it('does not see a split where there is none', () => {
    for (const c of ['', 'CW 24 dB 22 WPM CQ', 'TNX QSO 73', 'SETUP 1', 'UPPER 5', 'QRV UP TO 20 WPM', 'PWR UP 5W', 'UP 500', 'UP 0', 'QSX 14030', 'QSX 7025.0', 'D2UY wkd', 'DL1UP 599', 'GROUP 2']) {
      expect(splitFromComment(c, RX), c).toBeNull();
    }
    expect(splitFromComment('UP 1', 0)).toBeNull();
  });
});

describe('split typed by the operator', () => {
  const tx = (c: string) => splitFromInput(c, RX)?.txHz ?? null;

  it('takes a bare offset in kHz', () => {
    expect(tx('1')).toBe(7_026_000);
    expect(tx('+1.5')).toBe(7_026_500);
    expect(tx('-2')).toBe(7_023_000);
    expect(tx('2 khz')).toBe(7_027_000);
    expect(splitFromInput('-2', RX)?.label).toBe('DWN 2');
  });

  it('takes a bare frequency', () => {
    expect(tx('7027.5')).toBe(7_027_500);
    expect(tx('7027,5')).toBe(7_027_500);
    expect(splitFromInput('7027.5', RX)?.label).toBe('UP 2.5');
    expect(splitFromInput('7023', RX)?.label).toBe('DWN 2');
  });

  it('takes what a spot comment may hold', () => {
    expect(tx('up 1')).toBe(7_026_000);
    expect(tx('UP')).toBe(7_026_000);
    expect(tx('qsx 7027.5')).toBe(7_027_500);
  });

  it('refuses nonsense', () => {
    for (const c of ['', ' ', '0', 'abc', '14030', '7025', '+45', '-0']) expect(splitFromInput(c, RX), c).toBeNull();
  });
});

describe('helpers', () => {
  it('names an offset', () => {
    expect(offsetLabel(RX, 7_026_000)).toBe('UP 1');
    expect(offsetLabel(RX, 7_022_500)).toBe('DWN 2.5');
  });

  it('keeps the transmit frequency in the band of the receive frequency', () => {
    expect(splitProblem(RX, 7_026_000)).toBeNull();
    expect(splitProblem(7_001_000, 6_999_000)).toBe('split_other_band');
    expect(splitProblem(9_000_000, 9_001_000)).toBe('split_other_band');
    expect(splitProblem(RX, RX)).toBe('split_same_frequency');
  });
});
