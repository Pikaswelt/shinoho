<script lang="ts">
  import { bookmarkStore, SYSTEM_ROOT_TOOLBAR } from './bookmarkStore';
  import type { Bookmark } from './bookmarks.types';
  import { t, getLocale } from '../../i18n';

  interface Props {
    onNavigate?: (url: string) => void;
  }

  let { onNavigate }: Props = $props();

  let storeState = $state(bookmarkStore.getState());
  $effect(() => {
    return bookmarkStore.subscribe((s) => {
      storeState = s;
    });
  });

  $effect(() => {
    void bookmarkStore.hydrate();
  });

  let activeDropdownFolderId = $state<string | null>(null);

  // Helper for reactive translation
  const tr = (key: string, params?: Record<string, string | number>) => t(key, params, getLocale());

  const toolbarItems = $derived(
    storeState.bookmarks
      .filter((b) => b.parentId === SYSTEM_ROOT_TOOLBAR)
      .sort((a, b) => a.position - b.position),
  );

  function handleItemClick(item: Bookmark) {
    if (item.isFolder) {
      activeDropdownFolderId = activeDropdownFolderId === item.id ? null : item.id;
    } else if (item.url) {
      activeDropdownFolderId = null;
      if (onNavigate) {
        onNavigate(item.url);
      } else {
        window.open(item.url, '_blank');
      }
    }
  }

  function handleAddCurrent() {
    bookmarkStore.createBookmark({
      title: 'Neue Seite',
      url: 'https://example.com',
      parentId: SYSTEM_ROOT_TOOLBAR,
      tags: ['neu'],
    });
  }
</script>

<svelte:window onclick={() => (activeDropdownFolderId = null)} />

{#if storeState.isBarVisible}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="bookmark-bar" data-testid="bookmark-bar" onclick={(e) => e.stopPropagation()}>
    <div class="bar-items">
      {#if toolbarItems.length === 0}
        <span class="empty-hint">{tr('bookmarks.bar.empty')}</span>
      {:else}
        {#each toolbarItems as item (item.id)}
          <div class="item-wrapper">
            <button
              type="button"
              class="bar-item"
              class:is-folder={item.isFolder}
              class:is-active={activeDropdownFolderId === item.id}
              onclick={() => handleItemClick(item)}
              title={item.url || item.title}
              data-testid={`bookmark-bar-item-${item.id}`}
            >
              {#if item.isFolder}
                <span class="item-icon folder-icon">📁</span>
              {:else if item.faviconUrl}
                <img
                  src={item.faviconUrl}
                  alt=""
                  class="item-favicon"
                  onerror={(e) => {
                    (e.currentTarget as HTMLElement).style.display = 'none';
                  }}
                />
              {:else}
                <span class="item-icon fallback-icon">✦</span>
              {/if}
              <span class="item-title">{item.title}</span>
              {#if item.isFolder}
                <span class="folder-chevron">▾</span>
              {/if}
            </button>

            <!-- Dropdown Menu for Folder -->
            {#if item.isFolder && activeDropdownFolderId === item.id}
              {@const folderChildren = storeState.bookmarks
                .filter((b) => b.parentId === item.id)
                .sort((a, b) =>
                  a.isFolder === b.isFolder ? a.position - b.position : a.isFolder ? -1 : 1,
                )}
              <div class="folder-dropdown" data-testid={`folder-dropdown-${item.id}`}>
                {#if folderChildren.length === 0}
                  <div class="dropdown-empty">{tr('bookmarks.manager.emptyFolder')}</div>
                {:else}
                  {#each folderChildren as child (child.id)}
                    <button
                      type="button"
                      class="dropdown-item"
                      onclick={() => handleItemClick(child)}
                      title={child.url || child.title}
                      data-testid={`folder-dropdown-child-${child.id}`}
                    >
                      {#if child.isFolder}
                        <span class="item-icon">📁</span>
                      {:else if child.faviconUrl}
                        <img
                          src={child.faviconUrl}
                          alt=""
                          class="item-favicon"
                          onerror={(e) => {
                            (e.currentTarget as HTMLElement).style.display = 'none';
                          }}
                        />
                      {:else}
                        <span class="item-icon fallback-icon">✦</span>
                      {/if}
                      <span class="dropdown-title">{child.title}</span>
                    </button>
                  {/each}
                {/if}
              </div>
            {/if}
          </div>
        {/each}
      {/if}
    </div>

    <div class="bar-actions">
      <button
        type="button"
        class="action-btn"
        title={tr('bookmarks.bar.addCurrent')}
        onclick={handleAddCurrent}
        data-testid="bookmark-bar-add"
      >
        ★
      </button>
      <button
        type="button"
        class="action-btn"
        title={tr('bookmarks.bar.openManager')}
        onclick={() => bookmarkStore.openManager(SYSTEM_ROOT_TOOLBAR)}
        data-testid="bookmark-bar-manager-btn"
      >
        📑
      </button>
      <button
        type="button"
        class="action-btn close-btn"
        title={tr('bookmarks.bar.hideBar')}
        onclick={() => bookmarkStore.setBarVisibility(false)}
        data-testid="bookmark-bar-close"
      >
        ✕
      </button>
    </div>
  </div>
{/if}

<style>
  .bookmark-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 32px;
    padding: 0 8px;
    background: var(--color-bg-surface, #1e1e24);
    border-bottom: 1px solid var(--color-border, rgba(255, 255, 255, 0.08));
    font-size: 12px;
    user-select: none;
    z-index: 50;
    position: relative;
  }

  .bar-items {
    display: flex;
    align-items: center;
    gap: 4px;
    overflow-x: auto;
    overflow-y: hidden;
    scrollbar-width: none;
    flex: 1;
  }
  .bar-items::-webkit-scrollbar {
    display: none;
  }

  .empty-hint {
    color: var(--color-text-muted, #888);
    font-style: italic;
    font-size: 11px;
    padding: 0 4px;
  }

  .item-wrapper {
    position: relative;
  }

  .bar-item {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 3px 8px;
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--radius-sm, 4px);
    color: var(--color-text-main, #e0e0e0);
    cursor: pointer;
    font-size: 12px;
    white-space: nowrap;
    transition: all 0.15s ease;
  }

  .bar-item:hover,
  .bar-item.is-active {
    background: rgba(255, 255, 255, 0.07);
    border-color: var(--color-border, rgba(255, 255, 255, 0.1));
    color: var(--color-accent, #ff5e9a);
  }

  .item-favicon {
    width: 14px;
    height: 14px;
    border-radius: 2px;
    object-fit: contain;
  }

  .item-icon {
    font-size: 12px;
  }

  .fallback-icon {
    color: var(--color-accent, #ff5e9a);
  }

  .folder-chevron {
    font-size: 10px;
    color: var(--color-text-muted, #888);
    margin-left: 2px;
  }

  .item-title {
    max-width: 140px;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  /* Dropdown Menu */
  .folder-dropdown {
    position: absolute;
    top: calc(100% + 2px);
    left: 0;
    min-width: 180px;
    max-width: 280px;
    background: var(--color-bg-surface, #1e1e24);
    border: 1px solid var(--color-border, rgba(255, 255, 255, 0.15));
    border-radius: var(--radius-md, 6px);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5);
    padding: 4px;
    display: flex;
    flex-direction: column;
    gap: 2px;
    z-index: 100;
  }

  .dropdown-item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 6px 8px;
    background: transparent;
    border: none;
    border-radius: var(--radius-sm, 4px);
    color: var(--color-text-main, #e0e0e0);
    text-align: left;
    cursor: pointer;
    font-size: 12px;
    transition: background 0.15s ease;
  }

  .dropdown-item:hover {
    background: rgba(255, 255, 255, 0.08);
    color: var(--color-accent, #ff5e9a);
  }

  .dropdown-title {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .dropdown-empty {
    padding: 8px;
    color: var(--color-text-muted, #888);
    font-size: 11px;
    text-align: center;
    font-style: italic;
  }

  /* Actions */
  .bar-actions {
    display: flex;
    align-items: center;
    gap: 4px;
    margin-left: 8px;
  }

  .action-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    background: transparent;
    border: none;
    border-radius: var(--radius-sm, 4px);
    color: var(--color-text-muted, #888);
    cursor: pointer;
    font-size: 13px;
    transition: all 0.15s ease;
  }

  .action-btn:hover {
    background: rgba(255, 255, 255, 0.08);
    color: var(--color-text-main, #e0e0e0);
  }

  .close-btn {
    font-size: 11px;
  }
</style>
