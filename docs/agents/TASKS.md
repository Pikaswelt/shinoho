# Aufgaben-Board

Regeln zum Beanspruchen: [`AGENTS.md`](../../AGENTS.md) §2.
Status: `offen` · `in Arbeit (agent-x)` · `blockiert` · `review` · `fertig`

> ✅ Q1–Q8 sind beantwortet (Tauri 2, MPL-2.0, Windows + Linux zuerst …) – siehe
> [`OPEN-QUESTIONS.md`](../OPEN-QUESTIONS.md). Es kann losgehen!

## ⚠️ Aktive Verteilung (5 Agents) – vor dem Claimen lesen!

Alle Aufgaben aus Phase 0 und Phase 1 sind bereits **fest vergeben**, auch wenn sie unten
noch `offen` stehen (jeder Agent claimt nur eine Aufgabe gleichzeitig):

| Rolle | Reserviert (in dieser Reihenfolge) |
|---|---|
| agent-a – Kern | T-002, T-003, T-006 (Kern-Teil), T-101 |
| agent-b – UI | T-004, T-007, T-006 (UI-Teil), T-102 |
| agent-c – Infra/Repo | T-001, Rollen-PR für AGENTS.md, T-005, T-501, T-503, T-701, T-799 (Release) |
| agent-d – Kern-2 | T-103, T-104, T-106, T-108, T-109 (Backend) |
| agent-e – UI-2 | T-110, T-105, T-107, T-109 (UI) |
| agent-f – VPN & Performance | Phase 2 (Kern-Teil) und Phase 4 (Kern-Teil), siehe unten |

**Neuer Agent / Gast-Agent?** Reservierte Aufgaben nie übernehmen. Du darfst sofort
mitarbeiten an allen Aufgaben mit Revier **`frei`** – Ablauf in [`AGENTS.md`](../../AGENTS.md) §9.

## Phase 0 – Fundament

| ID | Aufgabe | Revier | Abhängig von | Status |
|---|---|---|---|---|
| T-001 | Lizenzdatei (MPL-2.0) + `.gitignore` + `.editorconfig` anlegen | C | – | fertig |
| T-002 | Tauri-2-Projekt anlegen (`apps/desktop`), leeres Fenster | A | – | fertig |
| T-003 | Rust-Workspace `core/` + `cargo fmt/clippy/test` einrichten | A | T-002 | fertig |
| T-004 | UI-Grundgerüst `ui/` (Svelte + TS, Lint, Prettier, Vitest) | B | – | fertig |
| T-005 | CI: Builds Windows + Linux (GitHub Actions); macOS unsigniert optional | C | T-002, T-004 | fertig |
| T-006 | Schnittstelle `core/api/` v0: Tab öffnen/schließen/navigieren, Events | gemeinsam | T-003, T-004 | fertig |
| T-007 | i18n-Grundgerüst (de, en, ja) | B | T-004 | fertig |

## Phase 1 – MVP-Browser

| ID | Aufgabe | Revier | Abhängig von | Status |
|---|---|---|---|---|
| T-101 | Tab-Manager im Kern (mehrere WebViews, Lebenszyklus) | A | T-006 | in Arbeit (agent-a) |
| T-102 | Tab-Leiste + Adressleiste UI | B | T-006 | in Arbeit (agent-b) |
| T-103 | SQLite-Storage (Verlauf, Lesezeichen, Einstellungen) | D | T-003 | fertig |
| T-104 | Lesezeichen: Backend (CRUD, Ordner, Import/Export HTML) | D | T-103 | fertig |
| T-105 | Lesezeichen: UI (Leiste, Manager, Suche) | E | T-104 | fertig |
| T-106 | Inkognito-Modus: Backend (flüchtiges Profil, kein Disk-Cache) | D | T-101 | fertig |
| T-107 | Inkognito-Modus: UI (eigenes Fenster, eigener Look) | E | T-106 | offen |
| T-108 | Adblock mit `adblock-rust` | D | T-101 | fertig |
| T-109 | Downloads-Manager (Backend + UI) | D + E | T-101 | in Arbeit (agent-d: Backend) |
| T-110 | Startseite „Shinoho Hub“ (Speed-Dial, Wallpaper) | E | T-102 | fertig |

## Phase 2 – Shinoho Control (Performance)

| ID | Aufgabe | Revier | Abhängig von | Status |
|---|---|---|---|---|
| T-201 | Limiter-Logik als Bibliothek `core/control`: Presets (Akku/Ausgewogen/Max/Gaming), Tab-Schlaf-Entscheidung, RAM-/CPU-Budget | F | – | review (agent-f) |
| T-202 | Messung RAM/CPU pro Tab bzw. WebView-Prozess | F | T-101 | offen |
| T-203 | Tab-Schlaf an Tab-Manager anbinden (einfrieren/aufwecken) | F | T-101, T-201 | offen |
| T-204 | Netzwerk-Limiter (Bandbreite pro Tab/gesamt) | F | T-101 | offen |
| T-205 | Gaming-Modus: laufende Spiele / Vollbild-Apps erkennen | F | T-201 | offen |
| T-206 | UI „Shinoho Control“ (Regler, Presets, Hot-Tabs-Killer) | UI (Rolle offen) | T-202, T-006 | offen |

## Phase 4 – VPN

| ID | Aufgabe | Revier | Abhängig von | Status |
|---|---|---|---|---|
| T-401 | `core/vpn`: WireGuard-`.conf`-Parser + Anbieterliste `providers.json` | F | – | fertig |
| T-402 | WireGuard-Tunnel (boringtun) + lokaler SOCKS5-Proxy | F | T-401 | offen |
| T-403 | Proxy an WebViews anbinden + Kill-Switch | F | T-101, T-402 | offen |
| T-404 | DNS über Tunnel, WebRTC-Leak-Schutz, Leak-Test-Seite `shinoho://vpn-test` | F | T-403 | offen |
| T-405 | Anbieter-Konditionen prüfen, `providers.json` + `docs/VPN.md` aktualisieren | F | T-401 | offen |
| T-406 | VPN-UI (Badge, Anbieter-Assistent, Config-Import) | UI (Rolle offen) | T-403, T-006 | offen |

## Phase 3 – Mods v1

| ID | Aufgabe | Revier | Abhängig von | Status |
|---|---|---|---|---|
| T-301 | Theme-Engine: Themes als CSS-Variablen-Pakete, Umschalten zur Laufzeit | frei | T-004 | offen |
| T-302 | Wallpaper- und Soundpacks (Startseite, UI-Sounds) | frei | T-301, T-110 | offen |
| T-303 | Seiten-CSS (Userstyles pro Domain) im Kern injizieren | frei | T-101 | offen |
| T-304 | Mod-Loader + Sandbox (Manifest, Rechte, Laden/Entladen) | frei | T-006 | offen |
| T-305 | SDK `sdk/` + Beispiel-Mod + `docs/MODDING.md` aktualisieren | frei | T-304 | offen |

## Phase 5 – Mobile (Android)

| ID | Aufgabe | Revier | Abhängig von | Status |
|---|---|---|---|---|
<<<<<<< HEAD
| T-501 | Tauri-Android-Ziel einrichten, APK lokal baubar (Debug-signiert) | C | T-002 | offen |
=======
| T-501 | Tauri-Android-Ziel einrichten, APK lokal baubar (Debug-signiert) | C | T-002 | in Arbeit (agent-c) |
>>>>>>> agent-b/T-006-core-api-v0
| T-502 | Touch-UI + Gesten (Tabs wischen, Adressleiste unten) | frei | T-102, T-501 | offen |
| T-503 | Android-APK-Build in der CI | C | T-005, T-501 | offen |

## Phase 6 – Anime & Gaming

| ID | Aufgabe | Revier | Abhängig von | Status |
|---|---|---|---|---|
| T-601 | Anime-Tracker (AniList-Login, Liste, Folgen-Fortschritt) | frei | T-006 | offen |
| T-602 | Release-Kalender (neue Folgen/Spiele der Woche) | frei | T-601 | offen |
| T-603 | Gaming-Modus-UI (Overlay, Performance-Preset) | frei | T-205 | offen |
| T-604 | Free-Games-Feed (Epic, Steam, GOG) auf dem Hub | frei | T-110 | in Arbeit (agent-m) |
| T-605 | Maskottchen Shino-chan (Platzhalter bis Q9 geklärt) | frei | T-110 | offen |
| T-606 | Achievements (lokal, Benachrichtigungen) | frei | T-103 | fertig |

## Phase 7 – Sync, Store & Release

| ID | Aufgabe | Revier | Abhängig von | Status |
|---|---|---|---|---|
| T-701 | Release-Workflow: Tag `v*` → Windows `.exe`/`.msi`, Linux AppImage/`.deb`, Android `.apk` als GitHub-Release | C | T-005, T-503 | offen |
| T-702 | Auto-Updates (`tauri-plugin-updater`, Update-Feed aus GitHub-Releases) | frei | T-701 | offen |
| T-703 | Sync ohne Server (verschlüsselter Export/Import von Lesezeichen + Einstellungen) | frei | T-104 | fertig |
| T-704 | Mod-Store (Index aus einem Git-Repo, Installieren/Entfernen) | frei | T-304 | offen |
| T-799 | **Release v0.1.0** mit `.exe` und `.apk` veröffentlichen | C | Phase 1, T-701 | offen |
