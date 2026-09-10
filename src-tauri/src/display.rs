use crate::error::AppResult;
use crate::models::MonitorInfo;
use std::time::Duration;
use tauri::{
    AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindowBuilder,
};

const WAITING_LABEL: &str = "waiting";

pub fn list_monitors(app: &AppHandle) -> AppResult<Vec<MonitorInfo>> {
    let primary = app.primary_monitor().ok().flatten();
    let primary_pos = primary.as_ref().map(|m| m.position());
    let mut list = Vec::new();
    for monitor in app.available_monitors().unwrap_or_default() {
        let position = monitor.position();
        let size = monitor.size();
        let name = monitor.name().map(|n| n.to_string()).unwrap_or_else(|| "Display".into());
        let is_primary = primary_pos
            .map(|p| p.x == position.x && p.y == position.y)
            .unwrap_or(false);
        list.push(MonitorInfo {
            id: monitor_id(&position, &size),
            name,
            is_primary,
            x: position.x,
            y: position.y,
            width: size.width,
            height: size.height,
        });
    }
    list.sort_by_key(|m| (m.is_primary as i8) * -1);
    Ok(list)
}

fn monitor_id(position: &PhysicalPosition<i32>, size: &PhysicalSize<u32>) -> String {
    format!("{}x{}@{},{}", size.width, size.height, position.x, position.y)
}

pub fn ensure_waiting_window(app: &AppHandle) -> AppResult<()> {
    let settings = {
        let state = app.state::<crate::state::AppState>();
        let conn = state.db.lock();
        crate::db::load_settings(&conn)?
    };
    let monitors = list_monitors(app)?;
    let secondary = pick_waiting_monitor(&monitors, &settings.waiting_monitor_id);

    match secondary {
        Some(monitor) => show_waiting_on(app, &monitor, settings.waiting_fullscreen),
        None => {
            if let Some(window) = app.get_webview_window(WAITING_LABEL) {
                let _ = window.close();
            }
            Ok(())
        }
    }
}

fn pick_waiting_monitor<'a>(
    monitors: &'a [MonitorInfo],
    preferred_id: &str,
) -> Option<&'a MonitorInfo> {
    if !preferred_id.is_empty() {
        if let Some(found) = monitors.iter().find(|m| m.id == preferred_id && !m.is_primary) {
            return Some(found);
        }
        if let Some(found) = monitors.iter().find(|m| m.id == preferred_id) {
            if monitors.len() > 1 && !found.is_primary {
                return Some(found);
            }
        }
    }
    monitors.iter().find(|m| !m.is_primary)
}

fn waiting_url() -> WebviewUrl {
    #[cfg(debug_assertions)]
    {
        WebviewUrl::External("http://localhost:1420/#/waiting".parse().expect("dev url"))
    }
    #[cfg(not(debug_assertions))]
    {
        WebviewUrl::App("index.html#/waiting".into())
    }
}

fn show_waiting_on(app: &AppHandle, monitor: &MonitorInfo, fullscreen: bool) -> AppResult<()> {
    let window = if let Some(existing) = app.get_webview_window(WAITING_LABEL) {
        existing
    } else {
        WebviewWindowBuilder::new(app, WAITING_LABEL, waiting_url())
            .title("Al-Sahil Waiting")
            .decorations(false)
            .skip_taskbar(true)
            .visible(true)
            .focused(false)
            .resizable(false)
            .build()
            .map_err(|e| crate::error::AppError::msg(format!("تعذر فتح شاشة الانتظار: {e}")))?
    };

    let _ = window.set_position(PhysicalPosition {
        x: monitor.x,
        y: monitor.y,
    });
    let _ = window.set_size(PhysicalSize {
        width: monitor.width,
        height: monitor.height,
    });
    let _ = window.set_fullscreen(fullscreen);
    let _ = window.show();
    Ok(())
}

pub fn start_monitor_watch(app: AppHandle) {
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(3));
        if let Err(err) = ensure_waiting_window(&app) {
            let _ = app.emit("display-error", err.to_string());
        }
    });
}

pub fn test_waiting_window(app: &AppHandle) -> AppResult<()> {
    ensure_waiting_window(app)?;
    if app.get_webview_window(WAITING_LABEL).is_none() {
        // Force on primary if no second display, so cashier can preview.
        let monitors = list_monitors(app)?;
        if let Some(primary) = monitors.iter().find(|m| m.is_primary).or(monitors.first()) {
            show_waiting_on(app, primary, true)?;
        }
    }
    Ok(())
}
