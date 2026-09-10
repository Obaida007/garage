use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::{
    format_ticket_number, now_iso, today, AppSettings, Bay, BoardService, CreateTicketResult,
    DailyReport, GarageSnapshot, Ticket, WaitingBoard,
};
use rusqlite::{params, Connection, OptionalExtension};

pub fn snapshot(conn: &Connection) -> AppResult<GarageSnapshot> {
    let settings = db::load_settings(conn)?;
    let bays = list_bays(conn)?;
    let waiting = tickets_by_status(conn, "WAITING")?;
    let in_service = tickets_by_status(conn, "IN_SERVICE")?;
    let board = waiting_board(conn, &settings, &waiting, &in_service)?;
    Ok(GarageSnapshot {
        waiting_count: waiting.len() as i64,
        needs_recovery: !in_service.is_empty(),
        bays,
        waiting,
        in_service,
        settings,
        board,
    })
}

pub fn list_bays(conn: &Connection) -> AppResult<Vec<Bay>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, status, current_ticket_id FROM bays ORDER BY id ASC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, Option<i64>>(3)?,
        ))
    })?;

    let mut bays = Vec::new();
    for row in rows {
        let (id, name, status, current_ticket_id) = row?;
        let current_ticket = match current_ticket_id {
            Some(ticket_id) => get_ticket(conn, ticket_id)?,
            None => None,
        };
        bays.push(Bay {
            id,
            name,
            status,
            current_ticket_id,
            current_ticket,
        });
    }
    Ok(bays)
}

pub fn tickets_by_status(conn: &Connection, status: &str) -> AppResult<Vec<Ticket>> {
    let sql = if status == "WAITING" {
        "SELECT id, ticket_number, sequence, status, bay_id, created_at, started_at, completed_at, cancelled_at
         FROM tickets WHERE status = ?1 ORDER BY sequence ASC"
    } else {
        "SELECT id, ticket_number, sequence, status, bay_id, created_at, started_at, completed_at, cancelled_at
         FROM tickets WHERE status = ?1 ORDER BY started_at ASC, sequence ASC"
    };
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map(params![status], map_ticket)?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(AppError::from)
}

pub fn get_ticket(conn: &Connection, id: i64) -> AppResult<Option<Ticket>> {
    conn.query_row(
        "SELECT id, ticket_number, sequence, status, bay_id, created_at, started_at, completed_at, cancelled_at
         FROM tickets WHERE id = ?1",
        params![id],
        map_ticket,
    )
    .optional()
    .map_err(AppError::from)
}

pub fn find_ticket_by_number(conn: &Connection, number: &str) -> AppResult<Option<Ticket>> {
    conn.query_row(
        "SELECT id, ticket_number, sequence, status, bay_id, created_at, started_at, completed_at, cancelled_at
         FROM tickets WHERE ticket_number = ?1",
        params![number.trim()],
        map_ticket,
    )
    .optional()
    .map_err(AppError::from)
}

pub fn recent_tickets(conn: &Connection, limit: i64) -> AppResult<Vec<Ticket>> {
    let mut stmt = conn.prepare(
        "SELECT id, ticket_number, sequence, status, bay_id, created_at, started_at, completed_at, cancelled_at
         FROM tickets ORDER BY id DESC LIMIT ?1",
    )?;
    let rows = stmt.query_map(params![limit], map_ticket)?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(AppError::from)
}

fn map_ticket(row: &rusqlite::Row<'_>) -> rusqlite::Result<Ticket> {
    Ok(Ticket {
        id: row.get(0)?,
        ticket_number: row.get(1)?,
        sequence: row.get(2)?,
        status: row.get(3)?,
        bay_id: row.get(4)?,
        created_at: row.get(5)?,
        started_at: row.get(6)?,
        completed_at: row.get(7)?,
        cancelled_at: row.get(8)?,
    })
}

pub fn create_ticket(conn: &mut Connection) -> AppResult<Ticket> {
    let tx = conn.transaction()?;
    let settings = db::load_settings(&tx)?;
    let mut sequence = settings.next_sequence.max(1);

    loop {
        let number = format_ticket_number(&settings.ticket_prefix, sequence);
        let exists: Option<i64> = tx
            .query_row(
                "SELECT id FROM tickets WHERE ticket_number = ?1 OR sequence = ?2",
                params![number, sequence],
                |row| row.get(0),
            )
            .optional()?;
        if exists.is_none() {
            let now = now_iso();
            tx.execute(
                "INSERT INTO tickets (ticket_number, sequence, status, bay_id, created_at)
                 VALUES (?1, ?2, 'WAITING', NULL, ?3)",
                params![number, sequence, now],
            )?;
            let id = tx.last_insert_rowid();
            db::set_setting(&tx, "next_sequence", &(sequence + 1).to_string())?;
            tx.commit()?;
            return Ok(Ticket {
                id,
                ticket_number: number,
                sequence,
                status: "WAITING".into(),
                bay_id: None,
                created_at: now,
                started_at: None,
                completed_at: None,
                cancelled_at: None,
            });
        }
        sequence += 1;
        if sequence > settings.next_sequence + 10_000 {
            return Err(AppError::msg("تعذر إنشاء رقم دور فريد"));
        }
    }
}

pub fn create_ticket_and_maybe_print<F>(
    conn: &mut Connection,
    printer: F,
) -> AppResult<CreateTicketResult>
where
    F: FnOnce(&Ticket, &AppSettings) -> Result<(), String>,
{
    let ticket = create_ticket(conn)?;
    let settings = db::load_settings(conn)?;
    let print_error = printer(&ticket, &settings).err();
    Ok(CreateTicketResult {
        ticket,
        print_error,
    })
}

pub fn first_ready_bay(conn: &Connection) -> AppResult<Option<i64>> {
    conn.query_row(
        "SELECT id FROM bays WHERE status = 'READY' AND current_ticket_id IS NULL ORDER BY id ASC LIMIT 1",
        [],
        |row| row.get(0),
    )
    .optional()
    .map_err(AppError::from)
}

pub fn assign_ticket_to_bay(conn: &Connection, ticket_id: i64, bay_id: i64) -> AppResult<Ticket> {
    let ticket = get_ticket(conn, ticket_id)?.ok_or_else(|| AppError::msg("الدور غير موجود"))?;
    if ticket.status != "WAITING" {
        return Err(AppError::msg("لا يمكن تعيين دور مستخدم مسبقاً"));
    }

    let bay_status: String = conn.query_row(
        "SELECT status FROM bays WHERE id = ?1",
        params![bay_id],
        |row| row.get(0),
    )?;
    let current: Option<i64> = conn.query_row(
        "SELECT current_ticket_id FROM bays WHERE id = ?1",
        params![bay_id],
        |row| row.get(0),
    )?;
    if bay_status != "READY" || current.is_some() {
        return Err(AppError::msg("الحفرة غير جاهزة"));
    }

    let started = now_iso();
    conn.execute(
        "UPDATE tickets SET status = 'IN_SERVICE', bay_id = ?1, started_at = ?2 WHERE id = ?3",
        params![bay_id, started, ticket_id],
    )?;
    conn.execute(
        "UPDATE bays SET status = 'BUSY', current_ticket_id = ?1 WHERE id = ?2",
        params![ticket_id, bay_id],
    )?;
    db::set_setting(conn, "last_called_ticket_id", &ticket_id.to_string())?;
    get_ticket(conn, ticket_id)?.ok_or_else(|| AppError::msg("فشل قراءة الدور بعد التعيين"))
}

pub fn call_next(conn: &Connection, bay_id: Option<i64>) -> AppResult<Ticket> {
    let target_bay = match bay_id {
        Some(id) => id,
        None => first_ready_bay(conn)?.ok_or_else(|| AppError::msg("لا توجد حفرة جاهزة"))?,
    };
    let next_id: i64 = conn
        .query_row(
            "SELECT id FROM tickets WHERE status = 'WAITING' ORDER BY sequence ASC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()?
        .ok_or_else(|| AppError::msg("لا يوجد دور في الانتظار"))?;
    assign_ticket_to_bay(conn, next_id, target_bay)
}

pub fn complete_ticket(conn: &Connection, ticket_id: i64) -> AppResult<Ticket> {
    let ticket = get_ticket(conn, ticket_id)?.ok_or_else(|| AppError::msg("الدور غير موجود"))?;
    if ticket.status != "IN_SERVICE" && ticket.status != "WAITING" {
        return Err(AppError::msg("لا يمكن إنهاء هذا الدور"));
    }
    let completed = now_iso();
    conn.execute(
        "UPDATE tickets SET status = 'COMPLETED', completed_at = ?1 WHERE id = ?2",
        params![completed, ticket_id],
    )?;
    if let Some(bay_id) = ticket.bay_id {
        conn.execute(
            "UPDATE bays SET status = 'READY', current_ticket_id = NULL WHERE id = ?1 AND current_ticket_id = ?2",
            params![bay_id, ticket_id],
        )?;
    }
    clear_last_called_if_needed(conn, ticket_id)?;
    get_ticket(conn, ticket_id)?.ok_or_else(|| AppError::msg("فشل قراءة الدور"))
}

pub fn complete_bay(conn: &Connection, bay_id: i64) -> AppResult<Ticket> {
    let ticket_id: Option<i64> = conn.query_row(
        "SELECT current_ticket_id FROM bays WHERE id = ?1",
        params![bay_id],
        |row| row.get(0),
    )?;
    let ticket_id = ticket_id.ok_or_else(|| AppError::msg("الحفرة فارغة"))?;
    complete_ticket(conn, ticket_id)
}

pub fn cancel_ticket(conn: &Connection, ticket_id: i64) -> AppResult<Ticket> {
    let ticket = get_ticket(conn, ticket_id)?.ok_or_else(|| AppError::msg("الدور غير موجود"))?;
    if ticket.status == "COMPLETED" || ticket.status == "CANCELLED" {
        return Err(AppError::msg("لا يمكن إلغاء هذا الدور"));
    }
    let cancelled = now_iso();
    conn.execute(
        "UPDATE tickets SET status = 'CANCELLED', cancelled_at = ?1, bay_id = CASE WHEN status = 'WAITING' THEN NULL ELSE bay_id END WHERE id = ?2",
        params![cancelled, ticket_id],
    )?;
    if let Some(bay_id) = ticket.bay_id {
        conn.execute(
            "UPDATE bays SET status = 'READY', current_ticket_id = NULL WHERE id = ?1 AND current_ticket_id = ?2",
            params![bay_id, ticket_id],
        )?;
    }
    clear_last_called_if_needed(conn, ticket_id)?;
    get_ticket(conn, ticket_id)?.ok_or_else(|| AppError::msg("فشل قراءة الدور"))
}

pub fn return_to_queue(conn: &Connection, ticket_id: i64) -> AppResult<Ticket> {
    let ticket = get_ticket(conn, ticket_id)?.ok_or_else(|| AppError::msg("الدور غير موجود"))?;
    if ticket.status != "IN_SERVICE" {
        return Err(AppError::msg("يمكن إرجاع الأدوار قيد الخدمة فقط"));
    }
    conn.execute(
        "UPDATE tickets SET status = 'WAITING', bay_id = NULL, started_at = NULL WHERE id = ?1",
        params![ticket_id],
    )?;
    if let Some(bay_id) = ticket.bay_id {
        conn.execute(
            "UPDATE bays SET status = 'READY', current_ticket_id = NULL WHERE id = ?1 AND current_ticket_id = ?2",
            params![bay_id, ticket_id],
        )?;
    }
    clear_last_called_if_needed(conn, ticket_id)?;
    get_ticket(conn, ticket_id)?.ok_or_else(|| AppError::msg("فشل قراءة الدور"))
}

pub fn move_ticket(conn: &Connection, ticket_id: i64, to_bay_id: i64) -> AppResult<Ticket> {
    let ticket = get_ticket(conn, ticket_id)?.ok_or_else(|| AppError::msg("الدور غير موجود"))?;
    if ticket.status != "IN_SERVICE" {
        return Err(AppError::msg("يمكن نقل الأدوار قيد الخدمة فقط"));
    }
    let from_bay = ticket.bay_id.ok_or_else(|| AppError::msg("الدور غير مرتبط بحفرة"))?;
    if from_bay == to_bay_id {
        return Ok(ticket);
    }
    let current: Option<i64> = conn.query_row(
        "SELECT current_ticket_id FROM bays WHERE id = ?1",
        params![to_bay_id],
        |row| row.get(0),
    )?;
    if current.is_some() {
        return Err(AppError::msg("الحفرة الهدف مشغولة"));
    }
    conn.execute(
        "UPDATE bays SET status = 'READY', current_ticket_id = NULL WHERE id = ?1",
        params![from_bay],
    )?;
    conn.execute(
        "UPDATE tickets SET bay_id = ?1 WHERE id = ?2",
        params![to_bay_id, ticket_id],
    )?;
    conn.execute(
        "UPDATE bays SET status = 'BUSY', current_ticket_id = ?1 WHERE id = ?2",
        params![ticket_id, to_bay_id],
    )?;
    db::set_setting(conn, "last_called_ticket_id", &ticket_id.to_string())?;
    get_ticket(conn, ticket_id)?.ok_or_else(|| AppError::msg("فشل قراءة الدور"))
}

pub fn update_settings(conn: &Connection, patch: &AppSettings) -> AppResult<AppSettings> {
    if patch.paper_width_mm != 58 && patch.paper_width_mm != 80 {
        return Err(AppError::msg("حجم الورق يجب أن يكون 58 أو 80"));
    }
    let current = db::load_settings(conn)?;
    if patch.next_sequence < 1 {
        return Err(AppError::msg("رقم الدور الابتدائي غير صالح"));
    }
    let max_seq: i64 = conn
        .query_row("SELECT COALESCE(MAX(sequence), 0) FROM tickets", [], |row| {
            row.get(0)
        })?;
    if patch.next_sequence <= max_seq {
        return Err(AppError::msg(format!(
            "لا يمكن تعيين الرقم التالي إلى {} لأنه مستخدم أو أصغر من آخر رقم ({max_seq})",
            patch.next_sequence
        )));
    }

    db::set_setting(conn, "garage_name", &patch.garage_name)?;
    db::set_setting(conn, "print_header", &patch.print_header)?;
    db::set_setting(conn, "ticket_prefix", &patch.ticket_prefix)?;
    db::set_setting(conn, "next_sequence", &patch.next_sequence.to_string())?;
    db::set_setting(conn, "printer_name", &patch.printer_name)?;
    db::set_setting(conn, "paper_width_mm", &patch.paper_width_mm.to_string())?;
    db::set_setting(conn, "waiting_monitor_id", &patch.waiting_monitor_id)?;
    db::set_setting(
        conn,
        "waiting_fullscreen",
        if patch.waiting_fullscreen {
            "true"
        } else {
            "false"
        },
    )?;
    db::set_setting(
        conn,
        "last_called_ticket_id",
        &current
            .last_called_ticket_id
            .map(|v| v.to_string())
            .unwrap_or_default(),
    )?;
    db::load_settings(conn)
}

pub fn daily_report(conn: &Connection, date: &str) -> AppResult<DailyReport> {
    let like = format!("{date}%");
    let count = |status: Option<&str>| -> AppResult<i64> {
        let (sql, params_vec): (&str, Vec<rusqlite::types::Value>) = match status {
            Some(st) => (
                "SELECT COUNT(*) FROM tickets WHERE created_at LIKE ?1 AND status = ?2",
                vec![like.clone().into(), st.to_string().into()],
            ),
            None => (
                "SELECT COUNT(*) FROM tickets WHERE created_at LIKE ?1",
                vec![like.clone().into()],
            ),
        };
        conn.query_row(sql, rusqlite::params_from_iter(params_vec), |row| row.get(0))
            .map_err(AppError::from)
    };

    Ok(DailyReport {
        date: date.to_string(),
        total: count(None)?,
        completed: count(Some("COMPLETED"))?,
        cancelled: count(Some("CANCELLED"))?,
        waiting: count(Some("WAITING"))?,
        in_service: count(Some("IN_SERVICE"))?,
        served: count(Some("COMPLETED"))?,
    })
}

pub fn reset_open_queue(conn: &Connection) -> AppResult<()> {
    conn.execute(
        "UPDATE tickets SET status = 'CANCELLED', cancelled_at = ?1 WHERE status IN ('WAITING','IN_SERVICE')",
        params![now_iso()],
    )?;
    conn.execute(
        "UPDATE bays SET status = 'READY', current_ticket_id = NULL",
        [],
    )?;
    db::set_setting(conn, "last_called_ticket_id", "")?;
    Ok(())
}

fn clear_last_called_if_needed(conn: &Connection, ticket_id: i64) -> AppResult<()> {
    let settings = db::load_settings(conn)?;
    if settings.last_called_ticket_id == Some(ticket_id) {
        let fallback: Option<i64> = conn
            .query_row(
                "SELECT id FROM tickets WHERE status = 'IN_SERVICE' ORDER BY started_at DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .optional()?;
        db::set_setting(
            conn,
            "last_called_ticket_id",
            &fallback.map(|v| v.to_string()).unwrap_or_default(),
        )?;
    }
    Ok(())
}

fn waiting_board(
    conn: &Connection,
    settings: &AppSettings,
    waiting: &[Ticket],
    in_service: &[Ticket],
) -> AppResult<WaitingBoard> {
    let mut current_ticket = None;
    let mut current_bay = None;
    if let Some(id) = settings.last_called_ticket_id {
        if let Some(ticket) = get_ticket(conn, id)? {
            if ticket.status == "IN_SERVICE" {
                current_ticket = Some(ticket.ticket_number.clone());
                current_bay = ticket.bay_id;
            }
        }
    }
    if current_ticket.is_none() {
        if let Some(ticket) = in_service.last() {
            current_ticket = Some(ticket.ticket_number.clone());
            current_bay = ticket.bay_id;
        }
    }
    Ok(WaitingBoard {
        garage_name: settings.garage_name.clone(),
        current_ticket,
        current_bay,
        next_ticket: waiting.first().map(|t| t.ticket_number.clone()),
        waiting_count: waiting.len() as i64,
        in_service: in_service
            .iter()
            .filter_map(|t| {
                t.bay_id.map(|bay_id| BoardService {
                    ticket_number: t.ticket_number.clone(),
                    bay_id,
                })
            })
            .collect(),
    })
}

pub fn report_today(conn: &Connection) -> AppResult<DailyReport> {
    daily_report(conn, &today())
}
