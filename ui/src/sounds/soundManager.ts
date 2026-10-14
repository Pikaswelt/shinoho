import type { SoundEvent, SoundPack, SoundPackId, SoundSettings } from './types';
import { BUILTIN_SOUND_PACKS, DEFAULT_SOUND_SETTINGS } from './presets';
import { synthesizeSound } from './audioSynthesis';

const STORAGE_KEY = 'shinoho:sounds';

export class SoundManager {
  private settings: SoundSettings = { ...DEFAULT_SOUND_SETTINGS };
  private packs = new Map<string, SoundPack>();
  private listeners = new Set<(settings: SoundSettings) => void>();
  private isInitialized = false;

  constructor() {
    for (const pack of BUILTIN_SOUND_PACKS) {
      this.packs.set(pack.id, pack);
    }
  }

  public init(initialSettings?: Partial<SoundSettings>): SoundSettings {
    if (this.isInitialized) {
      return this.settings;
    }

    if (typeof localStorage !== 'undefined') {
      try {
        const saved = localStorage.getItem(STORAGE_KEY);
        if (saved) {
          const parsed = JSON.parse(saved);
          this.settings = {
            ...this.settings,
            ...parsed,
          };
        }
      } catch {
        // localStorage not available or JSON parse failed
      }
    }

    if (initialSettings) {
      this.settings = {
        ...this.settings,
        ...initialSettings,
      };
    }

    // Clamp volume
    this.settings.volume = Math.max(0, Math.min(1, this.settings.volume));

    this.isInitialized = true;
    return this.settings;
  }

  public play(event: SoundEvent): void {
    if (!this.settings.enabled || this.settings.volume <= 0.001) {
      return;
    }

    const pack = this.packs.get(this.settings.activePackId);
    const multiplier = pack?.volumeMultiplier ?? 1.0;
    const effectiveVolume = this.settings.volume * multiplier;

    try {
      synthesizeSound(this.settings.activePackId, event, effectiveVolume);
    } catch (err) {
      // Audio playback should never crash the app
      console.warn('Failed to play UI sound:', err);
    }
  }

  public setEnabled(enabled: boolean): void {
    this.settings.enabled = enabled;
    this.saveAndNotify();
  }

  public setVolume(volume: number): void {
    this.settings.volume = Math.max(0, Math.min(1, volume));
    this.saveAndNotify();
  }

  public setSoundPack(packId: SoundPackId): void {
    this.settings.activePackId = packId;
    this.saveAndNotify();
  }

  public setSyncWithTheme(sync: boolean): void {
    this.settings.syncWithTheme = sync;
    this.saveAndNotify();
  }

  public handleThemeChange(defaultSoundPack?: string): void {
    if (this.settings.syncWithTheme && defaultSoundPack) {
      if (this.packs.has(defaultSoundPack) && this.settings.activePackId !== defaultSoundPack) {
        this.settings.activePackId = defaultSoundPack;
        this.saveAndNotify();
      }
    }
  }

  public getSettings(): SoundSettings {
    return { ...this.settings };
  }

  public getAvailablePacks(): SoundPack[] {
    return Array.from(this.packs.values());
  }

  public registerSoundPack(pack: SoundPack): void {
    this.packs.set(pack.id, pack);
  }

  public resetToDefaults(): void {
    this.settings = { ...DEFAULT_SOUND_SETTINGS };
    this.saveAndNotify();
  }

  public subscribe(listener: (settings: SoundSettings) => void): () => void {
    this.listeners.add(listener);
    listener(this.getSettings());
    return () => {
      this.listeners.delete(listener);
    };
  }

  private saveAndNotify(): void {
    if (typeof localStorage !== 'undefined') {
      try {
        localStorage.setItem(STORAGE_KEY, JSON.stringify(this.settings));
      } catch {
        // Storage write failed
      }
    }

    const snapshot = this.getSettings();
    for (const listener of this.listeners) {
      try {
        listener(snapshot);
      } catch (err) {
        console.error('Error in sound settings listener:', err);
      }
    }
  }
}

export const soundManager = new SoundManager();
