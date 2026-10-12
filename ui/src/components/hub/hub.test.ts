import { describe, it, expect, beforeEach } from 'vitest';
import { hubStore } from './hubStore';
import { tHub, setHubLocale, getHubLocale } from './i18n';

describe('Shinoho Hub Store', () => {
  beforeEach(() => {
    hubStore.resetToDefaults();
  });

  it('initializes with default state', () => {
    const state = hubStore.getState();
    expect(state.wallpaper.activeId).toBe('sakura');
    expect(state.wallpaper.particlesEnabled).toBe(true);
    expect(state.speedDial.length).toBeGreaterThan(0);
    expect(state.selectedSearchEngine).toBe('duckduckgo');
    expect(state.widgets.animeCountdown).toBe(true);
    expect(state.widgets.freeGames).toBe(true);
  });

  it('adds, updates, and deletes speed dial items', () => {
    const newItem = hubStore.addSpeedDial({
      title: 'AnimeBytes',
      url: 'https://animebytes.tv',
      category: 'anime',
      pinned: false,
    });

    expect(newItem.id).toBeDefined();
    expect(newItem.clicks).toBe(0);

    let state = hubStore.getState();
    const found = state.speedDial.find((s) => s.id === newItem.id);
    expect(found).toBeDefined();
    expect(found?.title).toBe('AnimeBytes');

    // Update
    hubStore.updateSpeedDial(newItem.id, { title: 'AB' });
    state = hubStore.getState();
    expect(state.speedDial.find((s) => s.id === newItem.id)?.title).toBe('AB');

    // Click increment
    hubStore.recordSpeedDialClick(newItem.id);
    state = hubStore.getState();
    expect(state.speedDial.find((s) => s.id === newItem.id)?.clicks).toBe(1);

    // Toggle Pin
    hubStore.togglePin(newItem.id);
    state = hubStore.getState();
    expect(state.speedDial.find((s) => s.id === newItem.id)?.pinned).toBe(true);

    // Delete
    const deleted = hubStore.removeSpeedDial(newItem.id);
    expect(deleted).toBe(true);
    state = hubStore.getState();
    expect(state.speedDial.find((s) => s.id === newItem.id)).toBeUndefined();
  });

  it('updates wallpaper settings', () => {
    hubStore.setActiveWallpaper('cyberpunk');
    let state = hubStore.getState();
    expect(state.wallpaper.activeId).toBe('cyberpunk');

    hubStore.updateWallpaperSettings({
      blur: 10,
      brightness: 0.8,
      particlesEnabled: false,
    });
    state = hubStore.getState();
    expect(state.wallpaper.blur).toBe(10);
    expect(state.wallpaper.brightness).toBe(0.8);
    expect(state.wallpaper.particlesEnabled).toBe(false);
  });

  it('builds search URLs correctly for all engines', () => {
    hubStore.setSelectedSearchEngine('duckduckgo');
    expect(hubStore.buildSearchUrl('anime')).toBe('https://duckduckgo.com/?q=anime');

    hubStore.setSelectedSearchEngine('google');
    expect(hubStore.buildSearchUrl('shinoho browser')).toBe(
      'https://www.google.com/search?q=shinoho%20browser',
    );

    hubStore.setSelectedSearchEngine('startpage');
    expect(hubStore.buildSearchUrl('test')).toBe('https://www.startpage.com/sp/search?query=test');
  });

  it('toggles widgets properly', () => {
    expect(hubStore.getState().widgets.animeCountdown).toBe(true);
    hubStore.toggleWidget('animeCountdown');
    expect(hubStore.getState().widgets.animeCountdown).toBe(false);
    hubStore.toggleWidget('animeCountdown');
    expect(hubStore.getState().widgets.animeCountdown).toBe(true);
  });
});

describe('Shinoho Hub i18n', () => {
  it('translates keys across German, English, and Japanese', () => {
    setHubLocale('de');
    expect(getHubLocale()).toBe('de');
    expect(tHub('hub.title')).toBe('Shinoho Hub');
    expect(tHub('hub.speedDial.title')).toBe('Schnellwahl');

    setHubLocale('en');
    expect(getHubLocale()).toBe('en');
    expect(tHub('hub.speedDial.title')).toBe('Speed Dial');

    setHubLocale('ja');
    expect(getHubLocale()).toBe('ja');
    expect(tHub('hub.speedDial.title')).toBe('スピードダイヤル');
  });

  it('interpolates parameters correctly', () => {
    setHubLocale('de');
    expect(tHub('hub.search.searchWith', { engine: 'DuckDuckGo' })).toBe('Suchen mit DuckDuckGo');
    expect(tHub('hub.speedDial.clicks', { count: 42 })).toBe('42 Klicks');

    setHubLocale('en');
    expect(tHub('hub.search.searchWith', { engine: 'Google' })).toBe('Search with Google');
  });

  it('gracefully falls back on missing keys', () => {
    expect(tHub('non.existent.key')).toBe('non.existent.key');
  });
});
