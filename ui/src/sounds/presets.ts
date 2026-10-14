import type { SoundPack } from './types';

export const BUILTIN_SOUND_PACKS: SoundPack[] = [
  {
    id: 'subtle',
    name: 'Subtle UI',
    i18nKey: 'sounds.packs.subtle',
    descriptionKey: 'sounds.packs.subtleDesc',
    volumeMultiplier: 0.9,
  },
  {
    id: 'kawaii',
    name: 'Kawaii Anime',
    i18nKey: 'sounds.packs.kawaii',
    descriptionKey: 'sounds.packs.kawaiiDesc',
    volumeMultiplier: 1.0,
  },
  {
    id: 'cyberpunk',
    name: 'Cyberpunk 2088',
    i18nKey: 'sounds.packs.cyberpunk',
    descriptionKey: 'sounds.packs.cyberpunkDesc',
    volumeMultiplier: 0.85,
  },
  {
    id: 'gaming',
    name: '8-Bit Retro Gaming',
    i18nKey: 'sounds.packs.gaming',
    descriptionKey: 'sounds.packs.gamingDesc',
    volumeMultiplier: 0.8,
  },
  {
    id: 'mecha',
    name: 'Sci-Fi Mecha',
    i18nKey: 'sounds.packs.mecha',
    descriptionKey: 'sounds.packs.mechaDesc',
    volumeMultiplier: 0.9,
  },
];

export const DEFAULT_SOUND_SETTINGS = {
  enabled: true,
  volume: 0.35,
  activePackId: 'subtle',
  syncWithTheme: true,
};
