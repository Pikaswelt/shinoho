# Aufgaben-Board

Regeln zum Beanspruchen: [`AGENTS.md`](../../AGENTS.md) §2.
Status: `offen` · `in Arbeit (agent-x)` · `blockiert` · `review` · `fertig`

> ✅ Q1–Q8 sind beantwortet (Tauri 2, MPL-2.0, Windows + Linux zuerst …) – siehe
> [`OPEN-QUESTIONS.md`](../OPEN-QUESTIONS.md). Es kann losgehen!

## Phase 0 – Fundament

| ID | Aufgabe | Revier | Abhängig von | Status |
|---|---|---|---|---|
| T-001 | Lizenzdatei (MPL-2.0) + `.gitignore` + `.editorconfig` anlegen | A | – | review (agent-a) |
| T-002 | Tauri-2-Projekt anlegen (`apps/desktop`), leeres Fenster | A | – | review (agent-a) |
| T-003 | Rust-Workspace `core/` + `cargo fmt/clippy/test` einrichten | A | T-002 | offen |
| T-004 | UI-Grundgerüst `ui/` (Svelte + TS, Lint, Prettier, Vitest) | B | – | offen |
| T-005 | CI: Builds Windows + Linux (GitHub Actions); macOS unsigniert optional | A | T-002, T-004 | offen |
| T-006 | Schnittstelle `core/api/` v0: Tab öffnen/schließen/navigieren, Events | gemeinsam | T-003, T-004 | offen |
| T-007 | i18n-Grundgerüst (de, en, ja) | B | T-004 | offen |

## Phase 1 – MVP-Browser

| ID | Aufgabe | Revier | Abhängig von | Status |
|---|---|---|---|---|
| T-101 | Tab-Manager im Kern (mehrere WebViews, Lebenszyklus) | A | T-006 | offen |
| T-102 | Tab-Leiste + Adressleiste UI | B | T-006 | offen |
| T-103 | SQLite-Storage (Verlauf, Lesezeichen, Einstellungen) | A | T-003 | offen |
| T-104 | Lesezeichen: Backend (CRUD, Ordner, Import/Export HTML) | A | T-103 | offen |
| T-105 | Lesezeichen: UI (Leiste, Manager, Suche) | B | T-104 | offen |
| T-106 | Inkognito-Modus: Backend (flüchtiges Profil, kein Disk-Cache) | A | T-101 | offen |
| T-107 | Inkognito-Modus: UI (eigenes Fenster, eigener Look) | B | T-106 | offen |
| T-108 | Adblock mit `adblock-rust` | A | T-101 | offen |
| T-109 | Downloads-Manager (Backend + UI) | A + B | T-101 | offen |
| T-110 | Startseite „Shinoho Hub“ (Speed-Dial, Wallpaper) | B | T-102 | offen |

## Spätere Phasen

Werden aus [`PLAN.md`](../PLAN.md) §5 in Aufgaben zerlegt, sobald Phase 1 zu ~70 % fertig ist.
