import type { ThemeDefinition } from './types';
import { BUILTIN_THEMES, DEFAULT_DARK_THEME } from './presets';

const STORAGE_KEY = 'shinoho:theme';

export class ThemeManager {
  private themes = new Map<string, ThemeDefinition>();
  private activeTheme: ThemeDefinition = DEFAULT_DARK_THEME;
  private listeners = new Set<(theme: ThemeDefinition) => void>();
  private isInitialized = false;

  constructor() {
    // Register all builtin themes
    for (const theme of BUILTIN_THEMES) {
      this.themes.set(theme.id, theme);
    }
  }

  public init(initialThemeId?: string): ThemeDefinition {
    if (this.isInitialized) {
      return this.activeTheme;
    }

    let targetId = initialThemeId;
    if (!targetId && typeof localStorage !== 'undefined') {
      try {
        const saved = localStorage.getItem(STORAGE_KEY);
        if (saved) {
          targetId = saved;
        }
      } catch {
        // localStorage unavailable or restricted
      }
    }

    const themeToApply = (targetId && this.themes.get(targetId)) || DEFAULT_DARK_THEME;
    this.applyTheme(themeToApply);
    this.isInitialized = true;
    return this.activeTheme;
  }

  public registerTheme(theme: ThemeDefinition): void {
    if (!theme || !theme.id) {
      throw new Error('Theme must have a valid id');
    }
    this.themes.set(theme.id, theme);
  }

  public unregisterTheme(id: string): boolean {
    if (id === DEFAULT_DARK_THEME.id) {
      return false; // Cannot remove default theme
    }
    const existed = this.themes.delete(id);
    if (existed && this.activeTheme.id === id) {
      this.setTheme(DEFAULT_DARK_THEME.id);
    }
    return existed;
  }

  public getTheme(id: string): ThemeDefinition | undefined {
    return this.themes.get(id);
  }

  public getAllThemes(): ThemeDefinition[] {
    return Array.from(this.themes.values());
  }

  public getCurrentTheme(): ThemeDefinition {
    return this.activeTheme;
  }

  public setTheme(id: string): ThemeDefinition {
    const target = this.themes.get(id);
    if (!target) {
      throw new Error(`Theme with id "${id}" not found`);
    }

    this.applyTheme(target);
    return target;
  }

  public resetToDefault(): ThemeDefinition {
    return this.setTheme(DEFAULT_DARK_THEME.id);
  }

  public onThemeChange(listener: (theme: ThemeDefinition) => void): () => void {
    this.listeners.add(listener);
    return () => {
      this.listeners.delete(listener);
    };
  }

  private applyTheme(theme: ThemeDefinition): void {
    this.activeTheme = theme;

    // Apply variables to DOM
    if (typeof document !== 'undefined' && document.documentElement) {
      const root = document.documentElement;
      for (const [key, value] of Object.entries(theme.variables)) {
        if (value !== undefined) {
          root.style.setProperty(key, value);
        }
      }
      root.setAttribute('data-theme', theme.id);
      root.setAttribute('data-theme-dark', String(theme.isDark));
    }

    // Persist choice
    if (typeof localStorage !== 'undefined') {
      try {
        localStorage.setItem(STORAGE_KEY, theme.id);
      } catch {
        // Ignored
      }
    }

    // Notify listeners
    for (const listener of this.listeners) {
      try {
        listener(theme);
      } catch (err) {
        console.error('Error in theme change listener:', err);
      }
    }

    // Dispatch DOM event
    if (typeof window !== 'undefined' && typeof CustomEvent !== 'undefined') {
      try {
        window.dispatchEvent(
          new CustomEvent('shinoho:theme-changed', {
            detail: { theme },
          }),
        );
      } catch {
        // Ignored
      }
    }
  }
}

export const themeManager = new ThemeManager();
