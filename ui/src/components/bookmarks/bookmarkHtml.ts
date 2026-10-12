import type { BookmarkNode } from './bookmarks.types';

export interface ParsedBookmarkItem {
  title: string;
  url?: string | null;
  isFolder: boolean;
  faviconUrl?: string | null;
  tags: string[];
  isToolbar?: boolean;
  children: ParsedBookmarkItem[];
}

function escapeHtml(str: string): string {
  return str
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

function unescapeHtml(str: string): string {
  return str
    .replace(/&quot;/g, '"')
    .replace(/&gt;/g, '>')
    .replace(/&lt;/g, '<')
    .replace(/&amp;/g, '&');
}

/**
 * Exports bookmark tree nodes into standard Netscape Bookmark File 1 format.
 * Compatible with Chrome, Firefox, Safari, Edge, and agent-d's Rust backend.
 */
export function exportToNetscapeHtml(nodes: BookmarkNode[]): string {
  let out = '<!DOCTYPE NETSCAPE-Bookmark-file-1>\n';
  out += '<!-- This is an automatically generated file.\n';
  out += '     It will be read and overwritten.\n';
  out += '     DO NOT EDIT! -->\n';
  out += '<META HTTP-EQUIV="Content-Type" CONTENT="text/html; charset=UTF-8">\n';
  out += '<TITLE>Bookmarks</TITLE>\n';
  out += '<H1>Bookmarks</H1>\n';
  out += '<DL><p>\n';

  for (const node of nodes) {
    writeNode(node, 1);
  }

  function writeNode(node: BookmarkNode, indentLevel: number) {
    const indent = '    '.repeat(indentLevel);
    const b = node.item;

    if (b.isFolder) {
      const addDate = Math.floor(b.createdAt / 1000);
      const lastModified = Math.floor(b.updatedAt / 1000);
      const toolbarAttr = b.id === 'toolbar' ? ' PERSONAL_TOOLBAR_FOLDER="true"' : '';

      out += `${indent}<DT><H3 ADD_DATE="${addDate}" LAST_MODIFIED="${lastModified}"${toolbarAttr}>${escapeHtml(b.title)}</H3>\n`;
      out += `${indent}<DL><p>\n`;
      for (const child of node.children) {
        writeNode(child, indentLevel + 1);
      }
      out += `${indent}</DL><p>\n`;
    } else {
      const addDate = Math.floor(b.createdAt / 1000);
      let attrs = `HREF="${escapeHtml(b.url || '')}" ADD_DATE="${addDate}"`;

      if (b.faviconUrl) {
        attrs += ` ICON="${escapeHtml(b.faviconUrl)}"`;
      }
      if (b.tags && b.tags.length > 0) {
        attrs += ` TAGS="${escapeHtml(b.tags.join(','))}"`;
      }

      out += `${indent}<DT><A ${attrs}>${escapeHtml(b.title)}</A>\n`;
    }
  }

  out += '</DL><p>\n';
  return out;
}

/**
 * Parses Netscape Bookmark HTML into a hierarchical structure.
 */
export function parseNetscapeHtml(html: string): ParsedBookmarkItem[] {
  const rootItems: ParsedBookmarkItem[] = [];
  const folderStack: { children: ParsedBookmarkItem[] }[] = [{ children: rootItems }];

  // Match tags of interest: <DT><H3...>, </DL>, <DT><A...>
  // Also tolerate variations without <DT> or lowercase
  const tagRegex = /(?:<DT>)?\s*(?:<H3([^>]*)>(.*?)<\/H3>|<A([^>]*)>(.*?)<\/A>|<\/DL>)/gi;
  let match: RegExpExecArray | null;

  while ((match = tagRegex.exec(html)) !== null) {
    const [fullMatch, h3Attrs, h3Title, aAttrs, aTitle] = match;

    if (fullMatch.toLowerCase().includes('</dl>')) {
      if (folderStack.length > 1) {
        folderStack.pop();
      }
      continue;
    }

    if (h3Attrs !== undefined && h3Title !== undefined) {
      // It's a folder
      const isToolbar = /PERSONAL_TOOLBAR_FOLDER\s*=\s*["']true["']/i.test(h3Attrs);
      const folderItem: ParsedBookmarkItem = {
        title: unescapeHtml(h3Title.trim()),
        isFolder: true,
        tags: [],
        isToolbar,
        children: [],
      };

      const currentParent = folderStack[folderStack.length - 1];
      currentParent.children.push(folderItem);
      // Push this folder onto stack so subsequent children go into it
      folderStack.push(folderItem);
    } else if (aAttrs !== undefined && aTitle !== undefined) {
      // It's a bookmark
      const hrefMatch = /HREF\s*=\s*["']([^"']*)["']/i.exec(aAttrs);
      const iconMatch = /ICON\s*=\s*["']([^"']*)["']/i.exec(aAttrs);
      const tagsMatch = /TAGS\s*=\s*["']([^"']*)["']/i.exec(aAttrs);

      const url = hrefMatch ? unescapeHtml(hrefMatch[1]) : '';
      const faviconUrl = iconMatch ? unescapeHtml(iconMatch[1]) : null;
      const tags = tagsMatch
        ? unescapeHtml(tagsMatch[1])
            .split(',')
            .map((t) => t.trim())
            .filter(Boolean)
        : [];

      const bookmarkItem: ParsedBookmarkItem = {
        title: unescapeHtml(aTitle.trim()) || url,
        url,
        isFolder: false,
        faviconUrl,
        tags,
        children: [],
      };

      const currentParent = folderStack[folderStack.length - 1];
      currentParent.children.push(bookmarkItem);
    }
  }

  return rootItems;
}
