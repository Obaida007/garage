mod commands;
mod db;
mod display;
mod error;
mod garage;
mod http_server;
mod models;
mod printing;
#[cfg(not(windows))]
mod printing_cups;
mod state;
mod ticket_layout;

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
            migrate_legacy_data_dir(&dir);
            std::fs::create_dir_all(&dir)?;
            let db_path = dir.join("garage.db");
            let conn = db::open(&db_path)?;
            app.manage(AppState {
                db: Mutex::new(conn),
                db_path,
                mobile_token: Mutex::new(None),
            });
            let _ = display::ensure_waiting_window(app.handle());
            display::start_monitor_watch(app.handle().clone());
            http_server::spawn(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_snapshot,
            commands::create_ticket,
            commands::create_priority_ticket,
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
            commands::rename_bay,
            commands::add_bay,
            commands::set_bay_active,
            commands::upload_logo,
            commands::list_printers,
            commands::test_print,
            commands::list_monitors,
            commands::refresh_waiting_display,
            commands::test_waiting_display,
            commands::backup_database,
            commands::restore_database,
            commands::get_report,
            commands::reset_open_queue,
            commands::reset_numbering,
            commands::set_autostart,
            commands::is_autostart_enabled,
            commands::db_path,
            commands::get_local_ips,
            commands::get_mobile_port,
            commands::list_ads,
            commands::add_ad,
            commands::remove_ad,
            commands::update_ad_duration,
            commands::reorder_ads,
        ])
        .run(tauri::generate_context!())
        .expect("error while running OS Tickets");
}

/// بيانات الإصدارات القديمة كانت مخزنة تحت معرّفات الحزم السابقة،
/// فننسخ ملفاتها مرة واحدة إلى مجلد المعرّف الجديد حتى لا يفقد المستخدم أدواره وإعداداته.
const LEGACY_IDENTIFIERS: &[&str] = &["com.albaroudi.garage", "com.osorders.garage"];

fn migrate_legacy_data_dir(dir: &std::path::Path) {
    if dir.join("garage.db").exists() {
        return;
    }
    let Some(parent) = dir.parent() else {
        return;
    };
    for legacy_identifier in LEGACY_IDENTIFIERS {
        let legacy_dir = parent.join(legacy_identifier);
        if legacy_dir == dir || !legacy_dir.is_dir() {
            continue;
        }
        if std::fs::create_dir_all(dir).is_err() {
            return;
        }
        for name in [
            "garage.db",
            "garage.db-wal",
            "garage.db-shm",
            "logo.png",
            "logo.jpg",
            "logo.jpeg",
        ] {
            let source = legacy_dir.join(name);
            let dest = dir.join(name);
            if source.is_file() && !dest.exists() {
                let _ = std::fs::copy(&source, &dest);
            }
        }
        if dir.join("garage.db").exists() {
            break;
        }
    }
}

#[cfg(test)]
mod tests;
