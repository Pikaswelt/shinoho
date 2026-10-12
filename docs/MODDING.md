# Shinoho Modding-Handbuch

> **Status: Entwurf (API v0).** Die Mod-API wird in Phase 3 gebaut. Dieses Dokument ist die
> Spezifikation, an der sich die Umsetzung orientiert – und die Anleitung, mit der Menschen
> **und AI-Agents** eigene Mods bauen können.

Mit Mods kann man Shinoho **komplett** umbauen: Aussehen, Sounds, Layout, Startseite,
Verhalten auf Websites und eigene Funktionen.

---

## 0. Kurzanleitung für AI-Agents

Wenn dich jemand bittet, „eine Shinoho-Mod zu bauen“:

1. Lies §1 (Aufbau) und §2 (Manifest) – das Manifest ist Pflicht.
2. Wähle die passenden Mod-Bausteine aus §3 (oft reicht ein Theme ohne JavaScript).
3. **Fordere nur Berechtigungen an, die wirklich nötig sind** (§5).
4. Halte dich an die Regeln in §7 (Performance, Sicherheit, keine geschützten Inhalte
   ohne Erlaubnis des Nutzers).
5. Prüfe mit `shinoho-mod validate` (§6) und liefere den Ordner bzw. die `.shinomod`-Datei aus.
6. Erkläre dem Nutzer kurz, wie man die Mod installiert (§6.3).

Vorlagen zum Kopieren: §8.

---

## 1. Aufbau einer Mod

Eine Mod ist ein Ordner (zum Weitergeben als ZIP mit Endung `.shinomod`):

```
meine-mod/
├── mod.json          # Pflicht: Manifest
├── theme.css         # optional: Theme (CSS-Variablen + eigene Regeln)
├── sounds/           # optional: Soundpack (.ogg / .mp3 / .wav)
├── wallpaper/        # optional: Bilder (.webp/.png/.jpg) oder Video (.webm/.mp4)
├── sites/            # optional: Seiten-Mods (CSS/JS pro Website)
├── scripts/          # optional: Mod-Logik (JavaScript/TypeScript → JS)
├── widgets/          # optional: Widgets für Startseite/Sidebar (HTML/CSS/JS)
├── i18n/             # optional: Übersetzungen (de.json, en.json, …)
└── icon.png          # optional: 256×256 Icon
```

---

## 2. Das Manifest `mod.json`

```json
{
  "id": "pikaswelt.sakura-night",
  "name": "Sakura Night",
  "version": "1.0.0",
  "apiVersion": 0,
  "author": "Pikaswelt",
  "description": "Dunkles Theme mit fallenden Kirschblüten",
  "license": "MIT",
  "icon": "icon.png",

  "theme": "theme.css",
  "sounds": { "pack": "sounds/" },
  "wallpaper": { "file": "wallpaper/night.webm", "dim": 0.3 },
  "sites": [
    { "match": ["*://*.youtube.com/*"], "css": ["sites/youtube.css"] }
  ],
  "scripts": { "background": "scripts/main.js" },
  "widgets": [
    { "id": "clock", "entry": "widgets/clock.html", "area": "hub" }
  ],
  "settings": [
    { "key": "petals", "type": "boolean", "default": true, "label": "Kirschblüten anzeigen" },
    { "key": "petalCount", "type": "number", "min": 0, "max": 200, "default": 60, "label": "Anzahl Blüten" }
  ],
  "permissions": ["ui.theme", "ui.sounds", "storage"]
}
```

| Feld | Pflicht | Beschreibung |
|---|---|---|
| `id` | ✅ | Eindeutig, Format `autor.mod-name` (Kleinbuchstaben, Ziffern, `-`, `.`) |
| `name` | ✅ | Anzeigename |
| `version` | ✅ | SemVer (`1.2.3`) |
| `apiVersion` | ✅ | Version der Mod-API, aktuell `0` |
| `author` | ✅ | Name des Erstellers |
| `description` | – | Kurzbeschreibung |
| `license` | – | SPDX-Kennung, z. B. `MIT` |
| `theme` | – | Pfad zur CSS-Datei |
| `sounds` | – | Soundpack (§3.2) |
| `wallpaper` | – | Hintergrund (§3.3) |
| `sites` | – | Seiten-Mods (§3.5) |
| `scripts` | – | Mod-Logik (§4) |
| `widgets` | – | Widgets (§3.6) |
| `settings` | – | Einstellungen, die Nutzer im Mod-Menü ändern können |
| `permissions` | – | Berechtigungen (§5); ohne Angabe: keine |

Erlaubte `settings.type`: `boolean`, `number`, `string`, `color`, `select` (mit `options`), `file`.

---

## 3. Mod-Bausteine

### 3.1 Themes (`theme.css`)

Die gesamte Oberfläche nutzt **CSS-Variablen**. Ein Theme überschreibt sie:

```css
:root {
  /* Farben */
  --sh-bg: #14101c;
  --sh-bg-elevated: #1e1729;
  --sh-fg: #f5e9f0;
  --sh-fg-muted: #a897ad;
  --sh-accent: #ff6fae;
  --sh-accent-contrast: #14101c;
  --sh-danger: #ff4d4d;

  /* Form */
  --sh-radius: 10px;
  --sh-tab-height: 34px;
  --sh-font-ui: "Nunito", system-ui, sans-serif;

  /* Effekte */
  --sh-blur: 12px;
  --sh-glow: 0 0 12px var(--sh-accent);
  --sh-transition: 150ms ease;
}
```

Wichtige Bereiche (stabile Klassen/Attribute, Teil der API):

| Selektor | Bereich |
|---|---|
| `[data-sh="window"]` | Ganzes Fenster |
| `[data-sh="tabbar"]`, `[data-sh="tab"]`, `[data-sh="tab"][data-active]` | Tab-Leiste, Tabs |
| `[data-sh="addressbar"]` | Adressleiste |
| `[data-sh="sidebar"]` | Seitenleiste |
| `[data-sh="hub"]` | Startseite |
| `[data-sh="bookmarks-bar"]` | Lesezeichen-Leiste |
| `[data-sh="window"][data-private]` | Inkognito-Fenster |

> Nur `data-sh`-Selektoren sind stabil. Interne Klassennamen können sich jederzeit ändern.

Vollständige Variablenliste: wird in Phase 3 aus `ui/themes/variables.css` generiert.

### 3.2 Soundpacks

```json
"sounds": {
  "pack": "sounds/",
  "volume": 0.6,
  "map": {
    "tab.open": "open.ogg",
    "tab.close": "close.ogg",
    "click": "click.ogg",
    "type": ["key1.ogg", "key2.ogg", "key3.ogg"],
    "download.done": "done.ogg",
    "hub.music": "lofi-loop.ogg"
  }
}
```

Bei einer Liste wird zufällig gewählt. Ereignisse: `click`, `hover`, `type`, `tab.open`,
`tab.close`, `tab.switch`, `download.start`, `download.done`, `error`, `vpn.on`, `vpn.off`,
`private.open`, `hub.music` (Schleife).

### 3.3 Wallpaper

```json
"wallpaper": { "file": "wallpaper/bg.webm", "dim": 0.3, "blur": 0, "where": ["hub", "ui"] }
```

`where`: `hub` (Startseite), `ui` (hinter Tab-/Adressleiste, braucht Transparenz im Theme).
Videos pausieren automatisch im Akku-Sparer- und Gaming-Modus.

### 3.4 Layout

```json
"layout": {
  "tabs": "top",
  "sidebar": "left",
  "bookmarksBar": "auto",
  "compact": false
}
```

`tabs`: `top` | `left` | `right` | `hidden` · `sidebar`: `left` | `right` | `hidden`.

### 3.5 Seiten-Mods

CSS/JS, das auf bestimmten Websites eingefügt wird (wie Userstyles/Userscripts):

```json
"sites": [
  {
    "match": ["*://*.crunchyroll.com/*"],
    "css": ["sites/crunchyroll.css"],
    "js": ["sites/crunchyroll.js"],
    "runAt": "document_idle"
  }
]
```

Seiten-JS braucht die Berechtigung `sites` **und** die passenden Host-Muster in `match`.

### 3.6 Widgets

Kleine HTML-Apps für Startseite (`hub`) oder Sidebar (`sidebar`). Sie laufen isoliert
(iframe-Sandbox) und kommunizieren nur über `shinoho.*` (§4).

---

## 4. Die JavaScript-API `shinoho.*` (Entwurf)

Skripte laufen in einer **Sandbox** (eigener Worker), ohne direkten Zugriff auf Dateisystem
oder Browser-Interna. Alles geht über das globale Objekt `shinoho`. Alle Funktionen sind
`async`. TypeScript-Typen: Paket `@shinoho/mod-sdk` (Phase 3).

```ts
// Einstellungen der eigenen Mod
const count = await shinoho.settings.get("petalCount");
shinoho.settings.onChange("petalCount", (v) => { /* … */ });

// Speicher (Berechtigung "storage")
await shinoho.storage.set("highscore", 42);

// Tabs (Berechtigung "tabs")
const tabs = await shinoho.tabs.list();
await shinoho.tabs.open("https://anilist.co");
shinoho.tabs.onActivated((tab) => { /* … */ });

// Oberfläche
await shinoho.ui.toast("Download fertig! 🎉");          // keine Berechtigung nötig
await shinoho.ui.setAccent("#00e5ff");                   // "ui.theme"
await shinoho.sounds.play("download.done");              // "ui.sounds"
await shinoho.ui.addCommand({                            // "ui.commands"
  id: "random-anime", title: "Zufälliger Anime", shortcut: "Ctrl+Shift+R",
  run: () => shinoho.tabs.open("https://anilist.co/search/anime")
});

// Performance-Infos (Berechtigung "system.stats")
const s = await shinoho.system.stats(); // { ramMB, cpuPercent, netKBs, gaming: boolean }
shinoho.system.onGamingMode((active) => { /* … */ });

// Netzwerk-Anfragen (Berechtigung "net" + Host in "hosts")
const res = await shinoho.net.fetch("https://graphql.anilist.co", { method: "POST", body: "…" });

// Lebenszyklus
shinoho.onUnload(() => { /* aufräumen */ });
```

| Namespace | Zweck | Berechtigung |
|---|---|---|
| `settings` | Mod-eigene Einstellungen | – |
| `storage` | Key-Value-Speicher der Mod | `storage` |
| `tabs` | Tabs lesen/öffnen/schließen | `tabs` |
| `bookmarks` | Lesezeichen lesen/anlegen | `bookmarks` |
| `ui` | Toasts, Theme, Befehle, Sidebar-Panels | `ui.theme`, `ui.commands`, `ui.sidebar` |
| `sounds` | Sounds abspielen | `ui.sounds` |
| `system` | RAM/CPU/Netz, Gaming-Modus | `system.stats` |
| `net` | HTTP-Anfragen | `net` + `hosts` |
| `vpn` | Status lesen (nicht steuern) | `vpn.status` |

---

## 5. Berechtigungen

Beim Installieren zeigt Shinoho alle Berechtigungen an; der Nutzer muss zustimmen.

```json
"permissions": ["storage", "tabs", "net"],
"hosts": ["https://graphql.anilist.co/*"]
```

- **Im Inkognito-Fenster** laufen Mods nur, wenn der Nutzer das pro Mod erlaubt.
- Mods sehen **nie**: Passwörter, Cookies, Verlauf des Inkognito-Modus, VPN-Schlüssel.
- Kritische Berechtigungen (`sites` mit `<all_urls>`, `net` ohne Host-Einschränkung) werden
  rot markiert.

---

## 6. Werkzeuge

### 6.1 CLI `shinoho-mod` (Phase 3)
```bash
npx shinoho-mod new meine-mod --template theme   # Vorlage: theme | soundpack | widget | script
npx shinoho-mod validate meine-mod/              # prüft Manifest, Dateien, Berechtigungen
npx shinoho-mod pack meine-mod/                  # → meine-mod-1.0.0.shinomod
```

### 6.2 Entwicklermodus
`Einstellungen → Mods → Entwicklermodus` → „Ordner laden“: Änderungen werden live neu geladen.
Mod-Konsole: `shinoho://mods/console`.

### 6.3 Installation
- `.shinomod`-Datei ins Browserfenster ziehen, oder
- `Einstellungen → Mods → Aus Datei installieren`, oder
- später: Shinoho-Mod-Store.

---

## 7. Regeln für gute Mods

1. **Performance:** keine Dauerschleifen, Animationen mit `requestAnimationFrame`/CSS,
   im Gaming-/Akku-Modus (`shinoho.system.onGamingMode`) Effekte pausieren.
   Wallpaper-Videos ≤ 1080p, ≤ 20 MB empfohlen.
2. **Sicherheit:** kein `eval`, kein Nachladen von Code aus dem Internet.
3. **Minimal-Berechtigungen:** nur anfordern, was gebraucht wird.
4. **Rechte:** Bilder, Musik und Figuren nur verwenden, wenn man die Rechte hat
   (oder der Nutzer die Dateien selbst einfügt).
5. **Barrierefreiheit:** ausreichender Kontrast zwischen `--sh-fg` und `--sh-bg`
   (mind. 4.5 : 1).
6. **Übersetzbar:** Texte in `i18n/*.json` statt fest im Code.

---

## 8. Vorlagen

### 8.1 Nur-Theme-Mod (kein JavaScript)

`mod.json`
```json
{
  "id": "beispiel.neon-tokyo",
  "name": "Neon Tokyo",
  "version": "1.0.0",
  "apiVersion": 0,
  "author": "Dein Name",
  "theme": "theme.css",
  "layout": { "tabs": "top", "sidebar": "left" }
}
```

`theme.css`
```css
:root {
  --sh-bg: #0b0b1a;
  --sh-bg-elevated: #15152b;
  --sh-fg: #e8e8ff;
  --sh-accent: #00e5ff;
  --sh-glow: 0 0 10px #00e5ff, 0 0 20px #ff00c8;
}
[data-sh="tab"][data-active] { box-shadow: var(--sh-glow); }
```

### 8.2 Skript-Mod: „Gaming-Modus-Ansage“

`mod.json`
```json
{
  "id": "beispiel.gaming-announcer",
  "name": "Gaming Announcer",
  "version": "1.0.0",
  "apiVersion": 0,
  "author": "Dein Name",
  "scripts": { "background": "scripts/main.js" },
  "sounds": { "pack": "sounds/", "map": { "custom.ready": "ready.ogg" } },
  "permissions": ["system.stats", "ui.sounds"]
}
```

`scripts/main.js`
```js
shinoho.system.onGamingMode(async (active) => {
  if (active) {
    await shinoho.sounds.play("custom.ready");
    await shinoho.ui.toast("Gaming-Modus an – viel Erfolg! 🎮");
  }
});
```

---

## 9. Versionierung der API

- `apiVersion` wird bei inkompatiblen Änderungen erhöht.
- Shinoho unterstützt immer die aktuelle und die vorherige `apiVersion`.
- Änderungen stehen in `sdk/CHANGELOG.md`.
