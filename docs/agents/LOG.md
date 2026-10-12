# Agent-LOG

Nur anhängen – nie ändern oder löschen. Format siehe [`AGENTS.md`](../../AGENTS.md) §3.
Neueste Einträge **unten**.

---

### 2026-10-12 · setup · –
Plan, Agent-Regeln, Aufgaben-Board und Mod-Doku angelegt.
Offene Grundsatzfragen stehen in `docs/OPEN-QUESTIONS.md` – **bevor Q1 (Engine/Framework)
beantwortet ist, wird kein Code geschrieben.**

### 2026-10-12 · setup · –
Q1–Q8 von der Projektleitung beantwortet: **Tauri 2**, **MPL-2.0**, **Windows + Linux zuerst**
(dann Android, dann macOS/iOS), Apple-Konto **später**, UI in **de/en/ja**, Maskottchen
**Shino-chan**, Sync **erst ohne Server**, Agents **Claude (A) + Claude (B)**.
Aufgaben T-001, T-002 und T-004 sind jetzt `offen`. Neu: PRs sofort öffnen, Ersatz-Agent-Regel (AGENTS.md §8).
Noch offen: Q9 (Shino-chan Stil/Zeichner) – blockiert nichts vor Phase 6.

### 2026-10-12 · projektleitung · –
5 Agents arbeiten parallel (agent-a bis agent-e). Alle Aufgaben aus Phase 0 und 1 sind fest
vergeben – siehe Abschnitt „Aktive Verteilung“ in `TASKS.md`. Neue Agents übernehmen keine
davon, ihre Rolle steht im Auftrag.

### 2026-10-12 · agent-c · T-001
Erledigt: LICENSE (MPL-2.0), .gitignore fuer Rust/Node/Tauri, .editorconfig angelegt. PR #3 eroeffnet.
Status: Gemergt.

### 2026-10-12 · agent-c · –
Erledigt: Rollen agent-c, agent-d und agent-e in AGENTS.md §1 und TASKS.md Revier-Spalte eingetragen. PR eroeffnet: "docs: Rollen agent-c/d/e".
Status: Gemergt.

<<<<<<< HEAD
### 2026-10-12 14:06 UTC · agent-b · T-004
Erledigt: UI-Grundgerüst in `ui/` aufgesetzt (Svelte 5 + TypeScript + Vite, ESLint flat config, Prettier, Vitest). Root package.json delegiert alle Skripte an ui/. Tests, Lint und Build laufen fehlerfrei.
Schnittstelle geändert: nein

### 2026-10-12 14:15 UTC · agent-a · T-002
Erledigt: Tauri-2-Grundgerüst in `apps/desktop` eingerichtet (Tauri 2.12, tauri-build 2.7, leeres Fenster mit Fallback-UI, Tauri-Capabilities, Icons). Rust-Workspace-Root in `Cargo.toml` angelegt.

### 2026-10-12 14:16 UTC · agent-b · T-007
Erledigt: i18n-Grundgerüst implementiert (`ui/i18n/de.json`, `en.json`, `ja.json` mit `_todo_review: true`). TypeScript-Modul `ui/src/i18n/` mit Reaktivität, Fallback, Sprachumschaltung und Parameterersetzung. App- und Header-Komponenten auf i18n umgestellt. Tests für alle 3 Sprachen und Konsistenzprüfung erfolgreich.
Schnittstelle geändert: nein

<<<<<<< HEAD
### 2026-10-12 · agent-f · T-401
Neu dabei: **agent-f – VPN & Performance** (Auftrag der Projektleitung: freie Aufgaben übernehmen).
Revier: `core/vpn/`, `core/control/`, `docs/VPN.md`. Phase 2 und 4 in Aufgaben zerlegt (T-201…T-206, T-401…T-406).
Erledigt (PR offen): T-401, Crate `core/vpn` mit WireGuard-Parser und `providers.json`; 11 Tests, fmt/clippy grün.
Hinweis: Meine Sitzung darf nur auf `claude/modest-darwin-cmmdbs` pushen, daher kein `agent-f/…`-Branch und kein
Claim-Commit auf `main`; der Claim steckt im PR.
@agent-a BITTE: Sobald der Root-Workspace steht (T-002/T-003), `core/vpn` (und später `core/control`) in
`members` eintragen. Dann entferne ich die eigene `[workspace]`-Tabelle in `core/vpn/Cargo.toml`.
@agent-c BITTE: In die Root-`.gitignore` gern `**/target/` aufnehmen; bis dahin hat `core/vpn` eine eigene.
Als Nächstes: T-201 (`core/control`), hängt von nichts ab.
### 2026-10-12 14:15 UTC · agent-a · T-002
Erledigt: Tauri-2-Grundgerüst in `apps/desktop` eingerichtet (Tauri 2.12, tauri-build 2.7, leeres Fenster mit Fallback-UI, Tauri-Capabilities, Icons). Rust-Workspace-Root in `Cargo.toml` angelegt.

### 2026-10-12 14:10 UTC · agent-d · T-103
Erledigt: Eigenständige Crate `core/storage` mit SQLite-Backend (rusqlite bundled), Schema-Migrationen, Verlauf (History Visits & Top Sites), Lesezeichen (Baumstruktur & CRUD), Einstellungen (KV & JSON) und Downloads implementiert. Tests, clippy und fmt erfolgreich.
Status: Fertig und in Root-Workspace eingebunden.

### 2026-10-12 · agent-c · T-005
Erledigt: GitHub Actions CI-Workflow (.github/workflows/ci.yml) für UI, Rust und Desktop-Builds (Linux, Windows, macOS) eingerichtet.
Abhängigkeiten T-002 und T-004 gemergt, main in Branch integriert. PR #9 fertiggestellt.

### 2026-10-12 14:25 UTC · agent-b · T-006 (UI-Teil)
Erledigt: TypeScript-Typen und Schnittstellendefinition in `core/api/` (types.ts, README.md, index.ts) für Tab- und Navigationsmanagement erstellt. Client-Service `ui/src/api/tabs.ts` mit Tauri-IPC und In-Memory-Fallback für lokale Entwicklung und Vitest umgesetzt (15 Tests grün).
Schnittstelle geändert: ja (neu: core/api v0 für Tabs/Navigation)
@agent-a BITTE: Prüfe die Schnittstelle in `core/api/README.md` und `core/api/types.ts` für die Rust-Implementierung von T-006/T-101 (Commands `create_tab`, `close_tab`, `activate_tab`, `navigate_tab`, `go_back`, `go_forward`, `reload_tab`, `get_tabs` und Events `shinoho://tab-*`). Bitte um kurzes "OK von agent-a" im LOG.

### 2026-10-12 · agent-c · –
Erledigt: .gitattributes mit merge=union fuer LOG.md und text=auto eol=lf angelegt, um Merge-Konflikte im LOG und CRLF-Probleme unter Windows zu verhindern.

### 2026-10-12 · projektleitung · –
Phasen 3, 5, 6 und 7 in Aufgaben zerlegt. Ziel: **Release v0.1.0 mit `.exe` und `.apk`** (T-701, T-799 → agent-c).
Neu: AGENTS.md §9 – Gast-Agents arbeiten nur an Aufgaben mit Revier `frei`. Alle Agents mergen
eigene PRs selbst, sobald die Checks grün sind.

