/**
 * Downloads-Bridge (T-112).
 *
 * Bindet an die echten Tauri-Commands aus `apps/desktop/src/commands.rs`
 * (start_download, pause_download, resume_download, cancel_download,
 * retry_download, get_download, list_downloads, delete_download,
 * clear_completed_downloads) und spiegelt die Download-Events
 * (`shinoho://download-*` aus `core/downloads/src/events.rs`).
 *
 * Ausserhalb der Desktop-App (Browser-Builds, Tests) liefern alle
 * Lesezugriffe leere Ergebnisse und Aktionen scheitern still, damit der
 * Build ohne Backend lauffaehig bleibt. Das vollstaendige
 * Downloads-Manager-UI gehoert zu T-109.
 */

export type DownloadState =
  'pending' | 'downloading' | 'paused' | 'completed' | 'cancelled' | 'failed';

export interface DownloadItem {
  id: string;
  url: string;
  target_path: string;
  filename: string;
  total_bytes: number | null;
  received_bytes: number;
  state: DownloadState;
  error_message: string | null;
  speed_bps: number;
  started_at: number;
  completed_at: number | null;
}

export interface DownloadOptions {
  customFilename?: string | null;
  targetDir?: string | null;
  headers?: Record<string, string>;
  allowOverwrite?: boolean;
}

export interface DownloadProgressEvent {
  id: string;
  url: string;
  filename: string;
  received_bytes: number;
  total_bytes: number | null;
  speed_bps: number;
  state: DownloadState;
  progress_percent: number | null;
}

export const DOWNLOAD_COMMANDS = {
  START: 'start_download',
  PAUSE: 'pause_download',
  RESUME: 'resume_download',
  CANCEL: 'cancel_download',
  RETRY: 'retry_download',
  GET: 'get_download',
  LIST: 'list_downloads',
  DELETE: 'delete_download',
  CLEAR_COMPLETED: 'clear_completed_downloads',
} as const;

export const DOWNLOAD_EVENTS = {
  STARTED: 'shinoho://download-started',
  PROGRESS: 'shinoho://download-progress',
  PAUSED: 'shinoho://download-paused',
  RESUMED: 'shinoho://download-resumed',
  COMPLETED: 'shinoho://download-completed',
  CANCELLED: 'shinoho://download-cancelled',
  FAILED: 'shinoho://download-failed',
} as const;

export type DownloadEventListener = (event: DownloadProgressEvent) => void;

export function isTauriRuntime(): boolean {
  return (
    typeof window !== 'undefined' &&
    '__TAURI_INTERNALS__' in window &&
    Boolean(window.__TAURI_INTERNALS__)
  );
}

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke } = await import('@tauri-apps/api/core');
  return invoke<T>(command, args);
}

function toBackendOptions(options?: DownloadOptions): Record<string, unknown> | undefined {
  if (!options) return undefined;
  return {
    custom_filename: options.customFilename ?? null,
    target_dir: options.targetDir ?? null,
    headers: options.headers ?? {},
    allow_overwrite: options.allowOverwrite ?? false,
  };
}

export async function startDownload(
  url: string,
  options?: DownloadOptions,
): Promise<DownloadItem | null> {
  if (!isTauriRuntime()) return null;
  try {
    return await call<DownloadItem>(DOWNLOAD_COMMANDS.START, {
      url,
      options: toBackendOptions(options),
    });
  } catch {
    return null;
  }
}

export async function pauseDownload(id: string): Promise<boolean> {
  if (!isTauriRuntime()) return false;
  try {
    await call(DOWNLOAD_COMMANDS.PAUSE, { id });
    return true;
  } catch {
    return false;
  }
}

export async function resumeDownload(id: string): Promise<boolean> {
  if (!isTauriRuntime()) return false;
  try {
    await call(DOWNLOAD_COMMANDS.RESUME, { id });
    return true;
  } catch {
    return false;
  }
}

export async function cancelDownload(id: string): Promise<boolean> {
  if (!isTauriRuntime()) return false;
  try {
    await call(DOWNLOAD_COMMANDS.CANCEL, { id });
    return true;
  } catch {
    return false;
  }
}

export async function retryDownload(id: string): Promise<DownloadItem | null> {
  if (!isTauriRuntime()) return null;
  try {
    return await call<DownloadItem>(DOWNLOAD_COMMANDS.RETRY, { id });
  } catch {
    return null;
  }
}

export async function getDownload(id: string): Promise<DownloadItem | null> {
  if (!isTauriRuntime()) return null;
  try {
    return await call<DownloadItem | null>(DOWNLOAD_COMMANDS.GET, { id });
  } catch {
    return null;
  }
}

export async function listDownloads(limit = 50, offset = 0): Promise<DownloadItem[]> {
  if (!isTauriRuntime()) return [];
  try {
    return await call<DownloadItem[]>(DOWNLOAD_COMMANDS.LIST, { limit, offset });
  } catch {
    return [];
  }
}

export async function deleteDownload(id: string, deleteFile = false): Promise<boolean> {
  if (!isTauriRuntime()) return false;
  try {
    return await call<boolean>(DOWNLOAD_COMMANDS.DELETE, { id, deleteFile });
  } catch {
    return false;
  }
}

export async function clearCompletedDownloads(): Promise<number> {
  if (!isTauriRuntime()) return 0;
  try {
    return await call<number>(DOWNLOAD_COMMANDS.CLEAR_COMPLETED);
  } catch {
    return 0;
  }
}

/**
 * Abonniert die Download-Lifecycle-Events des Backends. Ausserhalb von
 * Tauri ein No-op, das eine leere Unsubscribe-Funktion zurueckgibt.
 */
export function onDownloadEvent(handler: DownloadEventListener): () => void {
  if (!isTauriRuntime()) return () => {};
  let unlistenAll: (() => void) | undefined;
  import('@tauri-apps/api/event').then(({ listen }) => {
    const events = [
      DOWNLOAD_EVENTS.STARTED,
      DOWNLOAD_EVENTS.PROGRESS,
      DOWNLOAD_EVENTS.PAUSED,
      DOWNLOAD_EVENTS.RESUMED,
      DOWNLOAD_EVENTS.COMPLETED,
      DOWNLOAD_EVENTS.CANCELLED,
      DOWNLOAD_EVENTS.FAILED,
    ];
    Promise.all(
      events.map((evt) => listen(evt, (e) => handler(e.payload as DownloadProgressEvent))),
    ).then((unsubscribers) => {
      unlistenAll = () => unsubscribers.forEach((u) => u());
    });
  });
  return () => {
    if (unlistenAll) unlistenAll();
  };
}
