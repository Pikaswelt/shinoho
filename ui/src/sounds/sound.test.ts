import { describe, it, expect, beforeEach, vi } from 'vitest';
import { SoundManager } from './soundManager';
import { BUILTIN_SOUND_PACKS } from './presets';
import { setCustomAudioContext } from './audioSynthesis';
import type { SoundEvent } from './types';

function createMockAudioContext() {
  const oscMock = {
    type: 'sine',
    frequency: {
      setValueAtTime: vi.fn(),
      exponentialRampToValueAtTime: vi.fn(),
    },
    connect: vi.fn(),
    start: vi.fn(),
    stop: vi.fn(),
  };

  const gainMock = {
    gain: {
      setValueAtTime: vi.fn(),
      linearRampToValueAtTime: vi.fn(),
      exponentialRampToValueAtTime: vi.fn(),
    },
    connect: vi.fn(),
  };

  return {
    currentTime: 0,
    state: 'running',
    resume: vi.fn().mockResolvedValue(undefined),
    createOscillator: vi.fn().mockReturnValue(oscMock),
    createGain: vi.fn().mockReturnValue(gainMock),
    destination: {},
    _oscMock: oscMock,
    _gainMock: gainMock,
  } as unknown as AudioContext & { _oscMock: typeof oscMock; _gainMock: typeof gainMock };
}

describe('SoundManager', () => {
  let manager: SoundManager;
  let mockCtx: ReturnType<typeof createMockAudioContext>;

  beforeEach(() => {
    mockCtx = createMockAudioContext();
    setCustomAudioContext(mockCtx);
    localStorage.clear();
    manager = new SoundManager();
    manager.init();
  });

  it('initializes with default settings', () => {
    const settings = manager.getSettings();
    expect(settings.enabled).toBe(true);
    expect(settings.volume).toBe(0.35);
    expect(settings.activePackId).toBe('subtle');
    expect(settings.syncWithTheme).toBe(true);
  });

  it('lists all builtin soundpacks', () => {
    const packs = manager.getAvailablePacks();
    expect(packs.length).toBe(BUILTIN_SOUND_PACKS.length);
    const ids = packs.map((p) => p.id);
    expect(ids).toContain('subtle');
    expect(ids).toContain('kawaii');
    expect(ids).toContain('cyberpunk');
    expect(ids).toContain('gaming');
    expect(ids).toContain('mecha');
  });

  it('updates sound settings correctly', () => {
    manager.setEnabled(false);
    expect(manager.getSettings().enabled).toBe(false);

    manager.setVolume(0.8);
    expect(manager.getSettings().volume).toBe(0.8);

    // Clamping volume
    manager.setVolume(1.5);
    expect(manager.getSettings().volume).toBe(1.0);
    manager.setVolume(-0.2);
    expect(manager.getSettings().volume).toBe(0.0);

    manager.setSoundPack('kawaii');
    expect(manager.getSettings().activePackId).toBe('kawaii');

    manager.setSyncWithTheme(false);
    expect(manager.getSettings().syncWithTheme).toBe(false);
  });

  it('plays sounds when enabled and creates oscillators', () => {
    const events: SoundEvent[] = [
      'tab.open',
      'tab.close',
      'tab.switch',
      'download.start',
      'download.done',
      'click',
    ];

    for (const evt of events) {
      mockCtx.createOscillator = vi.fn().mockReturnValue(mockCtx._oscMock);
      manager.play(evt);
      expect(mockCtx.createOscillator).toHaveBeenCalled();
    }
  });

  it('does not play sounds when disabled or volume is zero', () => {
    manager.setEnabled(false);
    mockCtx.createOscillator = vi.fn();
    manager.play('tab.open');
    expect(mockCtx.createOscillator).not.toHaveBeenCalled();

    manager.setEnabled(true);
    manager.setVolume(0);
    manager.play('tab.open');
    expect(mockCtx.createOscillator).not.toHaveBeenCalled();
  });

  it('synthesizes sounds across all builtin soundpacks without error', () => {
    const packs = ['subtle', 'kawaii', 'cyberpunk', 'gaming', 'mecha'] as const;
    const events: SoundEvent[] = [
      'tab.open',
      'tab.close',
      'tab.switch',
      'download.start',
      'download.done',
      'click',
    ];

    for (const pack of packs) {
      manager.setSoundPack(pack);
      for (const evt of events) {
        expect(() => manager.play(evt)).not.toThrow();
      }
    }
  });

  it('handles theme change synchronization when syncWithTheme is true', () => {
    manager.setSyncWithTheme(true);
    manager.handleThemeChange('cyberpunk');
    expect(manager.getSettings().activePackId).toBe('cyberpunk');

    manager.handleThemeChange('kawaii');
    expect(manager.getSettings().activePackId).toBe('kawaii');

    // Ignores unknown pack
    manager.handleThemeChange('non-existent-pack');
    expect(manager.getSettings().activePackId).toBe('kawaii');

    // When syncWithTheme is false, does not change
    manager.setSyncWithTheme(false);
    manager.handleThemeChange('subtle');
    expect(manager.getSettings().activePackId).toBe('kawaii');
  });

  it('notifies subscribers on change', () => {
    const listener = vi.fn();
    const unsub = manager.subscribe(listener);

    // Initial call
    expect(listener).toHaveBeenCalledTimes(1);

    manager.setVolume(0.5);
    expect(listener).toHaveBeenCalledTimes(2);

    unsub();
    manager.setVolume(0.6);
    expect(listener).toHaveBeenCalledTimes(2);
  });

  it('resets to defaults', () => {
    manager.setEnabled(false);
    manager.setVolume(0.9);
    manager.setSoundPack('mecha');
    manager.setSyncWithTheme(false);

    manager.resetToDefaults();
    const settings = manager.getSettings();
    expect(settings.enabled).toBe(true);
    expect(settings.volume).toBe(0.35);
    expect(settings.activePackId).toBe('subtle');
    expect(settings.syncWithTheme).toBe(true);
  });
});
