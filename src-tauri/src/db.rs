use crate::error::{AppError, AppResult};
use crate::models::{AppSettings, BAY_COUNT};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;

pub fn open(path: &Path) -> AppResult<Connection> {
    let conn = Connection::open(path)?;
    configure(&conn)?;
    migrate(&conn)?;
    Ok(conn)
}

pub fn open_memory() -> AppResult<Connection> {
    let conn = Connection::open_in_memory()?;
    configure(&conn)?;
    migrate(&conn)?;
    Ok(conn)
}

fn configure(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        "
        PRAGMA foreign_keys = ON;
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = FULL;
        PRAGMA busy_timeout = 5000;
        ",
    )?;
    Ok(())
}

fn migrate(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS tickets (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            ticket_number TEXT NOT NULL UNIQUE,
            sequence INTEGER NOT NULL UNIQUE,
            status TEXT NOT NULL CHECK(status IN ('WAITING','IN_SERVICE','COMPLETED','CANCELLED')),
            bay_id INTEGER,
            created_at TEXT NOT NULL,
            started_at TEXT,
            completed_at TEXT,
            cancelled_at TEXT
        );

        CREATE TABLE IF NOT EXISTS bays (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            status TEXT NOT NULL CHECK(status IN ('READY','BUSY','OUT_OF_SERVICE')),
            current_ticket_id INTEGER,
            FOREIGN KEY(current_ticket_id) REFERENCES tickets(id)
        );

        CREATE INDEX IF NOT EXISTS idx_tickets_status ON tickets(status);
        CREATE INDEX IF NOT EXISTS idx_tickets_created ON tickets(created_at);
        "#,
    )?;

    // Migrate old bays table if it had restrictive check constraint
    let sql = "SELECT sql FROM sqlite_master WHERE type='table' AND name='bays'";
    if let Ok(table_sql) = conn.query_row(sql, [], |row| row.get::<_, String>(0)) {
        if !table_sql.contains("OUT_OF_SERVICE") {
            let _ = conn.execute_batch(
                r#"
                CREATE TABLE bays_temp (
                    id INTEGER PRIMARY KEY,
                    name TEXT NOT NULL,
                    status TEXT NOT NULL CHECK(status IN ('READY','BUSY','OUT_OF_SERVICE')),
                    current_ticket_id INTEGER,
                    FOREIGN KEY(current_ticket_id) REFERENCES tickets(id)
                );
                INSERT INTO bays_temp SELECT id, name, status, current_ticket_id FROM bays;
                DROP TABLE bays;
                ALTER TABLE bays_temp RENAME TO bays;
                "#,
            );
        }
    }

    seed_bays(conn)?;
    seed_settings(conn)?;
    Ok(())
}

fn seed_bays(conn: &Connection) -> AppResult<()> {
    for id in 1..=BAY_COUNT {
        conn.execute(
            "INSERT OR IGNORE INTO bays (id, name, status, current_ticket_id) VALUES (?1, ?2, 'READY', NULL)",
            params![id, format!("حفرة {id}")],
        )?;
    }
    Ok(())
}

fn seed_settings(conn: &Connection) -> AppResult<()> {
    let defaults = AppSettings::default();
    set_if_missing(conn, "garage_name", &defaults.garage_name)?;
    set_if_missing(conn, "print_header", &defaults.print_header)?;
    set_if_missing(conn, "ticket_prefix", &defaults.ticket_prefix)?;
    set_if_missing(conn, "next_sequence", &defaults.next_sequence.to_string())?;
    set_if_missing(conn, "printer_name", &defaults.printer_name)?;
    set_if_missing(conn, "paper_width_mm", &defaults.paper_width_mm.to_string())?;
    set_if_missing(conn, "waiting_monitor_id", &defaults.waiting_monitor_id)?;
    set_if_missing(conn, "waiting_fullscreen", "true")?;
    set_if_missing(conn, "last_called_ticket_id", "")?;
    set_if_missing(conn, "auto_assign", "true")?;
    Ok(())
}

fn set_if_missing(conn: &Connection, key: &str, value: &str) -> AppResult<()> {
    conn.execute(
        "INSERT OR IGNORE INTO settings (key, value) VALUES (?1, ?2)",
        params![key, value],
    )?;
    Ok(())
}

pub fn get_setting(conn: &Connection, key: &str) -> AppResult<Option<String>> {
    let value = conn
        .query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |row| row.get::<_, String>(0),
        )
        .optional()?;
    Ok(value)
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> AppResult<()> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

pub fn load_settings(conn: &Connection) -> AppResult<AppSettings> {
    let mut settings = AppSettings::default();
    if let Some(v) = get_setting(conn, "garage_name")? {
        settings.garage_name = v;
    }
    if let Some(v) = get_setting(conn, "print_header")? {
        settings.print_header = v;
    }
    if let Some(v) = get_setting(conn, "ticket_prefix")? {
        settings.ticket_prefix = v;
    }
    if let Some(v) = get_setting(conn, "next_sequence")? {
        settings.next_sequence = v.parse().unwrap_or(1);
    }
    if let Some(v) = get_setting(conn, "printer_name")? {
        settings.printer_name = v;
    }
    if let Some(v) = get_setting(conn, "paper_width_mm")? {
        settings.paper_width_mm = v.parse().unwrap_or(80);
    }
    if let Some(v) = get_setting(conn, "waiting_monitor_id")? {
        settings.waiting_monitor_id = v;
    }
    if let Some(v) = get_setting(conn, "waiting_fullscreen")? {
        settings.waiting_fullscreen = v == "true" || v == "1";
    }
    if let Some(v) = get_setting(conn, "last_called_ticket_id")? {
        settings.last_called_ticket_id = if v.is_empty() {
            None
        } else {
            v.parse().ok()
        };
    }
    if let Some(v) = get_setting(conn, "auto_assign")? {
        settings.auto_assign = v == "true" || v == "1";
    }
    Ok(settings)
}

pub fn vacuum_into(conn: &Connection, dest: &Path) -> AppResult<()> {
    conn.execute(
        "VACUUM INTO ?1",
        params![dest.to_string_lossy().to_string()],
    )?;
    Ok(())
}

pub fn backup_file(source: &Path, dest: &Path) -> AppResult<()> {
    std::fs::copy(source, dest)?;
    Ok(())
}

pub fn restore_file(source: &Path, dest: &Path) -> AppResult<()> {
    if !source.exists() {
        return Err(AppError::msg("ملف النسخة الاحتياطية غير موجود"));
    }
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::copy(source, dest)?;
    Ok(())
}
