use crate::db;
use crate::garage;
use crate::models::format_ticket_number;

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
