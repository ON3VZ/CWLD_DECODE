import { describe, expect, it } from 'vitest';
import { looksLikeCall, splitCalls } from './callsign';

describe('callsigns', () => {
  it('accepts common calls', () => {
    for (const c of ['ON3VZ', 'DL1ABC', 'F4XYZ', 'G3ABC', 'ON3VZ/P', 'W1AW', '9A1A', 'EA8/ON3VZ'.split('/')[1]]) {
      expect(looksLikeCall(c), c).toBe(true);
    }
  });
  it('rejects ordinary words and q-codes', () => {
    for (const w of ['CQ', 'DE', 'TNX', 'QRZ', 'RST', '599', 'NAME', 'K']) expect(looksLikeCall(w), w).toBe(false);
  });
  it('splits text and keeps whitespace', () => {
    const parts = splitCalls('CQ DE DL1ABC K');
    expect(parts.map((p) => p.text).join('')).toBe('CQ DE DL1ABC K');
    expect(parts.filter((p) => p.call).map((p) => p.text)).toEqual(['DL1ABC']);
  });
});
