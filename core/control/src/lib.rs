//! Shinoho Control (T-201): Performance-Presets und die Entscheidung, welche Tabs
//! schlafen gelegt oder gedrosselt werden.
//!
//! Die Regeln sind reine Logik: Die Messwerte (T-202) und das Einfrieren der Tabs
//! (T-203) kommen von außen, diese Crate sagt nur, *was* zu tun ist. Einzige Ausnahme
//! ist die Spiele-Erkennung für den Gaming-Modus (T-205), die die Prozessliste liest.

pub mod gaming;
pub mod monitor;
mod plan;
mod preset;
#[cfg(feature = "tauri")]
pub mod tauri_api;

pub use gaming::{GameRules, GameWatcher, GamingEvent};
pub use monitor::{ProcessStats, ResourceMonitor, ResourceStats};
pub use plan::{hot_tabs, plan, Action, TabSnapshot};
pub use preset::{Limits, Preset};
