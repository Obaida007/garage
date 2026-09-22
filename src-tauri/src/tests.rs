use crate::db;
use crate::garage;
use crate::models::{format_ticket_number, today, AppSettings, Ticket};
use crate::ticket_layout;
use rusqlite::params;

fn sample_ticket() -> Ticket {
    Ticket {
        id: 1,
        ticket_number: "007".into(),
        sequence: 7,
        status: "WAITING".into(),
        bay_id: None,
        created_at: "2026-09-18T09:30:00".into(),
        started_at: None,
        completed_at: None,
        cancelled_at: None,
        is_priority: false,
    }
}

#[test]
fn ticket_numbers_are_padded_and_prefixed() {
    assert_eq!(format_ticket_number("", 1), "001");
    assert_eq!(format_ticket_number("", 26), "026");
    assert_eq!(format_ticket_number("A", 3), "A003");
    assert_eq!(format_ticket_number("A", 1000), "A1000");
}

#[test]
fn creating_tickets_increments_and_never_repeats() {
    let mut conn = db::open_memory().unwrap();
    let first = garage::create_ticket(&mut conn).unwrap();
    let second = garage::create_ticket(&mut conn).unwrap();
    assert_eq!(first.ticket_number, "001");
    assert_eq!(second.ticket_number, "002");
    assert_ne!(first.ticket_number, second.ticket_number);
    let settings = db::load_settings(&conn).unwrap();
    assert_eq!(settings.next_sequence, 3);
}

#[test]
fn numbers_survive_reopen_like_restart() {
    let mut conn = db::open_memory().unwrap();
    garage::create_ticket(&mut conn).unwrap();
    for _ in 0..24 {
        garage::create_ticket(&mut conn).unwrap();
    }
    let last = garage::create_ticket(&mut conn).unwrap();
    assert_eq!(last.ticket_number, "026");
    let next = garage::create_ticket(&mut conn).unwrap();
    assert_eq!(next.ticket_number, "027");
}

#[test]
fn sequence_resets_when_the_calendar_day_changes() {
    let mut conn = db::open_memory().unwrap();
    let first = garage::create_ticket(&mut conn).unwrap();
    let second = garage::create_ticket(&mut conn).unwrap();
    assert_eq!(first.ticket_number, "001");
    assert_eq!(second.ticket_number, "002");

    // نحاكي مرور يوم فعلياً: الدوران القديمان أصبحا بتاريخ الأمس، و last_reset_date
    // يعود لتاريخ قديم بدل انتظار منتصف الليل فعلياً.
    conn.execute("UPDATE tickets SET created_at = '2000-01-01T09:00:00', status = 'COMPLETED'", [])
        .unwrap();
    db::set_setting(&conn, "last_reset_date", "2000-01-01").unwrap();

    let after_midnight = garage::create_ticket(&mut conn).unwrap();
    assert_eq!(
        after_midnight.ticket_number, "001",
        "رقم الدور العادي يجب أن يبدأ من جديد بعد تغيّر اليوم"
    );

    let settings = db::load_settings(&conn).unwrap();
    assert_eq!(
        settings.last_reset_date,
        today(),
        "last_reset_date يجب أن يُحدَّث لتاريخ اليوم بعد التصفير"
    );
}

#[test]
fn priority_sequence_resets_alongside_the_regular_sequence() {
    let mut conn = db::open_memory().unwrap();
    let mut settings = db::load_settings(&conn).unwrap();
    settings.priority_enabled = true;
    settings.priority_suffix = "A".into();
    garage::update_settings(&conn, &settings).unwrap();

    let first = garage::create_priority_ticket(&mut conn).unwrap();
    let second = garage::create_priority_ticket(&mut conn).unwrap();
    assert_eq!(first.ticket_number, "A001");
    assert_eq!(second.ticket_number, "A002");

    conn.execute("UPDATE tickets SET created_at = '2000-01-01T09:00:00', status = 'COMPLETED'", [])
        .unwrap();
    db::set_setting(&conn, "last_reset_date", "2000-01-01").unwrap();

    let after_midnight = garage::create_priority_ticket(&mut conn).unwrap();
    assert_eq!(
        after_midnight.ticket_number, "A001",
        "رقم دور الأولوية يجب أن يبدأ من جديد بعد تغيّر اليوم أيضاً"
    );
}

#[test]
fn find_ticket_by_number_prefers_todays_reused_number() {
    let mut conn = db::open_memory().unwrap();
    let yesterday_ticket = garage::create_ticket(&mut conn).unwrap();
    assert_eq!(yesterday_ticket.ticket_number, "001");

    // نجعل هذا الدور يبدو وكأنه من الأمس، ثم نحاكي تغيّر اليوم فعلياً.
    conn.execute(
        "UPDATE tickets SET created_at = '2000-01-01T09:00:00', status = 'COMPLETED' WHERE id = ?1",
        params![yesterday_ticket.id],
    )
    .unwrap();
    db::set_setting(&conn, "last_reset_date", "2000-01-01").unwrap();

    let todays_ticket = garage::create_ticket(&mut conn).unwrap();
    assert_eq!(
        todays_ticket.ticket_number, "001",
        "رقم اليوم يجب أن يُعاد استخدامه رغم وجود دور بنفس الرقم بالأمس"
    );

    let found = garage::find_ticket_by_number(&conn, "001")
        .unwrap()
        .expect("يجب إيجاد دور بهذا الرقم");
    assert_eq!(
        found.id, todays_ticket.id,
        "البحث يجب أن يجد دور اليوم النشط لا دور الأمس القديم"
    );
}

#[test]
fn queue_is_fifo_and_assigns_to_ready_bay() {
    let mut conn = db::open_memory().unwrap();
    let a = garage::create_ticket(&mut conn).unwrap();
    let b = garage::create_ticket(&mut conn).unwrap();
    let called = garage::call_next(&conn, Some(3)).unwrap();
    assert_eq!(called.id, a.id);
    assert_eq!(called.status, "IN_SERVICE");
    assert_eq!(called.bay_id, Some(3));
    let waiting = garage::tickets_by_status(&conn, "WAITING").unwrap();
    assert_eq!(waiting.len(), 1);
    assert_eq!(waiting[0].id, b.id);
}

#[test]
fn cannot_assign_already_used_ticket() {
    let mut conn = db::open_memory().unwrap();
    let ticket = garage::create_ticket(&mut conn).unwrap();
    garage::assign_ticket_to_bay(&conn, ticket.id, 1).unwrap();
    let err = garage::assign_ticket_to_bay(&conn, ticket.id, 2).unwrap_err();
    assert!(err.to_string().contains("مسبقا"));
}

#[test]
fn complete_and_cancel_update_bay_and_status() {
    let mut conn = db::open_memory().unwrap();
    let one = garage::create_ticket(&mut conn).unwrap();
    let two = garage::create_ticket(&mut conn).unwrap();
    garage::assign_ticket_to_bay(&conn, one.id, 1).unwrap();
    let done = garage::complete_bay(&conn, 1).unwrap();
    assert_eq!(done.status, "COMPLETED");
    assert!(done.completed_at.is_some());
    let cancelled = garage::cancel_ticket(&conn, two.id).unwrap();
    assert_eq!(cancelled.status, "CANCELLED");
    assert!(cancelled.cancelled_at.is_some());
    let bays = garage::list_bays(&conn).unwrap();
    assert!(bays.iter().all(|b| b.status == "READY"));
}

#[test]
fn print_failure_does_not_lose_or_renumber_ticket() {
    let mut conn = db::open_memory().unwrap();
    let result = garage::create_ticket_and_maybe_print(&mut conn, |_ticket, _settings| {
        Err("printer offline".into())
    })
    .unwrap();
    assert_eq!(result.ticket.ticket_number, "001");
    assert_eq!(result.print_error.as_deref(), Some("printer offline"));
    let again = garage::create_ticket_and_maybe_print(&mut conn, |_t, _s| Ok(())).unwrap();
    assert_eq!(again.ticket.ticket_number, "002");
    assert!(again.print_error.is_none());
}

#[test]
fn creating_while_all_bays_busy_still_queues() {
    let mut conn = db::open_memory().unwrap();
    for bay in 1..=6 {
        let ticket = garage::create_ticket(&mut conn).unwrap();
        garage::assign_ticket_to_bay(&conn, ticket.id, bay).unwrap();
    }
    let extra = garage::create_ticket(&mut conn).unwrap();
    assert_eq!(extra.status, "WAITING");
    let waiting = garage::tickets_by_status(&conn, "WAITING").unwrap();
    assert_eq!(waiting.len(), 1);
}

#[test]
fn snapshot_after_restart_keeps_in_service_for_recovery() {
    let mut conn = db::open_memory().unwrap();
    let ticket = garage::create_ticket(&mut conn).unwrap();
    garage::assign_ticket_to_bay(&conn, ticket.id, 2).unwrap();
    let snap = garage::snapshot(&conn).unwrap();
    assert!(snap.needs_recovery);
    assert_eq!(snap.in_service.len(), 1);
    assert_eq!(snap.bays[1].current_ticket.as_ref().unwrap().id, ticket.id);
}

#[test]
fn date_and_time_print_at_the_top_with_a_clear_label() {
    let settings = AppSettings::default();
    let lines = ticket_layout::build(&sample_ticket(), &settings).lines;

    let time_line = lines
        .iter()
        .position(|line| line.text.contains(ticket_layout::TIME_LABEL))
        .expect("سطر التاريخ والوقت مفقود من التذكرة");

    assert!(
        lines[time_line].text.contains("2026-09-18 09:30"),
        "سطر التاريخ والوقت لا يحتوي الوقت الفعلي: {}",
        lines[time_line].text
    );

    let number_line = lines
        .iter()
        .position(|line| line.text == "007")
        .expect("رقم الدور مفقود من التذكرة");

    assert!(
        time_line < number_line,
        "يجب أن يظهر التاريخ والوقت في الأعلى قبل رقم الدور"
    );
}

#[test]
fn header_and_time_come_right_after_the_logo() {
    let settings = AppSettings {
        print_header: "OS Tickets".into(),
        ..AppSettings::default()
    };
    let lines = ticket_layout::build(&sample_ticket(), &settings).lines;

    assert_eq!(lines[0].text, "OS Tickets");
    assert!(lines[1].text.starts_with(ticket_layout::TIME_LABEL));
}

#[test]
fn arabic_number_format_applies_to_the_time_line() {
    let settings = AppSettings {
        number_format: "ar".into(),
        ..AppSettings::default()
    };
    let lines = ticket_layout::build(&sample_ticket(), &settings).lines;

    let time_line = lines
        .iter()
        .find(|line| line.text.contains(ticket_layout::TIME_LABEL))
        .expect("سطر التاريخ والوقت مفقود من التذكرة");

    assert!(time_line.text.contains("٢٠٢٦-٠٩-١٨ ٠٩:٣٠"), "{}", time_line.text);
    assert!(lines.iter().any(|line| line.text == "٠٠٧"));
}

#[test]
fn manual_reset_restarts_numbering_but_keeps_open_ticket_numbers_reserved() {
    let mut conn = db::open_memory().unwrap();
    let first = garage::create_ticket(&mut conn).unwrap();
    let _second = garage::create_ticket(&mut conn).unwrap();
    assert_eq!(first.ticket_number, "001");

    conn.execute(
        "UPDATE tickets SET created_at = ?1",
        params![format!("{}T00:00:00", today())],
    )
    .unwrap();
    garage::complete_ticket(&conn, first.id).unwrap();

    garage::reset_numbering(&conn).unwrap();

    let again = garage::create_ticket(&mut conn).unwrap();
    assert_eq!(again.ticket_number, "001", "التصفير اليدوي يجب أن يعيد الترقيم من 001");
    let next = garage::create_ticket(&mut conn).unwrap();
    assert_eq!(
        next.ticket_number, "003",
        "الرقم 002 ما زال لدور مفتوح فيجب ألا يتكرر"
    );
}

#[test]
fn saving_settings_still_works_after_numbers_reset_with_history() {
    let mut conn = db::open_memory().unwrap();
    for _ in 0..3 {
        garage::create_ticket(&mut conn).unwrap();
    }
    conn.execute(
        "UPDATE tickets SET created_at = '2000-01-01T09:00:00', status = 'COMPLETED'",
        [],
    )
    .unwrap();
    db::set_setting(&conn, "last_reset_date", "2000-01-01").unwrap();
    garage::create_ticket(&mut conn).unwrap();

    let settings = db::load_settings(&conn).unwrap();
    garage::update_settings(&conn, &settings)
        .expect("حفظ الإعدادات يجب ألا يفشل بسبب أرقام أدوار الأيام السابقة");
}
