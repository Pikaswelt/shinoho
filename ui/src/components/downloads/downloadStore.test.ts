import { describe, it, expect, beforeEach } from 'vitest';
import { downloadStore } from './downloadStore';

describe('downloadStore', () => {
  beforeEach(() => {
    downloadStore.resetForTests();
  });

  it('starts with a closed manager', () => {
    expect(downloadStore.isManagerOpen).toBe(false);
    expect(downloadStore.getState().isManagerOpen).toBe(false);
  });

  it('opens and closes the manager and notifies subscribers', () => {
    const seen: boolean[] = [];
    const unsubscribe = downloadStore.subscribe((s) => seen.push(s.isManagerOpen));

    downloadStore.openManager();
    expect(downloadStore.isManagerOpen).toBe(true);

    downloadStore.closeManager();
    expect(downloadStore.isManagerOpen).toBe(false);

    // Initialer Sync-Callback (false) + open (true) + close (false).
    expect(seen).toEqual([false, true, false]);

    unsubscribe();
  });

  it('does not notify twice when opening an already open manager', () => {
    let calls = 0;
    const unsubscribe = downloadStore.subscribe(() => {
      calls += 1;
    });

    downloadStore.openManager();
    downloadStore.openManager();
    // 1x initial + 1x open.
    expect(calls).toBe(2);

    unsubscribe();
  });
});
