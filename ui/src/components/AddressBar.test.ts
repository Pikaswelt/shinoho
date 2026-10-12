import { describe, it, expect, vi } from 'vitest';
import { mount, unmount } from 'svelte';
import AddressBar from './AddressBar.svelte';

describe('AddressBar component', () => {
  it('renders navigation buttons with proper disabled states', () => {
    const target = document.createElement('div');
    document.body.appendChild(target);

    const component = mount(AddressBar, {
      target,
      props: {
        url: 'https://github.com',
        canGoBack: false,
        canGoForward: true,
        isLoading: false,
        onNavigate: vi.fn(),
        onBack: vi.fn(),
        onForward: vi.fn(),
        onReload: vi.fn(),
        onHome: vi.fn(),
      },
    });

    const backBtn = target.querySelector<HTMLButtonElement>('[data-testid="nav-back"]');
    const forwardBtn = target.querySelector<HTMLButtonElement>('[data-testid="nav-forward"]');
    const input = target.querySelector<HTMLInputElement>('[data-testid="address-input"]');

    expect(backBtn?.disabled).toBe(true);
    expect(forwardBtn?.disabled).toBe(false);
    expect(input?.value).toBe('https://github.com');

    unmount(component);
    document.body.removeChild(target);
  });

  it('triggers navigation callback with normalized URL on submit', () => {
    const target = document.createElement('div');
    document.body.appendChild(target);

    const onNavigate = vi.fn();
    const component = mount(AddressBar, {
      target,
      props: {
        url: 'shinoho://hub',
        canGoBack: true,
        canGoForward: false,
        isLoading: false,
        onNavigate,
        onBack: vi.fn(),
        onForward: vi.fn(),
        onReload: vi.fn(),
        onHome: vi.fn(),
      },
    });

    const input = target.querySelector<HTMLInputElement>('[data-testid="address-input"]');
    const form = target.querySelector<HTMLFormElement>('form');

    if (input) {
      input.value = 'vitejs.dev';
      input.dispatchEvent(new Event('input', { bubbles: true }));
    }

    form?.dispatchEvent(new Event('submit', { bubbles: true, cancelable: true }));
    expect(onNavigate).toHaveBeenCalledWith('https://vitejs.dev');

    unmount(component);
    document.body.removeChild(target);
  });
});
