use crate::error::{AppError, AppResult};
use crate::models::{Ad, AppSettings, BAY_COUNT};
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
            ticket_number TEXT NOT NULL,
            sequence INTEGER NOT NULL,
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
        CREATE INDEX IF NOT EXISTS idx_tickets_number_created ON tickets(ticket_number, created_at);

        CREATE TABLE IF NOT EXISTS ads (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            file_path TEXT NOT NULL,
            display_order INTEGER NOT NULL DEFAULT 0,
            duration_secs INTEGER NOT NULL DEFAULT 10,
            active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
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

    // Add the "active" column (soft delete/archive flag) if this DB predates it.
    let has_active_column = conn
        .prepare("SELECT active FROM bays LIMIT 1")
        .is_ok();
    if !has_active_column {
        conn.execute_batch("ALTER TABLE bays ADD COLUMN active INTEGER NOT NULL DEFAULT 1;")?;
    }

    // Add the "is_priority" column (priority/suffix tickets) if this DB predates it.
    let has_priority_column = conn
        .prepare("SELECT is_priority FROM tickets LIMIT 1")
        .is_ok();
    if !has_priority_column {
        conn.execute_batch(
            "ALTER TABLE tickets ADD COLUMN is_priority INTEGER NOT NULL DEFAULT 0;",
        )?;
    }

    // Migrate old "tickets" tables that still carry the legacy global UNIQUE
    // constraints on ticket_number/sequence. Those must only be unique per
    // day once numbers reset daily, not forever, otherwise every reset after
    // the first day silently fails (today's "001" collides with a past
    // day's "001" and the counter just keeps climbing).
    let tickets_sql = "SELECT sql FROM sqlite_master WHERE type='table' AND name='tickets'";
    if let Ok(table_sql) = conn.query_row(tickets_sql, [], |row| row.get::<_, String>(0)) {
        if table_sql.contains("UNIQUE") {
            conn.execute_batch("PRAGMA foreign_keys = OFF;")?;
            let migration = conn.execute_batch(
                r#"
                CREATE TABLE tickets_temp (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    ticket_number TEXT NOT NULL,
                    sequence INTEGER NOT NULL,
                    status TEXT NOT NULL CHECK(status IN ('WAITING','IN_SERVICE','COMPLETED','CANCELLED')),
                    bay_id INTEGER,
                    created_at TEXT NOT NULL,
                    started_at TEXT,
                    completed_at TEXT,
                    cancelled_at TEXT,
                    is_priority INTEGER NOT NULL DEFAULT 0
                );
                INSERT INTO tickets_temp
                    SELECT id, ticket_number, sequence, status, bay_id, created_at,
                           started_at, completed_at, cancelled_at, is_priority
                    FROM tickets;
                DROP TABLE tickets;
                ALTER TABLE tickets_temp RENAME TO tickets;
                CREATE INDEX IF NOT EXISTS idx_tickets_status ON tickets(status);
                CREATE INDEX IF NOT EXISTS idx_tickets_created ON tickets(created_at);
                CREATE INDEX IF NOT EXISTS idx_tickets_number_created ON tickets(ticket_number, created_at);
                "#,
            );
            conn.execute_batch("PRAGMA foreign_keys = ON;")?;
            migration?;
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

    // A pre-existing install already has a garage_name row (even an empty one the
    // user cleared intentionally). Only a brand-new database lacks it entirely, so
    // this is the signal for whether the first-run setup wizard should show.
    let is_existing_install = get_setting(conn, "garage_name")?.is_some();
    set_if_missing(
        conn,
        "setup_completed",
        if is_existing_install { "true" } else { "false" },
    )?;

    set_if_missing(conn, "waiting_layout", &defaults.waiting_layout)?;
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
    set_if_missing(conn, "last_reset_date", "")?;
    set_if_missing(conn, "logo_path", &defaults.logo_path)?;
    set_if_missing(conn, "number_format", &defaults.number_format)?;
    set_if_missing(conn, "settings_password", &defaults.settings_password)?;
    set_if_missing(conn, "priority_enabled", "false")?;
    set_if_missing(conn, "priority_suffix", &defaults.priority_suffix)?;
    set_if_missing(conn, "next_priority_sequence", &defaults.next_priority_sequence.to_string())?;
    set_if_missing(conn, "ads_enabled", "false")?;
    set_if_missing(conn, "board_duration_secs", &defaults.board_duration_secs.to_string())?;
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
    if let Some(v) = get_setting(conn, "last_reset_date")? {
        settings.last_reset_date = v;
    }
    if let Some(v) = get_setting(conn, "logo_path")? {
        settings.logo_path = v;
    }
    if let Some(v) = get_setting(conn, "number_format")? {
        settings.number_format = if v == "ar" { "ar".into() } else { "en".into() };
    }
    if let Some(v) = get_setting(conn, "settings_password")? {
        settings.settings_password = v;
    }
    if let Some(v) = get_setting(conn, "priority_enabled")? {
        settings.priority_enabled = v == "true" || v == "1";
    }
    if let Some(v) = get_setting(conn, "priority_suffix")? {
        settings.priority_suffix = v;
    }
    if let Some(v) = get_setting(conn, "next_priority_sequence")? {
        settings.next_priority_sequence = v.parse().unwrap_or(1);
    }
    if let Some(v) = get_setting(conn, "waiting_layout")? {
        settings.waiting_layout = if v == "table" { "table".into() } else { "cards".into() };
    }
    if let Some(v) = get_setting(conn, "setup_completed")? {
        settings.setup_completed = v == "true" || v == "1";
    }
    if let Some(v) = get_setting(conn, "ads_enabled")? {
        settings.ads_enabled = v == "true" || v == "1";
    }
    if let Some(v) = get_setting(conn, "board_duration_secs")? {
        settings.board_duration_secs = v.parse().unwrap_or(8).max(1);
    }
    Ok(settings)
}

// ─── Ads CRUD ────────────────────────────────────────────────────────────────

pub fn list_ads(conn: &Connection) -> AppResult<Vec<Ad>> {
    let mut stmt = conn.prepare(
        "SELECT id, file_path, display_order, duration_secs, active \
         FROM ads WHERE active = 1 ORDER BY display_order ASC, id ASC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Ad {
            id: row.get(0)?,
            file_path: row.get(1)?,
            display_order: row.get(2)?,
            duration_secs: row.get(3)?,
            active: row.get::<_, i64>(4)? != 0,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn add_ad(conn: &Connection, file_path: &str, duration_secs: i64) -> AppResult<Ad> {
    let next_order: i64 = conn.query_row(
        "SELECT COALESCE(MAX(display_order), -1) + 1 FROM ads WHERE active = 1",
        [],
        |row| row.get(0),
    )?;
    conn.execute(
        "INSERT INTO ads (file_path, display_order, duration_secs, active) VALUES (?1, ?2, ?3, 1)",
        params![file_path, next_order, duration_secs.max(1)],
    )?;
    let id = conn.last_insert_rowid();
    Ok(Ad {
        id,
        file_path: file_path.to_string(),
        display_order: next_order,
        duration_secs: duration_secs.max(1),
        active: true,
    })
}

pub fn remove_ad(conn: &Connection, id: i64) -> AppResult<String> {
    let file_path: String = conn
        .query_row("SELECT file_path FROM ads WHERE id = ?1", params![id], |r| r.get(0))
        .map_err(|_| AppError::msg("الإعلان غير موجود"))?;
    conn.execute("DELETE FROM ads WHERE id = ?1", params![id])?;
    Ok(file_path)
}

pub fn update_ad_duration(conn: &Connection, id: i64, duration_secs: i64) -> AppResult<Ad> {
    let dur = duration_secs.max(1);
    let updated = conn.execute(
        "UPDATE ads SET duration_secs = ?1 WHERE id = ?2",
        params![dur, id],
    )?;
    if updated == 0 {
        return Err(AppError::msg("الإعلان غير موجود"));
    }
    conn.query_row(
        "SELECT id, file_path, display_order, duration_secs, active FROM ads WHERE id = ?1",
        params![id],
        |row| {
            Ok(Ad {
                id: row.get(0)?,
                file_path: row.get(1)?,
                display_order: row.get(2)?,
                duration_secs: row.get(3)?,
                active: row.get::<_, i64>(4)? != 0,
            })
        },
    )
    .map_err(|e| AppError::msg(e.to_string()))
}

pub fn reorder_ads(conn: &Connection, ids: &[i64]) -> AppResult<()> {
    for (order, id) in ids.iter().enumerate() {
        conn.execute(
            "UPDATE ads SET display_order = ?1 WHERE id = ?2",
            params![order as i64, id],
        )?;
    }
    Ok(())
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
