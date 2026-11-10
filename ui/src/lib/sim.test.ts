import { describe, expect, it } from 'vitest';
import { distance, PracticeSim, Rand, STRENGTH_DB } from './sim';
import type { SimTx } from './sim';
import { emptyLogQso } from './types';
import type { ContestSettings, LogQso, PracticeSettings } from './types';

const cfg = (o: Partial<PracticeSettings> = {}): PracticeSettings => ({ exercise: 'they_cq', wpm_min: 18, wpm_max: 26, strength: 'medium', noise: true, qsb: false, pileup: 3, minutes: 10, decoder_help: true, volume: 0.5, quiet_radio: true, paddle: { exercise: 'chars', stats: [], sessions: [] }, koch: { chars: 'KMRSUAPTLOWINJEF0YVG5/Q9ZH38B?427C1D6X', char_wpm: 20, eff_wpm: 12, tone_hz: 650, listen: 'groups', stats: [], sessions: [] }, ...o });
const contest = (active: string, sent_index = 0): ContestSettings => ({ active, sent_index, session: 1, profiles: [], vars: [] });
const sim = (o: Partial<PracticeSettings> = {}, c?: ContestSettings, seed = 7) => new PracticeSim({ cfg: cfg(o), myCall: 'ON3VZ', myName: 'KRIS', toneHz: 700, contest: c, seed });
/** Lets every transmission end, as the band would. */
const finish = (s: PracticeSim, txs: SimTx[], now: number) => txs.flatMap((t) => s.done(t.call, now));

describe('helpers', () => {
  it('edit distance', () => {
    expect(distance('PA3ABC', 'PA3ABC')).toBe(0);
    expect(distance('PA3ABC', 'PA3ABD')).toBe(1);
    expect(distance('PA3ABC', 'PA3AB')).toBe(1);
  });
  it('the generator repeats from its seed', () => {
    const a = new Rand(5);
    const b = new Rand(5);
    expect([a.next(), a.next()]).toEqual([b.next(), b.next()]);
  });
});

describe('QSO exercise: they call CQ', () => {
  it('a station calls CQ, answers you, gives info and ends', () => {
    const s = sim();
    const t0 = s.start(0);
    expect(t0).toHaveLength(1);
    const st = s.current!;
    expect(t0[0].text).toMatch(new RegExp(`^CQ CQ .*DE ${st.call} ${st.call}`));
    expect(st.call).not.toBe('ON3VZ');
    const [lo, hi] = STRENGTH_DB.medium;
    expect(t0[0].snr_db).toBeGreaterThanOrEqual(lo);
    expect(t0[0].snr_db).toBeLessThanOrEqual(hi);
    expect(Math.abs(t0[0].tone_hz - 700)).toBeLessThanOrEqual(40);
    // While it is on air it does not react.
    expect(s.userSent(`${st.call} DE ON3VZ ON3VZ K`, 2)).toEqual([]);
    finish(s, t0, 5);
    const info = s.userSent(`${st.call} DE ON3VZ ON3VZ K`, 6);
    expect(info[0].text).toContain(`ON3VZ DE ${st.call}`);
    expect(info[0].text).toContain(`NAME ${st.name} ${st.name}`);
    expect(info[0].text).toContain(`QTH ${st.qth}`);
    finish(s, info, 20);
    // AGN repeats the same text.
    const again = s.userSent('AGN', 21);
    expect(again[0].text).toBe(info[0].text);
    finish(s, again, 30);
    const fin = s.userSent(`${st.call} DE ON3VZ R TNX UR 599 NAME KRIS 73`, 31);
    expect(fin[0].text).toContain('73');
    expect(st.worked).toBe(true);
    finish(s, fin, 40);
    expect(st.state).toBe('done');
    // A new station calls CQ a few seconds later.
    expect(s.tick(41)).toEqual([]);
    const next = s.tick(49);
    expect(next).toHaveLength(1);
    expect(s.current!.call).not.toBe(st.call);
  });

  it('repeats CQ when nobody answers, and gives up after a while', () => {
    const s = sim();
    let now = 0;
    let txs = s.start(now);
    const first = s.current!.call;
    for (let i = 0; i < 8; i++) {
      now += 4;
      finish(s, txs, now);
      now += 6;
      txs = s.tick(now);
    }
    expect(s.current!.call).not.toBe(first);
  });
});

describe('QSO exercise: you call CQ', () => {
  it('a station answers your CQ and a partial call makes it repeat', () => {
    const s = sim({ exercise: 'you_cq' });
    expect(s.start(0)).toEqual([]);
    expect(s.userSent('HELLO', 1)).toEqual([]);
    const call = s.userSent('CQ CQ DE ON3VZ ON3VZ K', 2);
    expect(call).toHaveLength(1);
    const st = s.current!;
    expect(call[0].text).toContain(st.call);
    finish(s, call, 5);
    const rep = s.userSent(`${st.call.slice(0, 3)}?`, 6);
    expect(rep[0].text).toContain(`${st.call} ${st.call}`);
    finish(s, rep, 8);
    const info = s.userSent(`${st.call} DE ON3VZ GM UR 599 599 NAME KRIS K`, 9);
    expect(info[0].text).toContain(`R GM KRIS TNX FER RPT`);
    finish(s, info, 25);
    const fin = s.userSent('R TNX 73 SK', 26);
    expect(fin[0].text).toContain('TU E E');
    expect(st.worked).toBe(true);
  });
});

describe('contest exercise (Run)', () => {
  const uba = contest('uba-dx-cw', 1); // abroad template: {CALL} {RST} {NR}

  it('your CQ brings callers, the exchange and TU complete a QSO', () => {
    const s = sim({ exercise: 'contest', pileup: 4 }, uba, 11);
    s.start(0);
    let txs = s.userSent('TEST UBA ON3VZ', 1);
    // Retry a few CQs: a quiet band can happen.
    for (let i = 0; !txs.length && i < 5; i++) txs = s.userSent('TEST UBA ON3VZ', 2 + i);
    expect(txs.length).toBeGreaterThan(0);
    expect(txs.length).toBeLessThanOrEqual(4);
    for (const t of txs) expect(Math.abs(t.tone_hz - 700)).toBeLessThanOrEqual(120);
    finish(s, txs, 5);
    const target = s.stations.find((x) => x.state === 'calling')!;
    expect(target.nr).toMatch(/^\d+$/);
    const exch = s.userSent(`${target.call} 5NN 001`, 6);
    expect(exch).toHaveLength(1);
    expect(exch[0].call).toBe(target.call);
    expect(exch[0].text).toMatch(/5NN|599/);
    expect(target.state).toBe('exch');
    finish(s, exch, 8);
    // NR? repeats the number.
    const nr = s.userSent('NR?', 9);
    expect(nr[0].text.replace(/T/g, '0').replace(/N/g, '9').split(' ').map((w) => w.replace(/^0+/, ''))).toContain(target.nr);
    finish(s, nr, 10);
    const tu = s.userSent('TU ON3VZ', 11);
    expect(target.state).toBe('done');
    expect(target.worked).toBe(true);
    expect(tu.every((t) => t.call !== target.call)).toBe(true);
  });

  it('a wrong call is corrected by the station', () => {
    const s = sim({ exercise: 'contest', pileup: 1 }, uba, 3);
    s.start(0);
    let txs: SimTx[] = [];
    for (let i = 0; !txs.length && i < 5; i++) txs = s.userSent('TEST UBA ON3VZ', 1 + i);
    finish(s, txs, 6);
    const target = s.stations.find((x) => x.state === 'calling')!;
    const wrong = target.call.slice(0, -1) + (target.call.endsWith('X') ? 'Y' : 'X');
    const fix = s.userSent(`${wrong} 5NN 001`, 7);
    expect(fix[0].text.startsWith(`${target.call} ${target.call}`)).toBe(true);
  });

  it('CQ WW callers send their zone, ARRL DX callers are W or VE with a state', () => {
    const ww = sim({ exercise: 'contest', pileup: 3 }, contest('cqww-cw'), 5);
    ww.start(0);
    for (let i = 0; i < 4; i++) ww.userSent('CQ TEST ON3VZ', i);
    for (const st of ww.stations) {
      expect(st.extra).toMatch(/^\d+$/);
      expect(st.nr).toBe('');
    }
    const arrl = sim({ exercise: 'contest', pileup: 3 }, contest('arrl-dx-cw'), 5);
    arrl.start(0);
    for (let i = 0; i < 4; i++) arrl.userSent('CQ TEST ON3VZ', i);
    expect(arrl.stations.length).toBeGreaterThan(0);
    for (const st of arrl.stations) {
      expect(st.call).toMatch(/^(K|W|N|AA|KB|VE|VA)/);
      expect(st.extra).toMatch(/^[A-Z]{2}$/);
    }
  });

  it('waiting callers call again after silence and leave when out of patience', () => {
    const s = sim({ exercise: 'contest', pileup: 3 }, uba, 9);
    s.start(0);
    let txs: SimTx[] = [];
    for (let i = 0; !txs.length && i < 5; i++) txs = s.userSent('TEST UBA ON3VZ', 1 + i);
    finish(s, txs, 6);
    let now = 6;
    for (let i = 0; i < 10; i++) {
      now += 6;
      finish(s, s.tick(now), now + 1);
    }
    expect(s.stations.every((x) => x.state === 'gone')).toBe(true);
  });
});

describe('score', () => {
  const q = (o: Partial<LogQso>): LogQso => ({ ...emptyLogQso(), ts: 100, ...o });

  it('counts correct, busted and wrong exchanges, and missed QSOs', () => {
    const s = sim({ exercise: 'contest', pileup: 3 }, contest('uba-dx-cw', 1), 21);
    s.start(0);
    let txs: SimTx[] = [];
    for (let i = 0; s.stations.length < 3 && i < 8; i++) txs = s.userSent('TEST UBA ON3VZ', 1 + i);
    finish(s, txs, 10);
    const [a, b, c] = s.stations;
    a.worked = b.worked = c.worked = true;
    const sc = s.score([
      q({ call: a.call, nr_rcvd: a.nr.padStart(3, '0'), extra_rcvd: a.extra }),
      q({ call: b.call.slice(0, -1) + (b.call.endsWith('Q') ? 'Z' : 'Q'), nr_rcvd: b.nr }),
      q({ call: c.call, nr_rcvd: String(Number(c.nr) + 1), extra_rcvd: c.extra }),
    ], 600);
    expect(sc.logged).toBe(3);
    expect(sc.correct).toBe(1);
    expect(sc.busted).toBe(1);
    expect(sc.lines[1].truth).toBe(b.call);
    expect(sc.wrongExchange).toBe(1);
    expect(sc.lines[2].problems).toEqual(['nr']);
    expect(sc.missed).toEqual([]);
    expect(sc.minutes).toBe(10);
    expect(sc.rate).toBe(6);
  });
});
