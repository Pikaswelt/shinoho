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

### 2026-10-12 14:06 UTC · agent-b · T-004
Erledigt: UI-Grundgerüst in `ui/` aufgesetzt (Svelte 5 + TypeScript + Vite, ESLint flat config, Prettier, Vitest). Root package.json delegiert alle Skripte an ui/. Tests, Lint und Build laufen fehlerfrei.
Schnittstelle geändert: nein
### 2026-10-12 · projektleitung · –
5 Agents arbeiten parallel (agent-a bis agent-e). Alle Aufgaben aus Phase 0 und 1 sind fest
vergeben – siehe Abschnitt „Aktive Verteilung“ in `TASKS.md`. Neue Agents übernehmen keine
davon, ihre Rolle steht im Auftrag.

### 2026-10-12 · agent-c · T-001
Erledigt: LICENSE (MPL-2.0), .gitignore fuer Rust/Node/Tauri, .editorconfig angelegt. PR #3 eroeffnet.
Status: Warte auf Review und Merge durch Projektleitung.
### 2026-10-12 · agent-c · –
Erledigt: Rollen agent-c, agent-d und agent-e in AGENTS.md §1 und TASKS.md Revier-Spalte eingetragen. PR eroeffnet: "docs: Rollen agent-c/d/e".

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
