import { describe, it, expect, beforeEach } from 'vitest';
import { mount, unmount, tick } from 'svelte';
import App from './App.svelte';
import { setLocale, DEFAULT_LOCALE } from './i18n';

describe('App component', () => {
  beforeEach(() => {
    setLocale(DEFAULT_LOCALE);
  });

  it('renders header, welcome card and language selector in default German', () => {
    const target = document.createElement('div');
    document.body.appendChild(target);

    const app = mount(App, { target });
    expect(target.querySelector('[data-testid="app-header"]')).not.toBeNull();
    expect(target.textContent).toContain('Shinoho');
    expect(target.textContent).toContain('Shinoho Browser UI bereit');
    expect(target.querySelector('[data-testid="language-selector"]')).not.toBeNull();

    unmount(app);
    document.body.removeChild(target);
  });

  it('switches language when clicking language button', async () => {
    const target = document.createElement('div');
    document.body.appendChild(target);

    const app = mount(App, { target });
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
});
