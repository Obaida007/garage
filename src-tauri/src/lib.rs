mod commands;
mod db;
mod display;
mod error;
mod garage;
mod models;
mod printing;
mod state;

use parking_lot::Mutex;
use state::AppState;
use tauri::Manager;
use tauri_plugin_autostart::MacosLauncher;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let db_path = dir.join("garage.db");
            let conn = db::open(&db_path)?;
            app.manage(AppState {
                db: Mutex::new(conn),
                db_path,
            });
            let _ = display::ensure_waiting_window(app.handle());
            display::start_monitor_watch(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_snapshot,
            commands::create_ticket,
            commands::reprint_ticket,
            commands::reprint_last,
            commands::search_ticket,
            commands::list_recent_tickets,
            commands::call_next,
            commands::assign_ticket,
            commands::set_bay_out_of_service,
            commands::complete_bay,
            commands::complete_ticket,
            commands::cancel_ticket,
            commands::return_to_queue,
            commands::move_ticket,
            commands::get_settings,
            commands::save_settings,
            commands::list_printers,
            commands::test_print,
            commands::list_monitors,
            commands::refresh_waiting_display,
            commands::test_waiting_display,
            commands::backup_database,
            commands::restore_database,
            commands::get_report,
            commands::reset_open_queue,
            commands::set_autostart,
            commands::is_autostart_enabled,
            commands::db_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Al-Baroudi Garage");
}

#[cfg(test)]
mod tests;
