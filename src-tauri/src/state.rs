use crate::db;
use parking_lot::Mutex;
use rusqlite::Connection;
use std::path::PathBuf;

pub struct AppState {
    pub db: Mutex<Connection>,
    pub db_path: PathBuf,
}

pub fn reopen(state: &AppState) -> crate::error::AppResult<()> {
    let conn = db::open(&state.db_path)?;
    *state.db.lock() = conn;
    Ok(())
}
