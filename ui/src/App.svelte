<script lang="ts">
  import { onMount } from 'svelte';
  import Header from './components/Header.svelte';
  import TabBar from './components/TabBar.svelte';
  import AddressBar from './components/AddressBar.svelte';
  import { tabService, type TabInfo, type TabId } from './api/tabs';
  import { t, getLocale, setLocale, SUPPORTED_LOCALES, type SupportedLocale } from './i18n';

  let currentLang = $state<SupportedLocale>(getLocale());
  let tabs = $state<TabInfo[]>([]);
  let activeTabId = $state<TabId | null>(null);

  const activeTab = $derived(tabs.find((item) => item.id === activeTabId) ?? tabs[0]);

  const tr = (key: string, params?: Record<string, string | number>) => t(key, params, currentLang);

  async function refreshTabs() {
    tabs = await tabService.getTabs();
    const active = await tabService.getActiveTab();
    activeTabId = active?.id ?? null;
  }

  onMount(() => {
    refreshTabs();
    const unsubscribe = tabService.onTabEvent(() => {
      refreshTabs();
    });
    return unsubscribe;
  });

  async function handleSelectTab(id: TabId) {
    await tabService.activateTab(id);
    await refreshTabs();
  }

  async function handleCloseTab(id: TabId) {
    await tabService.closeTab(id);
    await refreshTabs();
  }

  async function handleNewTab() {
    await tabService.createTab();
    await refreshTabs();
  }

  async function handleNavigate(url: string) {
    if (activeTabId) {
      await tabService.navigateTab(activeTabId, url);
      await refreshTabs();
    }
  }

  async function handleBack() {
    if (activeTabId) {
      await tabService.goBack(activeTabId);
      await refreshTabs();
    }
  }

  async function handleForward() {
    if (activeTabId) {
      await tabService.goForward(activeTabId);
      await refreshTabs();
    }
  }

  async function handleReload() {
    if (activeTabId) {
      await tabService.reloadTab(activeTabId);
      await refreshTabs();
    }
  }

  async function handleHome() {
    if (activeTabId) {
      await tabService.navigateTab(activeTabId, 'shinoho://hub');
      await refreshTabs();
    }
  }

  function changeLanguage(lang: SupportedLocale) {
    setLocale(lang);
    currentLang = lang;
  }
</script>

<div class="shinoho-app" data-testid="shinoho-app">
  <Header title={tr('app.title')} />
  <TabBar
    {tabs}
    {activeTabId}
    onSelectTab={handleSelectTab}
    onCloseTab={handleCloseTab}
    onNewTab={handleNewTab}
  />
  <AddressBar
    url={activeTab?.url ?? 'shinoho://hub'}
    canGoBack={activeTab?.canGoBack ?? false}
    canGoForward={activeTab?.canGoForward ?? false}
    isLoading={activeTab?.isLoading ?? false}
    onNavigate={handleNavigate}
    onBack={handleBack}
    onForward={handleForward}
    onReload={handleReload}
    onHome={handleHome}
  />

  <main class="content-viewport" data-testid="content-viewport">
    {#if !activeTab || activeTab.url === 'shinoho://hub'}
      <div class="hub-view" data-testid="hub-view">
        <div class="welcome-card">
          <h1>{tr('app.title')}</h1>
          <p class="tagline">{tr('app.tagline')}</p>
          <p class="status" data-testid="status-message">{tr('app.ready')}</p>

          <div class="quick-links">
            <button
              type="button"
              class="quick-link-item"
              onclick={() => handleNavigate('https://github.com')}
            >
              <span class="ql-icon">🐙</span>
              <span class="ql-title">GitHub</span>
            </button>
            <button
              type="button"
              class="quick-link-item"
              onclick={() => handleNavigate('https://duckduckgo.com')}
            >
              <span class="ql-icon">🦆</span>
              <span class="ql-title">DuckDuckGo</span>
            </button>
            <button
              type="button"
              class="quick-link-item"
              onclick={() => handleNavigate('https://wikipedia.org')}
            >
              <span class="ql-icon">📚</span>
              <span class="ql-title">Wikipedia</span>
            </button>
          </div>

          <div class="language-selector" data-testid="language-selector">
            {#each SUPPORTED_LOCALES as loc}
              <button
                type="button"
                class="lang-btn"
                class:active={currentLang === loc}
                onclick={() => changeLanguage(loc)}
                data-testid={`lang-btn-${loc}`}
              >
                {loc.toUpperCase()}
              </button>
            {/each}
          </div>
        </div>
      </div>
    {:else}
      <div class="webview-container" data-testid="webview-container">
        <div class="webview-placeholder">
          <p class="active-url-display">{activeTab.url}</p>
          <span class="webview-note">WebView: {activeTab.title}</span>
        </div>
      </div>
    {/if}
  </main>
</div>

<style>
  .shinoho-app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    width: 100vw;
    background-color: var(--color-bg-base);
  }

  .content-viewport {
    flex: 1;
    display: flex;
    overflow: hidden;
    position: relative;
  }

  .hub-view {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 24px;
  }

  .welcome-card {
    text-align: center;
    padding: 32px 48px;
    background: var(--color-bg-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.3);
  }

  .welcome-card h1 {
    font-size: 32px;
    margin-bottom: 8px;
    color: var(--color-accent);
    letter-spacing: 1px;
  }

  .tagline {
    font-size: 14px;
    color: var(--color-text-muted);
    margin-bottom: 6px;
  }

  .status {
    color: var(--color-text-main);
    font-size: 14px;
    margin-bottom: 20px;
  }

  .quick-links {
    display: flex;
    gap: 16px;
    justify-content: center;
    margin-bottom: 24px;
  }

  .quick-link-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    padding: 12px 16px;
    background: var(--color-bg-elevated);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    transition: all 0.2s ease;
  }

  .quick-link-item:hover {
    border-color: var(--color-accent);
    transform: translateY(-2px);
  }

  .ql-icon {
    font-size: 20px;
  }

  .ql-title {
    font-size: 12px;
    color: var(--color-text-main);
  }

  .language-selector {
    display: flex;
    justify-content: center;
    gap: 8px;
  }

  .lang-btn {
    padding: 4px 10px;
    font-size: 12px;
    font-weight: 600;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    color: var(--color-text-muted);
    transition: all 0.15s ease;
  }

  .lang-btn:hover {
    color: var(--color-text-main);
    border-color: var(--color-accent);
  }

  .lang-btn.active {
    background-color: var(--color-accent);
    border-color: var(--color-accent);
    color: #fff;
  }

  .webview-container {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: #0b0b0f;
  }

  .webview-placeholder {
    text-align: center;
    padding: 24px;
    background: var(--color-bg-surface);
    border: 1px dashed var(--color-border);
    border-radius: var(--radius-md);
  }

  .active-url-display {
    color: var(--color-accent);
    font-family: monospace;
    font-size: 14px;
    margin-bottom: 6px;
  }

  .webview-note {
    font-size: 12px;
    color: var(--color-text-muted);
  }
</style>
