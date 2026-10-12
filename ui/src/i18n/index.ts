import de from '../../i18n/de.json';
import en from '../../i18n/en.json';
import ja from '../../i18n/ja.json';

export type SupportedLocale = 'de' | 'en' | 'ja';

export const SUPPORTED_LOCALES: readonly SupportedLocale[] = ['de', 'en', 'ja'] as const;

export const DEFAULT_LOCALE: SupportedLocale = 'de';

type TranslationTree = {
  [key: string]: string | boolean | TranslationTree;
};

const translations: Record<SupportedLocale, TranslationTree> = {
  de: de as TranslationTree,
  en: en as TranslationTree,
  ja: ja as TranslationTree,
};

type Subscriber = (locale: SupportedLocale) => void;
const subscribers = new Set<Subscriber>();

let currentLocale: SupportedLocale = DEFAULT_LOCALE;

function getNestedValue(obj: TranslationTree, path: string[]): string | undefined {
  let current: string | boolean | TranslationTree | undefined = obj;
  for (const part of path) {
    if (typeof current !== 'object' || current === null || !(part in current)) {
      return undefined;
    }
    current = (current as TranslationTree)[part];
  }
  return typeof current === 'string' ? current : undefined;
}

export function getLocale(): SupportedLocale {
  return currentLocale;
}

export function setLocale(locale: SupportedLocale): void {
  if (SUPPORTED_LOCALES.includes(locale)) {
    currentLocale = locale;
    subscribers.forEach((fn) => fn(currentLocale));
  }
}

export function subscribeLocale(callback: Subscriber): () => void {
  subscribers.add(callback);
  callback(currentLocale);
  return () => {
    subscribers.delete(callback);
  };
}

export function t(
  key: string,
  params?: Record<string, string | number>,
  locale?: SupportedLocale,
): string {
  const targetLocale = locale ?? currentLocale;
  const path = key.split('.');

  let message = getNestedValue(translations[targetLocale], path);

  if (message === undefined && targetLocale !== 'en') {
    message = getNestedValue(translations.en, path);
  }

  if (message === undefined && targetLocale !== 'de') {
    message = getNestedValue(translations.de, path);
  }

  if (message === undefined) {
    return key;
  }

  if (params) {
    return Object.entries(params).reduce((acc, [k, v]) => {
      return acc.replaceAll(`{${k}}`, String(v));
    }, message);
  }

  return message;
}
