# Shinoho – Gesamtplan

> Dieses Dokument ist die zentrale Planung. Änderungen am Plan nur per Pull Request
> und mit Eintrag in [`agents/LOG.md`](agents/LOG.md).
> Punkte mit ❓ sind noch offen – siehe [`OPEN-QUESTIONS.md`](OPEN-QUESTIONS.md).

---

## 1. Vision

Ein Browser, der **sich wie ein Spiel anfühlt**: schnell, laut (wenn man will), bunt,
komplett umbaubar – und trotzdem privat und sicher.

**Zielgruppe:** Anime-Fans, Gamer, Streamer, Leute, die ihren Browser gern „tunen“.

**Kernversprechen**

1. **Modifizierbar wie Opera GX** – Themes, Sounds, Wallpaper, Layouts, Skripte.
2. **Eingebautes VPN** – mit Vorschlägen für gute kostenlose Anbieter.
3. **Lesezeichen & Inkognito** – selbstverständlich, sauber umgesetzt.
4. **Maximale Performance, frei einstellbar** – RAM/CPU/Netzwerk-Limiter, Presets.
5. **Überall** – Windows, Linux, macOS, Android (APK), iOS.
6. **Offen für Mods** – dokumentierte Mod-API, die auch AI-Agents nutzen können.

---

## 2. Technische Architektur (Vorschlag ❓)

### 2.1 Empfehlung: Tauri 2 + Rust-Kern + Web-UI

| Schicht | Technologie | Warum |
|---|---|---|
| App-Hülle | **Tauri 2** | Eine Codebasis für Windows, Linux, macOS, **Android und iOS** |
| Kern (Tabs, VPN, Adblock, Limiter, Sync) | **Rust** | Schnell, sicher, wenig RAM |
| Browser-UI (Tabs, Sidebar, Settings) | **TypeScript + Svelte** + CSS-Variablen | Leicht, schnell, perfekt für Themes/Mods |
| Rendering-Engine | System-WebView: WebView2 (Chromium) auf Windows, WebKitGTK auf Linux, WKWebView auf macOS/iOS, Android System WebView | Kein eigener 30-GB-Chromium-Build nötig |
| Adblock | `adblock-rust` (EasyList, uBlock-Listen) | Spart RAM & CPU, schnellere Seiten |
| VPN | WireGuard in Rust (`boringtun`) → lokaler Proxy nur für den Browser | Siehe [`VPN.md`](VPN.md) |
| Datenbank | SQLite (Lesezeichen, Verlauf, Einstellungen) | Bewährt, überall verfügbar |

**Wichtig:** Apple erlaubt auf iOS nur WebKit als Engine (Ausnahme: EU mit Sondergenehmigung).
Mit Tauri ist das automatisch erfüllt.

### 2.2 Alternativen (zur Entscheidung)

| Option | Vorteile | Nachteile |
|---|---|---|
| **A) Tauri 2** (empfohlen) | Alle 5 Plattformen, kleines Team machbar, sehr leicht | Engine je nach OS unterschiedlich, Chrome-Erweiterungen nicht direkt nutzbar |
| B) Chromium-Fork (wie Opera GX/Brave) | Beste Kompatibilität, Chrome-Extensions | Riesiger Aufwand, Build-Server nötig, iOS trotzdem WebKit |
| C) Electron (Desktop) + eigene Android/iOS-Apps | Einheitliches Chromium auf Desktop | Hoher RAM-Verbrauch (passt nicht zu „beste Performance“), 3 Codebasen |

### 2.3 Geplante Ordnerstruktur

```
shinoho/
├── AGENTS.md               # Regeln für AI-Agents
├── docs/                   # Plan, Doku, Agent-Board
├── core/                   # Rust: Tab-Manager, Settings, Storage, Limiter
│   ├── vpn/                # WireGuard + lokaler Proxy
│   ├── adblock/
│   └── mods/               # Mod-Loader, Sandbox, Berechtigungen
├── ui/                     # Svelte/TypeScript Browser-Oberfläche
│   ├── themes/             # Standard-Themes
│   └── components/
├── apps/
│   ├── desktop/            # Tauri-Konfiguration Windows/Linux/macOS
│   └── mobile/             # Tauri-Konfiguration Android/iOS
├── mods/                   # Offizielle Beispiel-Mods
├── sdk/                    # Mod-SDK (Typen, CLI `shinoho-mod`)
└── .github/workflows/      # CI: Builds für alle Plattformen
```

---

## 3. Feature-Übersicht

### 3.1 Basis-Browser (MVP)
- Tabs (inkl. Tab-Gruppen, angepinnte Tabs, vertikale Tabs optional)
- Adressleiste mit Suche (Standard-Suchmaschine wählbar, z. B. DuckDuckGo, Startpage, Google)
- Vor/Zurück/Neu laden, Downloads-Manager, Verlauf
- **Lesezeichen**: Ordner, Lesezeichen-Leiste, Import (Chrome/Firefox/Opera HTML), Export, Suche, Tags
- **Inkognito-Modus**: eigenes Fenster, nichts wird gespeichert (kein Verlauf, keine Cookies nach dem Schließen, kein Cache auf Disk), eigener Look (z. B. dunkles Lila + Ninja-Maskottchen 🥷), optional: VPN automatisch im Inkognito an
- Eingebauter Werbe- & Tracker-Blocker
- Startseite „Shinoho Hub“ (Speed-Dial, Wallpaper, Widgets)

### 3.2 Modifizieren wie Opera GX
- Themes: Farben, Schriften, Rundungen, Transparenz, Blur
- Live-Wallpaper (Video/animiert) auf Startseite und optional hinter der UI
- **Browser-Sounds** (Klicks, Tippen, Tab öffnen/schließen) – Soundpacks
- Hintergrundmusik-Option für die Startseite
- Layout-Editor: Sidebar links/rechts, Tabs oben/seitlich, Leisten ein/aus
- Seiten-Mods: eigenes CSS pro Website („Dark-Mode für alles“, Schriftart erzwingen)
- Mod-Store („Shinoho Mods“) – später; am Anfang Installation per Datei/Link
- Alles Details: [`MODDING.md`](MODDING.md)

### 3.3 Performance – „Shinoho Control“ (wie GX Control, aber mehr)
| Regler | Funktion |
|---|---|
| **RAM-Limiter** | Max. RAM festlegen; Tabs werden bei Bedarf schlafen gelegt |
| **CPU-Limiter** | Max. CPU-Last; Hintergrund-Tabs werden gedrosselt |
| **Netzwerk-Limiter** | Bandbreite begrenzen (z. B. damit Downloads nicht den Game-Ping killen) |
| **Tab-Schlaf** | Inaktive Tabs nach X Minuten einfrieren |
| **Hot-Tabs-Killer** | Zeigt die gierigsten Tabs, mit einem Klick beenden |
| **Presets** | 🔋 Akku-Sparer · ⚖️ Ausgewogen · 🚀 Max Power · 🎮 Gaming-Modus |
| **Hardware-Beschleunigung** | An/Aus, GPU-Auswahl |

Ziele (Messung per Benchmark in der CI):
- Kaltstart < 1 s auf Desktop, < 2 s auf Mittelklasse-Android
- Leerlauf-RAM < 150 MB (Desktop, 1 Tab)
- Installer < 20 MB (Desktop)

### 3.4 Eingebautes VPN
- Browser-VPN (nur Browser-Verkehr) über WireGuard
- Anbieter-Vorschläge mit kostenlosen Plänen, „Bring your own config“
- Kill-Switch für Tabs, Standort-Anzeige, Leak-Test-Seite
- Details: [`VPN.md`](VPN.md)

### 3.5 Anime- & Gaming-Extras
Siehe [`FEATURES-ANIME-GAMING.md`](FEATURES-ANIME-GAMING.md).

### 3.6 Sync & Datenschutz
- Optional: Ende-zu-Ende verschlüsselter Sync (Lesezeichen, Einstellungen, Mods) zwischen Geräten ❓
- Keine Telemetrie ohne ausdrückliche Zustimmung
- Export aller Daten jederzeit

---

## 4. Plattformen

| Plattform | Paket | Engine | Hinweise |
|---|---|---|---|
| Windows 10/11 | `.msi` / `.exe` | WebView2 (Chromium) | WebView2 ist auf Win 11 vorinstalliert |
| Linux | `.AppImage`, `.deb`, `.rpm`, Flatpak | WebKitGTK | |
| macOS | `.dmg` | WKWebView | Signierung braucht Apple-Developer-Konto (99 $/Jahr) ❓ |
| Android | **`.apk`** (+ später Play Store `.aab`) | Android System WebView | Sideload-APK über GitHub Releases |
| iOS/iPadOS | `.ipa` (App Store/TestFlight) | WKWebView (Pflicht) | Braucht Apple-Developer-Konto; VPN auf iOS nur über NetworkExtension ❓ |

---

## 5. Roadmap

| Phase | Inhalt | Ergebnis |
|---|---|---|
| **0 – Fundament** | Repo-Struktur, CI (GitHub Actions) für alle Plattformen, Lizenz, Coding-Style, Agent-Regeln | „Hallo Welt“-Fenster baut auf allen Plattformen |
| **1 – MVP-Browser** | Tabs, Adressleiste, Navigation, Verlauf, Downloads, Lesezeichen, Inkognito, Adblock | Benutzbarer Browser (Desktop) |
| **2 – Shinoho Control** | RAM/CPU/Netz-Limiter, Tab-Schlaf, Presets, Hot-Tabs-Killer | Performance-Features |
| **3 – Mods v1** | Theme-Engine, Soundpacks, Wallpaper, Seiten-CSS, Mod-Loader + Sandbox, SDK + Doku | Erste Community-Mods möglich |
| **4 – VPN** | WireGuard-Proxy, Anbieter-Assistent, Kill-Switch, Leak-Test | Eingebautes VPN |
| **5 – Mobile** | Android-APK, iOS-Build, Touch-UI, Gesten | Läuft auf Handy |
| **6 – Anime & Gaming** | Anime-Tracker, Release-Kalender, Gaming-Modus, Free-Games-Feed, Maskottchen, Achievements | Die „coolen Extras“ |
| **7 – Sync & Store** | E2E-Sync, Mod-Store, Auto-Updates | Version 1.0 |

Jede Phase wird in [`agents/TASKS.md`](agents/TASKS.md) in kleine Aufgaben zerlegt.

---

## 6. Arbeitsaufteilung für 2 AI-Agents

Damit sich zwei Agents nicht in die Quere kommen, bekommt jeder ein **eigenes Revier**
(Details und Regeln in [`../AGENTS.md`](../AGENTS.md)):

| | **Agent A – „Kern“** | **Agent B – „Oberfläche“** |
|---|---|---|
| Ordner | `core/`, `apps/`, `.github/workflows/` | `ui/`, `mods/`, `sdk/`, `docs/MODDING.md` |
| Themen | Tabs-Backend, Storage, VPN, Adblock, Limiter, Builds, Mobile-Hülle | UI, Themes, Sounds, Wallpaper, Mod-API (TS-Seite), Anime/Gaming-UI |
| Schnittstelle | Definiert gemeinsam in `core/api/` (Tauri-Commands + Events) – Änderungen nur nach Absprache im LOG |

---

## 7. Qualität & Sicherheit
- Jede Änderung per Pull Request, CI muss grün sein
- Rust: `cargo fmt`, `cargo clippy`, Tests · UI: `prettier`, `eslint`, `vitest`
- Mods laufen in einer Sandbox mit Berechtigungen (wie Browser-Extensions)
- Updates signiert (Tauri Updater)
- Keine urheberrechtlich geschützten Anime-Figuren in offiziellen Themes ohne Lizenz – eigenes Maskottchen statt dessen
