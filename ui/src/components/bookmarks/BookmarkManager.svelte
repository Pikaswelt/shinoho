<script lang="ts">
  import {
    bookmarkStore,
    SYSTEM_ROOT_TOOLBAR,
    SYSTEM_ROOT_OTHER,
    SYSTEM_ROOT_MOBILE,
  } from './bookmarkStore';
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

  const tr = (key: string, params?: Record<string, string | number>) => t(key, params, getLocale());

  // Modal State
  let isEditModalOpen = $state<boolean>(false);
  let modalIsFolder = $state<boolean>(false);
  let editingItemId = $state<string | null>(null);
  let formTitle = $state<string>('');
  let formUrl = $state<string>('');
  let formParentId = $state<string>(SYSTEM_ROOT_TOOLBAR);
  let formTags = $state<string>('');

  // Status message
  let statusBanner = $state<string | null>(null);
  let fileInputRef = $state<HTMLInputElement | null>(null);

  // Derived queries
  const allFolders = $derived(storeState.bookmarks.filter((b) => b.isFolder));
  const tagList = $derived(bookmarkStore.getAllTags());

  const displayedItems = $derived.by(() => {
    if (storeState.searchQuery.trim() || storeState.activeTag) {
      return bookmarkStore.search(storeState.searchQuery, storeState.activeTag);
    }
    return bookmarkStore.getItemsInFolder(storeState.activeFolderId);
  });

  const activeFolderName = $derived.by(() => {
    if (storeState.searchQuery.trim()) {
      return `Suche: "${storeState.searchQuery}"`;
    }
    if (storeState.activeTag) {
      return `Tag: #${storeState.activeTag}`;
    }
    if (storeState.activeFolderId === SYSTEM_ROOT_TOOLBAR) {
      return tr('bookmarks.folders.toolbar');
    }
    if (storeState.activeFolderId === SYSTEM_ROOT_OTHER) {
      return tr('bookmarks.folders.other');
    }
    if (storeState.activeFolderId === SYSTEM_ROOT_MOBILE) {
      return tr('bookmarks.folders.mobile');
    }
    const current = bookmarkStore.getById(storeState.activeFolderId);
    return current ? current.title : tr('bookmarks.manager.allBookmarks');
  });

  function selectFolder(folderId: string) {
    bookmarkStore.searchQuery = '';
    bookmarkStore.activeTag = null;
    bookmarkStore.activeFolderId = folderId;
  }

  function selectTag(tag: string) {
    if (storeState.activeTag === tag) {
      bookmarkStore.activeTag = null;
    } else {
      bookmarkStore.activeTag = tag;
      bookmarkStore.searchQuery = '';
    }
  }

  function openAddBookmark() {
    modalIsFolder = false;
    editingItemId = null;
    formTitle = '';
    formUrl = 'https://';
    formParentId = storeState.activeFolderId || SYSTEM_ROOT_TOOLBAR;
    formTags = '';
    isEditModalOpen = true;
  }

  function openAddFolder() {
    modalIsFolder = true;
    editingItemId = null;
    formTitle = '';
    formUrl = '';
    formParentId = storeState.activeFolderId || SYSTEM_ROOT_TOOLBAR;
    formTags = '';
    isEditModalOpen = true;
  }

  function openEdit(item: Bookmark) {
    editingItemId = item.id;
    modalIsFolder = item.isFolder;
    formTitle = item.title;
    formUrl = item.url || '';
    formParentId = item.parentId || SYSTEM_ROOT_TOOLBAR;
    formTags = item.tags.join(', ');
    isEditModalOpen = true;
  }

  function handleSave(e: SubmitEvent) {
    e.preventDefault();
    const tagsArray = formTags
      .split(',')
      .map((t) => t.trim())
      .filter(Boolean);

    if (editingItemId) {
      bookmarkStore.updateBookmark(editingItemId, {
        title: formTitle,
        url: modalIsFolder ? null : formUrl,
        parentId: formParentId,
        tags: tagsArray,
      });
    } else {
      bookmarkStore.createBookmark({
        title: formTitle,
        url: modalIsFolder ? null : formUrl,
        isFolder: modalIsFolder,
        parentId: formParentId,
        tags: tagsArray,
      });
    }

    isEditModalOpen = false;
  }

  function handleDelete(item: Bookmark) {
    const confirmMsg = item.isFolder
      ? tr('bookmarks.actions.deleteFolderConfirm', { title: item.title })
      : tr('bookmarks.actions.deleteConfirm', { title: item.title });

    if (window.confirm(confirmMsg)) {
      bookmarkStore.deleteBookmark(item.id);
    }
  }

  function handleItemClick(item: Bookmark) {
    if (item.isFolder) {
      selectFolder(item.id);
    } else if (item.url) {
      if (onNavigate) {
        onNavigate(item.url);
      } else {
        window.open(item.url, '_blank');
      }
    }
  }

  function handleExport() {
    const html = bookmarkStore.exportHtml();
    const blob = new Blob([html], { type: 'text/html;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = 'shinoho-bookmarks.html';
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    URL.revokeObjectURL(url);
  }

  function triggerImport() {
    if (fileInputRef) {
      fileInputRef.click();
    }
  }

  function handleFileSelected(e: Event) {
    const input = e.target as HTMLInputElement;
    if (!input.files || input.files.length === 0) return;

    const file = input.files[0];
    const reader = new FileReader();
    reader.onload = () => {
      const content = reader.result as string;
      try {
        const summary = bookmarkStore.importHtml(content, storeState.activeFolderId);
        statusBanner = tr('bookmarks.import.success', {
          bookmarks: summary.bookmarksCreated,
          folders: summary.foldersCreated,
        });
        setTimeout(() => {
          statusBanner = null;
        }, 4000);
      } catch {
        statusBanner = tr('bookmarks.import.error');
      }
    };
    reader.readAsText(file);
    input.value = '';
  }
</script>

{#if storeState.isManagerOpen}
  <!-- Hidden File Input for HTML Import -->
  <input
    type="file"
    accept=".html,.htm"
    style="display: none;"
    bind:this={fileInputRef}
    onchange={handleFileSelected}
    data-testid="bookmark-import-input"
  />

  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="manager-backdrop" onclick={() => bookmarkStore.closeManager()}>
    <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
    <div
      class="manager-window"
      onclick={(e) => e.stopPropagation()}
      data-testid="bookmark-manager-window"
    >
      <!-- Manager Header -->
      <div class="manager-topbar">
        <div class="topbar-title">
          <span class="icon">📑</span>
          <h2>{tr('bookmarks.manager.title')}</h2>
        </div>
        <button
          type="button"
          class="btn-close"
          title="Schließen"
          onclick={() => bookmarkStore.closeManager()}
          data-testid="bookmark-manager-close"
        >
          ✕
        </button>
      </div>

      {#if statusBanner}
        <div class="status-banner" data-testid="bookmark-status-banner">
          {statusBanner}
        </div>
      {/if}

      <!-- Manager Body -->
      <div class="manager-body">
        <!-- Sidebar Navigation -->
        <aside class="sidebar">
          <div class="search-box">
            <span class="search-icon">🔍</span>
            <input
              type="text"
              placeholder={tr('bookmarks.manager.searchPlaceholder')}
              bind:value={bookmarkStore.searchQuery}
              data-testid="bookmark-search-input"
            />
            {#if storeState.searchQuery}
              <button
                type="button"
                class="btn-clear-search"
                onclick={() => (bookmarkStore.searchQuery = '')}
              >
                ✕
              </button>
            {/if}
          </div>

          <div class="folder-section">
            <div class="section-label">{tr('bookmarks.folders.toolbar')}</div>
            <nav class="folder-nav">
              <button
                type="button"
                class="folder-item"
                class:active={storeState.activeFolderId === SYSTEM_ROOT_TOOLBAR &&
                  !storeState.activeTag &&
                  !storeState.searchQuery}
                onclick={() => selectFolder(SYSTEM_ROOT_TOOLBAR)}
                data-testid="folder-nav-toolbar"
              >
                <span class="folder-icon">📌</span>
                <span>{tr('bookmarks.folders.toolbar')}</span>
              </button>

              <button
                type="button"
                class="folder-item"
                class:active={storeState.activeFolderId === SYSTEM_ROOT_OTHER &&
                  !storeState.activeTag &&
                  !storeState.searchQuery}
                onclick={() => selectFolder(SYSTEM_ROOT_OTHER)}
                data-testid="folder-nav-other"
              >
                <span class="folder-icon">📁</span>
                <span>{tr('bookmarks.folders.other')}</span>
              </button>

              <button
                type="button"
                class="folder-item"
                class:active={storeState.activeFolderId === SYSTEM_ROOT_MOBILE &&
                  !storeState.activeTag &&
                  !storeState.searchQuery}
                onclick={() => selectFolder(SYSTEM_ROOT_MOBILE)}
                data-testid="folder-nav-mobile"
              >
                <span class="folder-icon">📱</span>
                <span>{tr('bookmarks.folders.mobile')}</span>
              </button>

              <!-- Custom Folders List -->
              {#each allFolders.filter((f) => f.id !== SYSTEM_ROOT_TOOLBAR && f.id !== SYSTEM_ROOT_OTHER && f.id !== SYSTEM_ROOT_MOBILE) as folder (folder.id)}
                <button
                  type="button"
                  class="folder-item custom-folder"
                  class:active={storeState.activeFolderId === folder.id &&
                    !storeState.activeTag &&
                    !storeState.searchQuery}
                  onclick={() => selectFolder(folder.id)}
                  data-testid={`folder-nav-${folder.id}`}
                >
                  <span class="folder-icon">📂</span>
                  <span class="folder-name">{folder.title}</span>
                </button>
              {/each}
            </nav>
          </div>

          <!-- Tag Cloud -->
          {#if tagList.length > 0}
            <div class="tags-section">
              <div class="section-label">Tags</div>
              <div class="tag-cloud">
                {#each tagList as { tag, count }}
                  <button
                    type="button"
                    class="tag-badge"
                    class:active={storeState.activeTag === tag}
                    onclick={() => selectTag(tag)}
                    data-testid={`tag-filter-${tag}`}
                  >
                    #{tag} <span class="tag-count">({count})</span>
                  </button>
                {/each}
              </div>
            </div>
          {/if}
        </aside>

        <!-- Main Content Area -->
        <main class="main-panel">
          <!-- Action Bar -->
          <div class="action-bar">
            <div class="current-path">
              <h3>{activeFolderName}</h3>
              <span class="items-count">
                {displayedItems.length === 1
                  ? tr('bookmarks.manager.itemsCountSingle')
                  : tr('bookmarks.manager.itemsCount', { count: displayedItems.length })}
              </span>
            </div>

            <div class="toolbar-actions">
              <button
                type="button"
                class="btn-action primary"
                onclick={openAddBookmark}
                data-testid="btn-add-bookmark"
              >
                + {tr('bookmarks.manager.newBookmark')}
              </button>
              <button
                type="button"
                class="btn-action secondary"
                onclick={openAddFolder}
                data-testid="btn-add-folder"
              >
                + {tr('bookmarks.manager.newFolder')}
              </button>
              <button
                type="button"
                class="btn-action secondary"
                onclick={triggerImport}
                data-testid="btn-import-html"
              >
                📥 {tr('bookmarks.manager.importHtml')}
              </button>
              <button
                type="button"
                class="btn-action secondary"
                onclick={handleExport}
                data-testid="btn-export-html"
              >
                📤 {tr('bookmarks.manager.exportHtml')}
              </button>
            </div>
          </div>

          <!-- Items Table / Grid -->
          <div class="items-container">
            {#if displayedItems.length === 0}
              <div class="empty-state">
                <span class="empty-icon">📂</span>
                <p>
                  {storeState.searchQuery
                    ? tr('bookmarks.manager.noResults')
                    : tr('bookmarks.manager.emptyFolder')}
                </p>
              </div>
            {:else}
              <table class="items-table" data-testid="bookmarks-table">
                <thead>
                  <tr>
                    <th class="col-icon"></th>
                    <th class="col-title">Titel</th>
                    <th class="col-url">URL</th>
                    <th class="col-tags">Tags</th>
                    <th class="col-actions">Aktionen</th>
                  </tr>
                </thead>
                <tbody>
                  {#each displayedItems as item (item.id)}
                    <tr
                      class="item-row"
                      class:is-folder={item.isFolder}
                      data-testid={`bookmark-row-${item.id}`}
                    >
                      <td class="col-icon">
                        {#if item.isFolder}
                          <span class="folder-icon">📁</span>
                        {:else if item.faviconUrl}
                          <img
                            src={item.faviconUrl}
                            alt=""
                            class="row-favicon"
                            onerror={(e) => {
                              (e.currentTarget as HTMLElement).style.display = 'none';
                            }}
                          />
                        {:else}
                          <span class="star-icon">✦</span>
                        {/if}
                      </td>
                      <td class="col-title">
                        <button
                          type="button"
                          class="link-title"
                          onclick={() => handleItemClick(item)}
                        >
                          {item.title}
                        </button>
                      </td>
                      <td class="col-url">
                        {#if !item.isFolder && item.url}
                          <a
                            href={item.url}
                            target="_blank"
                            rel="noreferrer"
                            class="url-text"
                            title={item.url}
                          >
                            {item.url}
                          </a>
                        {:else}
                          <span class="muted-text">–</span>
                        {/if}
                      </td>
                      <td class="col-tags">
                        <div class="tags-row">
                          {#each item.tags as tag}
                            <span class="item-tag">#{tag}</span>
                          {/each}
                        </div>
                      </td>
                      <td class="col-actions">
                        <button
                          type="button"
                          class="btn-row-action"
                          title={tr('bookmarks.actions.edit')}
                          onclick={() => openEdit(item)}
                          data-testid={`btn-edit-${item.id}`}
                        >
                          ✏️
                        </button>
                        <button
                          type="button"
                          class="btn-row-action btn-delete"
                          title={tr('bookmarks.actions.delete')}
                          onclick={() => handleDelete(item)}
                          data-testid={`btn-delete-${item.id}`}
                        >
                          🗑️
                        </button>
                      </td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            {/if}
          </div>
        </main>
      </div>
    </div>
  </div>
{/if}

<!-- Add / Edit Modal -->
{#if isEditModalOpen}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="modal-backdrop" onclick={() => (isEditModalOpen = false)}>
    <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
    <div
      class="edit-modal-card"
      onclick={(e) => e.stopPropagation()}
      data-testid="bookmark-edit-modal"
    >
      <h3>
        {#if modalIsFolder}
          {editingItemId
            ? tr('bookmarks.modal.editFolderTitle')
            : tr('bookmarks.modal.addFolderTitle')}
        {:else}
          {editingItemId
            ? tr('bookmarks.modal.editBookmarkTitle')
            : tr('bookmarks.modal.addBookmarkTitle')}
        {/if}
      </h3>

      <form onsubmit={handleSave}>
        <div class="form-group">
          <label for="bm-title">{tr('bookmarks.modal.nameLabel')}</label>
          <input
            id="bm-title"
            type="text"
            required
            bind:value={formTitle}
            data-testid="input-bookmark-title"
          />
        </div>

        {#if !modalIsFolder}
          <div class="form-group">
            <label for="bm-url">{tr('bookmarks.modal.urlLabel')}</label>
            <input
              id="bm-url"
              type="text"
              required
              bind:value={formUrl}
              data-testid="input-bookmark-url"
            />
          </div>
        {/if}

        <div class="form-group">
          <label for="bm-parent">{tr('bookmarks.modal.folderLabel')}</label>
          <select id="bm-parent" bind:value={formParentId} data-testid="select-bookmark-parent">
            <option value={SYSTEM_ROOT_TOOLBAR}>{tr('bookmarks.folders.toolbar')}</option>
            <option value={SYSTEM_ROOT_OTHER}>{tr('bookmarks.folders.other')}</option>
            <option value={SYSTEM_ROOT_MOBILE}>{tr('bookmarks.folders.mobile')}</option>
            {#each allFolders.filter((f) => f.id !== SYSTEM_ROOT_TOOLBAR && f.id !== SYSTEM_ROOT_OTHER && f.id !== SYSTEM_ROOT_MOBILE && f.id !== editingItemId) as folder}
              <option value={folder.id}>{folder.title}</option>
            {/each}
          </select>
        </div>

        <div class="form-group">
          <label for="bm-tags">{tr('bookmarks.modal.tagsLabel')}</label>
          <input
            id="bm-tags"
            type="text"
            placeholder={tr('bookmarks.modal.tagsHelp')}
            bind:value={formTags}
            data-testid="input-bookmark-tags"
          />
        </div>

        <div class="modal-actions">
          <button
            type="button"
            class="btn-cancel"
            onclick={() => (isEditModalOpen = false)}
            data-testid="btn-modal-cancel"
          >
            {tr('bookmarks.modal.cancel')}
          </button>
          <button type="submit" class="btn-save" data-testid="btn-modal-save">
            {tr('bookmarks.modal.save')}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}

<style>
  .manager-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.75);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 9999;
    padding: 24px;
  }

  .manager-window {
    width: 900px;
    max-width: 95vw;
    height: 600px;
    max-height: 90vh;
    background: var(--color-bg-surface, #1e1e24);
    border: 1px solid var(--color-border, rgba(255, 255, 255, 0.12));
    border-radius: var(--radius-lg, 12px);
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.6);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  /* Top Bar */
  .manager-topbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 20px;
    background: rgba(0, 0, 0, 0.2);
    border-bottom: 1px solid var(--color-border, rgba(255, 255, 255, 0.08));
  }

  .topbar-title {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .topbar-title h2 {
    font-size: 16px;
    font-weight: 600;
    margin: 0;
    color: var(--color-text-main, #e0e0e0);
  }

  .btn-close {
    background: transparent;
    border: none;
    color: var(--color-text-muted, #888);
    font-size: 14px;
    cursor: pointer;
    padding: 4px 8px;
    border-radius: var(--radius-sm, 4px);
    transition: all 0.15s ease;
  }
  .btn-close:hover {
    background: rgba(255, 255, 255, 0.1);
    color: #fff;
  }

  .status-banner {
    padding: 8px 16px;
    background: rgba(255, 94, 154, 0.2);
    border-bottom: 1px solid var(--color-accent, #ff5e9a);
    color: var(--color-accent, #ff5e9a);
    font-size: 13px;
    text-align: center;
  }

  /* Manager Body */
  .manager-body {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  /* Sidebar */
  .sidebar {
    width: 240px;
    background: rgba(0, 0, 0, 0.15);
    border-right: 1px solid var(--color-border, rgba(255, 255, 255, 0.08));
    display: flex;
    flex-direction: column;
    padding: 16px;
    gap: 16px;
    overflow-y: auto;
  }

  .search-box {
    position: relative;
    display: flex;
    align-items: center;
  }

  .search-icon {
    position: absolute;
    left: 8px;
    font-size: 12px;
    color: var(--color-text-muted, #888);
  }

  .search-box input {
    width: 100%;
    padding: 6px 24px 6px 26px;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid var(--color-border, rgba(255, 255, 255, 0.1));
    border-radius: var(--radius-sm, 4px);
    color: var(--color-text-main, #e0e0e0);
    font-size: 12px;
  }
  .search-box input:focus {
    outline: none;
    border-color: var(--color-accent, #ff5e9a);
  }

  .btn-clear-search {
    position: absolute;
    right: 6px;
    background: none;
    border: none;
    color: var(--color-text-muted, #888);
    cursor: pointer;
    font-size: 10px;
  }

  .section-label {
    font-size: 11px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--color-text-muted, #888);
    margin-bottom: 6px;
  }

  .folder-nav {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .folder-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 10px;
    background: transparent;
    border: none;
    border-radius: var(--radius-sm, 4px);
    color: var(--color-text-main, #e0e0e0);
    text-align: left;
    font-size: 12px;
    cursor: pointer;
    transition: all 0.15s ease;
  }
  .folder-item:hover {
    background: rgba(255, 255, 255, 0.05);
    color: var(--color-accent, #ff5e9a);
  }
  .folder-item.active {
    background: rgba(255, 94, 154, 0.15);
    color: var(--color-accent, #ff5e9a);
    font-weight: 600;
  }

  .custom-folder {
    padding-left: 20px;
  }

  .folder-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  /* Tags */
  .tag-cloud {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }

  .tag-badge {
    padding: 2px 6px;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid var(--color-border, rgba(255, 255, 255, 0.08));
    border-radius: 10px;
    color: var(--color-text-muted, #888);
    font-size: 11px;
    cursor: pointer;
    transition: all 0.15s ease;
  }
  .tag-badge:hover,
  .tag-badge.active {
    background: rgba(255, 94, 154, 0.2);
    border-color: var(--color-accent, #ff5e9a);
    color: var(--color-accent, #ff5e9a);
  }
  .tag-count {
    font-size: 9px;
    opacity: 0.8;
  }

  /* Main Panel */
  .main-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .action-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 20px;
    border-bottom: 1px solid var(--color-border, rgba(255, 255, 255, 0.08));
  }

  .current-path h3 {
    font-size: 15px;
    margin: 0;
    color: var(--color-text-main, #e0e0e0);
  }

  .items-count {
    font-size: 11px;
    color: var(--color-text-muted, #888);
  }

  .toolbar-actions {
    display: flex;
    gap: 8px;
  }

  .btn-action {
    padding: 6px 12px;
    font-size: 12px;
    font-weight: 500;
    border-radius: var(--radius-sm, 4px);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .btn-action.primary {
    background: var(--color-accent, #ff5e9a);
    border: 1px solid var(--color-accent, #ff5e9a);
    color: #fff;
  }
  .btn-action.primary:hover {
    opacity: 0.9;
  }

  .btn-action.secondary {
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid var(--color-border, rgba(255, 255, 255, 0.1));
    color: var(--color-text-main, #e0e0e0);
  }
  .btn-action.secondary:hover {
    background: rgba(255, 255, 255, 0.1);
  }

  /* Items Container */
  .items-container {
    flex: 1;
    overflow-y: auto;
    padding: 8px 16px;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--color-text-muted, #888);
    gap: 8px;
  }
  .empty-icon {
    font-size: 32px;
    opacity: 0.4;
  }

  .items-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 12px;
    text-align: left;
  }

  .items-table th {
    padding: 8px;
    color: var(--color-text-muted, #888);
    font-weight: 600;
    border-bottom: 1px solid var(--color-border, rgba(255, 255, 255, 0.08));
  }

  .items-table td {
    padding: 8px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.04);
    vertical-align: middle;
  }

  .item-row:hover td {
    background: rgba(255, 255, 255, 0.03);
  }

  .col-icon {
    width: 24px;
    text-align: center;
  }
  .row-favicon {
    width: 14px;
    height: 14px;
    object-fit: contain;
  }
  .star-icon {
    color: var(--color-accent, #ff5e9a);
  }

  .col-title {
    max-width: 200px;
  }
  .link-title {
    background: none;
    border: none;
    padding: 0;
    color: var(--color-text-main, #e0e0e0);
    font-weight: 500;
    cursor: pointer;
    text-align: left;
    font-size: 12px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    display: block;
    width: 100%;
  }
  .link-title:hover {
    color: var(--color-accent, #ff5e9a);
  }

  .col-url {
    max-width: 220px;
  }
  .url-text {
    color: var(--color-text-muted, #888);
    text-decoration: none;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    display: block;
  }
  .url-text:hover {
    color: var(--color-accent, #ff5e9a);
    text-decoration: underline;
  }

  .col-tags {
    max-width: 160px;
  }
  .tags-row {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }
  .item-tag {
    font-size: 10px;
    background: rgba(255, 255, 255, 0.06);
    padding: 2px 5px;
    border-radius: 4px;
    color: var(--color-accent, #ff5e9a);
  }

  .col-actions {
    width: 70px;
    text-align: right;
  }
  .btn-row-action {
    background: none;
    border: none;
    cursor: pointer;
    padding: 2px 4px;
    font-size: 12px;
    border-radius: 4px;
    transition: background 0.15s ease;
  }
  .btn-row-action:hover {
    background: rgba(255, 255, 255, 0.1);
  }
  .btn-delete:hover {
    background: rgba(255, 50, 50, 0.2);
  }

  /* Modal Form */
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 10000;
  }

  .edit-modal-card {
    width: 440px;
    background: var(--color-bg-surface, #1e1e24);
    border: 1px solid var(--color-border, rgba(255, 255, 255, 0.15));
    border-radius: var(--radius-md, 8px);
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.6);
    padding: 20px;
  }

  .edit-modal-card h3 {
    margin: 0 0 16px 0;
    font-size: 15px;
    color: var(--color-text-main, #e0e0e0);
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-bottom: 12px;
  }

  .form-group label {
    font-size: 12px;
    color: var(--color-text-muted, #888);
  }

  .form-group input,
  .form-group select {
    padding: 7px 10px;
    background: rgba(0, 0, 0, 0.3);
    border: 1px solid var(--color-border, rgba(255, 255, 255, 0.12));
    border-radius: var(--radius-sm, 4px);
    color: #fff;
    font-size: 12px;
  }
  .form-group input:focus,
  .form-group select:focus {
    outline: none;
    border-color: var(--color-accent, #ff5e9a);
  }

  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 16px;
  }

  .btn-cancel {
    padding: 6px 14px;
    background: transparent;
    border: 1px solid var(--color-border, rgba(255, 255, 255, 0.15));
    border-radius: var(--radius-sm, 4px);
    color: var(--color-text-muted, #888);
    font-size: 12px;
    cursor: pointer;
  }
  .btn-cancel:hover {
    background: rgba(255, 255, 255, 0.05);
    color: #fff;
  }

  .btn-save {
    padding: 6px 16px;
    background: var(--color-accent, #ff5e9a);
    border: 1px solid var(--color-accent, #ff5e9a);
    border-radius: var(--radius-sm, 4px);
    color: #fff;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
  }
  .btn-save:hover {
    opacity: 0.9;
  }
</style>
