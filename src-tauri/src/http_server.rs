use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tauri::{AppHandle, Emitter, Manager};

use crate::{db, garage, printing, state::AppState};

pub const PORT: u16 = 7878;
const MOBILE_HTML: &str = include_str!("mobile.html");

#[derive(Clone)]
struct S(AppHandle);

#[derive(Deserialize)]
struct AuthBody {
    password: String,
}

#[derive(Serialize)]
struct AuthResponse {
    token: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CallNextBody {
    bay_id: Option<i64>,
}

pub fn spawn(handle: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let app = Router::new()
            .route("/", get(serve_html))
            .route("/api/snapshot", get(api_snapshot))
            .route("/api/ticket", post(api_create_ticket))
            .route("/api/auth", post(api_auth))
            .route("/api/ticket-priority", post(api_create_priority_ticket))
            .route("/api/call-next", post(api_call_next))
            .route("/api/complete-bay/:bay_id", post(api_complete_bay))
            .route("/api/cancel/:ticket_id", post(api_cancel))
            .with_state(S(handle));

        let addr = SocketAddr::from(([0, 0, 0, 0], PORT));
        let Ok(listener) = tokio::net::TcpListener::bind(addr).await else {
            eprintln!("[mobile] failed to bind port {PORT}");
            return;
        };
        eprintln!("[mobile] listening on port {PORT}");
        let _ = axum::serve(listener, app).await;
    });
}

// ─── helpers ─────────────────────────────────────────────────────────────────

fn is_authorized(headers: &HeaderMap, handle: &AppHandle) -> bool {
    let state = handle.state::<AppState>();
    let conn = state.db.lock();
    let Ok(settings) = db::load_settings(&conn) else { return false; };
    drop(conn);

    if settings.settings_password.is_empty() {
        return true;
    }

    let auth = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let bearer = auth.trim_start_matches("Bearer ").trim();
    let stored = state.mobile_token.lock();
    stored.as_deref() == Some(bearer)
}

fn new_token() -> String {
    let t = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{:032x}", t.wrapping_mul(0x6c62272e07bb0142_u128))
}

fn emit_update(handle: &AppHandle) {
    let state = handle.state::<AppState>();
    let conn = state.db.lock();
    if let Ok(snap) = garage::snapshot(&conn) {
        drop(conn);
        let _ = handle.emit("garage-updated", &snap);
    }
}

// ─── routes ──────────────────────────────────────────────────────────────────

async fn serve_html() -> Html<&'static str> {
    Html(MOBILE_HTML)
}

async fn api_snapshot(State(S(handle)): State<S>) -> impl IntoResponse {
    let state = handle.state::<AppState>();
    let conn = state.db.lock();
    match garage::snapshot(&conn) {
        Ok(snap) => Json(snap).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn api_create_ticket(State(S(handle)): State<S>) -> impl IntoResponse {
    let h = handle.clone();
    let result = tokio::task::block_in_place(move || {
        let state = h.state::<AppState>();
        let mut conn = state.db.lock();
        let r = garage::create_ticket_and_maybe_print(&mut conn, |ticket, settings| {
            printing::print_ticket(ticket, settings)
        });
        if r.is_ok() {
            if let Ok(snap) = garage::snapshot(&conn) {
                drop(conn);
                let _ = h.emit("garage-updated", &snap);
            }
        }
        r
    });

    match result {
        Ok(r) => Json(r).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn api_auth(
    State(S(handle)): State<S>,
    Json(body): Json<AuthBody>,
) -> impl IntoResponse {
    let state = handle.state::<AppState>();
    let conn = state.db.lock();
    let Ok(settings) = db::load_settings(&conn) else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    drop(conn);

    if settings.settings_password.is_empty() {
        return StatusCode::NO_CONTENT.into_response();
    }

    if body.password != settings.settings_password {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let token = new_token();
    *state.mobile_token.lock() = Some(token.clone());
    Json(AuthResponse { token }).into_response()
}

async fn api_create_priority_ticket(State(S(handle)): State<S>) -> impl IntoResponse {
    let h = handle.clone();
    let result = tokio::task::block_in_place(move || {
        let state = h.state::<AppState>();
        let mut conn = state.db.lock();
        let r = garage::create_priority_ticket_and_maybe_print(&mut conn, |ticket, settings| {
            printing::print_ticket(ticket, settings)
        });
        if r.is_ok() {
            if let Ok(snap) = garage::snapshot(&conn) {
                drop(conn);
                let _ = h.emit("garage-updated", &snap);
            }
        }
        r
    });

    match result {
        Ok(r) => Json(r).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn api_call_next(
    State(S(handle)): State<S>,
    headers: HeaderMap,
    body: Option<Json<CallNextBody>>,
) -> impl IntoResponse {
    if !is_authorized(&headers, &handle) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let bay_id = body.and_then(|Json(b)| b.bay_id);
    let state = handle.state::<AppState>();
    let conn = state.db.lock();
    match garage::call_next(&conn, bay_id) {
        Ok(t) => {
            drop(conn);
            emit_update(&handle);
            Json(t).into_response()
        }
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}

async fn api_complete_bay(
    State(S(handle)): State<S>,
    headers: HeaderMap,
    Path(bay_id): Path<i64>,
) -> impl IntoResponse {
    if !is_authorized(&headers, &handle) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let state = handle.state::<AppState>();
    let conn = state.db.lock();
    match garage::complete_bay(&conn, bay_id) {
        Ok(t) => {
            drop(conn);
            emit_update(&handle);
            Json(t).into_response()
        }
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}

async fn api_cancel(
    State(S(handle)): State<S>,
    headers: HeaderMap,
    Path(ticket_id): Path<i64>,
) -> impl IntoResponse {
    if !is_authorized(&headers, &handle) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let state = handle.state::<AppState>();
    let conn = state.db.lock();
    match garage::cancel_ticket(&conn, ticket_id) {
        Ok(t) => {
            drop(conn);
            emit_update(&handle);
            Json(t).into_response()
        }
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}

// ─── IP helpers (used by the get_local_ips Tauri command) ────────────────────

pub fn local_ips() -> Vec<String> {
    if_addrs::get_if_addrs()
        .unwrap_or_default()
        .into_iter()
        .filter(|i| !i.is_loopback())
        .filter_map(|i| match i.addr {
            if_addrs::IfAddr::V4(ref a) => Some(a.ip.to_string()),
            _ => None,
        })
        .collect()
}
