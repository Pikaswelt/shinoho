import { describe, it, expect, vi } from 'vitest';
import {
  isTauriRuntime,
  startDownload,
  pauseDownload,
  resumeDownload,
  cancelDownload,
  retryDownload,
  getDownload,
  listDownloads,
  deleteDownload,
  clearCompletedDownloads,
  onDownloadEvent,
} from './downloads';

describe('downloads bridge', () => {
  it('is not running in Tauri during tests', () => {
    expect(isTauriRuntime()).toBe(false);
  });

  it('actions fail softly outside Tauri', async () => {
    await expect(startDownload('https://example.com/file.zip')).resolves.toBeNull();
    await expect(pauseDownload('id-1')).resolves.toBe(false);
    await expect(resumeDownload('id-1')).resolves.toBe(false);
    await expect(cancelDownload('id-1')).resolves.toBe(false);
    await expect(retryDownload('id-1')).resolves.toBeNull();
    await expect(getDownload('id-1')).resolves.toBeNull();
    await expect(deleteDownload('id-1')).resolves.toBe(false);
    await expect(clearCompletedDownloads()).resolves.toBe(0);
  });

  it('listDownloads falls back to an empty list outside Tauri', async () => {
    await expect(listDownloads()).resolves.toEqual([]);
  });

  it('onDownloadEvent is a safe no-op outside Tauri', () => {
    const unsubscribe = onDownloadEvent(vi.fn());
    expect(() => unsubscribe()).not.toThrow();
  });
});
