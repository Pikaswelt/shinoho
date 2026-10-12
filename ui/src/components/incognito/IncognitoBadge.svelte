<script lang="ts">
  import { incognitoStore } from './incognitoStore';
  import { t, getLocale } from '../../i18n';

  interface Props {
    onCloseSession?: () => void;
  }

  let { onCloseSession }: Props = $props();

  let isMenuOpen = $state(false);
  let storeState = $state(incognitoStore.getState());

  $effect(() => {
    return incognitoStore.subscribe((s) => {
      storeState = s;
    });
  });

  const tr = (key: string, params?: Record<string, string | number>) => t(key, params, getLocale());

  function handleBadgeClick() {
    isMenuOpen = !isMenuOpen;
  }

  function handleToggleVpn() {
    incognitoStore.toggleAutoVpn();
  }

  function handleClose() {
    isMenuOpen = false;
    if (onCloseSession) {
      onCloseSession();
    } else {
      incognitoStore.disableIncognito();
    }
  }
</script>

<svelte:window onclick={() => (isMenuOpen = false)} />

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
<div class="incognito-badge-wrapper" onclick={(e) => e.stopPropagation()}>
  <button
    type="button"
    class="incognito-pill"
    class:active={isMenuOpen}
    onclick={handleBadgeClick}
    title={tr('incognito.status')}
    data-testid="incognito-badge"
  >
    <span class="ninja-icon">🥷</span>
    <span class="badge-text">{tr('incognito.badge')}</span>
    {#if storeState.autoEnableVpn}
      <span class="vpn-dot" title="VPN aktiv"></span>
    {/if}
  </button>

  {#if isMenuOpen}
    <div class="badge-popover" data-testid="incognito-badge-menu">
      <div class="popover-header">
        <span class="popover-icon">🥷</span>
        <div>
          <h4>{tr('incognito.title')}</h4>
          <p>{tr('incognito.status')}</p>
        </div>
      </div>

      <div class="popover-body">
        <label class="toggle-option">
          <span>{tr('incognito.actions.vpnToggle')}</span>
          <input
            type="checkbox"
            checked={storeState.autoEnableVpn}
            onchange={handleToggleVpn}
            data-testid="incognito-vpn-toggle"
          />
        </label>
      </div>

      <div class="popover-actions">
        <button
          type="button"
          class="btn-popover close"
          onclick={handleClose}
          data-testid="incognito-close-session-btn"
        >
          {tr('incognito.actions.closeSession')}
        </button>
      </div>
    </div>
  {/if}
</div>

<style>
  .incognito-badge-wrapper {
    position: relative;
    display: inline-flex;
    align-items: center;
  }

  .incognito-pill {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 3px 10px;
    background: rgba(157, 78, 221, 0.2);
    border: 1px solid rgba(157, 78, 221, 0.4);
    border-radius: 20px;
    color: #c77dff;
    font-size: 11px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s ease;
    box-shadow: 0 0 10px rgba(157, 78, 221, 0.2);
  }

  .incognito-pill:hover,
  .incognito-pill.active {
    background: rgba(157, 78, 221, 0.35);
    border-color: #9d4edd;
    box-shadow: 0 0 16px rgba(157, 78, 221, 0.4);
    color: #fff;
  }

  .ninja-icon {
    font-size: 13px;
  }

  .vpn-dot {
    width: 6px;
    height: 6px;
    background-color: #00ffaa;
    border-radius: 50%;
    box-shadow: 0 0 6px #00ffaa;
  }

  .badge-popover {
    position: absolute;
    top: calc(100% + 6px);
    right: 0;
    width: 260px;
    background: #1b1130;
    border: 1px solid rgba(157, 78, 221, 0.3);
    border-radius: 8px;
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.7);
    padding: 12px;
    z-index: 1000;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .popover-header {
    display: flex;
    align-items: center;
    gap: 8px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
    padding-bottom: 8px;
  }

  .popover-icon {
    font-size: 20px;
  }

  .popover-header h4 {
    margin: 0;
    font-size: 13px;
    color: #e0e0e0;
  }

  .popover-header p {
    margin: 2px 0 0 0;
    font-size: 11px;
    color: #9d4edd;
  }

  .toggle-option {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 12px;
    color: #ddd;
    cursor: pointer;
  }

  .btn-popover {
    width: 100%;
    padding: 6px;
    font-size: 11px;
    font-weight: 600;
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .btn-popover.close {
    background: rgba(255, 50, 50, 0.2);
    border: 1px solid rgba(255, 50, 50, 0.4);
    color: #ff8888;
  }
  .btn-popover.close:hover {
    background: rgba(255, 50, 50, 0.4);
    color: #fff;
  }
</style>
