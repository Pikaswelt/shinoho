import { describe, it, expect, beforeEach } from 'vitest';
import { mount, unmount, tick } from 'svelte';
import DownloadsManager from './DownloadsManager.svelte';
import { downloadStore } from './downloadStore';
import { setLocale, DEFAULT_LOCALE } from '../../i18n';

describe('DownloadsManager component', () => {
  beforeEach(() => {
    setLocale(DEFAULT_LOCALE);
    downloadStore.resetForTests();
  });

  it('renders nothing while the manager is closed', async () => {
    const target = document.createElement('div');
    document.body.appendChild(target);

    const component = mount(DownloadsManager, { target });
    await tick();

    expect(target.querySelector('[data-testid="downloads-manager"]')).toBeNull();

    unmount(component);
    document.body.removeChild(target);
  });

  it('opens the modal with an empty state and closes it again', async () => {
    const target = document.createElement('div');
    document.body.appendChild(target);

    const component = mount(DownloadsManager, { target });
    await tick();

    downloadStore.openManager();
    await tick();
    await new Promise((resolve) => setTimeout(resolve, 50));
    await tick();

    expect(target.querySelector('[data-testid="downloads-manager"]')).not.toBeNull();
    // Browser-Umgebung ohne Tauri: leere Liste.
    expect(target.querySelector('[data-testid="downloads-empty"]')).not.toBeNull();

    const closeBtn = target.querySelector<HTMLButtonElement>('[data-testid="downloads-close"]');
    closeBtn?.click();
    await tick();

    expect(target.querySelector('[data-testid="downloads-manager"]')).toBeNull();
    expect(downloadStore.isManagerOpen).toBe(false);

    unmount(component);
    document.body.removeChild(target);
  });
});
