<script lang="ts">
  import { onMount } from 'svelte';
  import { gamingService, type GamingStatus } from '../../api/gaming';
  import { t, getLocale, subscribeLocale } from '../../i18n';

  let status = $state<GamingStatus>({
    active: false,
    game: null,
    auto: true,
    preset: 'balanced',
  });
  let dismissedFor = $state<string | null>(null);
  let locale = $state(getLocale());

  const tr = (key: string, params?: Record<string, string | number>) => t(key, params, locale);
  const visible = $derived(status.active && dismissedFor !== status.game);

  onMount(() => {
    void gamingService.getStatus().then((s) => (status = s));
    const off = gamingService.onChange((s) => (status = s));
    const offLocale = subscribeLocale((l) => (locale = l));
    return () => {
      off();
      offLocale();
    };
  });

  async function toggleAuto() {
    status = await gamingService.setAuto(!status.auto);
  }
</script>

{#if visible}
  <div class="gaming-banner" role="status" data-testid="gaming-banner">
    <span class="icon" aria-hidden="true">🎮</span>
    <span class="text">
      <strong>{tr('gaming.active', { game: status.game ?? '' })}</strong>
      <span class="hint">
        {status.auto ? tr('gaming.savingResources') : tr('gaming.autoOff')}
      </span>
    </span>
    <label class="auto">
      <input
        type="checkbox"
        checked={status.auto}
        onchange={toggleAuto}
        data-testid="gaming-auto-toggle"
      />
      {tr('gaming.autoLabel')}
    </label>
    <button
      type="button"
      class="close"
      aria-label={tr('gaming.dismiss')}
      onclick={() => (dismissedFor = status.game)}
      data-testid="gaming-dismiss"
    >
      ✕
    </button>
  </div>
{/if}

<style>
  .gaming-banner {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 6px 12px;
    background: linear-gradient(90deg, var(--color-bg-elevated), var(--color-bg-surface));
    border-bottom: 1px solid var(--color-accent);
    color: var(--color-text-main);
    font-size: 13px;
  }

  .icon {
    font-size: 18px;
  }

  .text {
    flex: 1;
    display: flex;
    gap: 8px;
    align-items: baseline;
    flex-wrap: wrap;
  }

  .hint {
    color: var(--color-text-muted);
    font-size: 12px;
  }

  .auto {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 12px;
    color: var(--color-text-muted);
    cursor: pointer;
  }

  .close {
    color: var(--color-text-muted);
    padding: 2px 6px;
    border-radius: var(--radius-sm);
  }

  .close:hover {
    color: var(--color-text-main);
  }
</style>
