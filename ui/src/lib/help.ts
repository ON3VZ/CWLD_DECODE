import type { Language } from './types';

/** The help page of the website. English at the root, Dutch under /nl/. */
export const SITE_URL = 'https://on3vz.github.io/CWLD_DECODE/';

export function helpUrl(lang: Language): string {
  return `${SITE_URL}${lang === 'nl' ? 'nl/' : ''}help.html`;
}
