import type {
  HubState,
  SpeedDialItem,
  WallpaperPreset,
  WallpaperSettings,
  SearchEngine,
  AnimeCountdownItem,
  FreeGameItem,
} from './hub.types';

export const WALLPAPER_PRESETS: WallpaperPreset[] = [
  {
    id: 'sakura',
    nameKey: 'hub.wallpaper.presets.sakura',
    type: 'sakura',
    gradient: 'radial-gradient(ellipse at bottom, #2b1120 0%, #120914 70%, #0a040b 100%)',
    previewColor: '#ff7597',
    descriptionKey: 'hub.wallpaper.presets.sakuraDesc',
  },
  {
    id: 'cyberpunk',
    nameKey: 'hub.wallpaper.presets.cyberpunk',
    type: 'cyberpunk',
    gradient: 'radial-gradient(circle at 50% 20%, #150a2e 0%, #080d21 60%, #02040d 100%)',
    previewColor: '#00e5ff',
    descriptionKey: 'hub.wallpaper.presets.cyberpunkDesc',
  },
  {
    id: 'lofi',
    nameKey: 'hub.wallpaper.presets.lofi',
    type: 'lofi',
    gradient: 'linear-gradient(135deg, #2c1654 0%, #462255 45%, #683257 80%, #ff8364 100%)',
    previewColor: '#ff8364',
    descriptionKey: 'hub.wallpaper.presets.lofiDesc',
  },
  {
    id: 'nebula',
    nameKey: 'hub.wallpaper.presets.nebula',
    type: 'nebula',
    gradient: 'radial-gradient(circle at 80% 30%, #1a1040 0%, #0d0a24 50%, #050410 100%)',
    previewColor: '#9d4edd',
    descriptionKey: 'hub.wallpaper.presets.nebulaDesc',
  },
  {
    id: 'minimal',
    nameKey: 'hub.wallpaper.presets.minimal',
    type: 'gradient',
    gradient: 'linear-gradient(180deg, #16161f 0%, #0c0c12 100%)',
    previewColor: '#495057',
    descriptionKey: 'hub.wallpaper.presets.minimalDesc',
  },
];

export const SEARCH_ENGINES: SearchEngine[] = [
  {
    id: 'duckduckgo',
    name: 'DuckDuckGo',
    queryUrl: 'https://duckduckgo.com/?q=%s',
    icon: '🦆',
  },
  {
    id: 'startpage',
    name: 'Startpage',
    queryUrl: 'https://www.startpage.com/sp/search?query=%s',
    icon: '🔒',
  },
  {
    id: 'google',
    name: 'Google',
    queryUrl: 'https://www.google.com/search?q=%s',
    icon: '🔍',
  },
  {
    id: 'ecosia',
    name: 'Ecosia',
    queryUrl: 'https://www.ecosia.org/search?q=%s',
    icon: '🌱',
  },
];

export const DEFAULT_SPEED_DIALS: SpeedDialItem[] = [
  {
    id: 'sd-anilist',
    title: 'AniList',
    url: 'https://anilist.co',
    category: 'anime',
    icon: '🌸',
    clicks: 12,
    pinned: true,
    createdAt: 1700000000000,
  },
  {
    id: 'sd-mal',
    title: 'MyAnimeList',
    url: 'https://myanimelist.net',
    category: 'anime',
    icon: '📜',
    clicks: 8,
    pinned: false,
    createdAt: 1700000001000,
  },
  {
    id: 'sd-crunchyroll',
    title: 'Crunchyroll',
    url: 'https://www.crunchyroll.com',
    category: 'anime',
    icon: '🍥',
    clicks: 15,
    pinned: true,
    createdAt: 1700000002000,
  },
  {
    id: 'sd-steam',
    title: 'Steam',
    url: 'https://store.steampowered.com',
    category: 'gaming',
    icon: '🎮',
    clicks: 25,
    pinned: true,
    createdAt: 1700000003000,
  },
  {
    id: 'sd-twitch',
    title: 'Twitch',
    url: 'https://www.twitch.tv',
    category: 'gaming',
    icon: '🟣',
    clicks: 19,
    pinned: true,
    createdAt: 1700000004000,
  },
  {
    id: 'sd-youtube',
    title: 'YouTube',
    url: 'https://www.youtube.com',
    category: 'social',
    icon: '▶️',
    clicks: 42,
    pinned: true,
    createdAt: 1700000005000,
  },
  {
    id: 'sd-github',
    title: 'GitHub',
    url: 'https://github.com',
    category: 'dev',
    icon: '🐙',
    clicks: 30,
    pinned: false,
    createdAt: 1700000006000,
  },
];

export const DEFAULT_ANIME_COUNTDOWN: AnimeCountdownItem[] = [
  {
    id: 'ani-1',
    title: "Frieren: Beyond Journey's End",
    nextEpisode: 29,
    airingAt: new Date(Date.now() + 2 * 86400000 + 4 * 3600000).toISOString(),
    siteUrl: 'https://anilist.co',
  },
  {
    id: 'ani-2',
    title: 'Solo Leveling S2: Arise from the Shadow',
    nextEpisode: 1,
    airingAt: new Date(Date.now() + 5 * 86400000).toISOString(),
    siteUrl: 'https://anilist.co',
  },
  {
    id: 'ani-3',
    title: 'Chainsaw Man: Reze Arc',
    nextEpisode: 1,
    airingAt: new Date(Date.now() + 12 * 86400000).toISOString(),
    siteUrl: 'https://anilist.co',
  },
];

export const DEFAULT_FREE_GAMES: FreeGameItem[] = [
  {
    id: 'game-1',
    title: 'Ghostrunner 2',
    platform: 'Epic Games',
    claimUntil: 'Sonntag 17:00',
    url: 'https://store.epicgames.com',
  },
  {
    id: 'game-2',
    title: 'Fallout: New Vegas Ultimate',
    platform: 'Prime Gaming',
    claimUntil: 'Ende des Monats',
    url: 'https://gaming.amazon.com',
  },
  {
    id: 'game-3',
    title: 'The Witcher: Enhanced Edition',
    platform: 'GOG',
    claimUntil: 'Dauerhaft kostenlos',
    url: 'https://www.gog.com',
  },
];

const STORAGE_KEY = 'shinoho_hub_state_v1';

export function createDefaultHubState(): HubState {
  return {
    wallpaper: {
      activeId: 'sakura',
      blur: 0,
      brightness: 1.0,
      opacity: 1.0,
      particlesEnabled: true,
      particleDensity: 40,
    },
    speedDial: [...DEFAULT_SPEED_DIALS],
    selectedSearchEngine: 'duckduckgo',
    widgets: {
      animeCountdown: true,
      freeGames: true,
      mascot: true,
      clock: true,
    },
  };
}

class HubStore {
  private state: HubState;
  private listeners = new Set<(state: HubState) => void>();

  constructor() {
    this.state = this.loadFromStorage();
  }

  private loadFromStorage(): HubState {
    if (typeof localStorage !== 'undefined') {
      try {
        const raw = localStorage.getItem(STORAGE_KEY);
        if (raw) {
          const parsed = JSON.parse(raw);
          return {
            ...createDefaultHubState(),
            ...parsed,
            wallpaper: {
              ...createDefaultHubState().wallpaper,
              ...(parsed.wallpaper || {}),
            },
            widgets: {
              ...createDefaultHubState().widgets,
              ...(parsed.widgets || {}),
            },
          };
        }
      } catch {
        // Fall back on error
      }
    }
    return createDefaultHubState();
  }

  private saveToStorage(): void {
    if (typeof localStorage !== 'undefined') {
      try {
        localStorage.setItem(STORAGE_KEY, JSON.stringify(this.state));
      } catch {
        // Ignore storage write errors (quota, incognito)
      }
    }
    this.notify();
  }

  private notify(): void {
    const current = this.getState();
    this.listeners.forEach((fn) => fn(current));
  }

  public getState(): HubState {
    return JSON.parse(JSON.stringify(this.state));
  }

  public subscribe(callback: (state: HubState) => void): () => void {
    this.listeners.add(callback);
    callback(this.getState());
    return () => {
      this.listeners.delete(callback);
    };
  }

  // --- Speed Dial CRUD ---
  public addSpeedDial(item: Omit<SpeedDialItem, 'id' | 'clicks' | 'createdAt'>): SpeedDialItem {
    const newItem: SpeedDialItem = {
      ...item,
      id: `sd-${Date.now()}-${Math.random().toString(36).substring(2, 7)}`,
      clicks: 0,
      createdAt: Date.now(),
    };
    this.state.speedDial.push(newItem);
    this.saveToStorage();
    return newItem;
  }

  public updateSpeedDial(id: string, updates: Partial<Omit<SpeedDialItem, 'id'>>): boolean {
    const index = this.state.speedDial.findIndex((sd) => sd.id === id);
    if (index !== -1) {
      this.state.speedDial[index] = {
        ...this.state.speedDial[index],
        ...updates,
      };
      this.saveToStorage();
      return true;
    }
    return false;
  }

  public removeSpeedDial(id: string): boolean {
    const initialLen = this.state.speedDial.length;
    this.state.speedDial = this.state.speedDial.filter((sd) => sd.id !== id);
    if (this.state.speedDial.length !== initialLen) {
      this.saveToStorage();
      return true;
    }
    return false;
  }

  public recordSpeedDialClick(id: string): void {
    const item = this.state.speedDial.find((sd) => sd.id === id);
    if (item) {
      item.clicks = (item.clicks || 0) + 1;
      this.saveToStorage();
    }
  }

  public togglePin(id: string): void {
    const item = this.state.speedDial.find((sd) => sd.id === id);
    if (item) {
      item.pinned = !item.pinned;
      this.saveToStorage();
    }
  }

  // --- Wallpaper Settings ---
  public setActiveWallpaper(presetId: string): void {
    this.state.wallpaper.activeId = presetId;
    this.saveToStorage();
  }

  public updateWallpaperSettings(settings: Partial<WallpaperSettings>): void {
    this.state.wallpaper = {
      ...this.state.wallpaper,
      ...settings,
    };
    this.saveToStorage();
  }

  // --- Search Engine ---
  public setSelectedSearchEngine(engineId: string): void {
    if (SEARCH_ENGINES.some((e) => e.id === engineId)) {
      this.state.selectedSearchEngine = engineId;
      this.saveToStorage();
    }
  }

  public buildSearchUrl(query: string): string {
    const engine =
      SEARCH_ENGINES.find((e) => e.id === this.state.selectedSearchEngine) || SEARCH_ENGINES[0];
    return engine.queryUrl.replace('%s', encodeURIComponent(query.trim()));
  }

  // --- Widgets ---
  public toggleWidget(widgetKey: keyof HubState['widgets']): void {
    if (widgetKey in this.state.widgets) {
      this.state.widgets[widgetKey] = !this.state.widgets[widgetKey];
      this.saveToStorage();
    }
  }

  public resetToDefaults(): void {
    this.state = createDefaultHubState();
    this.saveToStorage();
  }
}

export const hubStore = new HubStore();
