import type {
  Bookmark,
  BookmarkNode,
  CreateBookmarkInput,
  UpdateBookmarkInput,
  BookmarkImportSummary,
} from './bookmarks.types';
import { exportToNetscapeHtml, parseNetscapeHtml } from './bookmarkHtml';

const STORAGE_KEY = 'shinoho_bookmarks';
const BAR_VISIBLE_KEY = 'shinoho_bookmarks_bar_visible';

export const SYSTEM_ROOT_TOOLBAR = 'toolbar';
export const SYSTEM_ROOT_OTHER = 'other';
export const SYSTEM_ROOT_MOBILE = 'mobile';

export interface BookmarkStoreState {
  bookmarks: Bookmark[];
  isBarVisible: boolean;
  isManagerOpen: boolean;
  activeFolderId: string;
  searchQuery: string;
  activeTag: string | null;
}

const DEFAULT_BOOKMARKS: Bookmark[] = [
  // Built-in Folders
  {
    id: SYSTEM_ROOT_TOOLBAR,
    parentId: null,
    title: 'Lesezeichenleiste',
    url: null,
    isFolder: true,
    position: 0,
    tags: [],
    faviconUrl: null,
    createdAt: 1728740000000,
    updatedAt: 1728740000000,
  },
  {
    id: SYSTEM_ROOT_OTHER,
    parentId: null,
    title: 'Weitere Lesezeichen',
    url: null,
    isFolder: true,
    position: 1,
    tags: [],
    faviconUrl: null,
    createdAt: 1728740000000,
    updatedAt: 1728740000000,
  },
  {
    id: SYSTEM_ROOT_MOBILE,
    parentId: null,
    title: 'Mobile Lesezeichen',
    url: null,
    isFolder: true,
    position: 2,
    tags: [],
    faviconUrl: null,
    createdAt: 1728740000000,
    updatedAt: 1728740000000,
  },
  // Sub-folders on Toolbar
  {
    id: 'folder-anime',
    parentId: SYSTEM_ROOT_TOOLBAR,
    title: 'Anime & Manga',
    url: null,
    isFolder: true,
    position: 0,
    tags: ['anime'],
    faviconUrl: null,
    createdAt: 1728740100000,
    updatedAt: 1728740100000,
  },
  {
    id: 'folder-gaming',
    parentId: SYSTEM_ROOT_TOOLBAR,
    title: 'Gaming & Streams',
    url: null,
    isFolder: true,
    position: 1,
    tags: ['gaming'],
    faviconUrl: null,
    createdAt: 1728740110000,
    updatedAt: 1728740110000,
  },
  // Items inside Anime folder
  {
    id: 'bm-anilist',
    parentId: 'folder-anime',
    title: 'AniList',
    url: 'https://anilist.co',
    isFolder: false,
    position: 0,
    tags: ['anime', 'tracker'],
    faviconUrl: 'https://anilist.co/favicon.ico',
    createdAt: 1728740200000,
    updatedAt: 1728740200000,
  },
  {
    id: 'bm-crunchyroll',
    parentId: 'folder-anime',
    title: 'Crunchyroll',
    url: 'https://www.crunchyroll.com',
    isFolder: false,
    position: 1,
    tags: ['anime', 'streaming'],
    faviconUrl: 'https://www.crunchyroll.com/favicon.ico',
    createdAt: 1728740210000,
    updatedAt: 1728740210000,
  },
  {
    id: 'bm-mangadex',
    parentId: 'folder-anime',
    title: 'MangaDex',
    url: 'https://mangadex.org',
    isFolder: false,
    position: 2,
    tags: ['manga'],
    faviconUrl: 'https://mangadex.org/favicon.ico',
    createdAt: 1728740220000,
    updatedAt: 1728740220000,
  },
  // Items inside Gaming folder
  {
    id: 'bm-steam',
    parentId: 'folder-gaming',
    title: 'Steam Store',
    url: 'https://store.steampowered.com',
    isFolder: false,
    position: 0,
    tags: ['gaming', 'store'],
    faviconUrl: 'https://store.steampowered.com/favicon.ico',
    createdAt: 1728740300000,
    updatedAt: 1728740300000,
  },
  {
    id: 'bm-twitch',
    parentId: 'folder-gaming',
    title: 'Twitch',
    url: 'https://www.twitch.tv',
    isFolder: false,
    position: 1,
    tags: ['gaming', 'streams'],
    faviconUrl: 'https://static.twitchcdn.net/assets/favicon-32-e29e246c157142c94346.png',
    createdAt: 1728740310000,
    updatedAt: 1728740310000,
  },
  // Direct Items on Toolbar
  {
    id: 'bm-hub',
    parentId: SYSTEM_ROOT_TOOLBAR,
    title: 'Shinoho Hub',
    url: 'shinoho://hub',
    isFolder: false,
    position: 2,
    tags: ['shinoho', 'start'],
    faviconUrl: null,
    createdAt: 1728740400000,
    updatedAt: 1728740400000,
  },
  {
    id: 'bm-github',
    parentId: SYSTEM_ROOT_TOOLBAR,
    title: 'GitHub - Shinoho',
    url: 'https://github.com/Pikaswelt/shinoho',
    isFolder: false,
    position: 3,
    tags: ['dev', 'github'],
    faviconUrl: 'https://github.githubassets.com/favicons/favicon.png',
    createdAt: 1728740410000,
    updatedAt: 1728740410000,
  },
];

function loadFromStorage(): Bookmark[] {
  if (typeof localStorage === 'undefined') {
    return DEFAULT_BOOKMARKS;
  }
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return DEFAULT_BOOKMARKS;
    const parsed = JSON.parse(raw);
    return Array.isArray(parsed) && parsed.length > 0 ? parsed : DEFAULT_BOOKMARKS;
  } catch {
    return DEFAULT_BOOKMARKS;
  }
}

function loadBarVisibility(): boolean {
  if (typeof localStorage === 'undefined') return true;
  const raw = localStorage.getItem(BAR_VISIBLE_KEY);
  return raw !== null ? raw === 'true' : true;
}

export class BookmarkStore {
  private state: BookmarkStoreState;
  private listeners = new Set<(state: BookmarkStoreState) => void>();

  constructor() {
    this.state = {
      bookmarks: loadFromStorage(),
      isBarVisible: loadBarVisibility(),
      isManagerOpen: false,
      activeFolderId: SYSTEM_ROOT_TOOLBAR,
      searchQuery: '',
      activeTag: null,
    };
  }

  private saveToStorage(): void {
    if (typeof localStorage !== 'undefined') {
      try {
        localStorage.setItem(STORAGE_KEY, JSON.stringify(this.state.bookmarks));
        localStorage.setItem(BAR_VISIBLE_KEY, String(this.state.isBarVisible));
      } catch {
        // Ignore quota/incognito errors
      }
    }
  }

  private notify(): void {
    const current = this.getState();
    this.listeners.forEach((fn) => fn(current));
  }

  public getState(): BookmarkStoreState {
    return {
      ...this.state,
      bookmarks: [...this.state.bookmarks],
    };
  }

  public subscribe(callback: (state: BookmarkStoreState) => void): () => void {
    this.listeners.add(callback);
    callback(this.getState());
    return () => {
      this.listeners.delete(callback);
    };
  }

  // Getters for current state values
  public get bookmarks(): Bookmark[] {
    return this.state.bookmarks;
  }

  public get isBarVisible(): boolean {
    return this.state.isBarVisible;
  }

  public get isManagerOpen(): boolean {
    return this.state.isManagerOpen;
  }

  public get activeFolderId(): string {
    return this.state.activeFolderId;
  }

  public set activeFolderId(val: string) {
    this.state.activeFolderId = val;
    this.notify();
  }

  public get searchQuery(): string {
    return this.state.searchQuery;
  }

  public set searchQuery(val: string) {
    this.state.searchQuery = val;
    this.notify();
  }

  public get activeTag(): string | null {
    return this.state.activeTag;
  }

  public set activeTag(val: string | null) {
    this.state.activeTag = val;
    this.notify();
  }

  public getById(id: string): Bookmark | undefined {
    return this.state.bookmarks.find((b) => b.id === id);
  }

  public createBookmark(input: CreateBookmarkInput): Bookmark {
    const id = input.id || `bm-${Date.now()}-${Math.random().toString(36).substring(2, 7)}`;
    const parentId = input.parentId !== undefined ? input.parentId : SYSTEM_ROOT_TOOLBAR;
    const now = Date.now();

    const siblings = this.state.bookmarks.filter((b) => b.parentId === parentId);
    const maxPos = siblings.reduce((max, b) => Math.max(max, b.position), -1);
    const position = input.position !== undefined ? input.position : maxPos + 1;

    const newBookmark: Bookmark = {
      id,
      parentId,
      title: input.title.trim() || (input.url ? input.url : 'Neues Lesezeichen'),
      url: input.isFolder ? null : input.url?.trim() || null,
      isFolder: Boolean(input.isFolder),
      position,
      tags: input.tags || [],
      faviconUrl: input.faviconUrl || null,
      createdAt: now,
      updatedAt: now,
    };

    this.state.bookmarks = [...this.state.bookmarks, newBookmark];
    this.saveToStorage();
    this.notify();
    return newBookmark;
  }

  public updateBookmark(id: string, input: UpdateBookmarkInput): Bookmark | null {
    const current = this.getById(id);
    if (!current) return null;

    const updated: Bookmark = {
      ...current,
      title: input.title !== undefined ? input.title.trim() : current.title,
      url:
        input.url !== undefined
          ? current.isFolder
            ? null
            : input.url
              ? input.url.trim()
              : null
          : current.url,
      parentId: input.parentId !== undefined ? input.parentId : current.parentId,
      position: input.position !== undefined ? input.position : current.position,
      tags: input.tags !== undefined ? input.tags : current.tags,
      faviconUrl: input.faviconUrl !== undefined ? input.faviconUrl : current.faviconUrl,
      updatedAt: Date.now(),
    };

    this.state.bookmarks = this.state.bookmarks.map((b) => (b.id === id ? updated : b));
    this.saveToStorage();
    this.notify();
    return updated;
  }

  public deleteBookmark(id: string): boolean {
    if (id === SYSTEM_ROOT_TOOLBAR || id === SYSTEM_ROOT_OTHER || id === SYSTEM_ROOT_MOBILE) {
      return false;
    }

    const idsToDelete = new Set<string>([id]);
    const collectDescendants = (folderId: string) => {
      const children = this.state.bookmarks.filter((b) => b.parentId === folderId);
      for (const child of children) {
        idsToDelete.add(child.id);
        if (child.isFolder) {
          collectDescendants(child.id);
        }
      }
    };
    collectDescendants(id);

    this.state.bookmarks = this.state.bookmarks.filter((b) => !idsToDelete.has(b.id));
    this.saveToStorage();
    this.notify();
    return true;
  }

  public moveBookmark(id: string, targetParentId: string | null, newPosition?: number) {
    const item = this.getById(id);
    if (!item) return;

    if (item.isFolder && targetParentId) {
      let curr = this.getById(targetParentId);
      while (curr) {
        if (curr.id === id) return;
        curr = curr.parentId ? this.getById(curr.parentId) : undefined;
      }
    }

    this.updateBookmark(id, {
      parentId: targetParentId,
      position: newPosition,
    });
  }

  public getToolbarItems(): Bookmark[] {
    return this.state.bookmarks
      .filter((b) => b.parentId === SYSTEM_ROOT_TOOLBAR)
      .sort((a, b) => a.position - b.position);
  }

  public getItemsInFolder(folderId: string | null): Bookmark[] {
    return this.state.bookmarks
      .filter((b) => b.parentId === folderId)
      .sort((a, b) => {
        if (a.isFolder !== b.isFolder) {
          return a.isFolder ? -1 : 1;
        }
        return a.position - b.position;
      });
  }

  public getTree(rootId: string | null = null): BookmarkNode[] {
    const buildTree = (parentId: string | null): BookmarkNode[] => {
      return this.state.bookmarks
        .filter((b) => b.parentId === parentId)
        .sort((a, b) => a.position - b.position)
        .map((item) => ({
          item,
          children: item.isFolder ? buildTree(item.id) : [],
        }));
    };
    return buildTree(rootId);
  }

  public search(query: string, tag?: string | null): Bookmark[] {
    const q = query.trim().toLowerCase();
    return this.state.bookmarks.filter((b) => {
      if (
        b.id === SYSTEM_ROOT_TOOLBAR ||
        b.id === SYSTEM_ROOT_OTHER ||
        b.id === SYSTEM_ROOT_MOBILE
      ) {
        return false;
      }
      if (tag && !b.tags.includes(tag)) {
        return false;
      }
      if (!q) return true;

      const titleMatch = b.title.toLowerCase().includes(q);
      const urlMatch = b.url ? b.url.toLowerCase().includes(q) : false;
      const tagMatch = b.tags.some((t) => t.toLowerCase().includes(q));

      return titleMatch || urlMatch || tagMatch;
    });
  }

  public getAllTags(): { tag: string; count: number }[] {
    const counts = new Map<string, number>();
    for (const b of this.state.bookmarks) {
      for (const t of b.tags) {
        counts.set(t, (counts.get(t) || 0) + 1);
      }
    }
    return Array.from(counts.entries())
      .map(([tag, count]) => ({ tag, count }))
      .sort((a, b) => b.count - a.count);
  }

  public exportHtml(): string {
    const roots = this.getTree(null);
    return exportToNetscapeHtml(roots);
  }

  public importHtml(
    htmlContent: string,
    targetParentId = SYSTEM_ROOT_TOOLBAR,
  ): BookmarkImportSummary {
    const parsedRoots = parseNetscapeHtml(htmlContent);
    let foldersCreated = 0;
    let bookmarksCreated = 0;

    const insertRecursive = (items: typeof parsedRoots, parentId: string) => {
      for (const item of items) {
        if (item.isFolder) {
          const folder = this.createBookmark({
            title: item.title,
            isFolder: true,
            parentId: item.isToolbar ? SYSTEM_ROOT_TOOLBAR : parentId,
            tags: item.tags,
          });
          foldersCreated++;
          if (item.children && item.children.length > 0) {
            insertRecursive(item.children, folder.id);
          }
        } else {
          this.createBookmark({
            title: item.title,
            url: item.url,
            isFolder: false,
            parentId,
            tags: item.tags,
            faviconUrl: item.faviconUrl,
          });
          bookmarksCreated++;
        }
      }
    };

    insertRecursive(parsedRoots, targetParentId);
    return { foldersCreated, bookmarksCreated };
  }

  public toggleBarVisibility() {
    this.state.isBarVisible = !this.state.isBarVisible;
    this.saveToStorage();
    this.notify();
  }

  public setBarVisibility(visible: boolean) {
    this.state.isBarVisible = visible;
    this.saveToStorage();
    this.notify();
  }

  public openManager(folderId = SYSTEM_ROOT_TOOLBAR) {
    this.state.activeFolderId = folderId;
    this.state.isManagerOpen = true;
    this.notify();
  }

  public closeManager() {
    this.state.isManagerOpen = false;
    this.notify();
  }

  public resetToDefaults() {
    this.state = {
      bookmarks: [...DEFAULT_BOOKMARKS],
      isBarVisible: true,
      isManagerOpen: false,
      activeFolderId: SYSTEM_ROOT_TOOLBAR,
      searchQuery: '',
      activeTag: null,
    };
    this.saveToStorage();
    this.notify();
  }
}

export function createBookmarkStore(): BookmarkStore {
  return new BookmarkStore();
}

export const bookmarkStore = createBookmarkStore();
