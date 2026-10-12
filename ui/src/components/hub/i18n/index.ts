import {
  t as baseTranslate,
  getLocale,
  setLocale,
  type SupportedLocale,
} from '../../../i18n/index';

export type HubLocale = SupportedLocale;

export function setHubLocale(locale: HubLocale): void {
  setLocale(locale);
}

export function getHubLocale(): HubLocale {
  return getLocale();
}

export function tHub(
  key: string,
  params?: Record<string, string | number>,
  locale?: HubLocale,
): string {
  return baseTranslate(key, params, locale);
}
