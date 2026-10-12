# shinoho-vpn

Rust-Crate für das eingebaute VPN (Revier: **agent-f**). Plan und Hintergrund: [`docs/VPN.md`](../../docs/VPN.md).

| Datei | Inhalt |
|---|---|
| `providers.json` | Empfohlene Anbieter mit Gratis-Plan, wird in die App eingebettet |
| `src/providers.rs` | Lädt und prüft die Anbieterliste |
| `src/wgconfig.rs` | Liest WireGuard-`.conf`-Dateien (für „Bring your own config“) |
| `src/tunnel/` | WireGuard-Tunnel im Userspace (boringtun + smoltcp) mit SOCKS5-Proxy auf `127.0.0.1`, ohne Admin-Rechte |
| `tests/tunnel_echo.rs` | Ende-zu-Ende-Test mit lokalem Test-VPN-Server |

Private Schlüssel werden in `Debug`-Ausgaben nie angezeigt.

```rust
let config: shinoho_vpn::WgConfig = std::fs::read_to_string("proton.conf")?.parse()?;
let tunnel = shinoho_vpn::Tunnel::start(&config).await?;
// WebView-Proxy auf socks5://{tunnel.socks_addr()} setzen (T-403)
```

Noch offen: Domainnamen/DNS über den Tunnel (T-404), Anbindung an die WebViews + Kill-Switch (T-403).

```bash
cd core/vpn
cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
```

Bis die Crate im Root-Workspace steht, hat sie eine eigene `[workspace]`-Tabelle in `Cargo.toml`.
