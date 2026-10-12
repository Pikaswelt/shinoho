import { describe, it, expect, vi } from 'vitest';
import { mount, unmount } from 'svelte';
import TabBar from './TabBar.svelte';
import type { TabInfo } from '@core/api';

describe('TabBar component', () => {
  const mockTabs: TabInfo[] = [
    {
      id: 'tab-1',
      title: 'First Tab',
      url: 'shinoho://hub',
      isLoading: false,
      canGoBack: false,
      canGoForward: false,
      isPinned: false,
      isMuted: false,
      createdAt: Date.now(),
    },
    {
      id: 'tab-2',
      title: 'GitHub',
      url: 'https://github.com',
      isLoading: false,
      canGoBack: true,
      canGoForward: false,
      isPinned: false,
      isMuted: false,
      createdAt: Date.now(),
    },
  ];

  it('renders all tabs and highlights the active tab', () => {
    const target = document.createElement('div');
    document.body.appendChild(target);

    const component = mount(TabBar, {
      target,
      props: {
        tabs: mockTabs,
        activeTabId: 'tab-2',
        onSelectTab: vi.fn(),
        onCloseTab: vi.fn(),
        onNewTab: vi.fn(),
      },
    });

    const tab1 = target.querySelector('[data-testid="tab-tab-1"]');
    const tab2 = target.querySelector('[data-testid="tab-tab-2"]');

    expect(tab1).not.toBeNull();
    expect(tab2).not.toBeNull();
    expect(tab2?.classList.contains('active')).toBe(true);
    expect(tab1?.classList.contains('active')).toBe(false);

    unmount(component);
    document.body.removeChild(target);
  });

  it('triggers onSelectTab, onCloseTab and onNewTab events', () => {
    const target = document.createElement('div');
    document.body.appendChild(target);

    const onSelect = vi.fn();
    const onClose = vi.fn();
    const onNew = vi.fn();

    const component = mount(TabBar, {
      target,
      props: {
        tabs: mockTabs,
        activeTabId: 'tab-1',
        onSelectTab: onSelect,
        onCloseTab: onClose,
        onNewTab: onNew,
      },
    });

    const tab2 = target.querySelector<HTMLElement>('[data-testid="tab-tab-2"]');
    tab2?.click();
    expect(onSelect).toHaveBeenCalledWith('tab-2');

    const closeBtn = target.querySelector<HTMLButtonElement>('[data-testid="tab-close-tab-1"]');
    closeBtn?.click();
    expect(onClose).toHaveBeenCalledWith('tab-1');

    const newBtn = target.querySelector<HTMLButtonElement>('[data-testid="new-tab-btn"]');
    newBtn?.click();
    expect(onNew).toHaveBeenCalled();

    unmount(component);
    document.body.removeChild(target);
  });
});
