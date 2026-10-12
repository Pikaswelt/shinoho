<script lang="ts">
  import type { SpeedDialItem } from './hub.types';
  import { hubStore } from './hubStore';
  import { tHub } from './i18n';

  let { onNavigate }: { onNavigate?: (url: string) => void } = $props();

  let hubState = $state(hubStore.getState());
  hubStore.subscribe((s) => {
    hubState = s;
  });

  let selectedCategory = $state('all');
  let isAddModalOpen = $state(false);
  let editItemId: string | null = $state(null);

  let formTitle = $state('');
  let formUrl = $state('');
  let formCategory = $state('anime');

  const categories = ['all', 'anime', 'gaming', 'dev', 'social'] as const;

  let filteredItems = $derived(
    hubState.speedDial
      .filter(
        (item: SpeedDialItem) => selectedCategory === 'all' || item.category === selectedCategory,
      )
      .sort((a: SpeedDialItem, b: SpeedDialItem) => {
        if (a.pinned !== b.pinned) return a.pinned ? -1 : 1;
        return (b.clicks || 0) - (a.clicks || 0);
      }),
  );

  function handleClick(item: SpeedDialItem, e: MouseEvent) {
    hubStore.recordSpeedDialClick(item.id);
    if (onNavigate) {
      e.preventDefault();
      onNavigate(item.url);
    }
  }

  function openAddModal() {
    editItemId = null;
    formTitle = '';
    formUrl = '';
    formCategory = selectedCategory !== 'all' ? selectedCategory : 'anime';
    isAddModalOpen = true;
  }

  function openEditModal(item: SpeedDialItem, e: Event) {
    e.stopPropagation();
    e.preventDefault();
    editItemId = item.id;
    formTitle = item.title;
    formUrl = item.url;
    formCategory = item.category || 'anime';
    isAddModalOpen = true;
  }

  function handleSaveShortcut(e: Event) {
    e.preventDefault();
    if (!formTitle.trim() || !formUrl.trim()) return;

    let url = formUrl.trim();
    if (!url.startsWith('http://') && !url.startsWith('https://')) {
      url = `https://${url}`;
    }

    if (editItemId) {
      hubStore.updateSpeedDial(editItemId, {
        title: formTitle.trim(),
        url,
        category: formCategory,
      });
    } else {
      hubStore.addSpeedDial({
        title: formTitle.trim(),
        url,
        category: formCategory,
        pinned: false,
      });
    }

    isAddModalOpen = false;
  }

  function handleDelete(id: string, e: Event) {
    e.stopPropagation();
    e.preventDefault();
    if (confirm(tHub('hub.speedDial.deleteConfirm'))) {
      hubStore.removeSpeedDial(id);
    }
  }

  function handleTogglePin(id: string, e: Event) {
    e.stopPropagation();
    e.preventDefault();
    hubStore.togglePin(id);
  }

  function getDomain(url: string): string {
    try {
      return new URL(url).hostname.replace('www.', '');
    } catch {
      return url;
    }
  }
</script>

<div class="speed-dial-section" data-testid="speed-dial-section">
  <!-- Categories navigation -->
  <div class="category-tabs">
    {#each categories as cat}
      <button
        type="button"
        class="cat-tab"
        class:active={selectedCategory === cat}
        onclick={() => (selectedCategory = cat)}
        data-testid={`category-tab-${cat}`}
      >
        {tHub(`hub.speedDial.categories.${cat}`)}
      </button>
    {/each}
  </div>

  <!-- Grid of tiles -->
  <div class="tiles-grid" data-testid="speed-dial-grid">
    {#each filteredItems as item (item.id)}
      <a
        href={item.url}
        class="tile"
        class:pinned={item.pinned}
        data-testid={`speed-dial-item-${item.id}`}
        onclick={(e) => handleClick(item, e)}
      >
        <div class="tile-actions">
          <button
            type="button"
            class="action-btn pin-btn"
            title={item.pinned
              ? tHub('hub.speedDial.unpinShortcut')
              : tHub('hub.speedDial.pinShortcut')}
            onclick={(e) => handleTogglePin(item.id, e)}
          >
            {item.pinned ? '📌' : '📍'}
          </button>
          <button
            type="button"
            class="action-btn edit-btn"
            title={tHub('hub.speedDial.editShortcut')}
            onclick={(e) => openEditModal(item, e)}
          >
            ✏️
          </button>
          <button
            type="button"
            class="action-btn delete-btn"
            title={tHub('hub.speedDial.deleteShortcut')}
            onclick={(e) => handleDelete(item.id, e)}
          >
            ✕
          </button>
        </div>

        <div class="tile-icon-wrapper">
          {#if item.icon}
            <span class="custom-icon">{item.icon}</span>
          {:else}
            <img
              src={`https://www.google.com/s2/favicons?domain=${getDomain(item.url)}&sz=64`}
              alt={item.title}
              class="favicon-img"
              onerror={(e) => ((e.currentTarget as HTMLElement).style.display = 'none')}
            />
          {/if}
        </div>

        <div class="tile-info">
          <span class="tile-title">{item.title}</span>
          <span class="tile-domain">{getDomain(item.url)}</span>
        </div>
      </a>
    {/each}

    <!-- Add shortcut card -->
    <button
      type="button"
      class="tile add-tile"
      data-testid="add-speed-dial-button"
      onclick={openAddModal}
    >
      <div class="add-icon">+</div>
      <span class="add-text">{tHub('hub.speedDial.addShortcut')}</span>
    </button>
  </div>
</div>

<!-- Add/Edit Modal Dialog -->
{#if isAddModalOpen}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="modal-backdrop" onclick={() => (isAddModalOpen = false)}>
    <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
    <div class="modal-card" onclick={(e) => e.stopPropagation()} data-testid="speed-dial-modal">
      <h3>{editItemId ? tHub('hub.speedDial.editShortcut') : tHub('hub.speedDial.addShortcut')}</h3>

      <form onsubmit={handleSaveShortcut}>
        <div class="form-group">
          <label for="sd-title">{tHub('hub.speedDial.nameLabel')}</label>
          <input
            id="sd-title"
            type="text"
            required
            placeholder="z.B. AniList, Steam..."
            bind:value={formTitle}
          />
        </div>

        <div class="form-group">
          <label for="sd-url">{tHub('hub.speedDial.urlLabel')}</label>
          <input
            id="sd-url"
            type="text"
            required
            placeholder="https://example.com"
            bind:value={formUrl}
          />
        </div>

        <div class="form-group">
          <label for="sd-category">{tHub('hub.speedDial.categoryLabel')}</label>
          <select id="sd-category" bind:value={formCategory}>
            <option value="anime">{tHub('hub.speedDial.categories.anime')}</option>
            <option value="gaming">{tHub('hub.speedDial.categories.gaming')}</option>
            <option value="dev">{tHub('hub.speedDial.categories.dev')}</option>
            <option value="social">{tHub('hub.speedDial.categories.social')}</option>
          </select>
        </div>

        <div class="modal-actions">
          <button type="button" class="btn-cancel" onclick={() => (isAddModalOpen = false)}>
            {tHub('hub.speedDial.cancel')}
          </button>
          <button type="submit" class="btn-save">
            {tHub('hub.speedDial.save')}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}

<style>
  .speed-dial-section {
    width: 100%;
    max-width: 900px;
    margin: 24px auto 0 auto;
    position: relative;
    z-index: 10;
  }

  .category-tabs {
    display: flex;
    justify-content: center;
    gap: 8px;
    margin-bottom: 20px;
  }

  .cat-tab {
    background: rgba(24, 24, 34, 0.6);
    backdrop-filter: blur(8px);
    border: 1px solid var(--color-border, #2a2a3c);
    color: var(--color-text-muted, #8b8b9e);
    padding: 6px 14px;
    border-radius: 20px;
    font-size: 13px;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .cat-tab:hover {
    color: var(--color-text-main, #f0f0f5);
    background: rgba(255, 255, 255, 0.08);
  }

  .cat-tab.active {
    background: var(--color-accent, #ff4785);
    border-color: var(--color-accent, #ff4785);
    color: white;
    font-weight: 600;
    box-shadow: 0 4px 14px rgba(255, 71, 133, 0.35);
  }

  .tiles-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(130px, 1fr));
    gap: 16px;
    padding: 4px;
  }

  .tile {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    background: rgba(24, 24, 34, 0.65);
    backdrop-filter: blur(10px);
    border: 1px solid var(--color-border, #2a2a3c);
    border-radius: var(--radius-md, 8px);
    padding: 16px 12px 12px;
    text-decoration: none;
    position: relative;
    cursor: pointer;
    transition:
      transform 0.2s ease,
      border-color 0.2s ease,
      box-shadow 0.2s ease,
      background 0.2s ease;
  }

  .tile:hover {
    transform: translateY(-4px);
    border-color: rgba(255, 71, 133, 0.6);
    background: rgba(34, 34, 50, 0.85);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
  }

  .tile.pinned {
    border-color: rgba(255, 71, 133, 0.35);
  }

  .tile-actions {
    position: absolute;
    top: 6px;
    right: 6px;
    display: flex;
    gap: 2px;
    opacity: 0;
    transition: opacity 0.15s ease;
  }

  .tile:hover .tile-actions {
    opacity: 1;
  }

  .action-btn {
    background: rgba(15, 15, 20, 0.85);
    border: none;
    color: var(--color-text-muted, #8b8b9e);
    width: 22px;
    height: 22px;
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 11px;
    cursor: pointer;
    transition:
      color 0.15s,
      background 0.15s;
  }

  .action-btn:hover {
    color: white;
    background: var(--color-accent, #ff4785);
  }

  .tile-icon-wrapper {
    width: 48px;
    height: 48px;
    border-radius: 12px;
    background: rgba(255, 255, 255, 0.05);
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: 10px;
    overflow: hidden;
  }

  .custom-icon {
    font-size: 26px;
  }

  .favicon-img {
    width: 28px;
    height: 28px;
    border-radius: 4px;
  }

  .tile-info {
    text-align: center;
    width: 100%;
    overflow: hidden;
  }

  .tile-title {
    display: block;
    color: var(--color-text-main, #f0f0f5);
    font-size: 13px;
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .tile-domain {
    display: block;
    color: var(--color-text-muted, #8b8b9e);
    font-size: 11px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-top: 2px;
  }

  .add-tile {
    border: 1px dashed var(--color-border, #2a2a3c);
    background: rgba(24, 24, 34, 0.35);
  }

  .add-tile:hover {
    border-color: var(--color-accent, #ff4785);
    background: rgba(255, 71, 133, 0.08);
  }

  .add-icon {
    font-size: 28px;
    color: var(--color-accent, #ff4785);
    margin-bottom: 6px;
  }

  .add-text {
    font-size: 12px;
    color: var(--color-text-muted, #8b8b9e);
  }

  /* Modal */
  .modal-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    width: 100vw;
    height: 100vh;
    background: rgba(0, 0, 0, 0.7);
    backdrop-filter: blur(6px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }

  .modal-card {
    background: var(--color-bg-surface, #181822);
    border: 1px solid var(--color-border, #2a2a3c);
    border-radius: var(--radius-lg, 12px);
    padding: 24px;
    width: 100%;
    max-width: 420px;
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.5);
  }

  .modal-card h3 {
    margin: 0 0 18px 0;
    color: var(--color-text-main, #f0f0f5);
    font-size: 18px;
  }

  .form-group {
    margin-bottom: 14px;
  }

  .form-group label {
    display: block;
    font-size: 12px;
    color: var(--color-text-muted, #8b8b9e);
    margin-bottom: 6px;
  }

  .form-group input,
  .form-group select {
    width: 100%;
    padding: 8px 12px;
    background: var(--color-bg-base, #0f0f14);
    border: 1px solid var(--color-border, #2a2a3c);
    border-radius: var(--radius-sm, 4px);
    color: var(--color-text-main, #f0f0f5);
    font-size: 14px;
    box-sizing: border-box;
  }

  .form-group input:focus,
  .form-group select:focus {
    outline: none;
    border-color: var(--color-accent, #ff4785);
  }

  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    margin-top: 20px;
  }

  .btn-cancel {
    background: transparent;
    border: 1px solid var(--color-border, #2a2a3c);
    color: var(--color-text-muted, #8b8b9e);
    padding: 8px 16px;
    border-radius: var(--radius-sm, 4px);
    cursor: pointer;
  }

  .btn-save {
    background: var(--color-accent, #ff4785);
    border: none;
    color: white;
    padding: 8px 18px;
    border-radius: var(--radius-sm, 4px);
    font-weight: 600;
    cursor: pointer;
  }

  .btn-save:hover {
    background: var(--color-accent-hover, #ff6599);
  }
</style>
