use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Voreinstellungen aus „Shinoho Control“ (siehe docs/PLAN.md §3.3).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Preset {
    /// 🔋 Akku-Sparer
    BatterySaver,
    /// ⚖️ Ausgewogen (Standard)
    #[default]
    Balanced,
    /// 🚀 Max Power: keine Limits
    MaxPower,
    /// 🎮 Gaming-Modus: Shinoho macht sich so klein wie möglich
    Gaming,
}

/// Alle Regler von Shinoho Control. `None` heißt jeweils „kein Limit“.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Limits {
    /// Höchstens so viel RAM (MB) für alle Tabs zusammen.
    pub ram_limit_mb: Option<u32>,
    /// Höchstens so viel CPU (Prozent eines Kerns, summiert) für alle Tabs zusammen.
    pub cpu_limit_percent: Option<u32>,
    /// Bandbreite für alle Tabs zusammen (kbit/s), umgesetzt in T-204.
    pub net_limit_kbps: Option<u32>,
    /// Inaktive Tabs nach dieser Zeit schlafen legen.
    #[serde(with = "opt_secs")]
    pub sleep_after: Option<Duration>,
    /// Hintergrund-Tabs drosseln (Timer, Animationen).
    pub throttle_background: bool,
    /// Live-Wallpaper, Partikel und Mod-Animationen pausieren.
    pub pause_effects: bool,
}

impl Preset {
    pub const ALL: [Preset; 4] = [
        Preset::BatterySaver,
        Preset::Balanced,
        Preset::MaxPower,
        Preset::Gaming,
    ];

    /// Standardwerte des Presets. `total_ram_mb` ist der RAM des Geräts, damit die
    /// Limits auf einem 4-GB-Laptop anders ausfallen als auf einem 32-GB-PC.
    pub fn limits(self, total_ram_mb: u32) -> Limits {
        let share = |percent: u32| Some((total_ram_mb / 100 * percent).max(256));
        match self {
            Preset::BatterySaver => Limits {
                ram_limit_mb: share(20),
                cpu_limit_percent: Some(50),
                net_limit_kbps: None,
                sleep_after: Some(Duration::from_secs(5 * 60)),
                throttle_background: true,
                pause_effects: true,
            },
            Preset::Balanced => Limits {
                ram_limit_mb: share(40),
                cpu_limit_percent: None,
                net_limit_kbps: None,
                sleep_after: Some(Duration::from_secs(30 * 60)),
                throttle_background: true,
                pause_effects: false,
            },
            Preset::MaxPower => Limits {
                ram_limit_mb: None,
                cpu_limit_percent: None,
                net_limit_kbps: None,
                sleep_after: None,
                throttle_background: false,
                pause_effects: false,
            },
            Preset::Gaming => Limits {
                ram_limit_mb: share(10),
                cpu_limit_percent: Some(25),
                // Downloads sollen den Ping im Spiel nicht kaputt machen.
                net_limit_kbps: Some(5_000),
                sleep_after: Some(Duration::from_secs(2 * 60)),
                throttle_background: true,
                pause_effects: true,
            },
        }
    }
}

mod opt_secs {
    use std::time::Duration;

    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(v: &Option<Duration>, s: S) -> Result<S::Ok, S::Error> {
        match v {
            Some(d) => s.serialize_some(&d.as_secs()),
            None => s.serialize_none(),
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Duration>, D::Error> {
        Ok(Option::<u64>::deserialize(d)?.map(Duration::from_secs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_get_stricter_in_order() {
        let ram = 16_384;
        let max = Preset::MaxPower.limits(ram);
        let bal = Preset::Balanced.limits(ram);
        let bat = Preset::BatterySaver.limits(ram);
        let game = Preset::Gaming.limits(ram);
        assert_eq!(max.ram_limit_mb, None);
        assert!(bal.ram_limit_mb > bat.ram_limit_mb);
        assert!(bat.ram_limit_mb > game.ram_limit_mb);
        assert!(game.sleep_after < bat.sleep_after);
        assert!(game.pause_effects && bat.pause_effects && !bal.pause_effects);
    }

    #[test]
    fn small_devices_keep_a_minimum() {
        assert_eq!(Preset::Gaming.limits(1_024).ram_limit_mb, Some(256));
    }

    #[test]
    fn limits_roundtrip_as_json() {
        let l = Preset::Balanced.limits(8_192);
        let json = serde_json::to_string(&l).unwrap();
        assert!(json.contains("\"sleepAfter\":1800"));
        assert_eq!(serde_json::from_str::<Limits>(&json).unwrap(), l);
        assert_eq!(
            serde_json::to_string(&Preset::BatterySaver).unwrap(),
            "\"battery-saver\""
        );
    }
}
