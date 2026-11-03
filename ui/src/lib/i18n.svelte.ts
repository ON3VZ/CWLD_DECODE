import en from '../i18n/en.json';
import nl from '../i18n/nl.json';
import type { Language } from './types';

const dictionaries: Record<Language, Record<string, string>> = { en, nl };

class I18n {
  lang = $state<Language>('en');

  set(lang: Language) {
    this.lang = lang;
    if (typeof document !== 'undefined') document.documentElement.lang = lang;
  }

  /** Translates a key. Falls back to English, then to the key itself. */
  t = (key: string, params?: Record<string, string | number>): string => {
    const raw = dictionaries[this.lang][key] ?? dictionaries.en[key] ?? key;
    if (!params) return raw;
    return raw.replace(/\{(\w+)\}/g, (_m, k: string) => String(params[k] ?? `{${k}}`));
  };
}

export const i18n = new I18n();
export const t = i18n.t;
