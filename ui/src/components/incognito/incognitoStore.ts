import {
  type IncognitoSessionState,
  type IncognitoTheme,
  DEFAULT_INCOGNITO_CONFIG,
  DEFAULT_INCOGNITO_THEME,
} from './incognito.types';

export const NINJA_MASCOT_QUOTES = [
  'Senpai! In diesem Schattenreich hinterlassen wir keine Fußspuren. 🥷✨',
  'Lautlos wie ein Schatten im Bambuswald – dein Verlauf ist sicher! 🍃',
  'Keine Krümel, keine Kekse für neugierige Tracker! Ninja-Shino bewacht dich. ⚔️',
  'Schleichen wie ein Shinobi: Sobald das Fenster schließt, löst sich alles in Rauch auf. 💨',
  'Privatsphäre ist Ehrensache! Weder Verlauf noch Disk-Cache berühren deine Festplatte. 🛡️',
];

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
    this.state.sessionId = customSessionId || `incognito-session-${Date.now()}`;
    this.notify();
  }

  public disableIncognito(): void {
    this.state.isIncognito = false;
    this.state.sessionId = null;
    this.notify();
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
