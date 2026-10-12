# Shinoho VPN

## 1. Wie das VPN funktionieren soll

```
Tab ──► Shinoho-Kern ──► lokaler Proxy (127.0.0.1) ──► WireGuard-Tunnel ──► VPN-Server ──► Internet
```

- **Browser-VPN**: Nur der Verkehr von Shinoho läuft durch den Tunnel – Spiele, Discord usw.
  bleiben unberührt (wichtig für niedrigen Ping beim Zocken!).
- Technik: WireGuard in Rust (`boringtun`) im Userspace → keine Admin-Rechte auf Desktop nötig.
- Android: eigener `VpnService` nur für die Shinoho-App (per-App-VPN).
- iOS: über Apples `NetworkExtension` (braucht Apple-Developer-Konto und Berechtigung) ❓

### Funktionen
- Ein/Aus-Schalter in der Adressleiste (`VPN`-Badge wie bei Opera)
- Standort-Auswahl (je nach Anbieter)
- **Kill-Switch**: fällt der Tunnel aus, werden Tabs blockiert statt ungeschützt zu laden
- DNS über den Tunnel (keine DNS-Leaks), WebRTC-Leak-Schutz
- Eingebauter **Leak-Test** (`shinoho://vpn-test`)
- Regeln: „VPN immer im Inkognito“, „VPN nur für bestimmte Seiten“, „nie für Seite X“
- Profile: mehrere Anbieter/Configs speichern, schnell wechseln

## 2. Warum kein „eigenes Gratis-VPN“ wie Opera?

Opera betreibt eigene Server – das kostet viel Geld (Server + Traffic). Für den Start
**betreibt Shinoho keine eigenen Server**. Stattdessen:

1. **Anbieter-Assistent**: Shinoho schlägt seriöse Anbieter mit Gratis-Plan vor und erklärt
   Schritt für Schritt, wie man die WireGuard-Konfiguration holt.
2. **Bring your own config**: Jede WireGuard-`.conf`-Datei kann importiert werden
   (auch vom eigenen Router/Server).
3. Später optional: offizielle Partnerschaften mit Anbietern (API-Anbindung) ❓

## 3. Empfohlene kostenlose Anbieter

> ⚠️ Geprüft am **12.10.2026** (T-405). Konditionen ändern sich – **vor jedem Release erneut
> prüfen**. Maßgeblich ist `core/vpn/providers.json`; dort stehen pro Anbieter auch die Quellen.
> Einträge mit „unbestätigt“ ließen sich nicht aus einer offiziellen Quelle belegen.

| Anbieter | Datenlimit (Gratis) | Stärken | Einschränkungen | In Shinoho importierbar? |
|---|---|---|---|---|
| **Proton VPN Free** ⭐ | Unbegrenzt | Schweiz, unabhängig geprüfte No-Logs-Policy, keine Werbung | 1 Gerät, rund 10 Länder, Standort wird zugewiesen | ✅ **Ja** – auf account.protonvpn.com → Downloads → WireGuard eine `.conf` erzeugen und importieren ([Anleitung](https://protonvpn.com/support/wireguard-configurations/)) |
| **Windscribe Free** | 10 GB/Monat mit bestätigter E-Mail, sonst 2 GB | 10 Länder | WireGuard-Dateien nur für zahlende Kunden ([Quelle](https://windscribe.com/knowledge-base/articles/where-do-i-access-my-wireguard-configs)) | ❌ Nur mit eigener App |
| **hide.me Free** | 10 GB/Monat | Keine Werbung | 1 Gerät, wenige Standorte, laut Test keine manuellen Konfig-Dateien im Gratis-Plan | ❌ Nur mit eigener App (unbestätigt) |
| **PrivadoVPN Free** | 10 GB alle 30 Tage | Schweiz, 13 Standorte | Keine unabhängige Prüfung; was nach 10 GB passiert, ist widersprüchlich; Konfig-Dateien gelten 30 Tage | ❓ Download unter „Manual Configuration“ existiert, ob mit Gratis-Konto: unbestätigt |
| **Cloudflare WARP** | Unbegrenzt | Sehr schnell, verschlüsselt den Verkehr | **Kein Standortwechsel**; keine offizielle WireGuard-Datei, Drittwerkzeuge (wgcf) sind eine Grauzone | ❌ Nur mit der offiziellen 1.1.1.1-App |

**Fazit für den Anbieter-Assistenten:** Zurzeit ist **nur Proton VPN Free** direkt in Shinoho
nutzbar. Die anderen zeigt der Assistent als Alternativen mit Hinweis „nur mit eigener App“.
Für Gamer wichtig: Proton Free wählt den Server selbst, der Ping kann also schwanken.

**Nicht empfehlen / blockieren:** Gratis-VPNs ohne nachvollziehbare No-Logs-Policy, die
Daten verkaufen oder Nutzer-Bandbreite weiterverkaufen (z. B. Peer-to-Peer-„VPNs“).

### Bewertungskriterien für die Liste
1. Unabhängig geprüfte No-Logs-Policy
2. Gratis-Plan ohne Werbung/Datenverkauf
3. WireGuard-Unterstützung (oder offizielle API)
4. Firmensitz mit guten Datenschutzgesetzen
5. Gute Ping-Werte (wichtig für Gamer)

## 4. Optional später
- **Tor-Modus** für Inkognito-Fenster (wie Brave) ❓
- Eigene Shinoho-Server, falls das Projekt wächst
