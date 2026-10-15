<script lang="ts">
  import { onMount } from 'svelte';
  import { getTopSites, type TopSite } from '../../api/history';
  import { tHub } from './i18n';

  let { onNavigate }: { onNavigate?: (url: string) => void } = $props();

  let topSites = $state<TopSite[]>([]);
  let loaded = $state(false);

  onMount(async () => {
    topSites = await getTopSites(8);
    loaded = true;
  });

  function getDomain(url: string): string {
    try {
      return new URL(url).hostname.replace('www.', '');
    } catch {
      return url;
    }
  }

  function handleClick(site: TopSite, e: MouseEvent) {
    if (onNavigate) {
      e.preventDefault();
      onNavigate(site.url);
    }
  }
</script>

{#if loaded && topSites.length > 0}
  <section class="top-sites" data-testid="top-sites">
    <h2 class="top-sites-title">{tHub('hub.topSites.title')}</h2>
    <div class="top-sites-grid">
      {#each topSites as site (site.url)}
        <a
          href={site.url}
          class="top-site-card"
          data-testid={`top-site-${getDomain(site.url)}`}
          onclick={(e) => handleClick(site, e)}
        >
          <img
            src={`https://www.google.com/s2/favicons?domain=${getDomain(site.url)}&sz=64`}
            alt=""
            class="top-site-favicon"
            loading="lazy"
            onerror={(e) => ((e.currentTarget as HTMLElement).style.display = 'none')}
          />
          <span class="top-site-info">
            <span class="top-site-title">{site.title || getDomain(site.url)}</span>
            <span class="top-site-meta"
              >{tHub('hub.topSites.visits', { count: site.visit_count })}</span
            >
          </span>
        </a>
      {/each}
    </div>
  </section>
{/if}

<style>
  .top-sites {
    width: 100%;
    max-width: 900px;
    margin: 8px auto 0 auto;
    position: relative;
    z-index: 10;
  }

  .top-sites-title {
    font-size: 14px;
    font-weight: 600;
    color: var(--color-text-muted, #8b8b9e);
    margin: 0 0 12px 4px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }

  .top-sites-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 10px;
  }

  .top-site-card {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 12px;
    background: rgba(24, 24, 34, 0.65);
    backdrop-filter: blur(10px);
    border: 1px solid var(--color-border, #2a2a3c);
    border-radius: var(--radius-md, 8px);
    text-decoration: none;
    transition:
      transform 0.2s ease,
      border-color 0.2s ease;
  }

  .top-site-card:hover {
    transform: translateY(-2px);
    border-color: rgba(255, 71, 133, 0.6);
  }

  .top-site-favicon {
    width: 24px;
    height: 24px;
    border-radius: 6px;
    flex-shrink: 0;
  }

  .top-site-info {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .top-site-title {
    font-size: 13px;
    font-weight: 500;
    color: var(--color-text-main, #f0f0f5);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .top-site-meta {
    font-size: 11px;
    color: var(--color-text-muted, #8b8b9e);
  }
</style>
