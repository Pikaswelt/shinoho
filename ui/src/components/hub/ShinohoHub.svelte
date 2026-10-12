<script lang="ts">
  import { onMount } from 'svelte';
  import Wallpaper from './Wallpaper.svelte';
  import SearchBar from './SearchBar.svelte';
  import SpeedDial from './SpeedDial.svelte';
  import AnimeCountdownWidget from './AnimeCountdownWidget.svelte';
  import FreeGamesWidget from './FreeGamesWidget.svelte';
  import ShinoMascotWidget from './ShinoMascotWidget.svelte';
  import HubSettingsDrawer from './HubSettingsDrawer.svelte';
  import { hubStore } from './hubStore';

  let { onNavigate }: { onNavigate?: (url: string) => void } = $props();

  let hubState = $state(hubStore.getState());
  hubStore.subscribe((s) => {
    hubState = s;
  });

  let isSettingsOpen = $state(false);
  let currentTime = $state('');

  function updateClock() {
    const now = new Date();
    currentTime = now.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  }

  onMount(() => {
    updateClock();
    const interval = setInterval(updateClock, 1000);
    return () => clearInterval(interval);
  });
</script>

<div class="shinoho-hub" data-testid="shinoho-hub">
  <!-- Dynamic live wallpaper background -->
  <Wallpaper settings={hubState.wallpaper} />

  <!-- Hub Header / Quick Controls -->
  <header class="hub-header">
    <div class="header-left">
      {#if hubState.widgets.clock}
        <div class="clock-display" data-testid="clock-display">
          {currentTime}
        </div>
      {/if}
    </div>

    <div class="header-right">
      <button
        type="button"
        class="hub-icon-btn settings-btn"
        data-testid="hub-settings-toggle"
        title="Startseiten-Einstellungen"
        onclick={() => (isSettingsOpen = true)}
      >
        ⚙️
      </button>
    </div>
  </header>

  <!-- Central Hub Content -->
  <main class="hub-main-content">
    <!-- Shino-chan Mascot Banner -->
    {#if hubState.widgets.mascot}
      <div class="mascot-section">
        <ShinoMascotWidget />
      </div>
    {/if}

    <!-- Web Search Bar -->
    <div class="search-section">
      <SearchBar {onNavigate} />
    </div>

    <!-- Speed Dial Shortcuts Grid -->
    <div class="speed-dial-section">
      <SpeedDial {onNavigate} />
    </div>

    <!-- Anime & Gaming Widgets Row -->
    {#if hubState.widgets.animeCountdown || hubState.widgets.freeGames}
      <section class="widgets-section" data-testid="hub-widgets-section">
        {#if hubState.widgets.animeCountdown}
          <AnimeCountdownWidget />
        {/if}
        {#if hubState.widgets.freeGames}
          <FreeGamesWidget />
        {/if}
      </section>
    {/if}
  </main>

  <!-- Side Settings Drawer -->
  <HubSettingsDrawer isOpen={isSettingsOpen} onClose={() => (isSettingsOpen = false)} />
</div>

<style>
  .shinoho-hub {
    position: relative;
    width: 100%;
    min-height: 100vh;
    display: flex;
    flex-direction: column;
    overflow-x: hidden;
    color: var(--color-text-main, #f0f0f5);
    font-family: var(
      --font-family-base,
      -apple-system,
      BlinkMacSystemFont,
      'Segoe UI',
      Roboto,
      sans-serif
    );
    box-sizing: border-box;
  }

  .hub-header {
    position: relative;
    z-index: 10;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 28px;
  }

  .clock-display {
    font-size: 26px;
    font-weight: 700;
    letter-spacing: -0.5px;
    color: var(--color-text-main, #f0f0f5);
    text-shadow: 0 2px 10px rgba(0, 0, 0, 0.5);
  }

  .hub-icon-btn {
    background: rgba(24, 24, 34, 0.65);
    backdrop-filter: blur(8px);
    border: 1px solid var(--color-border, #2a2a3c);
    color: var(--color-text-main, #f0f0f5);
    width: 38px;
    height: 38px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 16px;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .hub-icon-btn:hover {
    background: var(--color-accent, #ff4785);
    border-color: var(--color-accent, #ff4785);
    transform: rotate(30deg);
  }

  .hub-main-content {
    position: relative;
    z-index: 10;
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 20px 24px 60px 24px;
    max-width: 1100px;
    margin: 0 auto;
    width: 100%;
    box-sizing: border-box;
  }

  .mascot-section {
    width: 100%;
    margin-bottom: 24px;
  }

  .search-section {
    width: 100%;
    margin-bottom: 12px;
  }

  .speed-dial-section {
    width: 100%;
    margin-bottom: 36px;
  }

  .widgets-section {
    width: 100%;
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: 20px;
  }
</style>
