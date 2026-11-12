import { describe, expect, it } from 'vitest';
import {
  checkOk,
  checkTemplate,
  chipSlots,
  cleanTemplate,
  contestSlotId,
  contestSlots,
  cqSlot,
  EMPTY_TEXTS,
  knownVars,
  moveChip,
  newOwnChip,
  pileupSlots,
  previewParts,
  removeOwnChip,
  restoreGroup,
  restoreSlot,
  setSlot,
  textsOf,
} from './texts';
import type { ContestSettings, SendTexts } from './types';

const contest = (active = '', sent_index = 0): ContestSettings => ({
  active,
  sent_index,
  session: 1,
  profiles: [],
  vars: [{ name: 'SEC', value: 'WLD' }, { name: 'ZONE', value: '14' }],
});
const fresh = (): SendTexts => structuredClone(EMPTY_TEXTS);

describe('slots start with the built-in texts', () => {
  it('has the six chips and four pile-up keys, nothing changed', () => {
    const chips = chipSlots(fresh());
    expect(chips.map((c) => c.id)).toEqual(['chip.cq', 'chip.answer', 'chip.report', 'chip.agn', 'chip.qrz', 'chip.bye']);
    expect(chips.every((c) => !c.changed && c.template === c.def)).toBe(true);
    expect(pileupSlots(fresh()).map((s) => s.fkey)).toEqual(['F1', 'F2', 'F3', 'F4']);
  });

  it('gives contest keys a slot per profile and mode', () => {
    const keys = contestSlots('run', contest('cqww-cw'), fresh());
    expect(keys[0].id).toBe('contest.cqww-cw.run.F1');
    expect(keys[0].template).toBe('CQ TEST {MYCALL}');
    expect(keys[1].template).toBe('{CALL} {RST} {ZONE}');
    expect(contestSlots('sp', contest(''), fresh())[0].id).toBe('contest.none.sp.F1');
  });

  it('gives F2 a slot per exchange when a profile has two', () => {
    expect(contestSlots('run', contest('uba-dx-cw', 0), fresh())[1].id).toBe('contest.uba-dx-cw.run.F2.s0');
    expect(contestSlots('run', contest('uba-dx-cw', 1), fresh())[1].id).toBe('contest.uba-dx-cw.run.F2.s1');
    expect(contestSlotId('x', 'run', 'F3')).toBe('contest.x.run.F3');
  });

  it('fills in what an older settings file lacks and keeps CQ repeat in range', () => {
    const t = textsOf({ send_texts: { cq_pause_s: 0, cq_max: 999 } as Partial<SendTexts> });
    expect(t.texts).toEqual({});
    expect(t.cq_pause_s).toBe(2);
    expect(t.cq_max).toBe(30);
    expect(t.paddle_wait).toBe(true);
    expect(textsOf(null).cq_pause_s).toBe(6);
  });
});

describe('changing and restoring', () => {
  it('a built-in contest key can be changed and restored', () => {
    const c = contest('cqww-cw');
    const f3 = contestSlots('run', c, fresh())[2];
    const t = setSlot(fresh(), f3, 'tu {mycall} test', '');
    const after = contestSlots('run', c, t)[2];
    expect(after.template).toBe('TU {MYCALL} TEST');
    expect(after.changed).toBe(true);
    expect(after.log).toBe(true);
    // Another profile keeps its own key.
    expect(contestSlots('run', contest('cqwpx-cw'), t)[2].template).toBe('TU {MYCALL}');
    expect(contestSlots('run', c, restoreSlot(t, f3.id))[2].changed).toBe(false);
  });

  it('saving the built-in text again stores nothing', () => {
    const cq = chipSlots(fresh())[0];
    const t = setSlot(fresh(), cq, cq.def, '');
    expect(t.texts).toEqual({});
    expect(t.labels).toEqual({});
  });

  it('renames a chip and restores a whole group', () => {
    const cq = chipSlots(fresh())[0];
    let t = setSlot(fresh(), cq, 'CQ CQ DE {MYCALL} K', 'Short CQ');
    const pu = pileupSlots(t)[2];
    t = setSlot(t, pu, 'TU {RST}', '');
    expect(chipSlots(t)[0].label).toBe('Short CQ');
    t = restoreGroup(t, 'chip.');
    expect(chipSlots(t)[0].changed).toBe(false);
    expect(pileupSlots(t)[2].template).toBe('TU {RST}');
  });

  it('cleans texts: capitals, one space, no line breaks', () => {
    expect(cleanTemplate('  cq  cq\n de {mycall} ')).toBe('CQ CQ DE {MYCALL}');
  });
});

describe('own chips', () => {
  it('adds, orders and removes an own chip', () => {
    const r = newOwnChip(fresh(), 'WX', 'wx sunny');
    let t = r.texts;
    expect(r.id).toBe('chip.own-1');
    let chips = chipSlots(t);
    expect(chips[chips.length - 1]).toMatchObject({ id: 'chip.own-1', label: 'WX', template: 'WX SUNNY', own: true });
    t = moveChip(t, 'chip.own-1', -1);
    chips = chipSlots(t);
    expect(chips[chips.length - 2].id).toBe('chip.own-1');
    t = removeOwnChip(t, 'chip.own-1');
    expect(chipSlots(t).some((c) => c.own)).toBe(false);
    expect(t.chip_order).not.toContain('chip.own-1');
  });

  it('moving the first chip up changes nothing', () => {
    const t = fresh();
    expect(moveChip(t, 'chip.cq', -1)).toBe(t);
  });
});

describe('checking and preview', () => {
  const known = knownVars(contest());

  it('knows the built-in and own variables', () => {
    expect(known).toEqual(expect.arrayContaining(['CALL', 'MYCALL', 'NR', 'RST', 'SEC', 'ZONE']));
  });

  it('accepts a normal text with a prosign', () => {
    expect(checkOk(checkTemplate('{CALL} DE {MYCALL} TU <SK>', known))).toBe(true);
  });

  it('flags unknown variables, unsendable characters, prosigns and loose braces', () => {
    const c = checkTemplate('{CALL} {FOO} HÉ! <ZZ> {BAR', known);
    expect(c.unknownVars).toEqual(['FOO']);
    expect(c.badChars).toEqual(['É', '!']);
    expect(c.badProsigns).toEqual(['ZZ']);
    expect(c.braces).toBe(true);
  });

  it('marks filled and missing variables in the preview', () => {
    const p = previewParts('{CALL} DE {MYCALL} {NAME}', { CALL: 'DL1ABC', MYCALL: 'ON3VZ', NAME: '' });
    expect(p).toEqual([
      { text: 'DL1ABC', kind: 'var' },
      { text: ' DE ', kind: 'plain' },
      { text: 'ON3VZ', kind: 'var' },
      { text: ' ', kind: 'plain' },
      { text: '{NAME}', kind: 'missing' },
    ]);
  });
});

describe('the text CQ repeat uses', () => {
  it('is the CQ chip outside contest mode, F1 in Run, and nothing in S&P', () => {
    expect(cqSlot(false, 'run', contest(), fresh())?.id).toBe('chip.cq');
    expect(cqSlot(true, 'run', contest('cqww-cw'), fresh())?.template).toBe('CQ TEST {MYCALL}');
    expect(cqSlot(true, 'sp', contest('cqww-cw'), fresh())).toBeNull();
  });
});
