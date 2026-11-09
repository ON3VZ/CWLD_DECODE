import { describe, expect, it } from 'vitest';
import {
  activeProfile,
  activeSent,
  allProfiles,
  BELGIAN_PREFIXES,
  BUILTIN_PROFILES,
  copyProfile,
  cutNumber,
  exportText,
  isBuiltin,
  mergeImport,
  ownVars,
  parseImport,
  plain,
  recvApplies,
  usesSerial,
  validVarName,
} from './contest';
import { expand } from './template';
import type { ContestSettings } from './types';

const settings = (over: Partial<ContestSettings> = {}): ContestSettings => ({
  active: '',
  sent_index: 0,
  session: 1,
  profiles: [],
  vars: [
    { name: 'SEC', value: 'WLD' },
    { name: 'ZONE', value: '14' },
    { name: 'ITU', value: '27' },
    { name: 'PWR', value: '20' },
  ],
  ...over,
});

describe('cut numbers', () => {
  it('0 becomes T and 9 becomes N, other digits stay', () => {
    expect(cutNumber('001')).toBe('TT1');
    expect(cutNumber('019')).toBe('T1N');
    expect(cutNumber('250')).toBe('25T');
    expect(cutNumber('999')).toBe('NNN');
  });
});

describe('own variable names', () => {
  it('accepts capitals, digits and underscore, starting with a letter', () => {
    expect(validVarName('SEC')).toBe(true);
    expect(validVarName('ZONE_2')).toBe(true);
    expect(validVarName('2ZONE')).toBe(false);
    expect(validVarName('sec')).toBe(false);
    expect(validVarName('')).toBe(false);
    expect(validVarName('A'.repeat(17))).toBe(false);
  });

  it('refuses the names CWLD fills in itself', () => {
    for (const n of ['CALL', 'MYCALL', 'NAME', 'QTH', 'RST', 'NR', 'MYNAME', 'MYQTH']) expect(validVarName(n)).toBe(false);
  });

  it('own variables never override reserved ones and values are upper case', () => {
    const v = ownVars(settings({ vars: [{ name: 'CALL', value: 'XX' }, { name: 'SEC', value: ' wld ' }, { name: 'bad name', value: 'x' }] }));
    expect(v).toEqual({ SEC: 'WLD' });
  });
});

describe('built-in profiles', () => {
  const vars = { CALL: 'PA3ABC', MYCALL: 'ON3VZ', RST: '5NN', NR: '001', ...ownVars(settings()) };

  it('there is a blank template and the nine profiles from the design', () => {
    const ids = BUILTIN_PROFILES.map((p) => p.id);
    expect(ids).toEqual(['uba-dx-cw', 'uba-on-cw', 'uba-spring-cw', 'uba-winter-cw', 'cqww-cw', 'cqwpx-cw', 'arrl-dx-cw', 'iaru-hf', 'wae-cw', 'blank']);
  });

  it('every sent template can be filled with the default variables', () => {
    for (const p of BUILTIN_PROFILES) {
      expect(p.sent.length, p.id).toBeGreaterThan(0);
      for (const s of p.sent) expect(expand(s.template, vars).missing, `${p.id} ${s.template}`).toEqual([]);
    }
  });

  it('UBA exchange from Belgium is call, report, number and section', () => {
    const p = BUILTIN_PROFILES.find((x) => x.id === 'uba-dx-cw')!;
    expect(expand(p.sent[0].template, vars).text).toBe('PA3ABC 5NN 001 WLD');
    expect(expand(p.sent[1].template, vars).text).toBe('PA3ABC 5NN 001');
    expect(p.calls).toEqual(['TEST UBA']);
  });

  it('the other contests send what their rules say', () => {
    const t = (id: string) => expand(BUILTIN_PROFILES.find((x) => x.id === id)!.sent[0].template, vars).text;
    expect(t('cqww-cw')).toBe('PA3ABC 5NN 14');
    expect(t('cqwpx-cw')).toBe('PA3ABC 5NN 001');
    expect(t('arrl-dx-cw')).toBe('PA3ABC 5NN 20');
    expect(t('iaru-hf')).toBe('PA3ABC 5NN 27');
    expect(t('wae-cw')).toBe('PA3ABC 5NN 001');
    expect(t('uba-winter-cw')).toBe('PA3ABC 5NN WLD');
  });

  it('ids are unique and built-in ones are recognised', () => {
    expect(new Set(BUILTIN_PROFILES.map((p) => p.id)).size).toBe(BUILTIN_PROFILES.length);
    expect(isBuiltin('cqww-cw')).toBe(true);
    expect(isBuiltin('own-1')).toBe(false);
  });
});

describe('active profile, template and serial number', () => {
  it('no profile: nothing active, serial assumed', () => {
    const c = settings();
    expect(activeProfile(c)).toBeNull();
    expect(activeSent(c)).toBeNull();
    expect(usesSerial(c)).toBe(true);
  });

  it('the chosen sent template is used, and an index out of range falls back to the first', () => {
    expect(activeSent(settings({ active: 'uba-dx-cw', sent_index: 1 }))).toBe('{CALL} {RST} {NR}');
    expect(activeSent(settings({ active: 'uba-dx-cw', sent_index: 9 }))).toBe('{CALL} {RST} {NR} {SEC}');
  });

  it('serial number only when the active template has {NR}', () => {
    expect(usesSerial(settings({ active: 'cqww-cw' }))).toBe(false);
    expect(usesSerial(settings({ active: 'cqwpx-cw' }))).toBe(true);
    expect(usesSerial(settings({ active: 'uba-winter-cw', sent_index: 0 }))).toBe(false);
    expect(usesSerial(settings({ active: 'uba-winter-cw', sent_index: 1 }))).toBe(true);
  });

  it('an unknown active id means no profile', () => {
    expect(activeProfile(settings({ active: 'gone' }))).toBeNull();
  });
});

describe('extra received field', () => {
  it('applies to Belgian prefixes only when the profile says so', () => {
    const f = BUILTIN_PROFILES.find((x) => x.id === 'uba-dx-cw')!.recv;
    expect(BELGIAN_PREFIXES).toContain('OO');
    expect(recvApplies(f, 'ON4ABC')).toBe(true);
    expect(recvApplies(f, 'oo7x')).toBe(true);
    expect(recvApplies(f, 'DL1ABC')).toBe(false);
    expect(recvApplies(null, 'ON4ABC')).toBe(false);
  });

  it('applies to every call when no prefixes are given', () => {
    const f = BUILTIN_PROFILES.find((x) => x.id === 'cqww-cw')!.recv;
    expect(recvApplies(f, 'DL1ABC')).toBe(true);
  });
});

describe('copy, export and import', () => {
  const tr = (k: string) => `T(${k})`;

  it('a copy gets a new id, a readable note and readable labels', () => {
    const src = BUILTIN_PROFILES.find((x) => x.id === 'uba-dx-cw')!;
    const c = copyProfile(src, 'Mine', allProfiles(settings()), tr);
    expect(c.id).toBe('own-11');
    expect(c.name).toBe('Mine');
    expect(c.note).toBe('T(contest.note.uba)');
    expect(c.sent[0].label).toBe('T(contest.sent.be)');
    expect(c.sent[0].template).toBe(src.sent[0].template);
    expect(isBuiltin(c.id)).toBe(false);
  });

  it('plain resolves i18n keys and leaves other text', () => {
    expect(plain('i18n:a.b', tr)).toBe('T(a.b)');
    expect(plain('Hello', tr)).toBe('Hello');
  });

  it('own profiles and variables survive an export and an import', () => {
    const src = BUILTIN_PROFILES.find((x) => x.id === 'uba-dx-cw')!;
    const mine = copyProfile(src, 'Club', allProfiles(settings()), tr);
    const c = settings({ profiles: [mine] });
    const text = exportText(c);
    const imp = parseImport(text);
    expect(imp.profiles).toHaveLength(1);
    expect(imp.profiles[0].name).toBe('Club');
    expect(imp.vars.map((v) => v.name)).toEqual(['SEC', 'ZONE', 'ITU', 'PWR']);
    expect(imp.skipped).toBe(0);
  });

  it('one profile can be exported alone, also a built-in one, without variables', () => {
    const text = exportText(settings(), 'cqww-cw');
    const imp = parseImport(text);
    expect(imp.profiles.map((p) => p.name)).toEqual(['CQ WW DX Contest CW']);
    expect(imp.vars).toEqual([]);
  });

  it('a file that is not an export is refused with a code', () => {
    expect(() => parseImport('nonsense')).toThrow('not_json');
    expect(() => parseImport('{"a":1}')).toThrow('not_export');
  });

  it('broken profiles and invalid variables are skipped, not guessed', () => {
    const text = JSON.stringify({
      cwld_contest: 1,
      profiles: [{ name: '', sent: [{ label: 'a', template: 'X' }] }, { name: 'NoSent', sent: [] }, { name: 'Good', sent: [{ label: 'a', template: '{call} {rst}' }] }],
      vars: [{ name: 'CALL', value: 'x' }, { name: 'AGE', value: '55' }],
    });
    const imp = parseImport(text);
    expect(imp.profiles.map((p) => p.name)).toEqual(['Good']);
    expect(imp.profiles[0].sent[0].template).toBe('{CALL} {RST}');
    expect(imp.vars).toEqual([{ name: 'AGE', value: '55' }]);
    expect(imp.skipped).toBe(3);
  });

  it('import adds a copy for the same name, or replaces when asked, and merges variables', () => {
    const src = BUILTIN_PROFILES.find((x) => x.id === 'cqwpx-cw')!;
    const mine = copyProfile(src, 'Club', allProfiles(settings()), tr);
    const c = settings({ profiles: [mine] });
    const imp = parseImport(exportText(c));
    imp.vars = [{ name: 'SEC', value: 'ANR' }, { name: 'AGE', value: '55' }];
    const added = mergeImport(c, imp, false);
    expect(added.profiles.map((p) => p.name)).toEqual(['Club', 'Club (2)']);
    expect(new Set(added.profiles.map((p) => p.id)).size).toBe(2);
    expect(added.vars.find((v) => v.name === 'SEC')?.value).toBe('ANR');
    expect(added.vars.find((v) => v.name === 'AGE')?.value).toBe('55');
    const replaced = mergeImport(c, imp, true);
    expect(replaced.profiles).toHaveLength(1);
    expect(replaced.profiles[0].id).toBe(mine.id);
  });

  it('an imported name equal to a built-in name does not collide', () => {
    const text = JSON.stringify({ cwld_contest: 1, profiles: [{ name: 'CQ WW DX Contest CW', sent: [{ label: 'a', template: '{CALL}' }] }], vars: [] });
    const merged = mergeImport(settings(), parseImport(text), false);
    expect(merged.profiles[0].name).toBe('CQ WW DX Contest CW (2)');
  });
});
