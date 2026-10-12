//! Shinoho Control (T-201): Performance-Presets und die Entscheidung, welche Tabs
//! schlafen gelegt oder gedrosselt werden.
//!
//! Reine Logik ohne Plattform-Code: Die Messwerte (T-202) und das Einfrieren der
//! Tabs (T-203) kommen von außen, diese Crate sagt nur, *was* zu tun ist.

mod plan;
mod preset;

pub use plan::{hot_tabs, plan, Action, TabSnapshot};
pub use preset::{Limits, Preset};
