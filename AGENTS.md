# AGENTS.md – Regeln & Workflow für KI-Agents im Shinoho-Projekt

Dieses Projekt wird kollaborativ von mehreren KI-Agents über **FancyAgent** gesteuert.
Das manuelle Editieren von `TASKS.md` und Committen auf `main` zum Task-Locking ist abgelöst.

> 🌐 **Dashboard:** [https://fancyagent.crazymine.de/app/p/shinoho](https://fancyagent.crazymine.de/app/p/shinoho)  
> 🔌 **MCP-Endpunkt:** `https://fancyagent.crazymine.de/mcp`

---

## 1. Verbindung über MCP

Agents binden FancyAgent als HTTP-MCP-Server ein:

```bash
claude mcp add --transport http fancyagent https://fancyagent.crazymine.de/mcp --header "Authorization: Bearer <DEIN_AGENT_TOKEN>"
```

### Start jeder Sitzung
1. **System-Prompt laden:** Rufe `get_system_prompt` auf. Dort stehen deine aktuellen Berechtigungen, Reviere und Projektanweisungen.
2. **Status setzen:** Rufe `set_status` auf (z. B. *„Prüfe offene Aufgaben für UI-Integration“*), damit Maintainer live im Dashboard sehen, woran du arbeitest.

---

## 2. Ablauf einer Aufgabe (Workflow)

```text
list_tasks 
  → claim_task (Aufgabe beanspruchen, Lease aktiv halten per heartbeat)
  → Branch erstellen: agent/<name>/T-xxx-beschreibung
  → Code implementieren & Tests ausführen (siehe §4 Qualitätsstandards)
  → Pull Request auf GitHub öffnen mit "Hub-Task: #<id>" im PR-Body
  → submit_pull_request aufrufen
  → Automatische Review-Aufgabe für Reviewer-Agents
  → Merge (Maintainer / FancyAgent mit sha-Commit-Bindung)
```

1. **Aufgabe wählen:** Rufe `list_tasks` auf.
2. **Beanspruchen:** Rufe `claim_task({ task_id: ... })` auf.
   * Keine direkten Commits auf `main` mehr nötig! FancyAgent sperrt die Aufgabe atomar für dich.
   * Halte deinen Lease bei längerer Arbeit mit `heartbeat({ task_id: ... })` aktiv.
3. **Branch & Entwicklung:**
   * Branch anlegen: `feat/T-xxx-...` oder `fix/T-xxx-...`
   * Conventional Commits verwenden (`feat(...)`, `fix(...)`, `chore(...)`).
4. **Pull Request einreichen:**
   * PR gegen `main` öffnen.
   * Im PR-Text zwingend den Marker hinterlegen:
     ```text
     Hub-Task: #<task_id>
     ```
   * Rufe `submit_pull_request({ task_id: ..., pull_number: ... })` auf.
   * FancyAgent prüft automatisch erlaubte/geschützte Pfade und erstellt die Review-Aufgabe.
5. **Review & Merge:**
   * Reviewer-Agents rufen `get_pull_request` und `submit_review({ task_id: ..., verdict: "approved" | "changes_requested" })` auf.
   * Sobald Freigaben vorliegen und CI grün ist, wird der PR via `merge_pull_request` gemergt.

---

## 3. Kommunikation & Dateien

* **Projekt-Chat:** Nutze `send_message` und `read_messages` / `inbox` für Absprachen, Fragen oder Übergaben an andere Agents.
* **Geteilte Notizen/Snippets:** Nutze `write_file` und `read_file` in FancyAgent, um Dumps, Logs oder Schnittstellen-Entwürfe projektweit bereitzustellen.
* Das alte Anhängen an `docs/agents/LOG.md` ist nicht mehr erforderlich, kann aber bei Bedarf ergänzend genutzt werden.

---

## 4. Architektur & Reviere

| Bereich | Pfade | Verantwortlichkeit |
|---|---|---|
| **Kern** | `core/`, `Cargo.toml` | Rust Workspace (storage, adblock, incognito, vpn, control, feeds) |
| **Desktop** | `apps/desktop/` | Tauri 2 Desktop-App & Shell |
| **Oberfläche** | `ui/`, `package.json` | Svelte 5 + TypeScript + Vite |
| **Schnittstelle** | `core/api/` | Tauri-Commands & Event-Typen (Kern ↔ UI) |
| **Infra/CI** | `.github/workflows/`, Build-Skripte | CI, Android-Builds, Releases |
| **Mods & SDK** | `sdk/`, `docs/MODDING.md` | Modding-Schnittstelle & Loader |

> **Wichtig für `core/api/`:** Änderungen an der Schnittstelle müssen abwärtskompatibel sein. Bei Breaking Changes immer zuerst im Chat (`send_message`) abstimmen!

---

## 5. Qualitätsstandards vor PR

Jeder PR muss lokal getestet werden, bevor `submit_pull_request` aufgerufen wird:

| Bereich | Pflicht-Checks |
|---|---|
| **Rust (`core/`, `apps/desktop`)** | `cargo fmt --check`<br>`cargo clippy -- -D warnings`<br>`cargo test` |
| **UI (`ui/`, `sdk/`)** | `npm run lint`<br>`npm run format:check`<br>`npm test` |
| **i18n (Mehrsprachigkeit)** | Texte **immer** über `ui/i18n/` (`de.json`, `en.json`, `ja.json`). Keine Hardcoded-Strings im UI-Code. |

---

## 6. Urheberschaft & Git-Regeln

* **Autoren:** Die offiziellen Autoren des Projekts sind **Senpai-TV** und **Pikaswelt**.
* **Keine AI-Signaturen:** Keine `Co-Authored-By`-Zeilen, Prompts, Modellnamen oder AI-Signaturen in Commits, PR-Beschreibungen oder Kommentaren.
* **Kein Force-Push:** Niemals `git push --force` auf `main`.
* **Geschützte Pfade:** `.github/**` ist geschützt und darf nur von Maintainern geändert werden.
