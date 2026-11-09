import { describe, expect, it } from 'vitest';
import { accuracy, analyseTiming, KOCH, markTarget, nextTarget, rng, sessionScore, updateStats, weakest, type Run } from './paddle';
import { pattern } from './morse';

/** Key runs for a text: element, character and word gaps in units, with optional jitter. */
function runsFor(text: string, wpm: number, o: { charGap?: number; wordGap?: number; dash?: number; jitter?: number; seed?: number } = {}): Run[] {
  const unit = 1200 / wpm;
  const r = rng(o.seed ?? 7);
  const j = (x: number) => x * (1 + (o.jitter ?? 0) * (r() * 2 - 1));
  const out: Run[] = [[false, 500]];
  const words = text.split(' ');
  words.forEach((w, wi) => {
    [...w].forEach((c, ci) => {
      const p = pattern(c)!;
      [...p].forEach((e, ei) => {
        out.push([true, j((e === '.' ? 1 : (o.dash ?? 3)) * unit)]);
        if (ei < p.length - 1) out.push([false, j(unit)]);
      });
      if (ci < w.length - 1) out.push([false, j((o.charGap ?? 3) * unit)]);
    });
    if (wi < words.length - 1) out.push([false, j((o.wordGap ?? 7) * unit)]);
  });
  out.push([false, 3000]);
  return out;
}

describe('paddle timing', () => {
  it('perfect keying at 20 WPM scores high and says a keyer made the elements', () => {
    const t = analyseTiming(runsFor('CQ CQ DE ON3VZ K', 20));
    expect(t.wpm).toBeCloseTo(20, 0);
    expect(t.dashDot).toBeCloseTo(3, 1);
    expect(t.elementGap).toBeCloseTo(1, 1);
    expect(t.charGap).toBeCloseTo(3, 1);
    expect(t.wordGap).toBeCloseTo(7, 1);
    expect(t.keyer).toBe(true);
    expect(t.score).toBeGreaterThanOrEqual(95);
    expect(t.tips).toContain('paddle.tip.keyer');
  });

  it('characters run together when the gap between them is too short', () => {
    const t = analyseTiming(runsFor('PARIS PARIS', 25, { charGap: 2.1 }));
    expect(t.tips).toContain('paddle.tip.char_gap_short');
    expect(t.score).toBeLessThan(90);
  });

  it('long pauses between characters and short word gaps are named', () => {
    const t = analyseTiming(runsFor('TEST DE ON3VZ', 18, { charGap: 5.5, wordGap: 9 }), 2);
    // The asked text has two word gaps: the two longest gaps are taken as word gaps.
    expect(t.tips).toContain('paddle.tip.char_gap_long');
    expect(t.wordGap).toBeCloseTo(9, 0);
    const short = analyseTiming(runsFor('TEST DE ON3VZ', 18, { wordGap: 4 }), 2);
    expect(short.tips).toContain('paddle.tip.word_gap_short');
  });

  it('a straight key with short dashes and uneven elements is not called a keyer', () => {
    const t = analyseTiming(runsFor('CQ CQ DE ON3VZ', 15, { dash: 2.3, jitter: 0.3 }));
    expect(t.keyer).toBe(false);
    expect(t.tips).toContain('paddle.tip.dash_short');
    expect(t.tips).toContain('paddle.tip.uneven');
  });

  it('only dashes still gives a speed', () => {
    const t = analyseTiming(runsFor('TMO TMO', 20));
    expect(t.wpm).toBeCloseTo(20, 0);
  });

  it('too little keying gives no score but a hint', () => {
    const t = analyseTiming([[true, 60], [false, 60], [true, 60]]);
    expect(t.score).toBe(0);
    expect(t.tips).toEqual(['paddle.tip.too_short']);
  });
});

describe('marking and statistics', () => {
  it('marks wrong, missing and split characters', () => {
    const m = markTarget('PARIS', 'PAEIS');
    expect(m.map((x) => x.ok)).toEqual([true, true, false, true, true]);
    expect(m[2].got).toBe('E');
    const missing = markTarget('ABC', 'AC');
    expect(missing.map((x) => x.ok)).toEqual([true, false, true]);
    // A pause inside A (.-) makes E T: A is wrong.
    const split = markTarget('AB', 'ETB');
    expect(split[1].ok).toBe(true);
    expect(split[0].ok).toBe(false);
    expect(accuracy(markTarget('CQ DE', 'CQ DE'))).toBe(1);
  });

  it('counts tries and errors and finds the weakest characters', () => {
    let s = updateStats([], markTarget('KKKMMM', 'KKKMMT'));
    s = updateStats(s, markTarget('MMM', 'TTM'));
    expect(s.find((x) => x.ch === 'M')).toEqual({ ch: 'M', tries: 6, errors: 3 });
    expect(weakest(s).map((x) => x.ch)).toEqual(['M']);
  });
});

describe('targets', () => {
  const ctx = { myCall: 'ON3VZ', name: 'Kristof', qth: 'Hoboken', stats: [] };
  it('gives groups, words, calls and phrases from the known characters', () => {
    const r = rng(42);
    const g = nextTarget('chars', ctx, r);
    expect(g.split(' ').length).toBe(4);
    expect([...g.replaceAll(' ', '')].every((c) => KOCH.includes(c))).toBe(true);
    expect(nextTarget('words', ctx, r).split(' ').length).toBe(5);
    expect(nextTarget('calls', ctx, r)).toMatch(/^[A-Z0-9]+\d[A-Z]+( [A-Z0-9]+\d[A-Z]+){2}$/);
    const p = nextTarget('phrases', { ...ctx }, rng(5));
    expect(p).not.toMatch(/[{}]/);
  });

  it('the weak exercise uses the weak characters', () => {
    const stats = [
      { ch: 'Q', tries: 10, errors: 6 },
      { ch: 'Y', tries: 10, errors: 4 },
      { ch: 'K', tries: 10, errors: 0 },
    ];
    const g = nextTarget('weak', { ...ctx, stats }, rng(3)).replaceAll(' ', '');
    expect([...g].every((c) => c === 'Q' || c === 'Y')).toBe(true);
  });

  it('a phrase never contains a made-up callsign', () => {
    for (let i = 1; i < 30; i++) expect(nextTarget('phrases', { myCall: '', name: '', qth: '', stats: [] }, rng(i))).not.toMatch(/ON3VZ/);
  });

  it('session score weighs right characters most', () => {
    const t = analyseTiming(runsFor('PARIS PARIS', 20));
    expect(sessionScore(1, t)).toBeGreaterThan(95);
    expect(sessionScore(0.5, t)).toBeLessThan(70);
  });
});
