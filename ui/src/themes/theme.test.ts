import { describe, it, expect, beforeEach, vi } from 'vitest';
import { themeManager } from './themeStore';
import { BUILTIN_THEMES, DEFAULT_DARK_THEME } from './presets';
import type { ThemeDefinition } from './types';

describe('Theme Engine', () => {
  beforeEach(() => {
    localStorage.clear();
    themeManager.resetToDefault();
  });

  it('registers all builtin themes on start', () => {
    const all = themeManager.getAllThemes();
    expect(all.length).toBe(BUILTIN_THEMES.length);

    const ids = all.map((t) => t.id);
    expect(ids).toContain('default-dark');
    expect(ids).toContain('sakura-neon');
    expect(ids).toContain('midnight-cyber');
    expect(ids).toContain('ninja-stealth');
    expect(ids).toContain('light-minimal');
  });

  it('initializes with default dark theme when no saved theme exists', () => {
    const active = themeManager.getCurrentTheme();
    expect(active.id).toBe('default-dark');
    expect(active.isDark).toBe(true);
    expect(document.documentElement.getAttribute('data-theme')).toBe('default-dark');
    expect(document.documentElement.getAttribute('data-theme-dark')).toBe('true');
  });

  it('applies theme variables to document root when switching themes', () => {
    const sakura = themeManager.setTheme('sakura-neon');
    expect(sakura.id).toBe('sakura-neon');
    expect(themeManager.getCurrentTheme().id).toBe('sakura-neon');

    expect(document.documentElement.getAttribute('data-theme')).toBe('sakura-neon');
    expect(document.documentElement.style.getPropertyValue('--color-accent')).toBe('#ff2a85');
    expect(document.documentElement.style.getPropertyValue('--color-bg-base')).toBe('#13091f');
  });

  it('correctly sets data-theme-dark to false for light theme', () => {
    const light = themeManager.setTheme('light-minimal');
    expect(light.isDark).toBe(false);
    expect(document.documentElement.getAttribute('data-theme')).toBe('light-minimal');
    expect(document.documentElement.getAttribute('data-theme-dark')).toBe('false');
    expect(document.documentElement.style.getPropertyValue('--color-bg-base')).toBe('#f3f4f6');
  });

  it('persists selected theme to localStorage', () => {
    themeManager.setTheme('midnight-cyber');
    expect(localStorage.getItem('shinoho:theme')).toBe('midnight-cyber');
  });

  it('notifies subscribers on theme change', () => {
    const listener = vi.fn();
    const unsubscribe = themeManager.onThemeChange(listener);

    themeManager.setTheme('ninja-stealth');
    expect(listener).toHaveBeenCalledTimes(1);
    expect(listener).toHaveBeenCalledWith(expect.objectContaining({ id: 'ninja-stealth' }));

    unsubscribe();
    themeManager.setTheme('default-dark');
    expect(listener).toHaveBeenCalledTimes(1);
  });

  it('allows registering and switching to a custom theme', () => {
    const customTheme: ThemeDefinition = {
      id: 'custom-anime-gold',
      name: 'Custom Gold',
      isDark: true,
      author: 'CommunityModder',
      variables: {
        ...DEFAULT_DARK_THEME.variables,
        '--color-accent': '#ffd700',
        '--color-bg-base': '#111000',
      },
    };

    themeManager.registerTheme(customTheme);
    expect(themeManager.getTheme('custom-anime-gold')).toBeDefined();

    themeManager.setTheme('custom-anime-gold');
    expect(themeManager.getCurrentTheme().id).toBe('custom-anime-gold');
    expect(document.documentElement.style.getPropertyValue('--color-accent')).toBe('#ffd700');

    // Clean up
    themeManager.unregisterTheme('custom-anime-gold');
    expect(themeManager.getTheme('custom-anime-gold')).toBeUndefined();
    expect(themeManager.getCurrentTheme().id).toBe('default-dark');
  });

  it('throws error when setting unknown theme ID', () => {
    expect(() => themeManager.setTheme('non-existent-theme')).toThrowError();
  });

  it('cannot unregister default theme', () => {
    const result = themeManager.unregisterTheme('default-dark');
    expect(result).toBe(false);
    expect(themeManager.getTheme('default-dark')).toBeDefined();
  });

  it('exposes defaultWallpaper and defaultSoundPack for builtin themes', () => {
    const sakura = themeManager.getTheme('sakura-neon');
    expect(sakura?.defaultWallpaper).toBe('sakura');
    expect(sakura?.defaultSoundPack).toBe('kawaii');

    const cyber = themeManager.getTheme('midnight-cyber');
    expect(cyber?.defaultWallpaper).toBe('cyberpunk');
    expect(cyber?.defaultSoundPack).toBe('cyberpunk');

    themeManager.setTheme('sakura-neon');
    expect(themeManager.getActiveWallpaper()).toBe('sakura');
    expect(themeManager.getActiveSoundPack()).toBe('kawaii');
  });
});
