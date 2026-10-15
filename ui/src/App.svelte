<script lang="ts">
  import { onMount } from 'svelte';
  import Header from './components/Header.svelte';
  import TabBar from './components/TabBar.svelte';
  import AddressBar from './components/AddressBar.svelte';
  import GamingModeBanner from './components/gaming/GamingModeBanner.svelte';
  import { BookmarkBar, BookmarkManager } from './components/bookmarks';
  import { IncognitoStartPage, incognitoStore } from './components/incognito';
  import { tabService, type TabInfo, type TabId } from './api/tabs';
  import { recordVisit } from './api/history';
  import { t, getLocale, setLocale, SUPPORTED_LOCALES, type SupportedLocale } from './i18n';
  import { themeManager } from './themes';
  import { soundManager } from './sounds';
  import { hubStore } from './components/hub/hubStore';

  let currentLang = $state<SupportedLocale>(getLocale());
  let tabs = $state<TabInfo[]>([]);
  let activeTabId = $state<TabId | null>(null);
  let incognitoState = $state(incognitoStore.getState());

  $effect(() => {
    return incognitoStore.subscribe((s) => {
      incognitoState = s;
    });
  });

  const activeTab = $derived(tabs.find((item) => item.id === activeTabId) ?? tabs[0]);

  const tr = (key: string, params?: Record<string, string | number>) => t(key, params, currentLang);

  async function refreshTabs() {
    tabs = await tabService.getTabs();
    const active = await tabService.getActiveTab();
    activeTabId = active?.id ?? null;
  }

  onMount(() => {
    const initialTheme = themeManager.init();
    soundManager.init();
    if (initialTheme) {
      soundManager.handleThemeChange(initialTheme.defaultSoundPack);
      hubStore.handleThemeChange(initialTheme.defaultWallpaper);
    }

    const unsubTheme = themeManager.onThemeChange((theme) => {
      soundManager.handleThemeChange(theme.defaultSoundPack);
      hubStore.handleThemeChange(theme.defaultWallpaper);
    });

    refreshTabs();
    const unsubTabs = tabService.onTabEvent(() => {
      refreshTabs();
    });

    return () => {
      unsubTheme();
      unsubTabs();
    };
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
    const defaultUrl = incognitoState.isIncognito ? 'shinoho://incognito' : 'shinoho://hub';
    await tabService.createTab({ url: defaultUrl });
    await refreshTabs();
  }

  async function handleNavigate(url: string) {
    if (activeTabId) {
      const tab = tabs.find((item) => item.id === activeTabId);
      await tabService.navigateTab(activeTabId, url);
      await refreshTabs();
      // Verlauf nur fuer echte Webseiten und nie im Inkognito-Modus.
      if (!incognitoState.isIncognito) {
        void recordVisit({
          url,
          title: tab?.title && tab.url === url ? tab.title : url,
          typed: true,
        });
      }
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
      const homeUrl = incognitoState.isIncognito ? 'shinoho://incognito' : 'shinoho://hub';
      await tabService.navigateTab(activeTabId, homeUrl);
      await refreshTabs();
    }
  }

  function handleOpenIncognitoWindow() {
    incognitoStore.openIncognitoWindow();
  }

  function changeLanguage(lang: SupportedLocale) {
    setLocale(lang);
    currentLang = lang;
  }
</script>

<div
  class="shinoho-app"
  class:theme-incognito={incognitoState.isIncognito}
  data-testid="shinoho-app"
>
  <Header
    title={incognitoState.isIncognito
      ? `${tr('app.title')} (${tr('incognito.badge')})`
      : tr('app.title')}
    onOpenIncognito={handleOpenIncognitoWindow}
  />
  <TabBar
    {tabs}
    {activeTabId}
    onSelectTab={handleSelectTab}
    onCloseTab={handleCloseTab}
    onNewTab={handleNewTab}
  />
  <AddressBar
    url={activeTab?.url ?? (incognitoState.isIncognito ? 'shinoho://incognito' : 'shinoho://hub')}
    canGoBack={activeTab?.canGoBack ?? false}
    canGoForward={activeTab?.canGoForward ?? false}
    isLoading={activeTab?.isLoading ?? false}
    onNavigate={handleNavigate}
    onBack={handleBack}
    onForward={handleForward}
    onReload={handleReload}
    onHome={handleHome}
  />

  <BookmarkBar onNavigate={handleNavigate} />
  <BookmarkManager onNavigate={handleNavigate} />

  <GamingModeBanner />

  <main class="content-viewport" data-testid="content-viewport">
    {#if incognitoState.isIncognito && (!activeTab || activeTab.url === 'shinoho://incognito' || activeTab.url === 'shinoho://hub')}
      <IncognitoStartPage onNavigate={handleNavigate} />
    {:else if !activeTab || activeTab.url === 'shinoho://hub'}
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
    {:else if activeTab.url === 'shinoho://incognito'}
      <IncognitoStartPage onNavigate={handleNavigate} />
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

  /* Incognito Look */
  .shinoho-app.theme-incognito {
    --color-bg-base: #150d24;
    --color-bg-surface: #1f1435;
    --color-border: rgba(157, 78, 221, 0.25);
    --color-accent: #9d4edd;
    background-color: #150d24;
  }

  .content-viewport {
    flex: 1;
    display: flex;
    position: relative;
    overflow: hidden;
  }

  .hub-view {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 24px;
    background: radial-gradient(ellipse at top, rgba(255, 94, 154, 0.08) 0%, transparent 70%);
  }

  .welcome-card {
    text-align: center;
    padding: 40px 56px;
    background: var(--color-bg-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.35);
    max-width: 560px;
    width: 100%;
  }

  .welcome-card h1 {
    font-size: 32px;
    margin: 0 0 8px 0;
    color: var(--color-accent);
    font-weight: 700;
  }

  .tagline {
    font-size: 15px;
    color: var(--color-text-muted);
    margin: 0 0 12px 0;
  }

  .status {
    color: var(--color-text-main);
    font-size: 13px;
    margin: 0 0 24px 0;
    padding: 6px 12px;
    background: rgba(255, 255, 255, 0.03);
    border-radius: var(--radius-sm);
    display: inline-block;
  }

  .quick-links {
    display: flex;
    justify-content: center;
    gap: 12px;
    margin-bottom: 24px;
  }

  .quick-link-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    padding: 12px 18px;
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    color: var(--color-text-main);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .quick-link-item:hover {
    background: rgba(255, 94, 154, 0.12);
    border-color: var(--color-accent);
    transform: translateY(-2px);
  }

  .ql-icon {
    font-size: 24px;
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
