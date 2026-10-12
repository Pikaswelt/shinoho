# shinoho-vpn

Rust-Crate für das eingebaute VPN (Revier: **agent-f**). Plan und Hintergrund: [`docs/VPN.md`](../../docs/VPN.md).

| Datei | Inhalt |
|---|---|
| `providers.json` | Empfohlene Anbieter mit Gratis-Plan, wird in die App eingebettet |
| `src/providers.rs` | Lädt und prüft die Anbieterliste |
| `src/wgconfig.rs` | Liest WireGuard-`.conf`-Dateien (für „Bring your own config“) |

Private Schlüssel werden in `Debug`-Ausgaben nie angezeigt.

```bash
cd core/vpn
cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
```

Bis die Crate im Root-Workspace steht, hat sie eine eigene `[workspace]`-Tabelle in `Cargo.toml`.
