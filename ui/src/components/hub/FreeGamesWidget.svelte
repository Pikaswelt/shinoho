<script lang="ts">
  import { DEFAULT_FREE_GAMES } from './hubStore';
  import { tHub } from './i18n';

  let games = $state(DEFAULT_FREE_GAMES);
</script>

<div class="free-games-widget" data-testid="free-games-widget">
  <div class="widget-header">
    <div class="widget-title">
      <span class="icon">🎮</span>
      <span>{tHub('hub.widgets.freeGames')}</span>
    </div>
    <span class="free-badge">{tHub('hub.widgets.free')}</span>
  </div>

  <div class="games-list">
    {#each games as game}
      <a href={game.url} class="game-card" target="_blank" rel="noreferrer">
        <div class="game-info">
          <span class="game-title">{game.title}</span>
          <span class="game-platform">{game.platform} · {game.claimUntil}</span>
        </div>
        <span class="claim-btn">{tHub('hub.widgets.claim')}</span>
      </a>
    {:else}
      <p class="games-empty">{tHub('hub.widgets.freeGamesEmpty')}</p>
    {/each}
  </div>
</div>

<style>
  .free-games-widget {
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

  .free-badge {
    background: rgba(0, 229, 255, 0.15);
    color: #00e5ff;
    font-size: 11px;
    font-weight: 600;
    padding: 2px 8px;
    border-radius: 10px;
  }

  .games-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .game-card {
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

  .game-card:hover {
    border-color: rgba(0, 229, 255, 0.4);
    background: rgba(0, 229, 255, 0.08);
    transform: translateX(4px);
  }

  .game-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    overflow: hidden;
  }

  .game-title {
    font-size: 13px;
    color: var(--color-text-main, #f0f0f5);
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .game-platform {
    font-size: 11px;
    color: var(--color-text-muted, #8b8b9e);
  }

  .games-empty {
    margin: 0;
    font-size: 12px;
    color: var(--color-text-muted, #8b8b9e);
  }

  .claim-btn {
    background: rgba(0, 229, 255, 0.15);
    color: #00e5ff;
    border-radius: 4px;
    padding: 4px 10px;
    font-size: 11px;
    font-weight: 600;
  }

  .game-card:hover .claim-btn {
    background: #00e5ff;
    color: #050410;
  }
</style>
