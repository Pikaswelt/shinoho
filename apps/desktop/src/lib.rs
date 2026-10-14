use shinoho_control::tauri_api::GamingState;
use shinoho_core::api::TabManagerState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(TabManagerState::new())
        .manage(GamingState::default())
        .invoke_handler(tauri::generate_handler![
            shinoho_core::api::create_tab,
            shinoho_core::api::close_tab,
            shinoho_core::api::activate_tab,
            shinoho_core::api::navigate_tab,
            shinoho_core::api::go_back,
            shinoho_core::api::go_forward,
            shinoho_core::api::reload_tab,
            shinoho_core::api::get_tabs,
            shinoho_core::api::get_active_tab,
            shinoho_core::api::move_tab,
            shinoho_core::api::set_pinned,
            shinoho_core::api::set_muted,
            shinoho_core::api::duplicate_tab,
            shinoho_core::api::close_other_tabs,
            shinoho_core::api::create_tab_group,
            shinoho_core::api::get_tab_groups,
            shinoho_core::api::assign_tab_to_group,
            shinoho_core::api::update_tab_group,
            shinoho_control::tauri_api::gaming_status,
            shinoho_control::tauri_api::gaming_set_auto,
            shinoho_control::tauri_api::gaming_set_rules,
        ])
        .setup(|app| {
            shinoho_control::tauri_api::start_watcher(app.handle());
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
