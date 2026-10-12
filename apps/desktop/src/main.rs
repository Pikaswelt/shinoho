// Unter Windows im Release-Build kein zusätzliches Konsolenfenster öffnen.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("Shinoho konnte nicht gestartet werden");
}
