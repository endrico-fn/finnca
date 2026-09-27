import type { en } from './en';

export type Locale = 'en' | 'id';
export type TranslationDict = typeof en;
export type TranslationKey = keyof TranslationDict;
