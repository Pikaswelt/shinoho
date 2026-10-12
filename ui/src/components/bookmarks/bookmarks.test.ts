import { describe, it, expect, beforeEach } from 'vitest';
import {
  createBookmarkStore,
  SYSTEM_ROOT_TOOLBAR,
  SYSTEM_ROOT_OTHER,
  SYSTEM_ROOT_MOBILE,
} from './bookmarkStore';
import { exportToNetscapeHtml, parseNetscapeHtml } from './bookmarkHtml';
import type { BookmarkNode } from './bookmarks.types';

describe('bookmarkStore', () => {
  let store: ReturnType<typeof createBookmarkStore>;

  beforeEach(() => {
    if (typeof localStorage !== 'undefined') {
      localStorage.clear();
    }
    store = createBookmarkStore();
    store.resetToDefaults();
  });

  it('initializes with default system roots and sample bookmarks', () => {
    const items = store.bookmarks;
    expect(items.length).toBeGreaterThan(0);

    const toolbar = store.getById(SYSTEM_ROOT_TOOLBAR);
    const other = store.getById(SYSTEM_ROOT_OTHER);
    const mobile = store.getById(SYSTEM_ROOT_MOBILE);

    expect(toolbar).toBeDefined();
    expect(toolbar?.isFolder).toBe(true);
    expect(other).toBeDefined();
    expect(mobile).toBeDefined();
  });

  it('can create a new bookmark in toolbar', () => {
    const bm = store.createBookmark({
      title: 'Anime News Network',
      url: 'https://animenewsnetwork.com',
      parentId: SYSTEM_ROOT_TOOLBAR,
      tags: ['anime', 'news'],
    });

    expect(bm.id).toBeDefined();
    expect(bm.title).toBe('Anime News Network');
    expect(bm.url).toBe('https://animenewsnetwork.com');
    expect(bm.tags).toContain('anime');

    const toolbarItems = store.getToolbarItems();
    expect(toolbarItems.some((b) => b.id === bm.id)).toBe(true);
  });

  it('can create a folder and place bookmarks inside it', () => {
    const folder = store.createBookmark({
      title: 'Manga Reads',
      isFolder: true,
      parentId: SYSTEM_ROOT_TOOLBAR,
    });

    expect(folder.isFolder).toBe(true);

    const mangaBm = store.createBookmark({
      title: 'V-Jump',
      url: 'https://vjump.shueisha.co.jp',
      parentId: folder.id,
      tags: ['manga'],
    });

    const folderChildren = store.getItemsInFolder(folder.id);
    expect(folderChildren.length).toBe(1);
    expect(folderChildren[0].id).toBe(mangaBm.id);
  });

  it('can update an existing bookmark', () => {
    const bm = store.createBookmark({
      title: 'Old Title',
      url: 'https://old.url',
      parentId: SYSTEM_ROOT_TOOLBAR,
    });

    const updated = store.updateBookmark(bm.id, {
      title: 'New Title',
      url: 'https://new.url',
      tags: ['updated'],
    });

    expect(updated?.title).toBe('New Title');
    expect(updated?.url).toBe('https://new.url');
    expect(updated?.tags).toEqual(['updated']);
  });

  it('can delete a bookmark', () => {
    const bm = store.createBookmark({
      title: 'To Delete',
      url: 'https://delete.me',
      parentId: SYSTEM_ROOT_TOOLBAR,
    });

    const deleted = store.deleteBookmark(bm.id);
    expect(deleted).toBe(true);
    expect(store.getById(bm.id)).toBeUndefined();
  });

  it('protects system folders from deletion', () => {
    expect(store.deleteBookmark(SYSTEM_ROOT_TOOLBAR)).toBe(false);
    expect(store.deleteBookmark(SYSTEM_ROOT_OTHER)).toBe(false);
    expect(store.deleteBookmark(SYSTEM_ROOT_MOBILE)).toBe(false);
  });

  it('searches bookmarks by query and tags', () => {
    store.createBookmark({
      title: 'Genshin Impact Official',
      url: 'https://genshin.hoyoverse.com',
      parentId: SYSTEM_ROOT_TOOLBAR,
      tags: ['gaming', 'gacha'],
    });

    const resultsByTitle = store.search('genshin');
    expect(resultsByTitle.length).toBe(1);
    expect(resultsByTitle[0].title).toContain('Genshin');

    const resultsByTag = store.search('', 'gacha');
    expect(resultsByTag.length).toBe(1);
    expect(resultsByTag[0].tags).toContain('gacha');
  });

  it('builds a hierarchical tree', () => {
    const tree = store.getTree(null);
    expect(tree.length).toBeGreaterThanOrEqual(3); // toolbar, other, mobile roots
    const toolbarNode = tree.find((n) => n.item.id === SYSTEM_ROOT_TOOLBAR);
    expect(toolbarNode).toBeDefined();
    expect(toolbarNode?.children.length).toBeGreaterThan(0);
  });

  it('toggles bar visibility', () => {
    const initial = store.isBarVisible;
    store.toggleBarVisibility();
    expect(store.isBarVisible).toBe(!initial);
    store.setBarVisibility(true);
    expect(store.isBarVisible).toBe(true);
  });
});

describe('Netscape Bookmark HTML format', () => {
  it('exports and serializes nodes into valid Netscape Bookmark HTML', () => {
    const sampleTree: BookmarkNode[] = [
      {
        item: {
          id: 'toolbar',
          parentId: null,
          title: 'Bookmarks Bar',
          url: null,
          isFolder: true,
          position: 0,
          tags: [],
          faviconUrl: null,
          createdAt: 1728740000000,
          updatedAt: 1728740000000,
        },
        children: [
          {
            item: {
              id: 'item-1',
              parentId: 'toolbar',
              title: 'AniList',
              url: 'https://anilist.co',
              isFolder: false,
              position: 0,
              tags: ['anime', 'tracker'],
              faviconUrl: 'https://anilist.co/favicon.ico',
              createdAt: 1728740000000,
              updatedAt: 1728740000000,
            },
            children: [],
          },
        ],
      },
    ];

    const html = exportToNetscapeHtml(sampleTree);
    expect(html).toContain('<!DOCTYPE NETSCAPE-Bookmark-file-1>');
    expect(html).toContain('PERSONAL_TOOLBAR_FOLDER="true"');
    expect(html).toContain('HREF="https://anilist.co"');
    expect(html).toContain('TAGS="anime,tracker"');
    expect(html).toContain('AniList</A>');
  });

  it('parses Netscape HTML bookmark format correctly', () => {
    const rawHtml = `
      <!DOCTYPE NETSCAPE-Bookmark-file-1>
      <TITLE>Bookmarks</TITLE>
      <H1>Bookmarks</H1>
      <DL><p>
        <DT><H3 ADD_DATE="1728740000" PERSONAL_TOOLBAR_FOLDER="true">Lesezeichenleiste</H3>
        <DL><p>
          <DT><A HREF="https://store.steampowered.com" ADD_DATE="1728740000" TAGS="gaming,store">Steam</A>
          <DT><H3>Unterordner</H3>
          <DL><p>
            <DT><A HREF="https://twitch.tv">Twitch</A>
          </DL><p>
        </DL><p>
      </DL><p>
    `;

    const parsed = parseNetscapeHtml(rawHtml);
    expect(parsed.length).toBe(1);
    expect(parsed[0].isFolder).toBe(true);
    expect(parsed[0].isToolbar).toBe(true);
    expect(parsed[0].title).toBe('Lesezeichenleiste');

    const toolbarChildren = parsed[0].children;
    expect(toolbarChildren.length).toBe(2);

    const steamBm = toolbarChildren[0];
    expect(steamBm.title).toBe('Steam');
    expect(steamBm.url).toBe('https://store.steampowered.com');
    expect(steamBm.tags).toContain('gaming');

    const subfolder = toolbarChildren[1];
    expect(subfolder.isFolder).toBe(true);
    expect(subfolder.title).toBe('Unterordner');
    expect(subfolder.children[0].title).toBe('Twitch');
    expect(subfolder.children[0].url).toBe('https://twitch.tv');
  });

  it('round-trips export and import in store', () => {
    const store = createBookmarkStore();
    store.resetToDefaults();

    const exported = store.exportHtml();
    expect(exported).toContain('<!DOCTYPE NETSCAPE-Bookmark-file-1>');

    // Import into fresh store
    const newStore = createBookmarkStore();
    newStore.resetToDefaults();
    const countBefore = newStore.bookmarks.length;

    const summary = newStore.importHtml(exported, SYSTEM_ROOT_TOOLBAR);
    expect(summary.bookmarksCreated).toBeGreaterThan(0);
    expect(newStore.bookmarks.length).toBeGreaterThan(countBefore);
  });
});
