import { describe, expect, it } from 'vitest';
import { helpUrl, SITE_URL } from './help';

describe('helpUrl', () => {
  it('points to the English help page at the root', () => {
    expect(helpUrl('en')).toBe('https://on3vz.github.io/CWLD_DECODE/help.html');
  });
  it('points to the Dutch help page under /nl/', () => {
    expect(helpUrl('nl')).toBe('https://on3vz.github.io/CWLD_DECODE/nl/help.html');
  });
  it('stays inside the address the Tauri permission allows', () => {
    expect(helpUrl('en').startsWith(SITE_URL)).toBe(true);
    expect(helpUrl('nl').startsWith(SITE_URL)).toBe(true);
  });
});
