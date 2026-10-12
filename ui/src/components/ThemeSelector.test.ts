import { describe, it, expect, beforeEach } from 'vitest';
import { mount, unmount, tick } from 'svelte';
import ThemeSelector from './ThemeSelector.svelte';
import { themeManager } from '../themes';

describe('ThemeSelector component', () => {
  beforeEach(() => {
    localStorage.clear();
    themeManager.resetToDefault();
  });

  it('renders trigger button with active theme', () => {
    const target = document.createElement('div');
    document.body.appendChild(target);

    const component = mount(ThemeSelector, { target });
    const triggerBtn = target.querySelector<HTMLButtonElement>('[data-testid="theme-trigger-btn"]');

    expect(triggerBtn).not.toBeNull();
    expect(triggerBtn?.getAttribute('aria-expanded')).toBe('false');

    unmount(component);
    document.body.removeChild(target);
  });

  it('opens dropdown and allows selecting another theme', async () => {
    const target = document.createElement('div');
    document.body.appendChild(target);

    const component = mount(ThemeSelector, { target });
    const triggerBtn = target.querySelector<HTMLButtonElement>('[data-testid="theme-trigger-btn"]');

    // Open dropdown
    triggerBtn?.dispatchEvent(new Event('click', { bubbles: true }));
    await tick();

    const dropdown = target.querySelector('[data-testid="theme-dropdown"]');
    expect(dropdown).not.toBeNull();

    // Select Sakura Neon theme
    const sakuraOption = target.querySelector<HTMLButtonElement>(
      '[data-testid="theme-option-sakura-neon"]',
    );
    expect(sakuraOption).not.toBeNull();

    sakuraOption?.dispatchEvent(new Event('click', { bubbles: true }));
    await tick();

    expect(themeManager.getCurrentTheme().id).toBe('sakura-neon');
    expect(document.documentElement.getAttribute('data-theme')).toBe('sakura-neon');

    unmount(component);
    document.body.removeChild(target);
  });
});
