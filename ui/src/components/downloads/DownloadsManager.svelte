<script lang="ts">
  import { onMount } from 'svelte';
  import { t } from '../../i18n';
  import {
    listDownloads,
    pauseDownload,
    resumeDownload,
    cancelDownload,
    retryDownload,
    deleteDownload,
    clearCompletedDownloads,
    onDownloadEvent,
    type DownloadItem,
    type DownloadState,
  } from '../../api/downloads';
  import { downloadStore } from './downloadStore';

  let uiState = $state(downloadStore.getState());
  let items = $state<DownloadItem[]>([]);
  let isLoading = $state(false);
  let isOpen = $derived(uiState.isManagerOpen);

  async function refresh(): Promise<void> {
    isLoading = true;
    try {
      items = await listDownloads(100, 0);
    } finally {
      isLoading = false;
    }
  }

  onMount(() => {
    const unsubStore = downloadStore.subscribe((s) => {
      const wasOpen = uiState.isManagerOpen;
      uiState = s;
      if (s.isManagerOpen && !wasOpen) {
        void refresh();
      }
    });
    const unsubEvents = onDownloadEvent(() => {
      if (downloadStore.isManagerOpen) {
        void refresh();
      }
    });
    return () => {
      unsubStore();
      unsubEvents();
    };
  });

  function stateLabel(state: DownloadState): string {
    return t(`downloads.states.${state}`);
  }

  function progressPercent(item: DownloadItem): number | null {
    if (item.total_bytes && item.total_bytes > 0) {
      return Math.min(100, Math.max(0, (item.received_bytes / item.total_bytes) * 100));
    }
    return null;
  }

  function formatBytes(bytes: number | null): string {
    if (bytes === null || bytes === undefined) return '–';
    if (bytes < 1024) return `${bytes} B`;
    const units = ['KB', 'MB', 'GB'];
    let value = bytes / 1024;
    let unit = 0;
    while (value >= 1024 && unit < units.length - 1) {
      value /= 1024;
      unit += 1;
    }
    return `${value.toFixed(1)} ${units[unit]}`;
  }

  function formatSpeed(item: DownloadItem): string {
    if (item.state !== 'downloading' || !item.speed_bps) return '';
    return `${formatBytes(item.speed_bps)}/s`;
  }

  async function handlePause(id: string) {
    await pauseDownload(id);
    await refresh();
  }

  async function handleResume(id: string) {
    await resumeDownload(id);
    await refresh();
  }

  async function handleCancel(id: string) {
    await cancelDownload(id);
    await refresh();
  }

  async function handleRetry(id: string) {
    await retryDownload(id);
    await refresh();
  }

  async function handleDelete(id: string) {
    if (!confirm(t('downloads.deleteConfirm'))) return;
    await deleteDownload(id, false);
    await refresh();
  }

  async function handleClearCompleted() {
    await clearCompletedDownloads();
    await refresh();
  }

  function handleClose() {
    downloadStore.closeManager();
  }

  let hasCompleted = $derived(items.some((i) => i.state === 'completed'));
  let activeCount = $derived(
    items.filter((i) => i.state === 'downloading' || i.state === 'pending').length,
  );
</script>

{#if isOpen}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="dl-backdrop" data-testid="downloads-manager" onclick={handleClose}>
    <div
      class="dl-modal"
      role="dialog"
      tabindex="-1"
      aria-modal="true"
      aria-label={t('downloads.title')}
      onclick={(e) => e.stopPropagation()}
    >
      <div class="dl-header">
        <h2>
          {t('downloads.title')}
          {#if activeCount > 0}
            <span class="dl-active-badge" data-testid="downloads-active-count">{activeCount}</span>
          {/if}
        </h2>
        <button
          type="button"
          class="dl-close"
          title={t('downloads.close')}
          onclick={handleClose}
          data-testid="downloads-close"
        >
          ×
        </button>
      </div>

      <div class="dl-body">
        {#if isLoading && items.length === 0}
          <p class="dl-empty">{t('downloads.loading')}</p>
        {:else if items.length === 0}
          <p class="dl-empty" data-testid="downloads-empty">{t('downloads.empty')}</p>
        {:else}
          <ul class="dl-list">
            {#each items as item (item.id)}
              {@const percent = progressPercent(item)}
              <li class="dl-item" data-testid={`download-item-${item.id}`}>
                <div class="dl-item-main">
                  <span class="dl-filename" title={item.filename}>{item.filename}</span>
                  <span class="dl-state dl-state-{item.state}">{stateLabel(item.state)}</span>
                </div>
                <div class="dl-progress" role="progressbar" aria-valuenow={percent ?? undefined}>
                  {#if percent !== null}
                    <div class="dl-progress-fill" style={`width: ${percent}%`}></div>
                  {:else}
                    <div class="dl-progress-fill dl-indeterminate"></div>
                  {/if}
                </div>
                <div class="dl-meta">
                  <span>{formatBytes(item.received_bytes)} / {formatBytes(item.total_bytes)}</span>
                  {#if formatSpeed(item)}
                    <span>{formatSpeed(item)}</span>
                  {/if}
                  {#if item.error_message}
                    <span class="dl-error" title={item.error_message}>{item.error_message}</span>
                  {/if}
                </div>
                <div class="dl-actions">
                  {#if item.state === 'downloading' || item.state === 'pending'}
                    <button type="button" onclick={() => handlePause(item.id)}>
                      {t('downloads.pause')}
                    </button>
                    <button type="button" onclick={() => handleCancel(item.id)}>
                      {t('downloads.cancel')}
                    </button>
                  {:else if item.state === 'paused'}
                    <button type="button" onclick={() => handleResume(item.id)}>
                      {t('downloads.resume')}
                    </button>
                    <button type="button" onclick={() => handleCancel(item.id)}>
                      {t('downloads.cancel')}
                    </button>
                  {:else if item.state === 'failed' || item.state === 'cancelled'}
                    <button type="button" onclick={() => handleRetry(item.id)}>
                      {t('downloads.retry')}
                    </button>
                    <button type="button" onclick={() => handleDelete(item.id)}>
                      {t('downloads.delete')}
                    </button>
                  {:else if item.state === 'completed'}
                    <button type="button" onclick={() => handleDelete(item.id)}>
                      {t('downloads.delete')}
                    </button>
                  {/if}
                </div>
              </li>
            {/each}
          </ul>
        {/if}
      </div>

      {#if hasCompleted}
        <div class="dl-footer">
          <button
            type="button"
            onclick={handleClearCompleted}
            data-testid="downloads-clear-completed"
          >
            {t('downloads.clearCompleted')}
          </button>
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .dl-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.7);
    backdrop-filter: blur(6px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }

  .dl-modal {
    background: var(--color-bg-surface, #181822);
    border: 1px solid var(--color-border, #2a2a3c);
    border-radius: var(--radius-lg, 12px);
    width: min(560px, calc(100vw - 48px));
    max-height: min(640px, calc(100vh - 96px));
    display: flex;
    flex-direction: column;
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.5);
  }

  .dl-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 20px;
    border-bottom: 1px solid var(--color-border, #2a2a3c);
  }

  .dl-header h2 {
    margin: 0;
    font-size: 16px;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .dl-active-badge {
    font-size: 11px;
    font-weight: 700;
    background: var(--color-accent, #ff4785);
    color: #fff;
    border-radius: 10px;
    padding: 1px 8px;
  }

  .dl-close {
    font-size: 20px;
    color: var(--color-text-muted, #8b8b9e);
    padding: 2px 8px;
    border-radius: var(--radius-sm, 4px);
  }

  .dl-close:hover {
    color: #fff;
    background: rgba(255, 255, 255, 0.08);
  }

  .dl-body {
    padding: 12px 20px;
    overflow-y: auto;
  }

  .dl-empty {
    text-align: center;
    color: var(--color-text-muted, #8b8b9e);
    font-size: 13px;
    padding: 24px 0;
  }

  .dl-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .dl-item {
    border: 1px solid var(--color-border, #2a2a3c);
    border-radius: var(--radius-md, 8px);
    padding: 10px 12px;
    background: rgba(255, 255, 255, 0.02);
  }

  .dl-item-main {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .dl-filename {
    font-size: 13px;
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .dl-state {
    font-size: 11px;
    padding: 1px 8px;
    border-radius: 10px;
    flex-shrink: 0;
  }

  .dl-state-downloading,
  .dl-state-pending {
    background: rgba(74, 222, 128, 0.15);
    color: #4ade80;
  }

  .dl-state-paused {
    background: rgba(250, 204, 21, 0.15);
    color: #facc15;
  }

  .dl-state-completed {
    background: rgba(255, 71, 133, 0.15);
    color: var(--color-accent, #ff4785);
  }

  .dl-state-failed,
  .dl-state-cancelled {
    background: rgba(248, 113, 113, 0.15);
    color: #f87171;
  }

  .dl-progress {
    height: 6px;
    border-radius: 3px;
    background: rgba(255, 255, 255, 0.08);
    margin-top: 8px;
    overflow: hidden;
  }

  .dl-progress-fill {
    height: 100%;
    background: var(--color-accent, #ff4785);
    transition: width 0.3s ease;
  }

  .dl-indeterminate {
    width: 40%;
    animation: dl-slide 1.2s ease-in-out infinite alternate;
  }

  @keyframes dl-slide {
    from {
      margin-left: 0;
    }
    to {
      margin-left: 60%;
    }
  }

  .dl-meta {
    display: flex;
    gap: 12px;
    font-size: 11px;
    color: var(--color-text-muted, #8b8b9e);
    margin-top: 6px;
    flex-wrap: wrap;
  }

  .dl-error {
    color: #f87171;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 100%;
  }

  .dl-actions {
    display: flex;
    gap: 8px;
    margin-top: 8px;
  }

  .dl-actions button {
    font-size: 12px;
    padding: 4px 12px;
    border: 1px solid var(--color-border, #2a2a3c);
    border-radius: var(--radius-sm, 4px);
    color: var(--color-text-main, #f0f0f5);
  }

  .dl-actions button:hover {
    border-color: var(--color-accent, #ff4785);
    color: var(--color-accent, #ff4785);
  }

  .dl-footer {
    padding: 12px 20px;
    border-top: 1px solid var(--color-border, #2a2a3c);
    display: flex;
    justify-content: flex-end;
  }

  .dl-footer button {
    font-size: 12px;
    padding: 6px 14px;
    border: 1px solid var(--color-border, #2a2a3c);
    border-radius: var(--radius-sm, 4px);
    color: var(--color-text-muted, #8b8b9e);
  }

  .dl-footer button:hover {
    border-color: var(--color-accent, #ff4785);
    color: var(--color-accent, #ff4785);
  }
</style>
