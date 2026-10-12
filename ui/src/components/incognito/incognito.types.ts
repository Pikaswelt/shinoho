export interface IncognitoTheme {
  primaryColor: string;
  accentColor: string;
  mascot: string;
}

export interface IncognitoConfig {
  autoEnableVpn: boolean;
  diskCacheEnabled: boolean;
  privateBrowsing: boolean;
  theme: IncognitoTheme;
}

export interface IncognitoSessionState {
  isIncognito: boolean;
  sessionId: string | null;
  autoEnableVpn: boolean;
  diskCacheEnabled: boolean;
  theme: IncognitoTheme;
}

export const DEFAULT_INCOGNITO_THEME: IncognitoTheme = {
  primaryColor: '#2d1b4e', // Dark purple matching core/incognito
  accentColor: '#9d4edd', // Neon purple
  mascot: 'ninja-shino', // Ninja mascot look from FEATURES-ANIME-GAMING.md
};

export const DEFAULT_INCOGNITO_CONFIG: IncognitoConfig = {
  autoEnableVpn: true,
  diskCacheEnabled: false,
  privateBrowsing: true,
  theme: DEFAULT_INCOGNITO_THEME,
};
