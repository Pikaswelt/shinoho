<script lang="ts">
  import { WALLPAPER_PRESETS, hubStore } from './hubStore';
  import { tHub } from './i18n';

  let { isOpen, onClose }: { isOpen: boolean; onClose: () => void } = $props();

  let hubState = $state(hubStore.getState());
  hubStore.subscribe((s) => {
    hubState = s;
  });

  function selectPreset(id: string) {
    hubStore.setActiveWallpaper(id);
  }

  function handleBlurChange(e: Event) {
    const val = parseFloat((e.target as HTMLInputElement).value);
    hubStore.updateWallpaperSettings({ blur: val });
  }

  function handleBrightnessChange(e: Event) {
    const val = parseFloat((e.target as HTMLInputElement).value);
    hubStore.updateWallpaperSettings({ brightness: val });
  }

  function handleParticlesToggle(e: Event) {
    const checked = (e.target as HTMLInputElement).checked;
    hubStore.updateWallpaperSettings({ particlesEnabled: checked });
  }

  function handleDensityChange(e: Event) {
    const val = parseInt((e.target as HTMLInputElement).value, 10);
    hubStore.updateWallpaperSettings({ particleDensity: val });
  }

  function handleCustomUrlChange(e: Event) {
    const val = (e.target as HTMLInputElement).value.trim();
    hubStore.updateWallpaperSettings({ customUrl: val || undefined });
  }
</script>

{#if isOpen}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="drawer-backdrop" onclick={onClose}>
    <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
    <div
      class="drawer-panel"
      onclick={(e) => e.stopPropagation()}
      data-testid="hub-settings-drawer"
    >
      <div class="drawer-header">
        <h3>{tHub('hub.settings.title')}</h3>
        <button type="button" class="close-btn" onclick={onClose} data-testid="drawer-close-button"
          >✕</button
        >
      </div>

      <div class="drawer-content">
        <!-- Wallpaper Presets -->
        <div class="setting-section">
          <h4>{tHub('hub.wallpaper.choosePreset')}</h4>
          <div class="presets-grid">
            {#each WALLPAPER_PRESETS as preset}
              <button
                type="button"
                class="preset-card"
                class:active={hubState.wallpaper.activeId === preset.id}
                data-testid={`preset-card-${preset.id}`}
                onclick={() => selectPreset(preset.id)}
              >
                <div
                  class="preset-preview"
                  style="background: {preset.gradient || preset.previewColor};"
                ></div>
                <span class="preset-name">{tHub(preset.nameKey)}</span>
              </button>
            {/each}
          </div>
        </div>

        <!-- Custom Image URL -->
        <div class="setting-section">
          <label for="custom-wp-url" class="field-label">{tHub('hub.wallpaper.customUrl')}</label>
          <input
            id="custom-wp-url"
            type="text"
            class="text-input"
            placeholder={tHub('hub.wallpaper.customUrlPlaceholder')}
            value={hubState.wallpaper.customUrl || ''}
            onchange={handleCustomUrlChange}
          />
        </div>

        <!-- Sliders -->
        <div class="setting-section">
          <div class="slider-group">
            <div class="slider-header">
              <span>{tHub('hub.wallpaper.blur')}</span>
              <span>{hubState.wallpaper.blur}px</span>
            </div>
            <input
              type="range"
              min="0"
              max="20"
              step="1"
              value={hubState.wallpaper.blur}
              oninput={handleBlurChange}
            />
          </div>

          <div class="slider-group">
            <div class="slider-header">
              <span>{tHub('hub.wallpaper.brightness')}</span>
              <span>{Math.round(hubState.wallpaper.brightness * 100)}%</span>
            </div>
            <input
              type="range"
              min="0.3"
              max="1.5"
              step="0.05"
              value={hubState.wallpaper.brightness}
              oninput={handleBrightnessChange}
            />
          </div>
        </div>

        <!-- Particles Toggle -->
        <div class="setting-section">
          <label class="toggle-row">
            <span>{tHub('hub.wallpaper.particles')}</span>
            <input
              type="checkbox"
              checked={hubState.wallpaper.particlesEnabled}
              onchange={handleParticlesToggle}
              data-testid="particles-toggle"
            />
          </label>

          {#if hubState.wallpaper.particlesEnabled}
            <div class="slider-group mt-2">
              <div class="slider-header">
                <span>{tHub('hub.wallpaper.particleDensity')}</span>
                <span>{hubState.wallpaper.particleDensity}</span>
              </div>
              <input
                type="range"
                min="10"
                max="80"
                step="5"
                value={hubState.wallpaper.particleDensity}
                oninput={handleDensityChange}
              />
            </div>
          {/if}
        </div>

        <!-- Widgets Toggle -->
        <div class="setting-section">
          <h4>{tHub('hub.settings.toggleWidgets')}</h4>
          <label class="toggle-row">
            <span>{tHub('hub.widgets.animeCountdown')}</span>
            <input
              type="checkbox"
              checked={hubState.widgets.animeCountdown}
              onchange={() => hubStore.toggleWidget('animeCountdown')}
            />
          </label>
          <label class="toggle-row">
            <span>{tHub('hub.widgets.freeGames')}</span>
            <input
              type="checkbox"
              checked={hubState.widgets.freeGames}
              onchange={() => hubStore.toggleWidget('freeGames')}
            />
          </label>
          <label class="toggle-row">
            <span>{tHub('hub.widgets.mascot')}</span>
            <input
              type="checkbox"
              checked={hubState.widgets.mascot}
              onchange={() => hubStore.toggleWidget('mascot')}
            />
          </label>
        </div>

        <!-- Reset Defaults -->
        <div class="reset-wrapper">
          <button
            type="button"
            class="btn-reset"
            data-testid="reset-hub-defaults"
            onclick={() => hubStore.resetToDefaults()}
          >
            {tHub('hub.settings.resetDefaults')}
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  .drawer-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    width: 100vw;
    height: 100vh;
    background: rgba(0, 0, 0, 0.5);
    backdrop-filter: blur(4px);
    z-index: 100;
  }

  .drawer-panel {
    position: absolute;
    top: 0;
    right: 0;
    width: 100%;
    max-width: 360px;
    height: 100%;
    background: var(--color-bg-surface, #181822);
    border-left: 1px solid var(--color-border, #2a2a3c);
    box-shadow: -10px 0 30px rgba(0, 0, 0, 0.5);
    display: flex;
    flex-direction: column;
    overflow-y: auto;
  }

  .drawer-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 20px;
    border-bottom: 1px solid var(--color-border, #2a2a3c);
  }

  .drawer-header h3 {
    margin: 0;
    font-size: 16px;
    color: var(--color-text-main, #f0f0f5);
  }

  .close-btn {
    background: transparent;
    border: none;
    color: var(--color-text-muted, #8b8b9e);
    font-size: 16px;
    cursor: pointer;
  }

  .close-btn:hover {
    color: white;
  }

  .drawer-content {
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .setting-section h4 {
    margin: 0 0 10px 0;
    font-size: 13px;
    color: var(--color-text-muted, #8b8b9e);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .field-label {
    display: block;
    font-size: 12px;
    color: var(--color-text-muted, #8b8b9e);
    margin-bottom: 6px;
  }

  .text-input {
    width: 100%;
    padding: 8px 12px;
    background: var(--color-bg-base, #0f0f14);
    border: 1px solid var(--color-border, #2a2a3c);
    border-radius: var(--radius-sm, 4px);
    color: var(--color-text-main, #f0f0f5);
    font-size: 13px;
    box-sizing: border-box;
  }

  .text-input:focus {
    outline: none;
    border-color: var(--color-accent, #ff4785);
  }

  .presets-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 10px;
  }

  .preset-card {
    background: var(--color-bg-base, #0f0f14);
    border: 1px solid var(--color-border, #2a2a3c);
    border-radius: var(--radius-sm, 4px);
    padding: 8px;
    cursor: pointer;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    transition: all 0.2s;
  }

  .preset-card:hover {
    border-color: rgba(255, 71, 133, 0.5);
  }

  .preset-card.active {
    border-color: var(--color-accent, #ff4785);
    background: rgba(255, 71, 133, 0.1);
  }

  .preset-preview {
    width: 100%;
    height: 36px;
    border-radius: 4px;
  }

  .preset-name {
    font-size: 11px;
    color: var(--color-text-main, #f0f0f5);
    text-align: center;
  }

  .slider-group {
    margin-bottom: 12px;
  }

  .slider-header {
    display: flex;
    justify-content: space-between;
    font-size: 12px;
    color: var(--color-text-muted, #8b8b9e);
    margin-bottom: 4px;
  }

  .slider-group input[type='range'] {
    width: 100%;
    accent-color: var(--color-accent, #ff4785);
  }

  .toggle-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 13px;
    color: var(--color-text-main, #f0f0f5);
    margin-bottom: 10px;
    cursor: pointer;
  }

  .toggle-row input[type='checkbox'] {
    accent-color: var(--color-accent, #ff4785);
    width: 16px;
    height: 16px;
    cursor: pointer;
  }

  .mt-2 {
    margin-top: 8px;
  }

  .reset-wrapper {
    margin-top: 10px;
    padding-top: 16px;
    border-top: 1px solid var(--color-border, #2a2a3c);
  }

  .btn-reset {
    width: 100%;
    background: transparent;
    border: 1px solid var(--color-border, #2a2a3c);
    color: var(--color-text-muted, #8b8b9e);
    padding: 8px 12px;
    border-radius: var(--radius-sm, 4px);
    font-size: 12px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .btn-reset:hover {
    border-color: #ff4785;
    color: #ff4785;
  }
</style>
