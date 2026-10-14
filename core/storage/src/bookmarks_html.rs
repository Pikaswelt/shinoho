use crate::bookmarks::{BookmarkNode, CreateBookmarkParams};
use crate::error::StorageError;
use crate::Storage;

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
pub struct ImportSummary {
    pub folders_created: usize,
    pub bookmarks_created: usize,
}

#[derive(Debug, Clone)]
pub struct ImportedItem {
    pub title: String,
    pub url: Option<String>,
    pub is_folder: bool,
    pub favicon_url: Option<String>,
    pub tags: Vec<String>,
    pub children: Vec<ImportedItem>,
}

pub struct BookmarkHtml;

impl BookmarkHtml {
    /// Export bookmark nodes into Netscape Bookmark File 1 format.
    pub fn export_tree(roots: &[BookmarkNode]) -> String {
        let mut out = String::new();
        out.push_str("<!DOCTYPE NETSCAPE-Bookmark-file-1>\n");
        out.push_str("<!-- This is an automatically generated file.\n");
        out.push_str("     It will be read and overwritten.\n");
        out.push_str("     DO NOT EDIT! -->\n");
        out.push_str("<META HTTP-EQUIV=\"Content-Type\" CONTENT=\"text/html; charset=UTF-8\">\n");
        out.push_str("<TITLE>Bookmarks</TITLE>\n");
        out.push_str("<H1>Bookmarks</H1>\n");
        out.push_str("<DL><p>\n");

        for node in roots {
            Self::write_node(&mut out, node, 1);
        }

        out.push_str("</DL><p>\n");
        out
    }

    fn write_node(out: &mut String, node: &BookmarkNode, indent_level: usize) {
        let indent = "    ".repeat(indent_level);
        let b = &node.item;

        if b.is_folder {
            let add_date = b.created_at / 1000;
            let last_modified = b.updated_at / 1000;
            let toolbar_attr = if b.id == "toolbar" {
                " PERSONAL_TOOLBAR_FOLDER=\"true\""
            } else {
                ""
            };

            out.push_str(&format!(
                "{}<DT><H3 ADD_DATE=\"{}\" LAST_MODIFIED=\"{}\"{}>{}</H3>\n",
                indent,
                add_date,
                last_modified,
                toolbar_attr,
                Self::escape_html(&b.title)
            ));
            out.push_str(&format!("{}<DL><p>\n", indent));

            for child in &node.children {
                Self::write_node(out, child, indent_level + 1);
            }

            out.push_str(&format!("{}</DL><p>\n", indent));
        } else if let Some(ref url) = b.url {
            let add_date = b.created_at / 1000;
            let icon_attr = match &b.favicon_url {
                Some(icon) => format!(" ICON=\"{}\"", Self::escape_html(icon)),
                None => String::new(),
            };
            let tags_attr = if b.tags.is_empty() {
                String::new()
            } else {
                format!(" TAGS=\"{}\"", Self::escape_html(&b.tags.join(",")))
            };

            out.push_str(&format!(
                "{}<DT><A HREF=\"{}\" ADD_DATE=\"{}\"{}{}>{}</A>\n",
                indent,
                Self::escape_html(url),
                add_date,
                icon_attr,
                tags_attr,
                Self::escape_html(&b.title)
            ));
        }
    }

    /// Parse Netscape Bookmark HTML into a hierarchy of `ImportedItem`s.
    pub fn parse(html: &str) -> Vec<ImportedItem> {
        let mut root_items: Vec<ImportedItem> = Vec::new();
        let mut stack: Vec<Vec<ImportedItem>> = Vec::new();
        stack.push(Vec::new()); // root container

        let mut pos = 0;
        let bytes = html.as_bytes();
        let len = bytes.len();

        while pos < len {
            // Find next tag start '<'
            if let Some(tag_start_rel) = html[pos..].find('<') {
                let tag_start = pos + tag_start_rel;
                if let Some(tag_end_rel) = html[tag_start..].find('>') {
                    let tag_end = tag_start + tag_end_rel;
                    let tag_content = &html[tag_start + 1..tag_end];
                    let tag_name = tag_content
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .to_ascii_uppercase();

                    if tag_name == "DL" {
                        // Enter a new child list
                        stack.push(Vec::new());
                        pos = tag_end + 1;
                        continue;
                    } else if tag_name == "/DL" {
                        // Close current child list
                        if stack.len() > 1 {
                            let children = stack.pop().unwrap();
                            // Attach children to the last folder in the parent container
                            if let Some(parent_container) = stack.last_mut() {
                                if let Some(last_item) = parent_container.last_mut() {
                                    if last_item.is_folder {
                                        last_item.children = children;
                                    } else {
                                        // If last item was somehow not a folder, promote children
                                        parent_container.extend(children);
                                    }
                                } else {
                                    parent_container.extend(children);
                                }
                            }
                        }
                        pos = tag_end + 1;
                        continue;
                    } else if tag_name.starts_with("H3") {
                        // Folder title: <H3 ...>Title</H3>
                        let title_end = html[tag_end + 1..]
                            .find("</")
                            .map(|idx| tag_end + 1 + idx)
                            .unwrap_or(len);
                        let raw_title = &html[tag_end + 1..title_end];
                        let title = Self::unescape_html(raw_title.trim());

                        let folder = ImportedItem {
                            title,
                            url: None,
                            is_folder: true,
                            favicon_url: None,
                            tags: Vec::new(),
                            children: Vec::new(),
                        };

                        if let Some(current_container) = stack.last_mut() {
                            current_container.push(folder);
                        }

                        pos = title_end;
                        continue;
                    } else if tag_name.starts_with('A') {
                        // Bookmark: <A HREF="..." ...>Title</A>
                        let href = Self::extract_attr(tag_content, "HREF").unwrap_or_default();
                        let icon = Self::extract_attr(tag_content, "ICON");
                        let tags_str = Self::extract_attr(tag_content, "TAGS");
                        let tags: Vec<String> = tags_str
                            .map(|s| {
                                s.split(',')
                                    .map(|t| t.trim().to_string())
                                    .filter(|t| !t.is_empty())
                                    .collect()
                            })
                            .unwrap_or_default();

                        let title_end = html[tag_end + 1..]
                            .find("</")
                            .map(|idx| tag_end + 1 + idx)
                            .unwrap_or(len);
                        let raw_title = &html[tag_end + 1..title_end];
                        let title = Self::unescape_html(raw_title.trim());

                        let item = ImportedItem {
                            title: if title.is_empty() {
                                href.clone()
                            } else {
                                title
                            },
                            url: Some(href),
                            is_folder: false,
                            favicon_url: icon,
                            tags,
                            children: Vec::new(),
                        };

                        if let Some(current_container) = stack.last_mut() {
                            current_container.push(item);
                        }

                        pos = title_end;
                        continue;
                    }

                    pos = tag_end + 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // Flush remaining stack
        while let Some(items) = stack.pop() {
            if stack.is_empty() {
                root_items.extend(items);
            } else if let Some(parent) = stack.last_mut() {
                parent.extend(items);
            }
        }

        root_items
    }

    /// Import parsed items into the database under `target_parent_id`.
    pub fn import_to_storage(
        storage: &Storage,
        items: &[ImportedItem],
        target_parent_id: Option<&str>,
    ) -> Result<ImportSummary, StorageError> {
        let mut summary = ImportSummary::default();
        let default_parent = target_parent_id.unwrap_or("toolbar");

        for item in items {
            Self::insert_recursive(storage, item, default_parent, &mut summary)?;
        }

        Ok(summary)
    }

    fn insert_recursive(
        storage: &Storage,
        item: &ImportedItem,
        parent_id: &str,
        summary: &mut ImportSummary,
    ) -> Result<(), StorageError> {
        if item.is_folder {
            let folder = storage.create_bookmark(CreateBookmarkParams {
                title: item.title.clone(),
                url: None,
                parent_id: Some(parent_id.to_string()),
                is_folder: true,
                ..Default::default()
            })?;
            summary.folders_created += 1;

            for child in &item.children {
                Self::insert_recursive(storage, child, &folder.id, summary)?;
            }
        } else {
            storage.create_bookmark(CreateBookmarkParams {
                title: item.title.clone(),
                url: item.url.clone(),
                parent_id: Some(parent_id.to_string()),
                is_folder: false,
                tags: item.tags.clone(),
                favicon_url: item.favicon_url.clone(),
                ..Default::default()
            })?;
            summary.bookmarks_created += 1;
        }

        Ok(())
    }

    fn extract_attr(tag: &str, attr_name: &str) -> Option<String> {
        let pattern = format!("{}=\"", attr_name);
        if let Some(idx) = tag.find(&pattern) {
            let val_start = idx + pattern.len();
            if let Some(quote_end) = tag[val_start..].find('"') {
                return Some(Self::unescape_html(&tag[val_start..val_start + quote_end]));
            }
        }

        // Try case-insensitive attribute match
        let tag_upper = tag.to_ascii_uppercase();
        let pattern_upper = format!("{}=\"", attr_name.to_ascii_uppercase());
        if let Some(idx) = tag_upper.find(&pattern_upper) {
            let val_start = idx + pattern_upper.len();
            if let Some(quote_end) = tag[val_start..].find('"') {
                return Some(Self::unescape_html(&tag[val_start..val_start + quote_end]));
            }
        }

        None
    }

    fn escape_html(input: &str) -> String {
        input
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
    }

    fn unescape_html(input: &str) -> String {
        input
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#39;", "'")
            .replace("&apos;", "'")
    }
}
