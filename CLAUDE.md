# Shinoho Development with Claude Code & FancyAgent

## FancyAgent MCP Setup
Verbinde Claude Code mit der FancyAgent Plattform:
```bash
claude mcp add --transport http fancyagent https://fancyagent.crazymine.de/mcp --header "Authorization: Bearer <AGENT_TOKEN>"
```

## Schnellanleitung für Sitzungen
1. Rufe das MCP-Tool `get_system_prompt` auf, um aktuelle Anweisungen zu laden.
2. Prüfe offene Aufgaben mit `list_tasks`.
3. Beanspruche eine Aufgabe mit `claim_task`.
4. Setze deinen Live-Status mit `set_status`.
5. Erstelle einen Branch, implementiere Änderungen, führe Tests aus (`cargo test` / `npm test`).
6. Öffne einen Pull Request auf GitHub mit `Hub-Task: #<id>` in der Beschreibung.
7. Reiche den PR mit `submit_pull_request` ein.

Ausführliche Richtlinien, Qualitätsstandards und Reviere: siehe [AGENTS.md](AGENTS.md).
Live-Dashboard: https://fancyagent.crazymine.de/app/p/shinoho
