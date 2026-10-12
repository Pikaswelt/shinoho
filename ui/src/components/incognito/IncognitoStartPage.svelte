<script lang="ts">
  import { incognitoStore } from './incognitoStore';
  import { t, getLocale } from '../../i18n';

  interface Props {
    onNavigate?: (url: string) => void;
    onOpenNormalWindow?: () => void;
  }

  let { onNavigate, onOpenNormalWindow }: Props = $props();

  let searchInput = $state('');
  let storeState = $state(incognitoStore.getState());
  let currentQuote = $state(incognitoStore.getRandomNinjaQuote());

  $effect(() => {
    return incognitoStore.subscribe((s) => {
      storeState = s;
    });
  });

  const tr = (key: string, params?: Record<string, string | number>) => t(key, params, getLocale());

  function handleSearch(e: SubmitEvent) {
    e.preventDefault();
    const query = searchInput.trim();
    if (!query) return;

    const url =
      query.startsWith('http://') || query.startsWith('https://')
        ? query
        : `https://duckduckgo.com/?q=${encodeURIComponent(query)}`;

    if (onNavigate) {
      onNavigate(url);
    } else {
      window.location.href = url;
    }
  }

  function handleOpenNormal() {
    if (onOpenNormalWindow) {
      onOpenNormalWindow();
    } else if (typeof window !== 'undefined') {
      const url = new URL(window.location.href);
      url.searchParams.delete('incognito');
      window.open(url.toString(), '_blank');
    }
  }
</script>

<div class="incognito-startpage" data-testid="incognito-startpage">
  <div class="ambient-glow"></div>

  <div class="startpage-content">
    <!-- Ninja Shino Mascot Section -->
    <div class="mascot-section">
      <div class="ninja-avatar" data-testid="ninja-avatar">
        <span class="avatar-emoji">🥷</span>
        <div class="ninja-aura"></div>
      </div>
      <div class="mascot-speech">
        <div class="speech-bubble">
          <p class="quote-text">{currentQuote}</p>
          <div class="bubble-arrow"></div>
        </div>
        <div class="mascot-badge">
          <span class="mascot-name">Shino-chan (Ninja-Modus)</span>
          <span class="stealth-indicator">✦ {tr('incognito.mascot.stealthMode')}</span>
        </div>
      </div>
    </div>

    <!-- Header Title -->
    <div class="header-section">
      <h1>{tr('incognito.title')}</h1>
      <p class="tagline">{tr('incognito.tagline')}</p>
    </div>

    <!-- Private Search Bar -->
    <form class="stealth-search" onsubmit={handleSearch} data-testid="incognito-search-form">
      <span class="search-icon">🔍</span>
      <input
        type="text"
        placeholder={tr('incognito.actions.searchPlaceholder')}
        bind:value={searchInput}
        data-testid="incognito-search-input"
      />
      <button type="submit" class="btn-search">
        {tr('nav.home') === 'Home' ? 'Search' : 'Suchen'}
      </button>
    </form>

    <!-- Privacy Guarantees Grid -->
    <div class="features-grid">
      <div class="feature-card">
        <div class="card-icon">🚫</div>
        <h3>{tr('incognito.features.noHistoryTitle')}</h3>
        <p>{tr('incognito.features.noHistoryDesc')}</p>
      </div>

      <div class="feature-card">
        <div class="card-icon">🍪</div>
        <h3>{tr('incognito.features.noCookiesTitle')}</h3>
        <p>{tr('incognito.features.noCookiesDesc')}</p>
      </div>

      <div class="feature-card">
        <div class="card-icon">💾</div>
        <h3>{tr('incognito.features.noCacheTitle')}</h3>
        <p>{tr('incognito.features.noCacheDesc')}</p>
      </div>

      <div class="feature-card">
        <div class="card-icon">🛡️</div>
        <h3>{tr('incognito.features.autoVpnTitle')}</h3>
        <p>{tr('incognito.features.autoVpnDesc')}</p>
        <label class="vpn-switch mt-2">
          <span>VPN: {storeState.autoEnableVpn ? 'AN' : 'AUS'}</span>
          <input
            type="checkbox"
            checked={storeState.autoEnableVpn}
            onchange={() => incognitoStore.toggleAutoVpn()}
            data-testid="incognito-card-vpn-toggle"
          />
        </label>
      </div>
    </div>

    <!-- Visibility Notice Box -->
    <div class="visibility-notice">
      <h4>{tr('incognito.visibility.title')}</h4>
      <p class="notice-desc">{tr('incognito.visibility.notice')}</p>
      <ul class="notice-list">
        <li>✦ {tr('incognito.visibility.itemIsp')}</li>
        <li>✦ {tr('incognito.visibility.itemEmployer')}</li>
        <li>✦ {tr('incognito.visibility.itemWebsites')}</li>
      </ul>
    </div>

    <!-- Actions -->
    <div class="bottom-actions">
      <button
        type="button"
        class="btn-action-normal"
        onclick={handleOpenNormal}
        data-testid="btn-open-normal-window"
      >
        🌐 {tr('incognito.actions.openNormal')}
      </button>
    </div>
  </div>
</div>

<style>
  .incognito-startpage {
    position: relative;
    display: flex;
    justify-content: center;
    align-items: center;
    min-height: 100%;
    width: 100%;
    background: radial-gradient(ellipse at top, #2d1b4e 0%, #150d24 60%, #0d0716 100%);
    color: #e0e0e0;
    padding: 32px 16px;
    box-sizing: border-box;
    overflow-y: auto;
  }

  .ambient-glow {
    position: absolute;
    top: 10%;
    left: 50%;
    transform: translateX(-50%);
    width: 600px;
    height: 300px;
    background: radial-gradient(ellipse, rgba(157, 78, 221, 0.15) 0%, transparent 70%);
    pointer-events: none;
  }

  .startpage-content {
    position: relative;
    z-index: 1;
    max-width: 800px;
    width: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 24px;
  }

  /* Mascot Section */
  .mascot-section {
    display: flex;
    align-items: center;
    gap: 20px;
  }

  .ninja-avatar {
    position: relative;
    width: 72px;
    height: 72px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    background: #1b1130;
    border: 2px solid #9d4edd;
    box-shadow: 0 0 20px rgba(157, 78, 221, 0.5);
  }

  .avatar-emoji {
    font-size: 38px;
  }

  .ninja-aura {
    position: absolute;
    inset: -6px;
    border-radius: 50%;
    border: 1px dashed rgba(157, 78, 221, 0.4);
    animation: spinAura 12s linear infinite;
  }

  @keyframes spinAura {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }

  .mascot-speech {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .speech-bubble {
    position: relative;
    background: rgba(45, 27, 78, 0.7);
    backdrop-filter: blur(8px);
    border: 1px solid rgba(157, 78, 221, 0.3);
    border-radius: 8px;
    padding: 10px 14px;
    max-width: 400px;
  }

  .quote-text {
    margin: 0;
    font-size: 13px;
    color: #f0e6ff;
    line-height: 1.4;
  }

  .bubble-arrow {
    position: absolute;
    left: -8px;
    top: 50%;
    transform: translateY(-50%);
    width: 0;
    height: 0;
    border-top: 6px solid transparent;
    border-bottom: 6px solid transparent;
    border-right: 8px solid rgba(157, 78, 221, 0.5);
  }

  .mascot-badge {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 11px;
  }

  .mascot-name {
    font-weight: 600;
    color: #c77dff;
  }

  .stealth-indicator {
    color: #00ffaa;
  }

  /* Header Section */
  .header-section {
    text-align: center;
  }

  .header-section h1 {
    font-size: 26px;
    font-weight: 700;
    margin: 0 0 6px 0;
    background: linear-gradient(135deg, #ffffff 0%, #c77dff 100%);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
  }

  .tagline {
    font-size: 14px;
    color: rgba(255, 255, 255, 0.6);
    margin: 0;
  }

  /* Search */
  .stealth-search {
    position: relative;
    display: flex;
    align-items: center;
    width: 100%;
    max-width: 580px;
  }

  .search-icon {
    position: absolute;
    left: 14px;
    font-size: 14px;
    color: #9d4edd;
  }

  .stealth-search input {
    width: 100%;
    padding: 12px 100px 12px 42px;
    background: rgba(0, 0, 0, 0.4);
    border: 1px solid rgba(157, 78, 221, 0.35);
    border-radius: 24px;
    color: #fff;
    font-size: 14px;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
    transition: all 0.2s ease;
  }
  .stealth-search input:focus {
    outline: none;
    border-color: #c77dff;
    box-shadow: 0 0 20px rgba(157, 78, 221, 0.4);
  }

  .btn-search {
    position: absolute;
    right: 6px;
    padding: 8px 16px;
    background: #9d4edd;
    border: none;
    border-radius: 20px;
    color: #fff;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.15s ease;
  }
  .btn-search:hover {
    background: #c77dff;
  }

  /* Features Grid */
  .features-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: 16px;
    width: 100%;
  }

  .feature-card {
    background: rgba(45, 27, 78, 0.35);
    border: 1px solid rgba(157, 78, 221, 0.2);
    border-radius: 10px;
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 6px;
    transition:
      transform 0.2s ease,
      border-color 0.2s ease;
  }
  .feature-card:hover {
    transform: translateY(-2px);
    border-color: rgba(157, 78, 221, 0.4);
  }

  .card-icon {
    font-size: 22px;
  }

  .feature-card h3 {
    margin: 0;
    font-size: 14px;
    color: #f0e6ff;
  }

  .feature-card p {
    margin: 0;
    font-size: 12px;
    color: rgba(255, 255, 255, 0.6);
    line-height: 1.4;
  }

  .vpn-switch {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 11px;
    color: #c77dff;
    cursor: pointer;
  }

  /* Visibility Notice */
  .visibility-notice {
    width: 100%;
    background: rgba(0, 0, 0, 0.25);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 8px;
    padding: 14px 18px;
    box-sizing: border-box;
  }

  .visibility-notice h4 {
    margin: 0 0 4px 0;
    font-size: 13px;
    color: #ffaa55;
  }

  .notice-desc {
    margin: 0 0 8px 0;
    font-size: 12px;
    color: rgba(255, 255, 255, 0.7);
  }

  .notice-list {
    margin: 0;
    padding: 0;
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 11px;
    color: rgba(255, 255, 255, 0.6);
  }

  /* Bottom Actions */
  .bottom-actions {
    display: flex;
    gap: 12px;
  }

  .btn-action-normal {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 16px;
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 6px;
    color: #e0e0e0;
    font-size: 12px;
    cursor: pointer;
    transition: all 0.15s ease;
  }
  .btn-action-normal:hover {
    background: rgba(255, 255, 255, 0.12);
    color: #fff;
  }
</style>
