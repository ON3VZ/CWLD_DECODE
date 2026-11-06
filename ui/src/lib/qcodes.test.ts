import { describe, expect, it } from 'vitest';
import { Q_ENTRIES, entryMeaning, lookup, searchEntries } from './qcodes';

describe('qcodes', () => {
  it('has no duplicate codes', () => {
    const codes = Q_ENTRIES.map((e) => e.kind + e.code);
    expect(new Set(codes).size).toBe(codes.length);
  });
  it('every entry has both languages and no dashes', () => {
    for (const e of Q_ENTRIES) {
      expect(e.en.length, e.code).toBeGreaterThan(0);
      expect(e.nl.length, e.code).toBeGreaterThan(0);
      for (const s of [e.en, e.nl, e.askEn ?? '', e.askNl ?? '']) expect(/[–—]/.test(s)).toBe(false);
      if (e.kind === 'q') {
        expect(e.code).toMatch(/^Q[A-Z]{2}$/);
        expect(e.askEn && e.askNl).toBeTruthy();
      }
    }
  });
  it('prosign patterns match the Morse of their letters', () => {
    const ar = Q_ENTRIES.find((e) => e.code === '<AR>');
    expect(ar?.pattern).toBe('.-.-.');
  });
  it('searches by code start first, then by meaning', () => {
    expect(searchEntries(Q_ENTRIES, 'qr', 'en')[0].code.startsWith('QR')).toBe(true);
    expect(searchEntries(Q_ENTRIES, 'qth?', 'en')[0].code).toBe('QTH');
    expect(searchEntries(Q_ENTRIES, 'locatie', 'nl').map((e) => e.code)).toContain('QTH');
    expect(searchEntries(Q_ENTRIES, 'frequency', 'en').map((e) => e.code)).toContain('QSY');
    expect(searchEntries(Q_ENTRIES, 'zzzzqq', 'en')).toEqual([]);
    expect(searchEntries(Q_ENTRIES, '  ', 'en').length).toBe(Q_ENTRIES.length);
  });
  it('gives the question form and language', () => {
    const e = lookup('QTH')!;
    expect(entryMeaning(e, 'en', true)).toBe('What is your location?');
    expect(entryMeaning(e, 'nl')).toContain('locatie');
    expect(lookup('ar')?.code).toBe('<AR>');
    expect(lookup('HW?')?.code).toBe('HW');
    expect(lookup('nonsense')).toBeUndefined();
  });
});

describe('codes used in a conversation', () => {
  it('lists each known code once, in order, with own words and without plain numbers', async () => {
    const { usedIn } = await import('./qcodes');
    const out = usedIn('CQ DE ON3VZ TNX UR 599 599 QTH? TNX 73 TNX2U', [{ code: 'TNX2U', meaning: 'Thanks to you' }]);
    expect(out.map((u) => u.code)).toEqual(['CQ', 'DE', 'TNX', 'UR', 'QTH', '73', 'TNX2U']);
    expect(out[6].own).toBe('Thanks to you');
  });
});
