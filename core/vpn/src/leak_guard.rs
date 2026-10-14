//! WebRTC-Leak-Schutz (T-404).
//!
//! WebRTC baut eigene UDP-Verbindungen auf (STUN/TURN). Die laufen **nicht** über den
//! SOCKS5-Proxy der Weiche und würden die echte IP-Adresse verraten, obwohl das VPN an ist.
//! Darum wird `RTCPeerConnection` gesperrt, solange das VPN an ist oder der Kill-Switch greift.
//!
//! Einbindung (T-111, `apps/desktop`):
//! 1. Jede WebView bekommt beim Erzeugen `.initialization_script(&webrtc_guard_script(blocked))`.
//! 2. Bei `shinoho://vpn-changed` und nach jedem Seitenladen auf allen WebViews
//!    `webview.eval(&webrtc_policy_js(blocked))` ausführen.
//!
//! Kamera und Mikrofon (`getUserMedia`) bleiben nutzbar; gesperrt wird nur der Verbindungsaufbau.

/// Skript, das vor jeder Seite läuft. `blocked` ist der Zustand beim Erzeugen der WebView.
pub fn webrtc_guard_script(blocked: bool) -> String {
    format!(
        r#"(() => {{
  if (window.__shinohoWebRtcGuard) return;
  window.__shinohoWebRtcGuard = true;
  window.__shinohoWebRtcBlocked = {blocked};
  const names = ['RTCPeerConnection', 'webkitRTCPeerConnection', 'mozRTCPeerConnection'];
  for (const name of names) {{
    const Original = window[name];
    if (typeof Original !== 'function') continue;
    const Guarded = function (...args) {{
      if (window.__shinohoWebRtcBlocked) {{
        throw new DOMException(
          'WebRTC ist gesperrt, solange das Shinoho-VPN aktiv ist (Schutz vor IP-Leaks).',
          'NotAllowedError'
        );
      }}
      return new Original(...args);
    }};
    Guarded.prototype = Original.prototype;
    Object.setPrototypeOf(Guarded, Original);
    Object.defineProperty(window, name, {{ value: Guarded, writable: false, configurable: false }});
  }}
}})();"#
    )
}

/// Schaltet die Sperre in einer bereits geladenen Seite um.
pub fn webrtc_policy_js(blocked: bool) -> String {
    format!("window.__shinohoWebRtcBlocked = {blocked};")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    /// Führt das Skript mit Node in einer nachgebauten `window`-Umgebung aus.
    fn run_in_node(body: &str) -> Option<String> {
        let program = format!(
            r#"
globalThis.window = globalThis;
globalThis.DOMException = class extends Error {{ constructor(m, n) {{ super(m); this.name = n; }} }};
globalThis.RTCPeerConnection = class {{ constructor(cfg) {{ this.cfg = cfg; }} }};
{body}
"#
        );
        let out = Command::new("node").arg("-e").arg(program).output().ok()?;
        Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
    }

    #[test]
    fn blocks_and_unblocks_peer_connections() {
        let script = format!(
            r#"
{guard}
const tryIt = () => {{ try {{ new RTCPeerConnection({{}}); return 'ok'; }} catch (e) {{ return e.name; }} }};
const a = tryIt();
{unblock}
const b = tryIt();
{block}
const c = tryIt();
{guard}
console.log([a, b, c, typeof RTCPeerConnection.prototype].join(','));
"#,
            guard = webrtc_guard_script(true),
            unblock = webrtc_policy_js(false),
            block = webrtc_policy_js(true),
        );
        let Some(out) = run_in_node(&script) else {
            eprintln!("node nicht gefunden – Test übersprungen");
            return;
        };
        assert_eq!(out, "NotAllowedError,ok,NotAllowedError,object");
    }

    #[test]
    fn page_cannot_restore_the_original() {
        let script = format!(
            r#"
{guard}
try {{ window.RTCPeerConnection = class {{}}; }} catch (e) {{}}
let r; try {{ new RTCPeerConnection({{}}); r = 'leak'; }} catch (e) {{ r = e.name; }}
console.log(r);
"#,
            guard = webrtc_guard_script(true),
        );
        let Some(out) = run_in_node(&script) else {
            return;
        };
        assert_eq!(out, "NotAllowedError");
    }
}
