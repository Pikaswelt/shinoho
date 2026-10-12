<script lang="ts">
  import Header from './components/Header.svelte';
  import { t, getLocale, setLocale, SUPPORTED_LOCALES, type SupportedLocale } from './i18n';

  let currentLang = $state<SupportedLocale>(getLocale());

  function changeLanguage(lang: SupportedLocale) {
    setLocale(lang);
    currentLang = lang;
  }

  // Reactive translation helper tied to currentLang state rune
  const tr = (key: string, params?: Record<string, string | number>) => t(key, params, currentLang);
</script>

<div class="shinoho-app" data-testid="shinoho-app">
  <Header title={tr('app.title')} />
  <main class="content">
    <div class="welcome-card">
      <h1>{tr('app.title')}</h1>
      <p class="tagline">{tr('app.tagline')}</p>
      <p class="status" data-testid="status-message">{tr('app.ready')}</p>

      <div class="language-selector" data-testid="language-selector">
        {#each SUPPORTED_LOCALES as loc}
          <button
            class="lang-btn"
            class:active={currentLang === loc}
            onclick={() => changeLanguage(loc)}
            data-testid={`lang-btn-${loc}`}
          >
            {loc.toUpperCase()}
          </button>
        {/each}
      </div>
    </div>
  </main>
</div>

<style>
  .shinoho-app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    width: 100vw;
  }

  .content {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 24px;
  }

  .welcome-card {
    text-align: center;
    padding: 32px 48px;
    background: var(--color-bg-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
  }

  .welcome-card h1 {
    font-size: 28px;
    margin-bottom: 8px;
    color: var(--color-accent);
  }

  .tagline {
    font-size: 14px;
    color: var(--color-text-muted);
    margin-bottom: 6px;
  }

  .status {
    color: var(--color-text-main);
    font-size: 14px;
    margin-bottom: 12px;
  }

  .language-selector {
    display: flex;
    justify-content: center;
    gap: 8px;
    margin-top: 16px;
  }

  .lang-btn {
    padding: 4px 10px;
    font-size: 12px;
    font-weight: 600;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    color: var(--color-text-muted);
    transition: all 0.15s ease;
  }

  .lang-btn:hover {
    color: var(--color-text-main);
    border-color: var(--color-accent);
  }

  .lang-btn.active {
    background-color: var(--color-accent);
    border-color: var(--color-accent);
    color: #fff;
  }
</style>
