export type SoundEvent =
  'tab.open' | 'tab.close' | 'tab.switch' | 'download.start' | 'download.done' | 'click';

export type SoundPackId = 'subtle' | 'kawaii' | 'cyberpunk' | 'gaming' | 'mecha' | string;

export interface SoundPack {
  id: SoundPackId;
  name: string;
  i18nKey: string;
  descriptionKey: string;
  volumeMultiplier?: number;
}

export interface SoundSettings {
  enabled: boolean;
  volume: number; // 0.0 to 1.0 (recommended default: 0.35)
  activePackId: SoundPackId;
  syncWithTheme: boolean;
}
