# Shinoho Desktop (Tauri 2)

Desktop-App für Windows und Linux (macOS unsigniert). Zeigt im Moment nur ein leeres
Fenster mit Platzhalter-Seite (`placeholder/`). Sobald `ui/` steht (T-004), wird
`build.frontendDist` in `tauri.conf.json` auf die gebaute Oberfläche umgestellt.

## Voraussetzungen

- Rust ≥ 1.77 (`rustup`)
- **Linux** (Debian/Ubuntu):
  ```bash
  sudo apt install libwebkit2gtk-4.1-dev libsoup-3.0-dev libjavascriptcoregtk-4.1-dev \
    librsvg2-dev build-essential pkg-config libssl-dev
  ```
- **Windows**: Visual Studio Build Tools (C++), WebView2 (bei Windows 11 schon dabei)

## Starten & prüfen (im Repo-Hauptordner)

```bash
cargo run -p shinoho-desktop          # Fenster öffnen
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Installer bauen (optional, braucht die Tauri-CLI):

```bash
cargo install tauri-cli --version "^2" --locked
cd apps/desktop && cargo tauri build
```
