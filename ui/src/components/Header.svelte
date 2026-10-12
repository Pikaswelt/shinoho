<script lang="ts">
  import { t } from '../i18n';
  import ThemeSelector from './ThemeSelector.svelte';

  interface Props {
    title?: string;
  }

  let { title = t('app.title') }: Props = $props();
</script>

<header class="app-header" data-testid="app-header">
  <div class="logo">
    <span class="logo-accent">✦</span>
    <span class="logo-text">{title}</span>
  </div>

  <div class="header-actions">
    <ThemeSelector />
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

  .header-actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }
</style>
