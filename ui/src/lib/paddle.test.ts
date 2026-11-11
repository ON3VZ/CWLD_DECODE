import { describe, expect, it } from 'vitest';
import { accuracy, analyseTiming, exerciseFits, KOCH, kochFirst, kochPool, kochReady, markTarget, nextTarget, rng, runsToText, sessionScore, updateStats, weakest, type Run } from './paddle';
import { gapUnit, pattern, schedule } from './morse';

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
    // Long pauses between the characters (every character its own word) do not make characters wrong.
    const slow = markTarget('AOWRW RAOWR', 'A O W R W R A M W R');
    expect(slow.filter((x) => x.ch !== ' ' && !x.ok).map((x) => `${x.ch}>${x.got}`)).toEqual(['O>M']);
    expect(accuracy(slow)).toBeCloseTo(0.9);
    expect(slow[5]).toEqual({ ch: ' ', ok: true, got: '' });
    expect(accuracy(markTarget('CQ DE', 'CQDE'))).toBe(1);
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

describe('reading the text from the key times (0.13.2)', () => {
  it('reads what was keyed, with word gaps', () => {
    expect(runsToText(runsFor('CQ CQ DE ON3VZ K', 20))).toBe('CQ CQ DE ON3VZ K');
    expect(runsToText(runsFor('PARIS KMRSU', 28, { jitter: 0.15, seed: 3 }))).toBe('PARIS KMRSU');
  });

  it('the first letter is right too (the live decoder often read NARIS)', () => {
    expect(runsToText(runsFor('PARIS', 20))[0]).toBe('P');
  });

  it('a pause inside a character splits it, as the keyer does', () => {
    // A (.-) with a 3 unit pause between dot and dash: E T.
    const unit = 60;
    const runs: Run[] = [[false, 500], [true, unit], [false, 3 * unit], [true, 3 * unit], [false, 3 * unit], [true, unit], [false, unit], [true, unit], [false, 2000]];
    expect(runsToText(runs)).toBe('ETI');
  });

  it('nothing keyed gives nothing, an unknown pattern gives a star', () => {
    expect(runsToText([[false, 3000]])).toBe('');
    const u = 60;
    const runs: Run[] = [[true, u], [false, u], [true, u], [false, u], [true, u], [false, u], [true, u], [false, u], [true, u], [false, u], [true, u], [false, u], [true, u], [false, u], [true, u], [false, 2000]];
    expect(runsToText(runs)).toBe('*');
  });
});

describe('Koch method', () => {
  const ctx = (chars: string) => ({ myCall: 'ON3VZ', name: 'KRISTOF', qth: 'HOBOKEN', stats: [], chars });

  it('uses only the chosen characters in every exercise', () => {
    const chars = kochFirst(12); // KMRSUAPTLOWI
    const pool = new Set(chars);
    for (const ex of ['chars', 'weak', 'words', 'calls', 'phrases'] as const) {
      const r = rng(5);
      for (let i = 0; i < 30; i++) {
        const text = nextTarget(ex, ctx(chars), r);
        for (const c of text.replace(/ /g, '')) expect(pool.has(c), `${ex}: ${c} in ${text}`).toBe(true);
      }
    }
  });

  it('falls back to groups when too few words fit', () => {
    expect(exerciseFits('words', 'KM')).toBe(false);
    const text = nextTarget('words', ctx('KM'), rng(3));
    expect(text.split(' ')).toHaveLength(4);
    expect(text.split(' ')[0]).toHaveLength(5);
    expect(exerciseFits('words', KOCH)).toBe(true);
    expect(exerciseFits('calls', 'KMRSUAPTLOWI')).toBe(false); // no digit
  });

  it('keeps Koch order and ignores unknown characters', () => {
    expect(kochPool('MK#').join('')).toBe('KM');
    expect(kochPool('').length).toBe(KOCH.length);
    expect(kochFirst(1).length).toBe(2);
  });

  it('asks the newest character more often', () => {
    const r = rng(11);
    let n = 0;
    let total = 0;
    for (let i = 0; i < 400; i++) for (const c of nextTarget('chars', ctx('KMRSU'), r).replace(/ /g, '')) { total++; if (c === 'U') n++; }
    expect(n / total).toBeGreaterThan(0.25);
  });

  it('proposes the next character after three good rounds with the same set, never earlier', () => {
    const s = (accuracy: number, n = 5) => ({ accuracy, n, chars: 20 });
    expect(kochReady([s(0.95), s(0.92)], 5)).toBe(false);
    expect(kochReady([s(0.95), s(0.92), s(0.9)], 5)).toBe(true);
    expect(kochReady([s(0.95), s(0.92), s(0.9, 4)], 5)).toBe(false);
    expect(kochReady([s(0.95), s(0.8), s(0.8)], 5)).toBe(false);
    expect(kochReady([s(1, 38), s(1, 38), s(1, 38)], 38)).toBe(false);
  });
});

describe('Farnsworth timing', () => {
  it('keeps the character speed and stretches the gaps so the text runs at the overall speed', () => {
    // PARIS and a following word gap is exactly 50 units: 60 / eff seconds.
    const s = schedule('PARIS', 20, 10);
    expect(s[1].off - s[1].on).toBeCloseTo(3 * 0.06, 6); // dash length stays at 20 WPM
    const end = s[s.length - 1].off;
    expect(end + 7 * gapUnit(20, 10)).toBeCloseTo(60 / 10, 5);
    expect(gapUnit(20)).toBeCloseTo(0.06, 6);
    expect(gapUnit(20, 30)).toBeCloseTo(0.06, 6);
  });
});
