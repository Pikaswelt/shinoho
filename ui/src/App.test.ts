import { describe, it, expect } from 'vitest';
import { mount, unmount } from 'svelte';
import App from './App.svelte';

describe('App component', () => {
  it('renders header and welcome card', () => {
    const target = document.createElement('div');
    document.body.appendChild(target);

    const app = mount(App, { target });
    expect(target.querySelector('[data-testid="app-header"]')).not.toBeNull();
    expect(target.textContent).toContain('Shinoho');
    expect(target.textContent).toContain('Shinoho Browser UI bereit');

    unmount(app);
    document.body.removeChild(target);
  });
});
