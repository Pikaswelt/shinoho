use shinoho_core::api::TabManagerState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(TabManagerState::new())
        .invoke_handler(tauri::generate_handler![
            shinoho_core::api::create_tab,
            shinoho_core::api::close_tab,
            shinoho_core::api::activate_tab,
            shinoho_core::api::navigate_tab,
            shinoho_core::api::go_back,
            shinoho_core::api::go_forward,
            shinoho_core::api::reload_tab,
            shinoho_core::api::get_tabs,
        ])
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Fehler beim Starten der Shinoho-Tauri-Anwendung");
}
