import { describe, expect, it } from 'vitest';
import { expand } from './template';

describe('expand', () => {
  it('fills known variables', () => {
    expect(expand('{CALL} DE {MYCALL}', { CALL: 'DL1ABC', MYCALL: 'ON3VZ' })).toEqual({
      text: 'DL1ABC DE ON3VZ',
      missing: [],
    });
  });
  it('reports missing variables once', () => {
    const r = expand('{CALL} {CALL} {NAME}', { NAME: '' });
    expect(r.missing).toEqual(['CALL', 'NAME']);
    expect(r.text).toBe('{CALL} {CALL} {NAME}');
  });
  it('leaves text without variables alone', () => {
    expect(expand('CQ CQ', {})).toEqual({ text: 'CQ CQ', missing: [] });
  });
});
