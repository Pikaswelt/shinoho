/**
 * Gaming-Modus (T-603): Status vom Kern holen und auf Änderungen hören.
 * Ohne Tauri (Browser-Entwicklung, Tests) arbeitet der Dienst mit einem Speicher-Zustand.
 */

export type Preset = 'battery-saver' | 'balanced' | 'max-power' | 'gaming';

export interface GamingStatus {
  active: boolean;
  game: string | null;
  auto: boolean;
  preset: Preset;
}

export const GAMING_COMMANDS = {
  STATUS: 'gaming_status',
  SET_AUTO: 'gaming_set_auto',
} as const;

export const GAMING_EVENT = 'shinoho://gaming-changed';

type Listener = (status: GamingStatus) => void;

const INITIAL: GamingStatus = { active: false, game: null, auto: true, preset: 'balanced' };

function isTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

class GamingService {
  private status: GamingStatus = { ...INITIAL };
  private listeners = new Set<Listener>();
  private unlistenTauri: (() => void) | null = null;

  async getStatus(): Promise<GamingStatus> {
    if (isTauri()) {
      const { invoke } = await import('@tauri-apps/api/core');
      this.status = await invoke<GamingStatus>(GAMING_COMMANDS.STATUS);
    }
    return { ...this.status };
  }

  async setAuto(enabled: boolean): Promise<GamingStatus> {
    if (isTauri()) {
      const { invoke } = await import('@tauri-apps/api/core');
      this.status = await invoke<GamingStatus>(GAMING_COMMANDS.SET_AUTO, { enabled });
    } else {
      this.status = {
        ...this.status,
        auto: enabled,
        preset: this.status.active && enabled ? 'gaming' : 'balanced',
      };
    }
    this.emit(this.status);
    return { ...this.status };
  }

  /** Meldet jede Änderung. Gibt eine Abmelde-Funktion zurück. */
  onChange(listener: Listener): () => void {
    this.listeners.add(listener);
    if (isTauri() && !this.unlistenTauri) {
      void import('@tauri-apps/api/event').then(async ({ listen }) => {
        this.unlistenTauri = await listen<GamingStatus>(GAMING_EVENT, (e) => {
          this.status = e.payload;
          this.emit(e.payload);
        });
      });
    }
    return () => {
      this.listeners.delete(listener);
      if (this.listeners.size === 0 && this.unlistenTauri) {
        this.unlistenTauri();
        this.unlistenTauri = null;
      }
    };
  }

  /** Nur für Tests und Browser-Entwicklung: simuliert eine Meldung vom Kern. */
  simulate(status: Partial<GamingStatus>): void {
    this.status = { ...this.status, ...status };
    this.emit(this.status);
  }

  resetMock(): void {
    this.status = { ...INITIAL };
    this.listeners.clear();
  }

  private emit(status: GamingStatus): void {
    this.listeners.forEach((l) => l({ ...status }));
  }
}

export const gamingService = new GamingService();
