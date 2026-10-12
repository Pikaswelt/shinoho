<script lang="ts">
  import { DEFAULT_ANIME_COUNTDOWN } from './hubStore';
  import { tHub } from './i18n';

  let items = $state(DEFAULT_ANIME_COUNTDOWN);

  function getRelativeTime(isoString: string): string {
    const diffMs = new Date(isoString).getTime() - Date.now();
    if (diffMs <= 0) return 'Jetzt';
    const hours = Math.floor(diffMs / 3600000);
    const days = Math.floor(hours / 24);
    const remHours = hours % 24;
    if (days > 0) {
      return `${days}T ${remHours}h`;
    }
    return `${hours}h`;
  }
</script>

<div class="anime-countdown-widget" data-testid="anime-countdown-widget">
  <div class="widget-header">
    <div class="widget-title">
      <span class="icon">🌸</span>
      <span>{tHub('hub.widgets.animeCountdown')}</span>
    </div>
    <span class="badge">AniList</span>
  </div>

  <div class="countdown-list">
    {#each items as item}
      <a href={item.siteUrl || '#'} class="countdown-card" target="_blank" rel="noreferrer">
        <div class="card-left">
          <span class="anime-title">{item.title}</span>
          <span class="ep-info">
            {tHub('hub.widgets.nextEpIn', {
              ep: item.nextEpisode,
              time: getRelativeTime(item.airingAt),
            })}
          </span>
        </div>
        <div class="countdown-pill">
          {getRelativeTime(item.airingAt)}
        </div>
      </a>
    {/each}
  </div>
</div>

<style>
  .anime-countdown-widget {
    background: rgba(24, 24, 34, 0.65);
    backdrop-filter: blur(12px);
    border: 1px solid var(--color-border, #2a2a3c);
    border-radius: var(--radius-lg, 12px);
    padding: 16px;
    width: 100%;
    max-width: 420px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
  }

  .widget-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 12px;
    padding-bottom: 8px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  }

  .widget-title {
    display: flex;
    align-items: center;
    gap: 8px;
    font-weight: 600;
    font-size: 14px;
    color: var(--color-text-main, #f0f0f5);
  }

  .icon {
    font-size: 16px;
  }

  .badge {
    background: rgba(255, 71, 133, 0.15);
    color: var(--color-accent, #ff4785);
    font-size: 11px;
    font-weight: 600;
    padding: 2px 8px;
    border-radius: 10px;
  }

  .countdown-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .countdown-card {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    background: rgba(15, 15, 20, 0.5);
    border-radius: var(--radius-sm, 4px);
    text-decoration: none;
    border: 1px solid transparent;
    transition: all 0.2s ease;
  }

  .countdown-card:hover {
    border-color: rgba(255, 71, 133, 0.4);
    background: rgba(255, 71, 133, 0.08);
    transform: translateX(4px);
  }

  .card-left {
    display: flex;
    flex-direction: column;
    gap: 2px;
    overflow: hidden;
  }

  .anime-title {
    font-size: 13px;
    color: var(--color-text-main, #f0f0f5);
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .ep-info {
    font-size: 11px;
    color: var(--color-text-muted, #8b8b9e);
  }

  .countdown-pill {
    background: rgba(255, 255, 255, 0.08);
    color: var(--color-accent, #ff4785);
    font-size: 11px;
    font-weight: 600;
    padding: 4px 8px;
    border-radius: 4px;
    white-space: nowrap;
  }
</style>
