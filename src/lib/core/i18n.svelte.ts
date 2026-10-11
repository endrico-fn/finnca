import { getPref, setPref } from '$lib/core/state/prefs';
import { en } from './i18n/en';
import { id } from './i18n/id';
import type { Locale, TranslationDict, TranslationKey } from './i18n/types';

export type { Locale, TranslationDict, TranslationKey };

const dict: Record<Locale, TranslationDict> = {
  en,
  id,
};

class I18nStore {
  locale = $state<Locale>('en');

  constructor() {
    const saved = getPref('finnca_locale', 'en');
    if (saved === 'en' || saved === 'id') {
      this.locale = saved;
    }
  }

  setLocale(l: Locale) {
    this.locale = l;
    setPref('finnca_locale', l);
  }

  get t(): TranslationDict {
    return dict[this.locale];
  }
}

export const i18n = new I18nStore();
