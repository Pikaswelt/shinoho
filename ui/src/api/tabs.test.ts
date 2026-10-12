import { describe, it, expect, beforeEach } from 'vitest';
import { tabService, type TabEvent } from './tabs';

describe('tabService', () => {
  beforeEach(() => {
    tabService.resetMock();
  });

  it('starts with a default hub tab', async () => {
    const tabs = await tabService.getTabs();
    expect(tabs.length).toBe(1);
    expect(tabs[0].url).toBe('shinoho://hub');

    const active = await tabService.getActiveTab();
    expect(active?.id).toBe(tabs[0].id);
  });

  it('creates new tabs and emits tab-created and tab-activated events', async () => {
    const events: TabEvent[] = [];
    const unsubscribe = tabService.onTabEvent((e) => events.push(e));

    const newTab = await tabService.createTab({ url: 'https://example.com' });
    expect(newTab.url).toBe('https://example.com');

    const tabs = await tabService.getTabs();
    expect(tabs.length).toBe(2);

    const active = await tabService.getActiveTab();
    expect(active?.id).toBe(newTab.id);

    expect(events.some((e) => e.type === 'tab-created')).toBe(true);
    expect(events.some((e) => e.type === 'tab-activated' && e.tabId === newTab.id)).toBe(true);

    unsubscribe();
  });

  it('navigates tabs and updates navigation history (back / forward)', async () => {
    const tab = await tabService.createTab({ url: 'https://alpha.com' });
    expect(tab.canGoBack).toBe(false);

    await tabService.navigateTab(tab.id, 'https://beta.com');
    let tabs = await tabService.getTabs();
    let current = tabs.find((t) => t.id === tab.id);
    expect(current?.url).toBe('https://beta.com');
    expect(current?.canGoBack).toBe(true);
    expect(current?.canGoForward).toBe(false);

    await tabService.goBack(tab.id);
    tabs = await tabService.getTabs();
    current = tabs.find((t) => t.id === tab.id);
    expect(current?.url).toBe('https://alpha.com');
    expect(current?.canGoBack).toBe(false);
    expect(current?.canGoForward).toBe(true);

    await tabService.goForward(tab.id);
    tabs = await tabService.getTabs();
    current = tabs.find((t) => t.id === tab.id);
    expect(current?.url).toBe('https://beta.com');
    expect(current?.canGoBack).toBe(true);
    expect(current?.canGoForward).toBe(false);
  });

  it('closes a tab and activates the remaining tab', async () => {
    const tabA = (await tabService.getTabs())[0];
    const tabB = await tabService.createTab({ url: 'https://beta.com' });

    expect((await tabService.getTabs()).length).toBe(2);

    await tabService.closeTab(tabB.id);
    const tabs = await tabService.getTabs();
    expect(tabs.length).toBe(1);
    expect(tabs[0].id).toBe(tabA.id);

    const active = await tabService.getActiveTab();
    expect(active?.id).toBe(tabA.id);
  });

  it('handles reloadTab', async () => {
    const tab = (await tabService.getTabs())[0];
    await tabService.reloadTab(tab.id);
    const refreshed = (await tabService.getTabs())[0];
    expect(refreshed.isLoading).toBe(false);
  });
});
