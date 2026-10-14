export type WallpaperType =
  | 'sakura'
  | 'cyberpunk'
  | 'lofi'
  | 'nebula'
  | 'gradient'
  | 'image'
  | 'video'
  | 'retrowave'
  | 'pixelart'
  | 'lofi-room';

export interface WallpaperPreset {
  id: string;
  nameKey: string;
  type: WallpaperType;
  src?: string;
  gradient?: string;
  previewColor: string;
  descriptionKey: string;
}

export interface WallpaperSettings {
  activeId: string;
  customUrl?: string;
  blur: number; // 0 to 20 px
  brightness: number; // 0.3 to 1.5
  opacity: number; // 0.1 to 1.0
  particlesEnabled: boolean;
  particleDensity: number; // 10 to 100
  syncWithTheme?: boolean;
}

export interface SpeedDialItem {
  id: string;
  title: string;
  url: string;
  icon?: string;
  category?: string;
  clicks: number;
  pinned: boolean;
  createdAt: number;
}

export interface SearchEngine {
  id: string;
  name: string;
  queryUrl: string; // '%s' replaced with encoded query
  suggestUrl?: string;
  icon: string;
}

export interface AnimeCountdownItem {
  id: string;
  title: string;
  nextEpisode: number;
  airingAt: string; // ISO date string
  bannerUrl?: string;
  siteUrl?: string;
}

export interface FreeGameItem {
  id: string;
  title: string;
  platform: 'Epic Games' | 'Steam' | 'GOG' | 'Prime Gaming';
  claimUntil: string;
  bannerUrl?: string;
  url: string;
}

export interface HubState {
  wallpaper: WallpaperSettings;
  speedDial: SpeedDialItem[];
  selectedSearchEngine: string;
  widgets: {
    animeCountdown: boolean;
    freeGames: boolean;
    mascot: boolean;
    clock: boolean;
  };
}
