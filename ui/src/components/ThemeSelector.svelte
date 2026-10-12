<script lang="ts">
  import { onMount } from 'svelte';
  import { themeManager, type ThemeDefinition } from '../themes';
  import { t } from '../i18n';

  let currentTheme = $state<ThemeDefinition>(themeManager.getCurrentTheme());
  let allThemes = $state<ThemeDefinition[]>(themeManager.getAllThemes());
  let isOpen = $state(false);

  onMount(() => {
    currentTheme = themeManager.getCurrentTheme();
    allThemes = themeManager.getAllThemes();

    const unsubscribe = themeManager.onThemeChange((theme) => {
      currentTheme = theme;
      allThemes = themeManager.getAllThemes();
    });

    return unsubscribe;
  });

  function selectTheme(themeId: string) {
    try {
      currentTheme = themeManager.setTheme(themeId);
      isOpen = false;
    } catch (err) {
      console.error('Failed to set theme:', err);
    }
  }

  function toggleOpen() {
    isOpen = !isOpen;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && isOpen) {
      isOpen = false;
    }
  }
</script>

<div class="theme-selector" data-testid="theme-selector">
  <button
    type="button"
    class="theme-trigger-btn"
    class:open={isOpen}
    onclick={toggleOpen}
    onkeydown={handleKeydown}
    title={t('themes.selectTheme')}
    aria-expanded={isOpen}
    aria-haspopup="listbox"
    data-testid="theme-trigger-btn"
  >
    <span class="theme-color-dot" style:background-color={currentTheme.variables['--color-accent']}
    ></span>
    <span class="theme-name">
      {t(currentTheme.i18nKey ?? '') || currentTheme.name}
    </span>
    <span class="chevron">{isOpen ? '▲' : '▼'}</span>
  </button>

  {#if isOpen}
    <div class="theme-dropdown" role="listbox" data-testid="theme-dropdown">
      {#each allThemes as theme (theme.id)}
        <button
          type="button"
          class="theme-option"
          class:selected={theme.id === currentTheme.id}
          onclick={() => selectTheme(theme.id)}
          role="option"
          aria-selected={theme.id === currentTheme.id}
          data-testid={`theme-option-${theme.id}`}
        >
          <div class="theme-preview-dots">
            <span
              class="preview-dot dot-bg"
              style:background-color={theme.variables['--color-bg-base']}
              title="Base Background"
            ></span>
            <span
              class="preview-dot dot-accent"
              style:background-color={theme.variables['--color-accent']}
              title="Accent Color"
            ></span>
          </div>
          <span class="theme-option-name">
            {t(theme.i18nKey ?? '') || theme.name}
          </span>
          {#if theme.id === currentTheme.id}
            <span class="check-mark">✓</span>
          {/if}
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .theme-selector {
    position: relative;
    display: inline-flex;
    align-items: center;
  }

  .theme-trigger-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 26px;
    padding: 0 8px;
    background-color: var(--color-bg-base);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    color: var(--color-text-main);
    font-size: 11px;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .theme-trigger-btn:hover,
  .theme-trigger-btn.open {
    border-color: var(--color-accent);
    background-color: var(--color-bg-elevated);
  }

  .theme-color-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    display: inline-block;
    box-shadow: 0 0 4px rgba(0, 0, 0, 0.4);
  }

  .theme-name {
    font-weight: 500;
  }

  .chevron {
    font-size: 8px;
    opacity: 0.7;
    margin-left: 2px;
  }

  .theme-dropdown {
    position: absolute;
    top: calc(100% + 4px);
    right: 0;
    min-width: 170px;
    background-color: var(--color-bg-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
    padding: 4px;
    z-index: 100;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .theme-option {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 8px;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--color-text-main);
    font-size: 12px;
    text-align: left;
    width: 100%;
    cursor: pointer;
    transition: all 0.12s ease;
  }

  .theme-option:hover {
    background-color: var(--color-bg-elevated);
  }

  .theme-option.selected {
    background-color: var(--color-bg-elevated);
    color: var(--color-accent);
    font-weight: 600;
  }

  .theme-preview-dots {
    display: flex;
    align-items: center;
    gap: 3px;
  }

  .preview-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    display: inline-block;
    border: 1px solid var(--color-border);
  }

  .theme-option-name {
    flex: 1;
    white-space: nowrap;
  }

  .check-mark {
    font-size: 11px;
    color: var(--color-accent);
  }
</style>
