export type TicketStatus = "WAITING" | "IN_SERVICE" | "COMPLETED" | "CANCELLED";
export type BayStatus = "READY" | "BUSY" | "OUT_OF_SERVICE";

export type Ticket = {
  id: number;
  ticketNumber: string;
  sequence: number;
  status: TicketStatus;
  bayId: number | null;
  createdAt: string;
  startedAt: string | null;
  completedAt: string | null;
  cancelledAt: string | null;
};

export type Bay = {
  id: number;
  name: string;
  status: BayStatus;
  currentTicketId: number | null;
  currentTicket: Ticket | null;
};

export type AppSettings = {
  garageName: string;
  printHeader: string;
  ticketPrefix: string;
  nextSequence: number;
  printerName: string;
  paperWidthMm: number;
  waitingMonitorId: string;
  waitingFullscreen: boolean;
  lastCalledTicketId: number | null;
  /** When true tickets are auto-assigned to ready bays. When false the cashier calls manually. */
  autoAssign: boolean;
};

export type BoardService = {
  ticketNumber: string;
  bayId: number;
};

export type WaitingBoard = {
  garageName: string;
  currentTicket: string | null;
  currentBay: number | null;
  nextTicket: string | null;
  waitingCount: number;
  inService: BoardService[];
};

export type GarageSnapshot = {
  bays: Bay[];
  waiting: Ticket[];
  inService: Ticket[];
  settings: AppSettings;
  waitingCount: number;
  board: WaitingBoard;
  needsRecovery: boolean;
};

export type CreateTicketResult = {
  ticket: Ticket;
  printError: string | null;
};

export type DailyReport = {
  date: string;
  total: number;
  completed: number;
  cancelled: number;
  waiting: number;
  inService: number;
  served: number;
  bayStats: BayReport[];
};

export type BayReport = {
  bayId: number;
  completed: number;
};

export type MonitorInfo = {
  id: string;
  name: string;
  isPrimary: boolean;
  x: number;
  y: number;
  width: number;
  height: number;
};

export type PrinterInfo = {
  name: string;
  isDefault: boolean;
};

export type Locale = "ar" | "en";
