/**
 * History-/Top-Sites-Bridge (T-112).
 *
 * Bindet die Hub-Verlaufsfunktionen an die echten Tauri-Commands aus
 * `apps/desktop/src/commands.rs` (record_visit, get_history, search_history,
 * get_top_sites, delete_history_visit, clear_history). Ausserhalb der
 * Desktop-App (Browser-Builds, Tests) bleibt das Modul ein harmloser No-op,
 * damit der Hub ohne Backend lauffaehig bleibt.
 */

export interface HistoryVisit {
  id: number;
  url: string;
  title: string;
  visited_at: number;
  visit_count: number;
  typed: boolean;
  favicon_url: string | null;
}

export interface TopSite {
  url: string;
  title: string;
  visit_count: number;
  last_visited: number;
}

export interface RecordVisitRequest {
  url: string;
  title: string;
  typed: boolean;
  faviconUrl?: string | null;
}

export const HISTORY_COMMANDS = {
  RECORD_VISIT: 'record_visit',
  GET_HISTORY: 'get_history',
  SEARCH_HISTORY: 'search_history',
  GET_TOP_SITES: 'get_top_sites',
  DELETE_HISTORY_VISIT: 'delete_history_visit',
  CLEAR_HISTORY: 'clear_history',
} as const;

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

/** Interne (shinoho://, about:) URLs werden nie in die Chronik geschrieben. */
export function isRecordableUrl(url: string): boolean {
  return url.startsWith('http://') || url.startsWith('https://');
}

/**
 * Zeichnet einen Seitenbesuch auf. Ausserhalb von Tauri sowie fuer interne
 * URLs ein No-op. Fehler werden geschluckt: Verlauf darf die Navigation nie
 * blockieren.
 */
export async function recordVisit(request: RecordVisitRequest): Promise<void> {
  if (!isTauriRuntime() || !isRecordableUrl(request.url)) return;
  try {
    await call(HISTORY_COMMANDS.RECORD_VISIT, {
      request: {
        url: request.url,
        title: request.title || request.url,
        typed: request.typed,
        faviconUrl: request.faviconUrl ?? null,
      },
    });
  } catch {
    // Verlauf ist Best-effort und darf Navigation nicht scheitern lassen.
  }
}

export async function getTopSites(limit = 8): Promise<TopSite[]> {
  if (!isTauriRuntime()) return [];
  try {
    return await call<TopSite[]>(HISTORY_COMMANDS.GET_TOP_SITES, { limit });
  } catch {
    return [];
  }
}

export async function getHistory(limit = 50, offset = 0): Promise<HistoryVisit[]> {
  if (!isTauriRuntime()) return [];
  try {
    return await call<HistoryVisit[]>(HISTORY_COMMANDS.GET_HISTORY, { limit, offset });
  } catch {
    return [];
  }
}

export async function searchHistory(query: string, limit = 50): Promise<HistoryVisit[]> {
  if (!isTauriRuntime()) return [];
  try {
    return await call<HistoryVisit[]>(HISTORY_COMMANDS.SEARCH_HISTORY, { query, limit });
  } catch {
    return [];
  }
}

export async function deleteHistoryVisit(id: number): Promise<boolean> {
  if (!isTauriRuntime()) return false;
  try {
    return await call<boolean>(HISTORY_COMMANDS.DELETE_HISTORY_VISIT, { id });
  } catch {
    return false;
  }
}
