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
| agent-c – Infra/Repo | T-001, Rollen-PR für AGENTS.md, T-005 |
| agent-d – Kern-2 | T-103, T-104, T-106, T-108, T-109 (Backend) |
| agent-e – UI-2 | T-110, T-105, T-107, T-109 (UI) |

**Neuer Agent?** Keine dieser Aufgaben übernehmen. Deine Rolle und Aufgabe bekommst du von
der Projektleitung im Auftrag. Steht keine drin: nachfragen, nicht raten (AGENTS.md §1).
Bis dahin höchstens offene PRs reviewen (nur Kommentare).

## Phase 0 – Fundament

| ID | Aufgabe | Revier | Abhängig von | Status |
|---|---|---|---|---|
| T-001 | Lizenzdatei (MPL-2.0) + `.gitignore` + `.editorconfig` anlegen | C | – | in Arbeit (agent-c) |
| T-002 | Tauri-2-Projekt anlegen (`apps/desktop`), leeres Fenster | A | – | in Arbeit (agent-a) |
| T-003 | Rust-Workspace `core/` + `cargo fmt/clippy/test` einrichten | A | T-002 | offen |
| T-004 | UI-Grundgerüst `ui/` (Svelte + TS, Lint, Prettier, Vitest) | B | – | review |
| T-005 | CI: Builds Windows + Linux (GitHub Actions); macOS unsigniert optional | C | T-002, T-004 | offen |
| T-006 | Schnittstelle `core/api/` v0: Tab öffnen/schließen/navigieren, Events | gemeinsam | T-003, T-004 | offen |
| T-007 | i18n-Grundgerüst (de, en, ja) | B | T-004 | in Arbeit (agent-b) |

## Phase 1 – MVP-Browser

| ID | Aufgabe | Revier | Abhängig von | Status |
|---|---|---|---|---|
| T-101 | Tab-Manager im Kern (mehrere WebViews, Lebenszyklus) | A | T-006 | offen |
| T-102 | Tab-Leiste + Adressleiste UI | B | T-006 | offen |
| T-103 | SQLite-Storage (Verlauf, Lesezeichen, Einstellungen) | D | T-003 | in Arbeit (agent-d) |
| T-104 | Lesezeichen: Backend (CRUD, Ordner, Import/Export HTML) | D | T-103 | in Arbeit (agent-d) |
| T-105 | Lesezeichen: UI (Leiste, Manager, Suche) | E | T-104 | offen |
| T-106 | Inkognito-Modus: Backend (flüchtiges Profil, kein Disk-Cache) | D | T-101 | offen |
| T-107 | Inkognito-Modus: UI (eigenes Fenster, eigener Look) | E | T-106 | offen |
| T-108 | Adblock mit `adblock-rust` | D | T-101 | offen |
| T-109 | Downloads-Manager (Backend + UI) | D + E | T-101 | offen |
| T-110 | Startseite „Shinoho Hub“ (Speed-Dial, Wallpaper) | E | T-102 | in Arbeit (agent-e) |

## Spätere Phasen

Werden aus [`PLAN.md`](../PLAN.md) §5 in Aufgaben zerlegt, sobald Phase 1 zu ~70 % fertig ist.
