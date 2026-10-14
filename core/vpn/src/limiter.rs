//! Netzwerk-Limiter (T-204): begrenzt die Bandbreite aller Tabs zusammen.
//!
//! Sitzt in der Proxy-Weiche, durch die jeder Tab-Verkehr läuft. Ein gemeinsamer
//! Token-Bucket gilt für Download und Upload aller Verbindungen. Damit lässt der
//! Gaming-Modus (5 Mbit/s) dem Spiel genug Leitung, auch wenn im Browser Downloads laufen.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// Gemeinsamer Bandbreiten-Begrenzer. `0` = unbegrenzt. Zur Laufzeit änderbar.
#[derive(Clone, Default)]
pub struct RateLimiter {
    inner: Arc<Inner>,
}

#[derive(Default)]
struct Inner {
    /// Grenze in Bytes pro Sekunde (0 = aus).
    bytes_per_sec: AtomicU64,
    bucket: Mutex<Bucket>,
}

struct Bucket {
    tokens: f64,
    last: Instant,
}

impl Default for Bucket {
    fn default() -> Self {
        Self {
            tokens: 0.0,
            last: Instant::now(),
        }
    }
}

/// Größte Stückelung, damit die Begrenzung auch bei großen Puffern gleichmäßig wirkt.
const CHUNK: usize = 16 * 1024;

impl RateLimiter {
    pub fn unlimited() -> Self {
        Self::default()
    }

    /// Setzt die Grenze in kbit/s (`None` oder 0 = unbegrenzt), passend zu `Limits::net_limit_kbps`.
    pub fn set_limit_kbps(&self, kbps: Option<u32>) {
        let bytes = u64::from(kbps.unwrap_or(0)) * 1000 / 8;
        self.inner.bytes_per_sec.store(bytes, Ordering::Relaxed);
    }

    pub fn limit_kbps(&self) -> Option<u32> {
        match self.inner.bytes_per_sec.load(Ordering::Relaxed) {
            0 => None,
            b => Some((b * 8 / 1000) as u32),
        }
    }

    /// Wartet, bis `n` Bytes gesendet werden dürfen.
    pub async fn acquire(&self, n: usize) {
        loop {
            let rate = self.inner.bytes_per_sec.load(Ordering::Relaxed);
            if rate == 0 {
                return;
            }
            let wait = {
                let mut b = self.inner.bucket.lock().unwrap_or_else(|e| e.into_inner());
                let now = Instant::now();
                let elapsed = now.duration_since(b.last).as_secs_f64();
                b.last = now;
                // Höchstens 1/4 Sekunde ansparen, damit nach Pausen kein großer Schwall kommt.
                let cap = rate as f64 / 4.0;
                b.tokens = (b.tokens + elapsed * rate as f64).min(cap.max(CHUNK as f64));
                if b.tokens >= n as f64 {
                    b.tokens -= n as f64;
                    return;
                }
                Duration::from_secs_f64((n as f64 - b.tokens) / rate as f64)
            };
            tokio::time::sleep(wait.min(Duration::from_millis(250))).await;
        }
    }

    /// Kopiert von `r` nach `w` und hält dabei die Grenze ein.
    pub async fn copy<R, W>(&self, r: &mut R, w: &mut W) -> std::io::Result<u64>
    where
        R: AsyncRead + Unpin,
        W: AsyncWrite + Unpin,
    {
        let mut buf = vec![0u8; CHUNK];
        let mut total = 0u64;
        loop {
            let n = r.read(&mut buf).await?;
            if n == 0 {
                w.shutdown().await?;
                return Ok(total);
            }
            self.acquire(n).await;
            w.write_all(&buf[..n]).await?;
            total += n as u64;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn unlimited_does_not_wait() {
        let l = RateLimiter::unlimited();
        let start = Instant::now();
        for _ in 0..100 {
            l.acquire(1_000_000).await;
        }
        assert!(start.elapsed() < Duration::from_millis(50));
        assert_eq!(l.limit_kbps(), None);
    }

    #[tokio::test]
    async fn limit_slows_down_transfers() {
        let l = RateLimiter::unlimited();
        l.set_limit_kbps(Some(800)); // 100 kB/s
        assert_eq!(l.limit_kbps(), Some(800));
        let start = Instant::now();
        let data = vec![7u8; 60_000];
        let (mut a, mut b) = tokio::io::duplex(1 << 20);
        let mut src = &data[..];
        l.copy(&mut src, &mut a).await.unwrap();
        let mut got = Vec::new();
        b.read_to_end(&mut got).await.unwrap();
        assert_eq!(got.len(), 60_000);
        let secs = start.elapsed().as_secs_f64();
        // 60 kB bei 100 kB/s ≈ 0,6 s (abzüglich kleinem Startguthaben).
        assert!(secs > 0.35, "zu schnell: {secs:.2}s");
        assert!(secs < 1.5, "zu langsam: {secs:.2}s");
    }

    #[tokio::test]
    async fn limit_can_be_lifted_at_runtime() {
        let l = RateLimiter::unlimited();
        l.set_limit_kbps(Some(8)); // 1 kB/s – sehr langsam
        let l2 = l.clone();
        let waiter = tokio::spawn(async move { l2.acquire(100_000).await });
        tokio::time::sleep(Duration::from_millis(100)).await;
        l.set_limit_kbps(None);
        tokio::time::timeout(Duration::from_secs(1), waiter)
            .await
            .expect("Aufheben der Grenze muss wartende Übertragungen freigeben")
            .unwrap();
    }
}
