export interface ThemeVariables {
  '--color-bg-base': string;
  '--color-bg-surface': string;
  '--color-bg-elevated': string;
  '--color-text-main': string;
  '--color-text-muted': string;
  '--color-accent': string;
  '--color-accent-hover': string;
  '--color-accent-active': string;
  '--color-border': string;
  '--color-tab-inactive': string;
  '--color-tab-active': string;
  '--color-tab-hover': string;
  '--radius-sm'?: string;
  '--radius-md'?: string;
  '--radius-lg'?: string;
  '--toolbar-height'?: string;
  '--tabbar-height'?: string;
  '--font-family-base'?: string;
  [key: `--${string}`]: string | undefined;
}

export interface ThemeDefinition {
  id: string;
  name: string;
  i18nKey?: string;
  description?: string;
  isDark: boolean;
  author?: string;
  version?: string;
  variables: ThemeVariables;
}
