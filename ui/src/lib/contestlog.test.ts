import { describe, expect, it } from 'vitest';
import { contestKeys, enterAction, entryComplete, nextSerial, rates, suggestFrom } from './contestlog';
import type { ContestSettings } from './types';

const c = (active: string, sent_index = 0): ContestSettings => ({ active, sent_index, session: 1, profiles: [], vars: [] });

describe('contest keys', () => {
  it('F1 is CQ in Run and your call in S&P, F3 logs', () => {
    const run = contestKeys('run', c('uba-dx-cw'));
    expect(run[0].template).toBe('TEST UBA {MYCALL}');
    expect(run[1].template).toBe('{CALL} {RST} {NR} {SEC}');
    expect(run[2].log).toBe(true);
    expect(run.map((k) => k.key)).toEqual(['F1', 'F2', 'F3', 'F4', 'F5', 'F6', 'F7', 'F8']);
    expect(contestKeys('sp', c('uba-dx-cw'))[0].template).toBe('{MYCALL}');
  });
  it('the exchange follows the chosen sent template', () => {
    expect(contestKeys('run', c('uba-dx-cw', 1))[1].template).toBe('{CALL} {RST} {NR}');
  });
  it('without a profile there are plain defaults', () => {
    expect(contestKeys('run', c(''))[0].template).toBe('CQ TEST {MYCALL}');
  });
});

describe('Enter (ESM)', () => {
  const st = (o: object) => ({ call: '', exchangeSent: false, mycallSent: false, complete: false, ...o });
  it('Run: empty sends CQ, a call sends the exchange', () => {
    expect(enterAction('run', st({}))).toEqual({ key: 'F1', log: false });
    expect(enterAction('run', st({ call: 'PA3ABC' }))).toEqual({ key: 'F2', log: false });
  });
  it('Run: after our exchange without his, AGN; with his, TU and log', () => {
    expect(enterAction('run', st({ call: 'PA3ABC', exchangeSent: true }))).toEqual({ key: 'F8', log: false });
    expect(enterAction('run', st({ call: 'PA3ABC', exchangeSent: true, complete: true }))).toEqual({ key: 'F3', log: true });
  });
  it('S&P: a call sends your call once, a complete exchange sends yours and logs', () => {
    expect(enterAction('sp', st({}))).toEqual({ key: null, log: false });
    expect(enterAction('sp', st({ call: 'PA3ABC' }))).toEqual({ key: 'F1', log: false });
    expect(enterAction('sp', st({ call: 'PA3ABC', mycallSent: true }))).toEqual({ key: null, log: false });
    expect(enterAction('sp', st({ call: 'PA3ABC', mycallSent: true, complete: true }))).toEqual({ key: 'F2', log: true });
  });
});

describe('complete entry', () => {
  it('UBA needs a number, and a section for Belgian stations only', () => {
    const u = c('uba-dx-cw');
    expect(entryComplete(u, 'PA3ABC', '012', '')).toBe(true);
    expect(entryComplete(u, 'PA3ABC', '', '')).toBe(false);
    expect(entryComplete(u, 'ON4XYZ', '012', '')).toBe(false);
    expect(entryComplete(u, 'ON4XYZ', '012', 'ANR')).toBe(true);
  });
  it('CQ WW needs the zone, no serial', () => {
    const w = c('cqww-cw');
    expect(entryComplete(w, 'DL1ABC', '', '14')).toBe(true);
    expect(entryComplete(w, 'DL1ABC', '', '')).toBe(false);
  });
  it('an empty call is never complete', () => {
    expect(entryComplete(c('cqww-cw'), '', '', '14')).toBe(false);
  });
});

describe('rate and serial', () => {
  it('counts QSOs in the last 10 and 60 minutes', () => {
    const now = 10_000;
    const r = rates([now - 60, now - 300, now - 1000, now - 5000], now);
    expect(r.rate10).toBe(12);
    expect(r.rate60).toBe(3);
  });
  it('serial goes up by one', () => {
    expect(nextSerial(41)).toBe(42);
    expect(nextSerial(9999)).toBe(9999);
  });
});

describe('suggestions from the reply', () => {
  it('finds report, number and section', () => {
    expect(suggestFrom('ON4XYZ 5NN 023 ANR', c('uba-dx-cw'), 'ON4XYZ')).toEqual({ rst: '5NN', nr: '023', extra: 'ANR' });
  });
  it('cut numbers are turned back into digits', () => {
    expect(suggestFrom('5NN T2N', c('uba-dx-cw'), 'PA3ABC').nr).toBe('029');
  });
  it('guesses nothing when the parts are missing', () => {
    expect(suggestFrom('QRZ', c('uba-dx-cw'), 'PA3ABC')).toEqual({});
  });
  it('CQ WW takes a zone, not a serial', () => {
    expect(suggestFrom('5NN 14', c('cqww-cw'), 'DL1ABC')).toEqual({ rst: '5NN', extra: '14' });
  });
});
