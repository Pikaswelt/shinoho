//! Messung von RAM und CPU (T-202).
//!
//! Gemessen wird der **ganze Shinoho-Prozessbaum**: der Hauptprozess und alle Kindprozesse,
//! also auch die WebView-Prozesse (WebKitGTK-Webprozesse unter Linux, `msedgewebview2` unter
//! Windows, WebKit-Prozesse unter macOS). So stimmt die Summe auf allen Plattformen, auch wenn
//! sich einzelne Prozesse nicht eindeutig einem Tab zuordnen lassen.

use std::collections::{HashMap, HashSet};

use serde::Serialize;
use sysinfo::{MemoryRefreshKind, Pid, ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System};

/// Ein Prozess aus dem Shinoho-Baum.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessStats {
    pub pid: u32,
    pub name: String,
    pub ram_mb: u32,
    /// Prozent eines CPU-Kerns (kann über 100 liegen).
    pub cpu_percent: f32,
}

/// Momentaufnahme für Shinoho Control.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceStats {
    /// RAM aller Shinoho-Prozesse zusammen.
    pub total_ram_mb: u32,
    /// CPU aller Shinoho-Prozesse zusammen (Prozent eines Kerns).
    pub total_cpu_percent: f32,
    /// Gesamter RAM des Geräts (für die Presets).
    pub device_ram_mb: u32,
    /// Alle Prozesse, größte zuerst.
    pub processes: Vec<ProcessStats>,
}

/// Misst wiederholt. CPU-Werte brauchen zwei Messungen; die erste liefert 0 %.
pub struct ResourceMonitor {
    system: System,
    root: Pid,
}

impl ResourceMonitor {
    /// Misst den Baum unterhalb des eigenen Prozesses.
    pub fn new() -> Self {
        Self::for_root(Pid::from_u32(std::process::id()))
    }

    /// Misst den Baum unterhalb von `root` (für Tests).
    pub fn for_root(root: Pid) -> Self {
        let system = System::new_with_specifics(
            RefreshKind::nothing().with_memory(MemoryRefreshKind::nothing().with_ram()),
        );
        Self { system, root }
    }

    pub fn sample(&mut self) -> ResourceStats {
        self.system.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing().with_memory().with_cpu(),
        );
        let procs = self.system.processes();

        // Kinder je Elternprozess, dann Baum ab `root` einsammeln.
        let mut children: HashMap<Pid, Vec<Pid>> = HashMap::new();
        for (pid, p) in procs {
            if let Some(parent) = p.parent() {
                children.entry(parent).or_default().push(*pid);
            }
        }
        let mut tree = Vec::new();
        let mut seen = HashSet::new();
        let mut stack = vec![self.root];
        while let Some(pid) = stack.pop() {
            if !seen.insert(pid) {
                continue;
            }
            if let Some(p) = procs.get(&pid) {
                tree.push(ProcessStats {
                    pid: pid.as_u32(),
                    name: p.name().to_string_lossy().into_owned(),
                    ram_mb: (p.memory() / (1024 * 1024)) as u32,
                    cpu_percent: p.cpu_usage(),
                });
            }
            if let Some(kids) = children.get(&pid) {
                stack.extend(kids);
            }
        }
        tree.sort_by(|a, b| b.ram_mb.cmp(&a.ram_mb).then(a.pid.cmp(&b.pid)));

        ResourceStats {
            total_ram_mb: tree.iter().map(|p| p.ram_mb).sum(),
            total_cpu_percent: tree.iter().map(|p| p.cpu_percent).sum(),
            device_ram_mb: (self.system.total_memory() / (1024 * 1024)) as u32,
            processes: tree,
        }
    }
}

impl Default for ResourceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    #[test]
    fn measures_own_process() {
        let mut m = ResourceMonitor::new();
        let s = m.sample();
        assert!(s.device_ram_mb > 0);
        assert!(s.total_ram_mb > 0, "eigener Prozess braucht RAM");
        assert_eq!(
            s.processes[0].ram_mb,
            s.processes.iter().map(|p| p.ram_mb).max().unwrap()
        );
        assert!(s.processes.iter().any(|p| p.pid == std::process::id()));
    }

    #[test]
    fn includes_child_processes() {
        // Ein Kindprozess (wie ein WebView-Prozess) muss mitgezählt werden.
        let Ok(mut child) = Command::new("sleep").arg("5").spawn() else {
            return; // kein `sleep` (z. B. Windows) – Test überspringen
        };
        let mut m = ResourceMonitor::new();
        let s = m.sample();
        let found = s.processes.iter().any(|p| p.pid == child.id());
        let _ = child.kill();
        let _ = child.wait();
        assert!(found, "Kindprozess fehlt in der Messung");
    }

    #[test]
    fn unknown_root_gives_empty_tree() {
        let mut m = ResourceMonitor::for_root(Pid::from_u32(u32::MAX - 7));
        let s = m.sample();
        assert!(s.processes.is_empty());
        assert_eq!(s.total_ram_mb, 0);
    }
}
