import { describe, it, expect, beforeEach } from 'vitest';
import { mount, unmount, flushSync, tick } from 'svelte';
import GamingModeBanner from './GamingModeBanner.svelte';
import { gamingService, type GamingStatus } from '../../api/gaming';
import { setLocale } from '../../i18n';

async function settle() {
  await Promise.resolve();
  await tick();
  flushSync();
}

describe('gamingService (ohne Tauri)', () => {
  beforeEach(() => gamingService.resetMock());

  it('startet ohne laufendes Spiel', async () => {
    const s = await gamingService.getStatus();
    expect(s).toEqual({ active: false, game: null, auto: true, preset: 'balanced' });
  });

  it('meldet Änderungen und schaltet das Preset nur mit Auto um', async () => {
    const seen: GamingStatus[] = [];
    const off = gamingService.onChange((s) => seen.push(s));
    gamingService.simulate({ active: true, game: 'cs2', preset: 'gaming' });
    const manual = await gamingService.setAuto(false);
    expect(manual.preset).toBe('balanced');
    const auto = await gamingService.setAuto(true);
    expect(auto.preset).toBe('gaming');
    expect(seen.map((s) => s.auto)).toEqual([true, false, true]);
    off();
  });
});

describe('GamingModeBanner', () => {
  beforeEach(() => {
    gamingService.resetMock();
    setLocale('de');
    document.body.innerHTML = '';
  });

  it('ist unsichtbar, solange kein Spiel läuft', async () => {
    const c = mount(GamingModeBanner, { target: document.body });
    await settle();
    expect(document.querySelector('[data-testid="gaming-banner"]')).toBeNull();
    unmount(c);
  });

  it('erscheint beim Spielstart mit Spielnamen und lässt sich ausblenden', async () => {
    const c = mount(GamingModeBanner, { target: document.body });
    await settle();
    gamingService.simulate({ active: true, game: 'Minecraft', preset: 'gaming' });
    await settle();
    const banner = document.querySelector('[data-testid="gaming-banner"]');
    expect(banner?.textContent).toContain('Gaming-Modus: Minecraft');

    (document.querySelector('[data-testid="gaming-dismiss"]') as HTMLButtonElement).click();
    await settle();
    expect(document.querySelector('[data-testid="gaming-banner"]')).toBeNull();

    // Ein anderes Spiel zeigt den Hinweis wieder an.
    gamingService.simulate({ game: 'cs2' });
    await settle();
    expect(document.querySelector('[data-testid="gaming-banner"]')).not.toBeNull();
    unmount(c);
  });

  it('schaltet automatisches Sparen über die Checkbox um', async () => {
    const c = mount(GamingModeBanner, { target: document.body });
    gamingService.simulate({ active: true, game: 'cs2', preset: 'gaming' });
    await settle();
    (document.querySelector('[data-testid="gaming-auto-toggle"]') as HTMLInputElement).click();
    await settle();
    await settle();
    expect((await gamingService.getStatus()).auto).toBe(false);
    expect(document.querySelector('[data-testid="gaming-banner"]')?.textContent).toContain(
      'Automatisches Sparen ist aus.',
    );
    unmount(c);
  });
});
