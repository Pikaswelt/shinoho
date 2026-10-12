import { describe, it, expect, beforeEach } from 'vitest';
import { createIncognitoStore, NINJA_MASCOT_QUOTES } from './incognitoStore';
import { DEFAULT_INCOGNITO_THEME, DEFAULT_INCOGNITO_CONFIG } from './incognito.types';

describe('incognitoStore', () => {
  let store: ReturnType<typeof createIncognitoStore>;

  beforeEach(() => {
    store = createIncognitoStore();
  });

  it('initializes with default privacy configuration matching core/incognito', () => {
    const state = store.getState();
    expect(state.autoEnableVpn).toBe(true);
    expect(state.diskCacheEnabled).toBe(false);
    expect(state.theme.primaryColor).toBe('#2d1b4e');
    expect(state.theme.accentColor).toBe('#9d4edd');
    expect(state.theme.mascot).toBe('ninja-shino');
  });

  it('can enable and disable incognito session', () => {
    expect(store.isIncognito).toBe(false);
    expect(store.sessionId).toBeNull();

    store.enableIncognito('custom-test-session');
    expect(store.isIncognito).toBe(true);
    expect(store.sessionId).toBe('custom-test-session');

    store.disableIncognito();
    expect(store.isIncognito).toBe(false);
    expect(store.sessionId).toBeNull();
  });

  it('toggles and sets auto-VPN state', () => {
    expect(store.autoEnableVpn).toBe(true);

    store.toggleAutoVpn();
    expect(store.autoEnableVpn).toBe(false);

    store.setAutoVpn(true);
    expect(store.autoEnableVpn).toBe(true);
  });

  it('notifies subscribers on state updates', () => {
    const states: boolean[] = [];
    const unsubscribe = store.subscribe((s) => states.push(s.isIncognito));

    store.enableIncognito();
    store.disableIncognito();
    unsubscribe();
    store.enableIncognito();

    expect(states).toEqual([false, true, false]);
  });

  it('returns valid random ninja mascot quotes', () => {
    const quote = store.getRandomNinjaQuote();
    expect(NINJA_MASCOT_QUOTES).toContain(quote);
    expect(quote.length).toBeGreaterThan(10);
  });

  it('exports valid default constants', () => {
    expect(DEFAULT_INCOGNITO_THEME.mascot).toBe('ninja-shino');
    expect(DEFAULT_INCOGNITO_CONFIG.privateBrowsing).toBe(true);
  });
});
