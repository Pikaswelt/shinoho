import { describe, it, expect } from 'vitest';
import { flattenRawTree, toUiBookmark, type RawBookmark } from './bookmarkBackend';

const raw = (overrides: Partial<RawBookmark> = {}): RawBookmark => ({
  id: 'bm-1',
  parent_id: 'toolbar',
  title: 'Example',
  url: 'https://example.com',
  is_folder: false,
  position: 2,
  tags: ['dev'],
  favicon_url: null,
  created_at: 10,
  updated_at: 20,
  ...overrides,
});

describe('bookmarkBackend mapping', () => {
  it('maps snake_case fields to the UI shape', () => {
    expect(toUiBookmark(raw())).toEqual({
      id: 'bm-1',
      parentId: 'toolbar',
      title: 'Example',
      url: 'https://example.com',
      isFolder: false,
      position: 2,
      tags: ['dev'],
      faviconUrl: null,
      createdAt: 10,
      updatedAt: 20,
    });
  });

  it('flattens a nested tree depth-first', () => {
    const tree = [
      {
        item: raw({ id: 'folder', is_folder: true, url: null, parent_id: null }),
        children: [{ item: raw({ id: 'child' }), children: [] }],
      },
      { item: raw({ id: 'sibling' }), children: [] },
    ];
    expect(flattenRawTree(tree).map((b) => b.id)).toEqual(['folder', 'child', 'sibling']);
  });
});
