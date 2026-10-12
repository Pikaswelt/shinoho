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
