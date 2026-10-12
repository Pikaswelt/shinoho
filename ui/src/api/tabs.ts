import {
  type TabId,
  type TabInfo,
  type CreateTabRequest,
  type NavigateTabRequest,
  type TabActionRequest,
  type ReloadTabRequest,
  type TabEvent,
  TAB_COMMANDS,
  TAB_EVENTS,
} from '@core/api';

export * from '@core/api';

type TabEventListener = (event: TabEvent) => void;

class TabService {
  private inMemoryTabs: TabInfo[] = [];
  private activeTabId: TabId | null = null;
  private listeners: Set<TabEventListener> = new Set();
  private historyMap: Map<TabId, { stack: string[]; index: number }> = new Map();

  constructor() {
    this.initDefaultTab();
  }

  private isTauriAvailable(): boolean {
    return (
      typeof window !== 'undefined' &&
      '__TAURI_INTERNALS__' in window &&
      Boolean(window.__TAURI_INTERNALS__)
    );
  }

  public initDefaultTab(): void {
    if (this.inMemoryTabs.length === 0) {
      const defaultTab: TabInfo = {
        id: 'tab-1',
        title: 'Neuer Tab',
        url: 'shinoho://hub',
        isLoading: false,
        canGoBack: false,
        canGoForward: false,
        isPinned: false,
        isMuted: false,
        createdAt: Date.now(),
      };
      this.inMemoryTabs = [defaultTab];
      this.activeTabId = defaultTab.id;
      this.historyMap.set(defaultTab.id, { stack: [defaultTab.url], index: 0 });
    }
  }

  public resetMock(): void {
    this.inMemoryTabs = [];
    this.activeTabId = null;
    this.listeners.clear();
    this.historyMap.clear();
    this.initDefaultTab();
  }

  private emitEvent(event: TabEvent): void {
    this.listeners.forEach((listener) => listener(event));
  }

  public async getTabs(): Promise<TabInfo[]> {
    if (this.isTauriAvailable()) {
      const { invoke } = await import('@tauri-apps/api/core');
      return invoke<TabInfo[]>(TAB_COMMANDS.GET_TABS);
    }
    return [...this.inMemoryTabs];
  }

  public async getActiveTab(): Promise<TabInfo | undefined> {
    const tabs = await this.getTabs();
    return tabs.find((t) => t.id === this.activeTabId) ?? tabs[0];
  }

  public async createTab(options?: CreateTabRequest): Promise<TabInfo> {
    if (this.isTauriAvailable()) {
      const { invoke } = await import('@tauri-apps/api/core');
      return invoke<TabInfo>(TAB_COMMANDS.CREATE_TAB, { request: options });
    }

    const newId = `tab-${Date.now()}-${Math.floor(Math.random() * 1000)}`;
    const url = options?.url ?? 'shinoho://hub';
    const newTab: TabInfo = {
      id: newId,
      title: url === 'shinoho://hub' ? 'Neuer Tab' : url,
      url,
      isLoading: false,
      canGoBack: false,
      canGoForward: false,
      isPinned: Boolean(options?.isPinned),
      isMuted: false,
      createdAt: Date.now(),
    };

    this.inMemoryTabs.push(newTab);
    this.historyMap.set(newId, { stack: [url], index: 0 });

    if (options?.activate !== false) {
      this.activeTabId = newId;
    }

    this.emitEvent({ type: 'tab-created', tab: { ...newTab } });
    if (options?.activate !== false) {
      this.emitEvent({ type: 'tab-activated', tabId: newId });
    }

    return { ...newTab };
  }

  public async closeTab(tabId: TabId): Promise<void> {
    if (this.isTauriAvailable()) {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke(TAB_COMMANDS.CLOSE_TAB, { request: { tabId } as TabActionRequest });
      return;
    }

    const index = this.inMemoryTabs.findIndex((t) => t.id === tabId);
    if (index === -1) return;

    this.inMemoryTabs.splice(index, 1);
    this.historyMap.delete(tabId);

    if (this.activeTabId === tabId) {
      if (this.inMemoryTabs.length > 0) {
        const nextActive = this.inMemoryTabs[Math.max(0, index - 1)];
        this.activeTabId = nextActive.id;
        this.emitEvent({ type: 'tab-activated', tabId: nextActive.id });
      } else {
        this.activeTabId = null;
      }
    }

    this.emitEvent({ type: 'tab-closed', tabId });
  }

  public async activateTab(tabId: TabId): Promise<void> {
    if (this.isTauriAvailable()) {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke(TAB_COMMANDS.ACTIVATE_TAB, { request: { tabId } as TabActionRequest });
      return;
    }

    const tab = this.inMemoryTabs.find((t) => t.id === tabId);
    if (!tab) return;

    this.activeTabId = tabId;
    this.emitEvent({ type: 'tab-activated', tabId });
  }

  public async navigateTab(tabId: TabId, url: string): Promise<void> {
    if (this.isTauriAvailable()) {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke(TAB_COMMANDS.NAVIGATE_TAB, {
        request: { tabId, url } as NavigateTabRequest,
      });
      return;
    }

    const tab = this.inMemoryTabs.find((t) => t.id === tabId);
    if (!tab) return;

    let history = this.historyMap.get(tabId);
    if (!history) {
      history = { stack: [tab.url], index: 0 };
      this.historyMap.set(tabId, history);
    }

    history.stack = history.stack.slice(0, history.index + 1);
    history.stack.push(url);
    history.index += 1;

    tab.url = url;
    tab.title = url.startsWith('http') ? new URL(url).hostname : url;
    tab.canGoBack = history.index > 0;
    tab.canGoForward = false;

    this.emitEvent({ type: 'tab-updated', tab: { ...tab } });
  }

  public async goBack(tabId: TabId): Promise<void> {
    if (this.isTauriAvailable()) {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke(TAB_COMMANDS.GO_BACK, { request: { tabId } as TabActionRequest });
      return;
    }

    const tab = this.inMemoryTabs.find((t) => t.id === tabId);
    const history = this.historyMap.get(tabId);
    if (!tab || !history || history.index <= 0) return;

    history.index -= 1;
    tab.url = history.stack[history.index];
    tab.title = tab.url;
    tab.canGoBack = history.index > 0;
    tab.canGoForward = history.index < history.stack.length - 1;

    this.emitEvent({ type: 'tab-updated', tab: { ...tab } });
  }

  public async goForward(tabId: TabId): Promise<void> {
    if (this.isTauriAvailable()) {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke(TAB_COMMANDS.GO_FORWARD, { request: { tabId } as TabActionRequest });
      return;
    }

    const tab = this.inMemoryTabs.find((t) => t.id === tabId);
    const history = this.historyMap.get(tabId);
    if (!tab || !history || history.index >= history.stack.length - 1) return;

    history.index += 1;
    tab.url = history.stack[history.index];
    tab.title = tab.url;
    tab.canGoBack = history.index > 0;
    tab.canGoForward = history.index < history.stack.length - 1;

    this.emitEvent({ type: 'tab-updated', tab: { ...tab } });
  }

  public async reloadTab(tabId: TabId, ignoreCache = false): Promise<void> {
    if (this.isTauriAvailable()) {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke(TAB_COMMANDS.RELOAD_TAB, {
        request: { tabId, ignoreCache } as ReloadTabRequest,
      });
      return;
    }

    const tab = this.inMemoryTabs.find((t) => t.id === tabId);
    if (!tab) return;

    tab.isLoading = true;
    this.emitEvent({ type: 'tab-updated', tab: { ...tab } });

    tab.isLoading = false;
    this.emitEvent({ type: 'tab-updated', tab: { ...tab } });
  }

  public onTabEvent(handler: TabEventListener): () => void {
    this.listeners.add(handler);
    let tauriUnlisten: (() => void) | undefined;

    if (this.isTauriAvailable()) {
      import('@tauri-apps/api/event').then(({ listen }) => {
        const events = [
          TAB_EVENTS.TAB_CREATED,
          TAB_EVENTS.TAB_UPDATED,
          TAB_EVENTS.TAB_CLOSED,
          TAB_EVENTS.TAB_ACTIVATED,
        ];

        Promise.all(events.map((evt) => listen(evt, (e) => handler(e.payload as TabEvent)))).then(
          (unsubscribers) => {
            tauriUnlisten = () => unsubscribers.forEach((u) => u());
          },
        );
      });
    }

    return () => {
      this.listeners.delete(handler);
      if (tauriUnlisten) {
        tauriUnlisten();
      }
    };
  }
}

export const tabService = new TabService();
