import { describe, expect, it } from 'vitest';
import en from '../i18n/en.json';
import nl from '../i18n/nl.json';

describe('translations', () => {
  it('English and Dutch have exactly the same keys', () => {
    expect(Object.keys(nl).sort()).toEqual(Object.keys(en).sort());
  });
  it('no empty texts', () => {
    for (const d of [en, nl]) for (const [k, v] of Object.entries(d)) expect(v.trim(), k).not.toBe('');
  });
  it('placeholders match between languages', () => {
    const ph = (s: string) => (s.match(/\{\w+\}/g) ?? []).sort().join(',');
    for (const k of Object.keys(en) as Array<keyof typeof en>) expect(ph(nl[k]), k).toBe(ph(en[k]));
  });
  it('no em or en dashes in any text', () => {
    for (const d of [en, nl]) for (const v of Object.values(d)) expect(/[\u2013\u2014]/.test(v)).toBe(false);
  });
});
