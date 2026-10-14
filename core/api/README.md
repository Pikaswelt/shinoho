# Schnittstelle `core/api/` (v0)

Dieses Verzeichnis definiert die gemeinsame Schnittstelle zwischen dem **Rust-Kern** (`core/` / `apps/desktop`)
und der **TypeScript-UI** (`ui/`).

## 1. Tauri Commands

| Command | Argumente | Rückgabe | Beschreibung |
|---|---|---|---|
| `create_tab` | `request: CreateTabRequest` | `TabInfo` | Erzeugt einen neuen Tab (optional mit Start-URL). |
| `close_tab` | `request: TabActionRequest` | `void` | Schließt den angegebenen Tab. |
| `activate_tab` | `request: TabActionRequest` | `void` | Setzt den Tab als aktiv/fokussiert. |
| `navigate_tab` | `request: NavigateTabRequest` | `void` | Navigiert den Tab zu einer neuen URL. |
| `go_back` | `request: TabActionRequest` | `void` | Navigiert in der Tab-Historie zurück. |
| `go_forward` | `request: TabActionRequest` | `void` | Navigiert in der Tab-Historie vorwärts. |
| `reload_tab` | `request: ReloadTabRequest` | `void` | Lädt den Tab neu (optional Cache ignorieren). |
| `get_tabs` | keine | `Vec<TabInfo>` | Liefert die Liste aller aktuell offenen Tabs. |

## 2. Tauri Events

| Event-Name | Payload | Wann gefeuert |
|---|---|---|
| `shinoho://tab-created` | `{ tab: TabInfo }` | Sobald ein neuer Tab angelegt wurde. |
| `shinoho://tab-updated` | `{ tab: TabInfo }` | Wenn sich URL, Titel, Ladestatus oder Favicon ändert. |
| `shinoho://tab-closed` | `{ tabId: string }` | Wenn ein Tab geschlossen wurde. |
| `shinoho://tab-activated` | `{ tabId: string }` | Wenn ein anderer Tab aktiv wurde. |

## 3. Typ-Mapping Rust <-> TypeScript

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabInfo {
    pub id: String,
    pub title: String,
    pub url: String,
    pub is_loading: bool,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub is_pinned: bool,
    pub is_muted: bool,
    pub favicon: Option<String>,
    pub created_at: i64,
}
```

## 4. Persistente Dienste (T-111, abwärtskompatibel)

Diese Commands sind in `apps/desktop/src/commands.rs` implementiert. Sie greifen auf den
SQLite-Speicher (`shinoho-storage`), Inkognito (`shinoho-incognito`), Adblock
(`shinoho-adblock`) und Downloads (`shinoho-downloads`) zu, die im `setup` als Tauri-State
angelegt werden (`apps/desktop/src/services.rs`). Die v0-Commands aus Abschnitt 1 bleiben unverändert.

| Bereich | Commands |
|---|---|
| Historie | `record_visit`, `get_history`, `search_history`, `get_top_sites`, `delete_history_visit`, `clear_history`, `clear_history_range` |
| Lesezeichen | `create_bookmark`, `get_bookmark`, `update_bookmark`, `delete_bookmark`, `list_bookmarks`, `get_bookmark_tree`, `search_bookmarks`, `export_bookmarks_html`, `import_bookmarks_html` |
| Einstellungen | `get_setting`, `set_setting`, `get_all_settings`, `delete_setting` |
| Inkognito | `create_incognito_session`, `close_incognito_session`, `close_all_incognito_sessions`, `list_incognito_sessions`, `register_incognito_tab`, `unregister_incognito_tab` |
| Adblock | `get_adblock_stats`, `set_adblock_enabled`, `whitelist_domain`, `check_network_request`, `get_hidden_elements_css`, `add_adblock_filter_list` |
| Downloads | `start_download`, `pause_download`, `resume_download`, `cancel_download`, `retry_download`, `get_download`, `list_downloads`, `delete_download`, `clear_completed_downloads` |

Eine vollständige Argumentliste steht bei der jeweiligen Funktion in `commands.rs`. Fehler werden als
`String` zurückgegeben.

### Download-Events

Der `DownloadManager` sendet seine Events über einen Kanal. `services.rs` leitet jeden Event als
Tauri-Event an alle Fenster weiter. Der Payload ist der getaggte Rust-Enum `DownloadEvent`
(`#[serde(tag = "type", content = "payload")]`), das Feld `type` kennzeichnet die Variante.

| Event-Name | Variante (`DownloadEvent`) | Wann gefeuert |
|---|---|---|
| `shinoho://download-started` | `Started(DownloadItem)` | Download wurde gestartet. |
| `shinoho://download-progress` | `Progress(DownloadProgressEvent)` | Fortschritt hat sich geändert. |
| `shinoho://download-paused` | `Paused { id }` | Download pausiert. |
| `shinoho://download-resumed` | `Resumed { id }` | Download fortgesetzt. |
| `shinoho://download-completed` | `Completed(DownloadItem)` | Download abgeschlossen. |
| `shinoho://download-cancelled` | `Cancelled { id }` | Download abgebrochen. |
| `shinoho://download-failed` | `Failed { id, error }` | Download fehlgeschlagen. |
