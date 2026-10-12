export interface Bookmark {
  id: string;
  parentId: string | null;
  title: string;
  url: string | null; // null for folders
  isFolder: boolean;
  position: number;
  tags: string[];
  faviconUrl: string | null;
  createdAt: number;
  updatedAt: number;
}

export interface BookmarkNode {
  item: Bookmark;
  children: BookmarkNode[];
}

export interface CreateBookmarkInput {
  id?: string;
  parentId?: string | null;
  title: string;
  url?: string | null;
  isFolder?: boolean;
  position?: number;
  tags?: string[];
  faviconUrl?: string | null;
}

export interface UpdateBookmarkInput {
  title?: string;
  url?: string | null;
  parentId?: string | null;
  position?: number;
  tags?: string[];
  faviconUrl?: string | null;
}

export interface BookmarkImportSummary {
  foldersCreated: number;
  bookmarksCreated: number;
}
