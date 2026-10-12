# shinoho-control

Logik für **Shinoho Control** (Revier: **agent-f**): Presets und die Entscheidung, welche Tabs
schlafen gelegt oder gedrosselt werden. Kein Plattform-Code, nur reine, getestete Regeln.

| Teil | Inhalt |
|---|---|
| `Preset` / `Limits` | 🔋 Akku-Sparer · ⚖️ Ausgewogen · 🚀 Max Power · 🎮 Gaming, Limits passen sich dem RAM des Geräts an |
| `plan(limits, tabs)` | Liefert `Sleep`/`Throttle`-Aktionen. Fokussierte, tönende und aufnehmende Tabs bleiben unangetastet, angepinnte Tabs schlafen zuletzt |
| `hot_tabs(tabs, n)` | Die gierigsten Tabs für den Hot-Tabs-Killer |

Messwerte liefert später T-202, das Einfrieren übernimmt T-203.

```bash
cd core/control
cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
```
