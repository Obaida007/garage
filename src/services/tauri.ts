import { invoke } from "@tauri-apps/api/core";
import type {
  AppSettings,
  Bay,
  CreateTicketResult,
  DailyReport,
  GarageSnapshot,
  MonitorInfo,
  PrinterInfo,
  Ticket,
} from "@/types";

export const api = {
  snapshot: () => invoke<GarageSnapshot>("get_snapshot"),
  createTicket: () => invoke<CreateTicketResult>("create_ticket"),
  reprint: (ticketId: number) => invoke<void>("reprint_ticket", { ticketId }),
  reprintLast: () => invoke<void>("reprint_last"),
  searchTicket: (number: string) => invoke<Ticket | null>("search_ticket", { number }),
  recentTickets: (limit = 50) => invoke<Ticket[]>("list_recent_tickets", { limit }),
  callNext: (bayId?: number | null) => invoke<Ticket>("call_next", { bayId: bayId ?? null }),
  assign: (ticketId: number, bayId: number) =>
    invoke<Ticket>("assign_ticket", { ticketId, bayId }),
  setBayOutOfService: (bayId: number, outOfService: boolean) =>
    invoke<Bay>("set_bay_out_of_service", { bayId, outOfService }),
  completeBay: (bayId: number) => invoke<Ticket>("complete_bay", { bayId }),
  completeTicket: (ticketId: number) => invoke<Ticket>("complete_ticket", { ticketId }),
  cancelTicket: (ticketId: number) => invoke<Ticket>("cancel_ticket", { ticketId }),
  returnToQueue: (ticketId: number) => invoke<Ticket>("return_to_queue", { ticketId }),
  moveTicket: (ticketId: number, toBayId: number) =>
    invoke<Ticket>("move_ticket", { ticketId, toBayId }),
  getSettings: () => invoke<AppSettings>("get_settings"),
  saveSettings: (settings: AppSettings) => invoke<AppSettings>("save_settings", { settings }),
  printers: () => invoke<PrinterInfo[]>("list_printers"),
  testPrint: () => invoke<void>("test_print"),
  monitors: () => invoke<MonitorInfo[]>("list_monitors"),
  refreshDisplay: () => invoke<void>("refresh_waiting_display"),
  testDisplay: () => invoke<void>("test_waiting_display"),
  backup: (dest: string) => invoke<string>("backup_database", { dest }),
  restore: (source: string) => invoke<void>("restore_database", { source }),
  report: (date?: string) => invoke<DailyReport>("get_report", { date: date ?? null }),
  resetQueue: () => invoke<void>("reset_open_queue"),
  setAutostart: (enabled: boolean) => invoke<boolean>("set_autostart", { enabled }),
  isAutostart: () => invoke<boolean>("is_autostart_enabled"),
  dbPath: () => invoke<string>("db_path"),
};
