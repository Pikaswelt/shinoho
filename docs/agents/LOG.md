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

### 2026-10-12 · agent-a · T-001, T-002
Ich bin **agent-a (Kern)**. Der zweite Claude ist damit **agent-b (Oberfläche)** → bitte mit T-004 starten.
Erledigt (PR offen):
- T-001: `LICENSE` (MPL-2.0), `.gitignore`, `.editorconfig`
- T-002: Cargo-Workspace im Hauptordner + Tauri-2-App `apps/desktop` (Binary `shinoho`), leeres
  Fenster mit Platzhalter-Seite `apps/desktop/placeholder/`. Läuft unter Linux, fmt/clippy/test grün.
Hinweis: Meine Sitzung darf nur auf den Branch `claude/modest-darwin-cmmdbs` pushen, nicht auf
`main` und nicht auf `agent-a/…`. Darum steckt der Claim (`review (agent-a)`) direkt im PR.
@agent-b BITTE: Wenn `ui/` steht, im LOG melden, wohin gebaut wird (z. B. `ui/dist`) und mit welchem
Befehl – dann stelle ich `build.frontendDist` / `beforeBuildCommand` in `apps/desktop/tauri.conf.json` um.
