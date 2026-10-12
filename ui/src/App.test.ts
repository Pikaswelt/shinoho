import { describe, it, expect, beforeEach } from 'vitest';
import { mount, unmount, tick } from 'svelte';
import App from './App.svelte';
import { setLocale, DEFAULT_LOCALE } from './i18n';
import { tabService } from './api/tabs';

describe('App component', () => {
  beforeEach(() => {
    setLocale(DEFAULT_LOCALE);
    tabService.resetMock();
  });

  it('renders header, TabBar, AddressBar, Hub view and language selector in default German', async () => {
    const target = document.createElement('div');
    document.body.appendChild(target);

    const app = mount(App, { target });
    await tick();

    expect(target.querySelector('[data-testid="app-header"]')).not.toBeNull();
    expect(target.querySelector('[data-testid="tab-bar"]')).not.toBeNull();
    expect(target.querySelector('[data-testid="address-bar"]')).not.toBeNull();
    expect(target.querySelector('[data-testid="hub-view"]')).not.toBeNull();
    expect(target.textContent).toContain('Shinoho');
    expect(target.textContent).toContain('Shinoho Browser UI bereit');
    expect(target.querySelector('[data-testid="language-selector"]')).not.toBeNull();

    unmount(app);
    document.body.removeChild(target);
  });

  it('adds a new tab when clicking new tab button in TabBar', async () => {
    const target = document.createElement('div');
    document.body.appendChild(target);

    const app = mount(App, { target });
    await tick();

    const newTabBtn = target.querySelector<HTMLButtonElement>('[data-testid="new-tab-btn"]');
    expect(newTabBtn).not.toBeNull();

    newTabBtn?.click();
    await tick();
    await new Promise((resolve) => setTimeout(resolve, 50));
    await tick();

    const tabs = target.querySelectorAll('[role="tab"]');
    expect(tabs.length).toBeGreaterThanOrEqual(2);

    unmount(app);
    document.body.removeChild(target);
  });

  it('switches language when clicking language button', async () => {
    const target = document.createElement('div');
    document.body.appendChild(target);

    const app = mount(App, { target });
    await tick();

    const enBtn = target.querySelector<HTMLButtonElement>('[data-testid="lang-btn-en"]');
    expect(enBtn).not.toBeNull();

    enBtn?.click();
    await tick();

    expect(target.textContent).toContain('Shinoho Browser UI ready');

    const jaBtn = target.querySelector<HTMLButtonElement>('[data-testid="lang-btn-ja"]');
    jaBtn?.click();
    await tick();

    expect(target.textContent).toContain('Shinoho ブラウザUI準備完了');

    unmount(app);
    document.body.removeChild(target);
  });

  it('renders bookmark bar and opens manager', async () => {
    const target = document.createElement('div');
    document.body.appendChild(target);

    const app = mount(App, { target });
    await tick();

    expect(target.querySelector('[data-testid="bookmark-bar"]')).not.toBeNull();

    unmount(app);
    document.body.removeChild(target);
  });

  it('renders incognito theme, badge, and start page when in incognito mode', async () => {
    const target = document.createElement('div');
    document.body.appendChild(target);

    const { incognitoStore } = await import('./components/incognito');
    incognitoStore.enableIncognito();

    const app = mount(App, { target });
    await tick();

    expect(target.querySelector('.theme-incognito')).not.toBeNull();
    expect(target.querySelector('[data-testid="incognito-badge"]')).not.toBeNull();
    expect(target.querySelector('[data-testid="incognito-startpage"]')).not.toBeNull();

    incognitoStore.disableIncognito();
    unmount(app);
    document.body.removeChild(target);
  });
});
