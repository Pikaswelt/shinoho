<script lang="ts">
  import { t, getLocale } from '../i18n';
  import { incognitoStore } from './incognito/incognitoStore';
  import IncognitoBadge from './incognito/IncognitoBadge.svelte';

  interface Props {
    title?: string;
    onOpenIncognito?: () => void;
  }

  let { title = t('app.title'), onOpenIncognito }: Props = $props();

  let incognitoState = $state(incognitoStore.getState());

  $effect(() => {
    return incognitoStore.subscribe((s) => {
      incognitoState = s;
    });
  });

  const tr = (key: string, params?: Record<string, string | number>) => t(key, params, getLocale());

  function handleOpenIncognito() {
    if (onOpenIncognito) {
      onOpenIncognito();
    } else {
      incognitoStore.openIncognitoWindow();
    }
  }
</script>

<header class="app-header" class:is-incognito={incognitoState.isIncognito} data-testid="app-header">
  <div class="logo">
    <span class="logo-accent">{incognitoState.isIncognito ? '🥷' : '✦'}</span>
    <span class="logo-text">{title}</span>
    {#if incognitoState.isIncognito}
      <span class="stealth-tag">{tr('incognito.badge')}</span>
    {/if}
  </div>

  <div class="header-right">
    {#if incognitoState.isIncognito}
      <IncognitoBadge />
    {:else}
      <button
        type="button"
        class="btn-incognito-launch"
        title={tr('incognito.actions.newWindow')}
        onclick={handleOpenIncognito}
        data-testid="header-btn-incognito"
      >
        <span class="ninja-icon">🥷</span>
        <span class="launch-label">{tr('incognito.badge')}</span>
      </button>
    {/if}
  </div>
</header>

<style>
  .app-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 12px;
    height: 36px;
    background-color: var(--color-bg-surface);
    border-bottom: 1px solid var(--color-border);
    transition: all 0.2s ease;
  }

  .app-header.is-incognito {
    background-color: #1f1435;
    border-bottom-color: rgba(157, 78, 221, 0.3);
  }

  .logo {
    display: flex;
    align-items: center;
    gap: 6px;
    font-weight: 600;
    font-size: 13px;
  }

  .logo-accent {
    color: var(--color-accent);
  }

  .logo-text {
    letter-spacing: 0.5px;
  }

  .stealth-tag {
    font-size: 11px;
    font-weight: 700;
    padding: 2px 6px;
    background: rgba(157, 78, 221, 0.2);
    border: 1px solid rgba(157, 78, 221, 0.4);
    border-radius: 10px;
    color: #c77dff;
    margin-left: 6px;
  }

  .header-right {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .btn-incognito-launch {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 3px 8px;
    background: transparent;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm, 4px);
    color: var(--color-text-muted);
    font-size: 11px;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .btn-incognito-launch:hover {
    background: rgba(157, 78, 221, 0.15);
    border-color: #9d4edd;
    color: #c77dff;
  }

  .ninja-icon {
    font-size: 12px;
  }
</style>
