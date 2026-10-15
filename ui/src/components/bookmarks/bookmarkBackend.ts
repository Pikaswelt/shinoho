import type { Bookmark } from './bookmarks.types';

/** Rust-Seite (core/storage) serialisiert Bookmarks in snake_case. */
export interface RawBookmark {
  id: string;
  parent_id: string | null;
  title: string;
  url: string | null;
  is_folder: boolean;
  position: number;
  tags: string[];
  favicon_url: string | null;
  created_at: number;
  updated_at: number;
}

export interface RawBookmarkNode {
  item: RawBookmark;
  children: RawBookmarkNode[];
}

export const BOOKMARK_COMMANDS = {
  CREATE: 'create_bookmark',
  UPDATE: 'update_bookmark',
  DELETE: 'delete_bookmark',
  GET_TREE: 'get_bookmark_tree',
} as const;

export function isTauriRuntime(): boolean {
  return (
    typeof window !== 'undefined' &&
    '__TAURI_INTERNALS__' in window &&
    Boolean(window.__TAURI_INTERNALS__)
  );
}

export function toUiBookmark(raw: RawBookmark): Bookmark {
  return {
    id: raw.id,
    parentId: raw.parent_id,
    title: raw.title,
    url: raw.url,
    isFolder: raw.is_folder,
    position: raw.position,
    tags: raw.tags,
    faviconUrl: raw.favicon_url,
    createdAt: raw.created_at,
    updatedAt: raw.updated_at,
  };
}

/** Flacht den Baum aus get_bookmark_tree zu einer Liste ab. */
export function flattenRawTree(nodes: RawBookmarkNode[]): Bookmark[] {
  const out: Bookmark[] = [];
  const walk = (list: RawBookmarkNode[]) => {
    for (const node of list) {
      out.push(toUiBookmark(node.item));
      walk(node.children);
    }
  };
  walk(nodes);
  return out;
}

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke } = await import('@tauri-apps/api/core');
  return invoke<T>(command, args);
}

export async function fetchBookmarkTree(): Promise<Bookmark[]> {
  const tree = await call<RawBookmarkNode[]>(BOOKMARK_COMMANDS.GET_TREE, { rootId: null });
  return flattenRawTree(tree);
}

export async function createBookmarkRemote(bookmark: Bookmark): Promise<void> {
  await call(BOOKMARK_COMMANDS.CREATE, {
    request: {
      id: bookmark.id,
      parentId: bookmark.parentId,
      title: bookmark.title,
      url: bookmark.url,
      isFolder: bookmark.isFolder,
      position: bookmark.position,
      tags: bookmark.tags,
      faviconUrl: bookmark.faviconUrl,
    },
  });
}

/** Leere Strings loeschen url/faviconUrl in der Rust-Seite. */
export async function updateBookmarkRemote(
  id: string,
  patch: {
    title?: string;
    url?: string | null;
    parentId?: string | null;
    position?: number;
    tags?: string[];
    faviconUrl?: string | null;
  },
): Promise<void> {
  await call(BOOKMARK_COMMANDS.UPDATE, {
    id,
    request: {
      title: patch.title,
      url: patch.url === undefined ? undefined : (patch.url ?? ''),
      parentId: patch.parentId ?? undefined,
      position: patch.position,
      tags: patch.tags,
      faviconUrl: patch.faviconUrl === undefined ? undefined : (patch.faviconUrl ?? ''),
    },
  });
}

export async function deleteBookmarkRemote(id: string): Promise<void> {
  await call(BOOKMARK_COMMANDS.DELETE, { id });
}
