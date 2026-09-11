use crate::db;
use crate::display;
use crate::error::AppResult;
use crate::garage;
use crate::models::{
    AppSettings, Bay, CreateTicketResult, DailyReport, GarageSnapshot, MonitorInfo, PrinterInfo, Ticket,
};
use crate::printing;
use crate::state::AppState;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_autostart::ManagerExt;

fn emit_update(app: &AppHandle) -> AppResult<GarageSnapshot> {
    let state = app.state::<AppState>();
    let conn = state.db.lock();
    let snapshot = garage::snapshot(&conn)?;
    drop(conn);
    let _ = app.emit("garage-updated", &snapshot);
    Ok(snapshot)
}

#[tauri::command]
pub fn get_snapshot(state: State<AppState>) -> AppResult<GarageSnapshot> {
    let conn = state.db.lock();
    garage::snapshot(&conn)
}

#[tauri::command]
pub fn create_ticket(app: AppHandle) -> AppResult<CreateTicketResult> {
    let result = {
        let state = app.state::<AppState>();
        let mut conn = state.db.lock();
        garage::create_ticket_and_maybe_print(&mut conn, |ticket, settings| {
            printing::print_ticket(ticket, settings)
        })?
    };
    let _ = emit_update(&app);
    Ok(result)
}

#[tauri::command]
pub fn reprint_ticket(app: AppHandle, ticket_id: i64) -> AppResult<()> {
    let state = app.state::<AppState>();
    let conn = state.db.lock();
    let ticket = garage::get_ticket(&conn, ticket_id)?
        .ok_or_else(|| crate::error::AppError::msg("الدور غير موجود"))?;
    let settings = db::load_settings(&conn)?;
    drop(conn);
    printing::print_ticket(&ticket, &settings).map_err(crate::error::AppError::msg)?;
    let _ = emit_update(&app);
    Ok(())
}

#[tauri::command]
pub fn reprint_last(app: AppHandle) -> AppResult<()> {
    let state = app.state::<AppState>();
    let conn = state.db.lock();
    let tickets = garage::recent_tickets(&conn, 1)?;
    let ticket = tickets
        .into_iter()
        .next()
        .ok_or_else(|| crate::error::AppError::msg("لا يوجد دور لإعادة طباعته"))?;
    let settings = db::load_settings(&conn)?;
    drop(conn);
    printing::print_ticket(&ticket, &settings).map_err(crate::error::AppError::msg)
}

#[tauri::command]
pub fn search_ticket(state: State<AppState>, number: String) -> AppResult<Option<Ticket>> {
    let conn = state.db.lock();
    garage::find_ticket_by_number(&conn, &number)
}

#[tauri::command]
pub fn list_recent_tickets(state: State<AppState>, limit: i64) -> AppResult<Vec<Ticket>> {
    let conn = state.db.lock();
    garage::recent_tickets(&conn, limit.max(1).min(200))
}

#[tauri::command]
pub fn call_next(app: AppHandle, bay_id: Option<i64>) -> AppResult<Ticket> {
    let ticket = {
        let state = app.state::<AppState>();
        let conn = state.db.lock();
        garage::call_next(&conn, bay_id)?
    };
    let _ = emit_update(&app);
    Ok(ticket)
}

#[tauri::command]
pub fn assign_ticket(app: AppHandle, ticket_id: i64, bay_id: i64) -> AppResult<Ticket> {
    let ticket = {
        let state = app.state::<AppState>();
        let conn = state.db.lock();
        garage::assign_ticket_to_bay(&conn, ticket_id, bay_id)?
    };
    let _ = emit_update(&app);
    Ok(ticket)
}

#[tauri::command]
pub fn set_bay_out_of_service(app: AppHandle, bay_id: i64, out_of_service: bool) -> AppResult<Bay> {
    let bay = {
        let state = app.state::<AppState>();
        let conn = state.db.lock();
        garage::set_bay_out_of_service(&conn, bay_id, out_of_service)?
    };
    let _ = emit_update(&app);
    Ok(bay)
}

#[tauri::command]
pub fn complete_bay(app: AppHandle, bay_id: i64) -> AppResult<Ticket> {
    let ticket = {
        let state = app.state::<AppState>();
        let conn = state.db.lock();
        garage::complete_bay(&conn, bay_id)?
    };
    let _ = emit_update(&app);
    Ok(ticket)
}

#[tauri::command]
pub fn complete_ticket(app: AppHandle, ticket_id: i64) -> AppResult<Ticket> {
    let ticket = {
        let state = app.state::<AppState>();
        let conn = state.db.lock();
        garage::complete_ticket(&conn, ticket_id)?
    };
    let _ = emit_update(&app);
    Ok(ticket)
}

#[tauri::command]
pub fn cancel_ticket(app: AppHandle, ticket_id: i64) -> AppResult<Ticket> {
    let ticket = {
        let state = app.state::<AppState>();
        let conn = state.db.lock();
        garage::cancel_ticket(&conn, ticket_id)?
    };
    let _ = emit_update(&app);
    Ok(ticket)
}

#[tauri::command]
pub fn return_to_queue(app: AppHandle, ticket_id: i64) -> AppResult<Ticket> {
    let ticket = {
        let state = app.state::<AppState>();
        let conn = state.db.lock();
        garage::return_to_queue(&conn, ticket_id)?
    };
    let _ = emit_update(&app);
    Ok(ticket)
}

#[tauri::command]
pub fn move_ticket(app: AppHandle, ticket_id: i64, to_bay_id: i64) -> AppResult<Ticket> {
    let ticket = {
        let state = app.state::<AppState>();
        let conn = state.db.lock();
        garage::move_ticket(&conn, ticket_id, to_bay_id)?
    };
    let _ = emit_update(&app);
    Ok(ticket)
}

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> AppResult<AppSettings> {
    let conn = state.db.lock();
    db::load_settings(&conn)
}

#[tauri::command]
pub fn save_settings(app: AppHandle, settings: AppSettings) -> AppResult<AppSettings> {
    let saved = {
        let state = app.state::<AppState>();
        let conn = state.db.lock();
        garage::update_settings(&conn, &settings)?
    };
    let _ = display::ensure_waiting_window(&app);
    let _ = emit_update(&app);
    Ok(saved)
}

#[tauri::command]
pub fn list_printers() -> AppResult<Vec<PrinterInfo>> {
    printing::list_printers()
}

#[tauri::command]
pub fn test_print(state: State<AppState>) -> AppResult<()> {
    let conn = state.db.lock();
    let settings = db::load_settings(&conn)?;
    printing::test_print(&settings).map_err(crate::error::AppError::msg)
}

#[tauri::command]
pub fn list_monitors(app: AppHandle) -> AppResult<Vec<MonitorInfo>> {
    display::list_monitors(&app)
}

#[tauri::command]
pub fn refresh_waiting_display(app: AppHandle) -> AppResult<()> {
    display::ensure_waiting_window(&app)
}

#[tauri::command]
pub fn test_waiting_display(app: AppHandle) -> AppResult<()> {
    display::test_waiting_window(&app)
}

#[tauri::command]
pub fn backup_database(state: State<AppState>, dest: String) -> AppResult<String> {
    let dest_path = PathBuf::from(&dest);
    {
        let conn = state.db.lock();
        conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
    }
    db::backup_file(&state.db_path, &dest_path)?;
    Ok(dest_path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn restore_database(app: AppHandle, source: String) -> AppResult<()> {
    let source_path = PathBuf::from(source);
    {
        let state = app.state::<AppState>();
        {
            let mut conn = state.db.lock();
            let placeholder = rusqlite::Connection::open_in_memory()?;
            let old = std::mem::replace(&mut *conn, placeholder);
            drop(old);
            db::restore_file(&source_path, &state.db_path)?;
            *conn = db::open(&state.db_path)?;
        }
    }
    let _ = emit_update(&app);
    Ok(())
}

#[tauri::command]
pub fn get_report(state: State<AppState>, date: Option<String>) -> AppResult<DailyReport> {
    let conn = state.db.lock();
    match date {
        Some(d) if !d.is_empty() => garage::daily_report(&conn, &d),
        _ => garage::report_today(&conn),
    }
}

#[tauri::command]
pub fn reset_open_queue(app: AppHandle) -> AppResult<()> {
    {
        let state = app.state::<AppState>();
        let conn = state.db.lock();
        garage::reset_open_queue(&conn)?;
    }
    let _ = emit_update(&app);
    Ok(())
}

#[tauri::command]
pub fn set_autostart(app: AppHandle, enabled: bool) -> AppResult<bool> {
    let manager = app.autolaunch();
    if enabled {
        manager
            .enable()
            .map_err(|e| crate::error::AppError::msg(e.to_string()))?;
    } else {
        manager
            .disable()
            .map_err(|e| crate::error::AppError::msg(e.to_string()))?;
    }
    Ok(enabled)
}

#[tauri::command]
pub fn is_autostart_enabled(app: AppHandle) -> AppResult<bool> {
    app.autolaunch()
        .is_enabled()
        .map_err(|e| crate::error::AppError::msg(e.to_string()))
}

#[tauri::command]
pub fn db_path(state: State<AppState>) -> AppResult<String> {
    Ok(state.db_path.to_string_lossy().to_string())
}
