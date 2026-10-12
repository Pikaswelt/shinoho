<script lang="ts">
  import { SEARCH_ENGINES, hubStore } from './hubStore';
  import { tHub } from './i18n';

  let { onNavigate }: { onNavigate?: (url: string) => void } = $props();

  let query = $state('');
  let isMenuOpen = $state(false);
  let hubState = $state(hubStore.getState());

  hubStore.subscribe((s) => {
    hubState = s;
  });

  let selectedEngine = $derived(
    SEARCH_ENGINES.find((e) => e.id === hubState.selectedSearchEngine) || SEARCH_ENGINES[0],
  );

  function handleSubmit(e: Event) {
    e.preventDefault();
    const trimmed = query.trim();
    if (!trimmed) return;

    let targetUrl: string;
    if (trimmed.startsWith('http://') || trimmed.startsWith('https://')) {
      targetUrl = trimmed;
    } else if (trimmed.includes('.') && !trimmed.includes(' ')) {
      targetUrl = `https://${trimmed}`;
    } else {
      targetUrl = hubStore.buildSearchUrl(trimmed);
    }

    if (onNavigate) {
      onNavigate(targetUrl);
    } else {
      window.location.href = targetUrl;
    }
  }

  function selectEngine(id: string) {
    hubStore.setSelectedSearchEngine(id);
    isMenuOpen = false;
  }
</script>

<div class="search-bar-wrapper" data-testid="search-bar">
  <form class="search-form" onsubmit={handleSubmit}>
    <div class="engine-selector">
      <button
        type="button"
        class="engine-btn"
        data-testid="engine-button"
        onclick={() => (isMenuOpen = !isMenuOpen)}
        title={tHub('hub.search.searchWith', { engine: selectedEngine.name })}
      >
        <span class="engine-icon">{selectedEngine.icon}</span>
        <span class="engine-arrow">▾</span>
      </button>

      {#if isMenuOpen}
        <div class="engine-dropdown" data-testid="engine-dropdown">
          {#each SEARCH_ENGINES as engine}
            <button
              type="button"
              class="engine-option"
              class:selected={engine.id === selectedEngine.id}
              onclick={() => selectEngine(engine.id)}
            >
              <span class="option-icon">{engine.icon}</span>
              <span class="option-name">{engine.name}</span>
            </button>
          {/each}
        </div>
      {/if}
    </div>

    <input
      type="text"
      class="search-input"
      data-testid="search-input"
      placeholder={tHub('hub.search.placeholder')}
      bind:value={query}
    />

    <button
      type="submit"
      class="submit-btn"
      data-testid="search-submit"
      aria-label="Suchen"
      title="Suchen"
    >
      <svg
        viewBox="0 0 24 24"
        width="18"
        height="18"
        fill="none"
        stroke="currentColor"
        stroke-width="2.5"
      >
        <circle cx="11" cy="11" r="7" />
        <line x1="21" y1="21" x2="16.65" y2="16.65" />
      </svg>
    </button>
  </form>
</div>

<style>
  .search-bar-wrapper {
    width: 100%;
    max-width: 620px;
    margin: 0 auto;
    position: relative;
    z-index: 10;
  }

  .search-form {
    display: flex;
    align-items: center;
    background: rgba(24, 24, 34, 0.75);
    backdrop-filter: blur(12px);
    border: 1px solid var(--color-border, #2a2a3c);
    border-radius: var(--radius-lg, 12px);
    padding: 6px 10px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.35);
    transition:
      border-color 0.2s ease,
      box-shadow 0.2s ease;
  }

  .search-form:focus-within {
    border-color: var(--color-accent, #ff4785);
    box-shadow: 0 8px 32px rgba(255, 71, 133, 0.25);
  }

  .engine-selector {
    position: relative;
  }

  .engine-btn {
    background: transparent;
    border: none;
    color: var(--color-text-main, #f0f0f5);
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 6px 8px;
    border-radius: var(--radius-sm, 4px);
    transition: background 0.15s ease;
  }

  .engine-btn:hover {
    background: rgba(255, 255, 255, 0.08);
  }

  .engine-icon {
    font-size: 18px;
  }

  .engine-arrow {
    font-size: 11px;
    color: var(--color-text-muted, #8b8b9e);
  }

  .engine-dropdown {
    position: absolute;
    top: calc(100% + 8px);
    left: 0;
    background: var(--color-bg-surface, #181822);
    border: 1px solid var(--color-border, #2a2a3c);
    border-radius: var(--radius-md, 8px);
    box-shadow: 0 10px 25px rgba(0, 0, 0, 0.4);
    min-width: 150px;
    z-index: 50;
    overflow: hidden;
  }

  .engine-option {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 8px 12px;
    background: transparent;
    border: none;
    color: var(--color-text-main, #f0f0f5);
    font-size: 13px;
    cursor: pointer;
    text-align: left;
    transition: background 0.15s ease;
  }

  .engine-option:hover {
    background: rgba(255, 255, 255, 0.08);
  }

  .engine-option.selected {
    background: rgba(255, 71, 133, 0.15);
    color: var(--color-accent, #ff4785);
    font-weight: 600;
  }

  .search-input {
    flex: 1;
    background: transparent;
    border: none;
    outline: none;
    color: var(--color-text-main, #f0f0f5);
    font-size: 15px;
    padding: 6px 12px;
  }

  .search-input::placeholder {
    color: var(--color-text-muted, #8b8b9e);
  }

  .submit-btn {
    background: var(--color-accent, #ff4785);
    border: none;
    color: white;
    cursor: pointer;
    width: 36px;
    height: 36px;
    border-radius: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition:
      background 0.15s ease,
      transform 0.1s ease;
  }

  .submit-btn:hover {
    background: var(--color-accent-hover, #ff6599);
    transform: scale(1.04);
  }

  .submit-btn:active {
    background: var(--color-accent-active, #e0326e);
    transform: scale(0.96);
  }
</style>
