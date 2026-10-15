import { describe, it, expect } from 'vitest';
import {
  isRecordableUrl,
  isTauriRuntime,
  recordVisit,
  getTopSites,
  getHistory,
  searchHistory,
  deleteHistoryVisit,
} from './history';

describe('history bridge', () => {
  it('is not running in Tauri during tests', () => {
    expect(isTauriRuntime()).toBe(false);
  });

  it('only records http(s) urls', () => {
    expect(isRecordableUrl('https://example.com')).toBe(true);
    expect(isRecordableUrl('http://localhost:3000/x')).toBe(true);
    expect(isRecordableUrl('shinoho://hub')).toBe(false);
    expect(isRecordableUrl('shinoho://incognito')).toBe(false);
    expect(isRecordableUrl('about:blank')).toBe(false);
  });

  it('recordVisit is a no-op outside Tauri and never throws', async () => {
    await expect(
      recordVisit({ url: 'https://example.com', title: 'Example', typed: true }),
    ).resolves.toBeUndefined();
    await expect(
      recordVisit({ url: 'shinoho://hub', title: 'Hub', typed: false }),
    ).resolves.toBeUndefined();
  });

  it('read accessors fall back to empty results outside Tauri', async () => {
    await expect(getTopSites()).resolves.toEqual([]);
    await expect(getHistory()).resolves.toEqual([]);
    await expect(searchHistory('example')).resolves.toEqual([]);
    await expect(deleteHistoryVisit(1)).resolves.toBe(false);
  });
});
