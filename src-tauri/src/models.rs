use serde::{Deserialize, Serialize};

pub const BAY_COUNT: i64 = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TicketStatus {
    Waiting,
    InService,
    Completed,
    Cancelled,
}

impl TicketStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Waiting => "WAITING",
            Self::InService => "IN_SERVICE",
            Self::Completed => "COMPLETED",
            Self::Cancelled => "CANCELLED",
        }
    }

    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "WAITING" => Ok(Self::Waiting),
            "IN_SERVICE" => Ok(Self::InService),
            "COMPLETED" => Ok(Self::Completed),
            "CANCELLED" => Ok(Self::Cancelled),
            other => Err(format!("unknown ticket status: {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BayStatus {
    Ready,
    Busy,
    OutOfService,
}

impl BayStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "READY",
            Self::Busy => "BUSY",
            Self::OutOfService => "OUT_OF_SERVICE",
        }
    }

    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "READY" => Ok(Self::Ready),
            "BUSY" => Ok(Self::Busy),
            "OUT_OF_SERVICE" => Ok(Self::OutOfService),
            other => Err(format!("unknown bay status: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ticket {
    pub id: i64,
    pub ticket_number: String,
    pub sequence: i64,
    pub status: String,
    pub bay_id: Option<i64>,
    pub created_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub cancelled_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bay {
    pub id: i64,
    pub name: String,
    pub status: String,
    pub current_ticket_id: Option<i64>,
    pub current_ticket: Option<Ticket>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub garage_name: String,
    pub print_header: String,
    pub ticket_prefix: String,
    pub next_sequence: i64,
    pub printer_name: String,
    pub paper_width_mm: i64,
    pub waiting_monitor_id: String,
    pub waiting_fullscreen: bool,
    pub last_called_ticket_id: Option<i64>,
    /// When true, tickets are auto-assigned to ready bays immediately.
    /// When false, the cashier must manually call each ticket.
    pub auto_assign: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            garage_name: "كراج البارودي".into(),
            print_header: "كراج البارودي".into(),
            ticket_prefix: String::new(),
            next_sequence: 1,
            printer_name: String::new(),
            paper_width_mm: 80,
            waiting_monitor_id: String::new(),
            waiting_fullscreen: true,
            last_called_ticket_id: None,
            auto_assign: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WaitingBoard {
    pub garage_name: String,
    pub current_ticket: Option<String>,
    pub current_bay: Option<i64>,
    pub next_ticket: Option<String>,
    pub waiting_count: i64,
    pub in_service: Vec<BoardService>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardService {
    pub ticket_number: String,
    pub bay_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GarageSnapshot {
    pub bays: Vec<Bay>,
    pub waiting: Vec<Ticket>,
    pub in_service: Vec<Ticket>,
    pub settings: AppSettings,
    pub waiting_count: i64,
    pub board: WaitingBoard,
    pub needs_recovery: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTicketResult {
    pub ticket: Ticket,
    pub print_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyReport {
    pub date: String,
    pub total: i64,
    pub completed: i64,
    pub cancelled: i64,
    pub waiting: i64,
    pub in_service: i64,
    pub served: i64,
    pub bay_stats: Vec<BayReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BayReport {
    pub bay_id: i64,
    pub completed: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorInfo {
    pub id: String,
    pub name: String,
    pub is_primary: bool,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrinterInfo {
    pub name: String,
    pub is_default: bool,
}

pub fn format_ticket_number(prefix: &str, sequence: i64) -> String {
    let body = if sequence < 0 {
        format!("{sequence}")
    } else if sequence < 1000 {
        format!("{sequence:03}")
    } else {
        sequence.to_string()
    };
    format!("{}{}", prefix.trim(), body)
}

pub fn now_iso() -> String {
    chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string()
}

pub fn today() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}
