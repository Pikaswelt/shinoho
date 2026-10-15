<script lang="ts">
  import { t } from '../i18n';
  import { downloadStore } from './downloads/downloadStore';

  interface Props {
    url: string;
    canGoBack: boolean;
    canGoForward: boolean;
    isLoading: boolean;
    onNavigate: (url: string) => void;
    onBack: () => void;
    onForward: () => void;
    onReload: () => void;
    onHome: () => void;
  }

  let {
    url,
    canGoBack,
    canGoForward,
    isLoading,
    onNavigate,
    onBack,
    onForward,
    onReload,
    onHome,
  }: Props = $props();

  let userTyped = $state<string | null>(null);
  let isEditing = $state(false);

  let displayValue = $derived(userTyped !== null ? userTyped : url);

  function normalizeUrl(input: string): string {
    const trimmed = input.trim();
    if (!trimmed) return 'shinoho://hub';

    if (trimmed.startsWith('shinoho://') || trimmed.startsWith('about:')) {
      return trimmed;
    }

    if (/^https?:\/\//i.test(trimmed)) {
      return trimmed;
    }

    // Check if domain like example.com or localhost
    if (
      /^[a-zA-Z0-9-]+(\.[a-zA-Z0-9-]+)+(:\d+)?(\/.*)?$/.test(trimmed) ||
      /^localhost(:\d+)?(\/.*)?$/.test(trimmed)
    ) {
      return `https://${trimmed}`;
    }

    // Default to search engine
    return `https://duckduckgo.com/?q=${encodeURIComponent(trimmed)}`;
  }

  function handleSubmit(e: Event) {
    e.preventDefault();
    isEditing = false;
    const finalUrl = userTyped !== null ? userTyped : url;
    userTyped = null;
    onNavigate(normalizeUrl(finalUrl));
  }

  function handleFocus() {
    isEditing = true;
  }

  function handleBlur() {
    isEditing = false;
    userTyped = null;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      userTyped = null;
      isEditing = false;
    }
  }
</script>

<div class="address-bar" data-testid="address-bar">
  <div class="nav-controls">
    <button
      type="button"
      class="nav-btn"
      disabled={!canGoBack}
      title={t('nav.back')}
      onclick={onBack}
      data-testid="nav-back"
    >
      ←
    </button>
    <button
      type="button"
      class="nav-btn"
      disabled={!canGoForward}
      title={t('nav.forward')}
      onclick={onForward}
      data-testid="nav-forward"
    >
      →
    </button>
    <button
      type="button"
      class="nav-btn"
      title={t('nav.reload')}
      onclick={onReload}
      data-testid="nav-reload"
    >
      {#if isLoading}
        <span class="spinning">⟳</span>
      {:else}
        <span>↻</span>
      {/if}
    </button>
    <button
      type="button"
      class="nav-btn"
      title={t('nav.home')}
      onclick={onHome}
      data-testid="nav-home"
    >
      ⌂
    </button>
  </div>

  <form class="omnibox-form" onsubmit={handleSubmit}>
    <div class="omnibox-container" class:focused={isEditing}>
      <span class="security-badge" title="Security Status">
        {#if url.startsWith('https://')}
          <span class="badge-secure">🔒</span>
        {:else if url.startsWith('shinoho://')}
          <span class="badge-internal">✦</span>
        {:else}
          <span class="badge-standard">ℹ</span>
        {/if}
      </span>

      <input
        type="text"
        class="omnibox-input"
        value={displayValue}
        placeholder={t('addressbar.placeholder')}
        oninput={(e) => {
          userTyped = (e.target as HTMLInputElement).value;
        }}
        onfocus={handleFocus}
        onblur={handleBlur}
        onkeydown={handleKeydown}
        data-testid="address-input"
      />

      <button type="submit" class="go-btn" title={t('addressbar.go')} data-testid="address-go">
        ↵
      </button>
    </div>
  </form>

  <div class="quick-actions">
    <button
      type="button"
      class="action-btn"
      title={t('common.bookmarks')}
      data-testid="action-bookmarks"
    >
      ★
    </button>
    <button
      type="button"
      class="action-btn"
      title={t('downloads.title')}
      onclick={() => downloadStore.openManager()}
      data-testid="action-downloads"
    >
      ⬇
    </button>
  </div>
</div>

<style>
  .address-bar {
    display: flex;
    align-items: center;
    gap: 8px;
    height: var(--toolbar-height, 44px);
    background-color: var(--color-bg-surface);
    padding: 0 10px;
    border-bottom: 1px solid var(--color-border);
  }

  .nav-controls {
    display: flex;
    align-items: center;
    gap: 2px;
  }

  .nav-btn,
  .action-btn {
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 14px;
    border-radius: var(--radius-sm);
    color: var(--color-text-main);
    transition: all 0.15s ease;
  }

  .nav-btn:hover:not(:disabled),
  .action-btn:hover {
    background-color: var(--color-bg-elevated);
    color: var(--color-accent);
  }

  .nav-btn:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }

  .spinning {
    display: inline-block;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }

  .omnibox-form {
    flex: 1;
    display: flex;
  }

  .omnibox-container {
    width: 100%;
    display: flex;
    align-items: center;
    background-color: var(--color-bg-base);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    padding: 0 8px;
    height: 30px;
    transition: all 0.15s ease;
  }

  .omnibox-container.focused {
    border-color: var(--color-accent);
    box-shadow: 0 0 0 2px rgba(255, 71, 133, 0.2);
  }

  .security-badge {
    font-size: 12px;
    margin-right: 6px;
    display: flex;
    align-items: center;
  }

  .badge-internal {
    color: var(--color-accent);
  }

  .badge-secure {
    color: #4ade80;
  }

  .omnibox-input {
    flex: 1;
    background: transparent;
    border: none;
    outline: none;
    font-size: 13px;
    color: var(--color-text-main);
  }

  .go-btn {
    font-size: 13px;
    color: var(--color-text-muted);
    padding: 0 4px;
    transition: color 0.15s ease;
  }

  .go-btn:hover {
    color: var(--color-accent);
  }

  .quick-actions {
    display: flex;
    align-items: center;
  }
</style>
