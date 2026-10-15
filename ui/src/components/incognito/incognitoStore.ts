import {
  type IncognitoSessionState,
  type IncognitoTheme,
  DEFAULT_INCOGNITO_CONFIG,
  DEFAULT_INCOGNITO_THEME,
} from './incognito.types';
import { isTauriRuntime } from '../bookmarks/bookmarkBackend';

export const NINJA_MASCOT_QUOTES = [
  'Senpai! In diesem Schattenreich hinterlassen wir keine Fußspuren. 🥷✨',
  'Lautlos wie ein Schatten im Bambuswald – dein Verlauf ist sicher! 🍃',
  'Keine Krümel, keine Kekse für neugierige Tracker! Ninja-Shino bewacht dich. ⚔️',
  'Schleichen wie ein Shinobi: Sobald das Fenster schließt, löst sich alles in Rauch auf. 💨',
  'Privatsphäre ist Ehrensache! Weder Verlauf noch Disk-Cache berühren deine Festplatte. 🛡️',
];

// Rust-Struct `IncognitoConfig` nutzt snake_case-Felder.
function toBackendConfig(state: IncognitoSessionState) {
  return {
    auto_enable_vpn: state.autoEnableVpn,
    disk_cache_enabled: state.diskCacheEnabled,
    private_browsing: DEFAULT_INCOGNITO_CONFIG.privateBrowsing,
    theme: {
      primary_color: state.theme.primaryColor,
      accent_color: state.theme.accentColor,
      mascot: state.theme.mascot,
    },
  };
}

function detectInitialIncognito(): boolean {
  if (typeof window === 'undefined') return false;
  try {
    const params = new URLSearchParams(window.location.search);
    if (params.get('incognito') === 'true') return true;
    if (window.location.hash.includes('incognito')) return true;
  } catch {
    // Ignore URL parse errors
  }
  return false;
}

export class IncognitoStore {
  private state: IncognitoSessionState;
  private listeners = new Set<(state: IncognitoSessionState) => void>();

  constructor() {
    const isIncognito = detectInitialIncognito();
    this.state = {
      isIncognito,
      sessionId: isIncognito ? `incognito-session-${Date.now()}` : null,
      autoEnableVpn: DEFAULT_INCOGNITO_CONFIG.autoEnableVpn,
      diskCacheEnabled: DEFAULT_INCOGNITO_CONFIG.diskCacheEnabled,
      theme: { ...DEFAULT_INCOGNITO_THEME },
    };
  }

  private notify(): void {
    const current = this.getState();
    this.listeners.forEach((fn) => fn(current));
  }

  public getState(): IncognitoSessionState {
    return {
      ...this.state,
      theme: { ...this.state.theme },
    };
  }

  public subscribe(callback: (state: IncognitoSessionState) => void): () => void {
    this.listeners.add(callback);
    callback(this.getState());
    return () => {
      this.listeners.delete(callback);
    };
  }

  public get isIncognito(): boolean {
    return this.state.isIncognito;
  }

  public get sessionId(): string | null {
    return this.state.sessionId;
  }

  public get autoEnableVpn(): boolean {
    return this.state.autoEnableVpn;
  }

  public get theme(): IncognitoTheme {
    return this.state.theme;
  }

  public enableIncognito(customSessionId?: string): void {
    this.state.isIncognito = true;
    const localId = customSessionId || `incognito-session-${Date.now()}`;
    this.state.sessionId = localId;
    this.notify();
    if (isTauriRuntime()) {
      // Desktop: Sitzung im Rust-Backend anlegen und deren ID übernehmen.
      this.invokeBackend<string>('create_incognito_session', {
        customId: customSessionId ?? null,
        config: toBackendConfig(this.state),
      })
        .then((backendId) => {
          if (this.state.isIncognito && this.state.sessionId === localId) {
            this.state.sessionId = backendId;
            this.notify();
          }
        })
        .catch((err) =>
          console.warn('[incognito] Sitzung konnte nicht im Backend angelegt werden', err),
        );
    }
  }

  public disableIncognito(): void {
    const previousId = this.state.sessionId;
    this.state.isIncognito = false;
    this.state.sessionId = null;
    this.notify();
    if (isTauriRuntime() && previousId) {
      this.invokeBackend<void>('close_incognito_session', { sessionId: previousId }).catch((err) =>
        console.warn('[incognito] Sitzung konnte nicht geschlossen werden', err),
      );
    }
  }

  private async invokeBackend<T>(command: string, args: Record<string, unknown>): Promise<T> {
    const { invoke } = await import('@tauri-apps/api/core');
    return invoke<T>(command, args);
  }

  public toggleAutoVpn(): void {
    this.state.autoEnableVpn = !this.state.autoEnableVpn;
    this.notify();
  }

  public setAutoVpn(enabled: boolean): void {
    this.state.autoEnableVpn = enabled;
    this.notify();
  }

  public openIncognitoWindow(): void {
    // Try Tauri IPC if available, else open new browser window with query param
    if (typeof window !== 'undefined') {
      const url = new URL(window.location.href);
      url.searchParams.set('incognito', 'true');
      window.open(url.toString(), '_blank', 'width=1200,height=800');
    }
  }

  public getRandomNinjaQuote(): string {
    const idx = Math.floor(Math.random() * NINJA_MASCOT_QUOTES.length);
    return NINJA_MASCOT_QUOTES[idx];
  }
}

export function createIncognitoStore(): IncognitoStore {
  return new IncognitoStore();
}

export const incognitoStore = createIncognitoStore();
