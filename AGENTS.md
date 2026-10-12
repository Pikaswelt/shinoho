# AGENTS.md – Regeln für AI-Agents im Shinoho-Projekt

Diese Datei gilt für **jeden** AI-Agent (Claude, Codex, Copilot, Gemini, …), der an Shinoho
arbeitet. Ziel: Zwei (oder mehr) Agents arbeiten **gleichzeitig** auf GitHub, ohne sich in
die Quere zu kommen.

> **Lies vor jeder Arbeitssitzung:** diese Datei → [`docs/agents/TASKS.md`](docs/agents/TASKS.md)
> → die letzten Einträge in [`docs/agents/LOG.md`](docs/agents/LOG.md) → [`docs/PLAN.md`](docs/PLAN.md).

---

## 1. Rollen & Reviere

| Agent | Name im Projekt | Revier (darf frei ändern) |
|---|---|---|
| **Agent A** | `agent-a` – „Kern“ | `core/`, `apps/`, `.github/workflows/`, `Cargo.toml`, Build-Skripte |
| **Agent B** | `agent-b` – „Oberfläche“ | `ui/`, `mods/`, `sdk/`, `docs/MODDING.md`, `package.json` |
| **Gemeinsam** | – | `core/api/` (Schnittstelle Kern ↔ UI), `docs/PLAN.md`, `AGENTS.md`, `README.md` |
| **Projektleitung** | Senpai-TV & Pikaswelt (Menschen) | Entscheidet alles, was in `docs/OPEN-QUESTIONS.md` steht |

**Regel:** Dateien im Revier des anderen Agents werden **nicht** geändert. Braucht man dort
etwas → Eintrag im LOG mit `@agent-x BITTE:` und warten bzw. an anderer Aufgabe weiterarbeiten.

Welcher Agent bin ich? → Steht in deinem Auftrag. Wenn nicht: nachfragen, **nicht raten**.
Kommt ein dritter Agent dazu, wird hier eine neue Zeile ergänzt (per PR).

---

## 2. Ablauf einer Aufgabe

1. **Aktualisieren:** `git fetch origin && git checkout main && git pull`
2. **Aufgabe wählen** in `docs/agents/TASKS.md` – nur Aufgaben aus dem eigenen Revier
   mit Status `offen`, deren Abhängigkeiten `fertig` sind.
3. **Aufgabe beanspruchen (Claim):** Status auf `in Arbeit (agent-x)` setzen, direkt auf
   `main` committen und pushen: `chore(tasks): agent-x übernimmt T-012`.
   - Push abgelehnt? → `git pull --rebase`, prüfen, ob die Aufgabe inzwischen der andere
     Agent genommen hat. Wenn ja: andere Aufgabe wählen.
   - Das ist der **einzige** direkte Commit auf `main`. Alles andere läuft über PRs.
4. **Branch anlegen:** `agent-a/T-012-kurzer-name` bzw. `agent-b/T-012-kurzer-name`
5. **Arbeiten** – kleine, thematische Commits (siehe §4).
6. **Pull Request** gegen `main` öffnen. Titel: `T-012: <Beschreibung>`.
   Im PR-Text: was, warum, wie getestet, betroffene Schnittstellen.
7. **CI muss grün sein.** Der andere Agent darf reviewen (Kommentar), die Projektleitung
   mergt bzw. gibt Merge frei.
8. Nach Merge: Status in `TASKS.md` auf `fertig` setzen, kurzer LOG-Eintrag.

**Max. 1 Aufgabe gleichzeitig pro Agent** (Ausnahme: Aufgabe ist blockiert → im LOG
vermerken, dann darf eine zweite begonnen werden).

---

## 3. Kommunikation – `docs/agents/LOG.md`

- **Nur anhängen**, nie alte Einträge ändern oder löschen.
- Format:
  ```
  ### 2026-10-12 14:30 UTC · agent-a · T-012
  Erledigt: …
  Schnittstelle geändert: …   (falls ja)
  @agent-b BITTE: …            (falls etwas gebraucht wird)
  @projektleitung FRAGE: …          (falls eine Entscheidung nötig ist)
  ```
- Fragen an die Projektleitung zusätzlich in `docs/OPEN-QUESTIONS.md` eintragen.
  **Bei offenen Grundsatzfragen nicht selbst entscheiden – fragen!**
- LOG-Einträge dürfen im eigenen PR mitkommen; bei Konflikten in `LOG.md` einfach
  **beide** Einträge behalten (chronologisch).

---

## 4. Git-Regeln

- **Nie** `git push --force` auf `main` oder auf Branches des anderen Agents.
- **Nie** den Branch des anderen Agents ändern.
- Commit-Nachrichten im Stil *Conventional Commits*, Deutsch oder Englisch:
  `feat(vpn): WireGuard-Proxy hinzugefügt`, `fix(ui): Tab-Leiste flackert`, `docs: …`
- Vor dem PR: `main` in den eigenen Branch mergen (oder rebasen, solange nur man selbst
  auf dem Branch arbeitet) und Konflikte lösen.
- Keine generierten Dateien committen (Builds, `node_modules/`, `target/`).
- Keine Geheimnisse (API-Keys, Tokens, VPN-Configs mit Private Keys) committen.

### Urheberschaft / Mitwirkende
- **Keine** `Co-Authored-By`-Zeilen, Signaturen oder Hinweise auf AI-Agents in Commits,
  PRs, Code-Kommentaren oder Doku.
- Autoren des Projekts sind **Senpai-TV** und **Pikaswelt**. AI-Agents tragen sich **nicht** als Contributor ein
  (keine `CONTRIBUTORS`-, `AUTHORS`-Einträge o. Ä.).

---

## 5. Schnittstelle Kern ↔ UI (`core/api/`)

- Alle Tauri-Commands und Events sind in `core/api/` beschrieben (Rust-Typen + generierte
  TypeScript-Typen für `ui/`).
- Änderung an der Schnittstelle = **eigene Aufgabe** im Board, Vorschlag im LOG,
  Bestätigung des anderen Agents (`OK von agent-b`) abwarten.
- Nur **abwärtskompatible** Erweiterungen ohne Absprache erlaubt (neues Feld optional,
  neuer Command). Umbenennen/Löschen nie ohne Absprache.

---

## 6. Qualitätsregeln

| Bereich | Muss vor PR laufen |
|---|---|
| Rust (`core/`) | `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` |
| UI (`ui/`, `sdk/`) | `npm run lint`, `npm run format:check`, `npm test` |
| Doku | Links funktionieren, Beispiele passen zum Code |

- Neue Funktionen bekommen Tests.
- Performance ist Kernversprechen: keine schweren Abhängigkeiten ohne Begründung im PR.
- UI-Texte über i18n-Dateien (`ui/i18n/de.json`, `en.json`), nie fest im Code.

---

## 7. Wenn etwas schiefgeht

- **Merge-Konflikt im fremden Revier:** nicht selbst lösen → LOG-Eintrag, anderen Agent bitten.
- **CI auf `main` rot:** wer es kaputt gemacht hat, repariert sofort; alle anderen mergen
  nichts, bis `main` wieder grün ist.
- **Unklarheit / Widerspruch im Plan:** stoppen, `@projektleitung FRAGE:` ins LOG und in
  `OPEN-QUESTIONS.md`, an anderer Aufgabe weiterarbeiten.
