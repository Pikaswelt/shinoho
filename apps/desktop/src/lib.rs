use shinoho_control::tauri_api::{GamingState, MonitorState};
use shinoho_core::api::TabManagerState;
use shinoho_core::userstyles::UserStylesState;
use shinoho_vpn::tauri_api::VpnState;
use tauri::Manager;

mod commands;
mod services;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(TabManagerState::new())
        .manage(GamingState::default())
        .manage(MonitorState::default())
        .manage(VpnState::default())
        .manage(UserStylesState::new())
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
            shinoho_control::tauri_api::control_stats,
            shinoho_vpn::tauri_api::vpn_status,
            shinoho_vpn::tauri_api::vpn_providers,
            shinoho_vpn::tauri_api::vpn_import_config,
            shinoho_vpn::tauri_api::vpn_connect,
            shinoho_vpn::tauri_api::vpn_disconnect,
            shinoho_vpn::tauri_api::vpn_set_kill_switch,
            shinoho_core::api::add_userstyle,
            shinoho_core::api::update_userstyle,
            shinoho_core::api::toggle_userstyle,
            shinoho_core::api::remove_userstyle,
            shinoho_core::api::get_userstyles,
            shinoho_core::api::get_styles_for_url,
            shinoho_core::api::inject_styles_for_tab,
            commands::record_visit,
            commands::get_history,
            commands::search_history,
            commands::get_top_sites,
            commands::delete_history_visit,
            commands::clear_history,
            commands::clear_history_range,
            commands::create_bookmark,
            commands::get_bookmark,
            commands::update_bookmark,
            commands::delete_bookmark,
            commands::list_bookmarks,
            commands::get_bookmark_tree,
            commands::search_bookmarks,
            commands::export_bookmarks_html,
            commands::import_bookmarks_html,
            commands::get_setting,
            commands::set_setting,
            commands::get_all_settings,
            commands::delete_setting,
            commands::create_incognito_session,
            commands::close_incognito_session,
            commands::close_all_incognito_sessions,
            commands::list_incognito_sessions,
            commands::register_incognito_tab,
            commands::unregister_incognito_tab,
            commands::get_adblock_stats,
            commands::set_adblock_enabled,
            commands::whitelist_domain,
            commands::check_network_request,
            commands::get_hidden_elements_css,
            commands::add_adblock_filter_list,
            commands::start_download,
            commands::pause_download,
            commands::resume_download,
            commands::cancel_download,
            commands::retry_download,
            commands::get_download,
            commands::list_downloads,
            commands::delete_download,
            commands::clear_completed_downloads,
        ])
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let download_dir = app
                .path()
                .download_dir()
                .unwrap_or_else(|_| data_dir.join("downloads"));

            let services =
                services::AppServices::new(&data_dir.join("shinoho.sqlite3"), download_dir)?;
            services::forward_download_events(app.handle().clone(), &services.downloads);
            app.manage(services);

            shinoho_control::tauri_api::start_watcher(app.handle());
            // Proxy-Weiche vor allen WebViews starten; diese nutzen VpnState::proxy_url().
            shinoho_vpn::tauri_api::init(app.handle())?;
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
