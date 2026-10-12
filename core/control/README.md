# shinoho-control

Logik für **Shinoho Control** (Revier: **agent-f**): Presets, die Entscheidung, welche Tabs
schlafen gelegt oder gedrosselt werden, und die Spiele-Erkennung für den Gaming-Modus.

| Teil | Inhalt |
|---|---|
| `Preset` / `Limits` | 🔋 Akku-Sparer · ⚖️ Ausgewogen · 🚀 Max Power · 🎮 Gaming, Limits passen sich dem RAM des Geräts an |
| `plan(limits, tabs)` | Liefert `Sleep`/`Throttle`-Aktionen. Fokussierte, tönende und aufnehmende Tabs bleiben unangetastet, angepinnte Tabs schlafen zuletzt |
| `hot_tabs(tabs, n)` | Die gierigsten Tabs für den Hot-Tabs-Killer |
| `GameWatcher` / `GameRules` | Gaming-Modus: erkennt laufende Spiele (Steam, Epic, GOG, Riot, Ubisoft, EA, Battle.net, Proton/Wine, Minecraft, bekannte Namen) und meldet Start/Ende. Eigene Spiele und Ausnahmen einstellbar |

Messwerte liefert später T-202, das Einfrieren übernimmt T-203.

```bash
cd core/control
cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
```
