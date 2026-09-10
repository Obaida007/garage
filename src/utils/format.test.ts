import { describe, expect, it } from "vitest";
import { backupFileName, formatTicketNumber } from "./format";

describe("ticket numbering", () => {
  it("pads to 3 digits", () => {
    expect(formatTicketNumber("", 1)).toBe("001");
    expect(formatTicketNumber("", 26)).toBe("026");
  });

  it("supports prefix", () => {
    expect(formatTicketNumber("A", 3)).toBe("A003");
  });
});

describe("backup name", () => {
  it("uses garage-backup date format", () => {
    expect(backupFileName("2026-09-05")).toBe("garage-backup-2026-09-05.db");
  });
});
