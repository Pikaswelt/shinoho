<script lang="ts">
  import type { TabInfo, TabId } from '@core/api';
  import { t } from '../i18n';
  import { soundManager } from '../sounds';

  interface Props {
    tabs: TabInfo[];
    activeTabId: TabId | null;
    onSelectTab: (id: TabId) => void;
    onCloseTab: (id: TabId) => void;
    onNewTab: () => void;
  }

  let { tabs, activeTabId, onSelectTab, onCloseTab, onNewTab }: Props = $props();

  function handleSelect(id: TabId) {
    soundManager.play('tab.switch');
    onSelectTab(id);
  }

  function handleClose(e: MouseEvent, tabId: TabId) {
    e.stopPropagation();
    soundManager.play('tab.close');
    onCloseTab(tabId);
  }

  function handleNew() {
    soundManager.play('tab.open');
    onNewTab();
  }
</script>

<div class="tab-bar" data-testid="tab-bar">
  <div class="tabs-list" role="tablist">
    {#each tabs as tab (tab.id)}
      <div
        class="tab-item"
        class:active={tab.id === activeTabId}
        class:pinned={tab.isPinned}
        class:loading={tab.isLoading}
        role="tab"
        aria-selected={tab.id === activeTabId}
        tabindex="0"
        onclick={() => handleSelect(tab.id)}
        onkeydown={(e) => {
          if (e.key === 'Enter' || e.key === ' ') handleSelect(tab.id);
        }}
        data-testid={`tab-${tab.id}`}
      >
        <span class="tab-favicon">
          {#if tab.isLoading}
            <span class="spinner" aria-label="loading">⏳</span>
          {:else if tab.favicon}
            <img src={tab.favicon} alt="" class="favicon-img" />
          {:else if tab.url.startsWith('shinoho://')}
            <span class="hub-icon">✦</span>
          {:else}
            <span class="globe-icon">🌐</span>
          {/if}
        </span>

        <span class="tab-title" title={tab.title || t('tabs.untitled')}>
          {tab.title || t('tabs.untitled')}
        </span>

        <button
          type="button"
          class="tab-close"
          title={t('tabs.close')}
          onclick={(e) => handleClose(e, tab.id)}
          data-testid={`tab-close-${tab.id}`}
        >
          ×
        </button>
      </div>
    {/each}
  </div>

  <button
    type="button"
    class="new-tab-btn"
    title={t('tabs.newTab')}
    onclick={handleNew}
    data-testid="new-tab-btn"
  >
    +
  </button>
</div>

<style>
  .tab-bar {
    display: flex;
    align-items: center;
    background-color: var(--color-bg-base);
    height: var(--tabbar-height, 38px);
    padding: 0 8px;
    border-bottom: 1px solid var(--color-border);
    gap: 4px;
    overflow-x: auto;
  }

  .tabs-list {
    display: flex;
    align-items: center;
    gap: 4px;
    flex: 1;
    overflow-x: auto;
  }

  .tab-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 0 10px;
    height: 30px;
    min-width: 120px;
    max-width: 200px;
    background-color: var(--color-tab-inactive);
    border: 1px solid transparent;
    border-radius: var(--radius-sm);
    color: var(--color-text-muted);
    font-size: 12px;
    cursor: pointer;
    transition: all 0.15s ease;
    user-select: none;
  }

  .tab-item:hover {
    background-color: var(--color-tab-hover);
    color: var(--color-text-main);
  }

  .tab-item.active {
    background-color: var(--color-tab-active);
    color: var(--color-text-main);
    border-color: var(--color-border);
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
  }

  .tab-favicon {
    font-size: 12px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .hub-icon {
    color: var(--color-accent);
  }

  .tab-title {
    flex: 1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .tab-close {
    font-size: 14px;
    width: 18px;
    height: 18px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    color: var(--color-text-muted);
    opacity: 0.7;
    transition: all 0.15s ease;
  }

  .tab-close:hover {
    opacity: 1;
    background-color: rgba(255, 255, 255, 0.1);
    color: var(--color-accent);
  }

  .new-tab-btn {
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 16px;
    border-radius: var(--radius-sm);
    color: var(--color-text-muted);
    transition: all 0.15s ease;
  }

  .new-tab-btn:hover {
    background-color: var(--color-tab-hover);
    color: var(--color-accent);
  }
</style>
