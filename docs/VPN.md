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

> ⚠️ Stand: Oktober 2026 – Konditionen ändern sich. **Vor jedem Release prüfen** und diese
> Tabelle aktualisieren. Die Liste wird im Browser aus einer Datei geladen
> (`core/vpn/providers.json`), damit sie ohne neues Release aktualisiert werden kann.

| Anbieter | Datenlimit (Gratis) | Stärken | Einschränkungen | Einbindung in Shinoho |
|---|---|---|---|---|
| **Proton VPN Free** ⭐ | Unbegrenzt | Schweiz, geprüfte No-Logs-Policy, keine Werbung, Open-Source-Apps | Wenige Länder, Server-Wahl automatisch, 1 Gerät | WireGuard-Config im Proton-Konto herunterladen → importieren |
| **Windscribe Free** | 10 GB/Monat (mit E-Mail) | Viele Länder für Gratis-Plan, beliebige Geräte | Datenlimit | Prüfen, ob Config-Generator im Gratis-Plan verfügbar ist ❓ |
| **hide.me Free** | 10 GB/Monat | No-Logs, keine Werbung | Datenlimit, wenige Standorte | WireGuard-Einrichtung prüfen ❓ |
| **PrivadoVPN Free** | 10 GB/Monat | Schweiz, gute Geschwindigkeit | Datenlimit, wenige Standorte | WireGuard-Einrichtung prüfen ❓ |
| **Cloudflare WARP** | Unbegrenzt | Sehr schnell, verschlüsselt WLAN-Verkehr | **Kein echter Standortwechsel**, eher „sicheres DNS + Verschlüsselung“ | Nur als Option „Schnell & sicher“, nicht als Anonymisierung |

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
