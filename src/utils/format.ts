export function formatTicketNumber(prefix: string, sequence: number): string {
  const body = sequence < 1000 ? String(sequence).padStart(3, "0") : String(sequence);
  return `${prefix.trim()}${body}`;
}

export function todayIso(): string {
  const now = new Date();
  const y = now.getFullYear();
  const m = String(now.getMonth() + 1).padStart(2, "0");
  const d = String(now.getDate()).padStart(2, "0");
  return `${y}-${m}-${d}`;
}

export function backupFileName(date = todayIso()): string {
  return `garage-backup-${date}.db`;
}
