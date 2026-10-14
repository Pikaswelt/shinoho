<script lang="ts">
  import { WALLPAPER_PRESETS, hubStore } from './hubStore';
  import { soundManager, BUILTIN_SOUND_PACKS } from '../../sounds';
  import { tHub } from './i18n';

  let { isOpen, onClose }: { isOpen: boolean; onClose: () => void } = $props();

  let hubState = $state(hubStore.getState());
  hubStore.subscribe((s) => {
    hubState = s;
  });

  let soundState = $state(soundManager.getSettings());
  soundManager.subscribe((s) => {
    soundState = s;
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

  function handleWallpaperThemeSync(e: Event) {
    const checked = (e.target as HTMLInputElement).checked;
    hubStore.updateWallpaperSettings({ syncWithTheme: checked });
  }

  function handleSoundToggle(e: Event) {
    const checked = (e.target as HTMLInputElement).checked;
    soundManager.setEnabled(checked);
    if (checked) {
      soundManager.play('click');
    }
  }

  function handleSoundVolume(e: Event) {
    const val = parseFloat((e.target as HTMLInputElement).value);
    soundManager.setVolume(val);
  }

  function handleSoundThemeSync(e: Event) {
    const checked = (e.target as HTMLInputElement).checked;
    soundManager.setSyncWithTheme(checked);
  }

  function selectSoundPack(id: string) {
    soundManager.setSoundPack(id);
    soundManager.play('tab.open');
  }

  function testSound() {
    soundManager.play('download.done');
  }

  function handleResetAll() {
    hubStore.resetToDefaults();
    soundManager.resetToDefaults();
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

        <!-- Wallpaper Theme Sync -->
        <div class="setting-section">
          <label class="toggle-row">
            <span>{tHub('hub.wallpaper.syncWithTheme')}</span>
            <input
              type="checkbox"
              checked={hubState.wallpaper.syncWithTheme ?? true}
              onchange={handleWallpaperThemeSync}
              data-testid="wallpaper-theme-sync-toggle"
            />
          </label>
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

        <!-- UI-Sounds & Soundpacks -->
        <div class="setting-section" data-testid="sound-settings-section">
          <h4>{tHub('sounds.title')}</h4>
          <label class="toggle-row">
            <span>{tHub('sounds.enabled')}</span>
            <input
              type="checkbox"
              checked={soundState.enabled}
              onchange={handleSoundToggle}
              data-testid="sound-enabled-toggle"
            />
          </label>

          {#if soundState.enabled}
            <label class="toggle-row">
              <span>{tHub('sounds.syncWithTheme')}</span>
              <input
                type="checkbox"
                checked={soundState.syncWithTheme}
                onchange={handleSoundThemeSync}
                data-testid="sound-theme-sync-toggle"
              />
            </label>

            <div class="slider-group">
              <div class="slider-header">
                <span>{tHub('sounds.volume')}</span>
                <span>{Math.round(soundState.volume * 100)}%</span>
              </div>
              <input
                type="range"
                min="0"
                max="1"
                step="0.05"
                value={soundState.volume}
                oninput={handleSoundVolume}
                data-testid="sound-volume-slider"
              />
            </div>

            <div class="soundpacks-subhead">
              <span>{tHub('sounds.choosePack')}</span>
            </div>
            <div class="soundpacks-grid">
              {#each BUILTIN_SOUND_PACKS as pack}
                <button
                  type="button"
                  class="soundpack-card"
                  class:active={soundState.activePackId === pack.id}
                  data-testid={`soundpack-card-${pack.id}`}
                  onclick={() => selectSoundPack(pack.id)}
                >
                  <span class="soundpack-name">{tHub(pack.i18nKey)}</span>
                  <span class="soundpack-desc">{tHub(pack.descriptionKey)}</span>
                </button>
              {/each}
            </div>

            <button
              type="button"
              class="btn-test-sound"
              data-testid="test-sound-btn"
              onclick={testSound}
            >
              🔊 {tHub('sounds.test')}
            </button>
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
            onclick={handleResetAll}
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
    font-size: 18px;
    cursor: pointer;
    padding: 4px 8px;
    border-radius: 4px;
    transition: all 0.2s;
  }

  .close-btn:hover {
    color: var(--color-text-main, #f0f0f5);
    background: rgba(255, 255, 255, 0.05);
  }

  .drawer-content {
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .setting-section {
    display: flex;
    flex-direction: column;
  }

  .setting-section h4 {
    margin: 0 0 12px 0;
    font-size: 13px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--color-accent, #ff4785);
  }

  .presets-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 8px;
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

  .soundpacks-subhead {
    font-size: 12px;
    color: var(--color-text-muted, #8b8b9e);
    margin: 6px 0 8px 0;
  }

  .soundpacks-grid {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-bottom: 12px;
  }

  .soundpack-card {
    background: var(--color-bg-base, #0f0f14);
    border: 1px solid var(--color-border, #2a2a3c);
    border-radius: var(--radius-sm, 4px);
    padding: 8px 10px;
    cursor: pointer;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    text-align: left;
    gap: 2px;
    transition: all 0.2s;
  }

  .soundpack-card:hover {
    border-color: rgba(255, 71, 133, 0.5);
  }

  .soundpack-card.active {
    border-color: var(--color-accent, #ff4785);
    background: rgba(255, 71, 133, 0.1);
  }

  .soundpack-name {
    font-size: 12px;
    font-weight: 600;
    color: var(--color-text-main, #f0f0f5);
  }

  .soundpack-desc {
    font-size: 11px;
    color: var(--color-text-muted, #8b8b9e);
  }

  .btn-test-sound {
    background: var(--color-bg-base, #0f0f14);
    border: 1px solid var(--color-border, #2a2a3c);
    color: var(--color-accent, #ff4785);
    padding: 8px 12px;
    border-radius: var(--radius-sm, 4px);
    font-size: 12px;
    cursor: pointer;
    font-weight: 500;
    transition: all 0.2s;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
  }

  .btn-test-sound:hover {
    border-color: var(--color-accent, #ff4785);
    background: rgba(255, 71, 133, 0.1);
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

  .field-label {
    font-size: 12px;
    color: var(--color-text-muted, #8b8b9e);
    margin-bottom: 6px;
  }

  .text-input {
    background: var(--color-bg-base, #0f0f14);
    border: 1px solid var(--color-border, #2a2a3c);
    color: var(--color-text-main, #f0f0f5);
    padding: 8px 10px;
    border-radius: var(--radius-sm, 4px);
    font-size: 12px;
    width: 100%;
    box-sizing: border-box;
  }

  .text-input:focus {
    outline: none;
    border-color: var(--color-accent, #ff4785);
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
