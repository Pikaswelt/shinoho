import { describe, it, expect, beforeEach } from 'vitest';
import {
  t,
  setLocale,
  getLocale,
  DEFAULT_LOCALE,
  SUPPORTED_LOCALES,
  subscribeLocale,
} from './index';
import de from '../../i18n/de.json';
import en from '../../i18n/en.json';
import ja from '../../i18n/ja.json';

describe('i18n runtime', () => {
  beforeEach(() => {
    setLocale(DEFAULT_LOCALE);
  });

  it('defaults to German', () => {
    expect(SUPPORTED_LOCALES).toEqual(['de', 'en', 'ja']);
    expect(getLocale()).toBe('de');
    expect(t('app.title')).toBe('Shinoho');
    expect(t('app.ready')).toBe('Shinoho Browser UI bereit');
  });

  it('switches between supported languages (de, en, ja)', () => {
    setLocale('en');
    expect(getLocale()).toBe('en');
    expect(t('app.ready')).toBe('Shinoho Browser UI ready');
    expect(t('nav.reload')).toBe('Reload');

    setLocale('ja');
    expect(getLocale()).toBe('ja');
    expect(t('app.ready')).toBe('Shinoho ブラウザUI準備完了');
    expect(t('nav.reload')).toBe('再読み込み');
  });

  it('notifies subscribers on locale change', () => {
    const received: string[] = [];
    const unsubscribe = subscribeLocale((loc) => received.push(loc));

    setLocale('en');
    setLocale('ja');
    unsubscribe();
    setLocale('de');

    expect(received).toEqual(['de', 'en', 'ja']);
  });

  it('interpolates parameters correctly', () => {
    expect(t('addressbar.placeholder', { search: 'duck' })).toBe(
      'Webadresse eingeben oder suchen...',
    );
  });

  it('falls back to English or German for missing keys', () => {
    expect(t('nonexistent.key.xyz')).toBe('nonexistent.key.xyz');
  });
});

describe('translation consistency', () => {
  function getLeaves(obj: Record<string, unknown>, prefix = ''): string[] {
    const keys: string[] = [];
    for (const [key, value] of Object.entries(obj)) {
      if (key.startsWith('_')) continue;
      const fullKey = prefix ? `${prefix}.${key}` : key;
      if (typeof value === 'object' && value !== null) {
        keys.push(...getLeaves(value as Record<string, unknown>, fullKey));
      } else {
        keys.push(fullKey);
      }
    }
    return keys.sort();
  }

  it('has identical keys across de and en', () => {
    const deKeys = getLeaves(de as Record<string, unknown>);
    const enKeys = getLeaves(en as Record<string, unknown>);
    expect(deKeys).toEqual(enKeys);
  });

  it('has identical keys across de and ja', () => {
    const deKeys = getLeaves(de as Record<string, unknown>);
    const jaKeys = getLeaves(ja as Record<string, unknown>);
    expect(deKeys).toEqual(jaKeys);
  });

  it('marks ja.json with _todo_review', () => {
    expect((ja as { _todo_review?: boolean })._todo_review).toBe(true);
  });
});
