import { create } from "zustand";
import { listen } from "@tauri-apps/api/event";
import { api } from "@/services/tauri";
import { speakTicketCall } from "@/utils/audio";
import type { GarageSnapshot, Locale } from "@/types";

type GarageStore = {
  locale: Locale;
  snapshot: GarageSnapshot | null;
  loading: boolean;
  error: string | null;
  setLocale: (locale: Locale) => void;
  refresh: () => Promise<void>;
  start: () => Promise<() => void>;
};

export const useGarageStore = create<GarageStore>((set, get) => ({
  locale: "ar",
  snapshot: null,
  loading: true,
  error: null,
  setLocale: (locale) => {
    set({ locale });
    document.documentElement.lang = locale;
    document.documentElement.dir = locale === "ar" ? "rtl" : "ltr";
  },
  refresh: async () => {
    try {
      const snapshot = await api.snapshot();
      set({ snapshot, loading: false, error: null });
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : String(error);
      
      // Fallback for standard web browsers (no Tauri backend)
      if (errorMsg.includes("invoke") || errorMsg.includes("window.__TAURI_IPC__")) {
        const demoSnapshot: GarageSnapshot = {
          bays: [
            { id: 1, name: "حفرة 1", status: "BUSY", currentTicketId: 1, currentTicket: { id: 1, ticketNumber: "001", sequence: 1, status: "IN_SERVICE", bayId: 1, createdAt: "", startedAt: null, completedAt: null, cancelledAt: null } },
            { id: 2, name: "حفرة 2", status: "BUSY", currentTicketId: 2, currentTicket: { id: 2, ticketNumber: "002", sequence: 2, status: "IN_SERVICE", bayId: 2, createdAt: "", startedAt: null, completedAt: null, cancelledAt: null } },
            { id: 3, name: "حفرة 3", status: "READY", currentTicketId: null, currentTicket: null },
            { id: 4, name: "حفرة 4", status: "BUSY", currentTicketId: 3, currentTicket: { id: 3, ticketNumber: "004", sequence: 4, status: "IN_SERVICE", bayId: 4, createdAt: "", startedAt: null, completedAt: null, cancelledAt: null } },
            { id: 5, name: "حفرة 5", status: "READY", currentTicketId: null, currentTicket: null },
            { id: 6, name: "حفرة 6", status: "READY", currentTicketId: null, currentTicket: null },
          ],
          waiting: [],
          inService: [],
          settings: { garageName: "كراج البارودي", printHeader: "", ticketPrefix: "", nextSequence: 5, printerName: "", paperWidthMm: 80, waitingMonitorId: "", waitingFullscreen: true, lastCalledTicketId: null, autoAssign: true },
          waitingCount: 3,
          board: {
            garageName: "كراج البارودي",
            currentTicket: "004",
            currentBay: 4,
            nextTicket: "005",
            waitingCount: 3,
            inService: [
              { ticketNumber: "001", bayId: 1 },
              { ticketNumber: "002", bayId: 2 },
              { ticketNumber: "004", bayId: 4 },
            ],
          },
          needsRecovery: false,
        };
        set({ snapshot: demoSnapshot, loading: false, error: null });
        return;
      }

      set({
        loading: false,
        error: errorMsg,
      });
    }
  },
  start: async () => {
    await get().refresh();
    const unlisten = await listen<GarageSnapshot>("garage-updated", (event) => {
      const prevSnapshot = get().snapshot;
      const newSnapshot = event.payload;

      // Detect tickets that just moved into a bay (were not there before)
      if (prevSnapshot) {
        for (const newBay of newSnapshot.bays) {
          const prevBay = prevSnapshot.bays.find((b) => b.id === newBay.id);
          const prevTicketId = prevBay?.currentTicket?.id ?? null;
          const newTicketId = newBay.currentTicket?.id ?? null;

          // A new ticket just arrived in this bay
          if (
            newTicketId !== null &&
            newTicketId !== prevTicketId &&
            newBay.currentTicket
          ) {
            speakTicketCall(newBay.currentTicket.ticketNumber, newBay.id);
          }
        }
      }

      set({ snapshot: newSnapshot, loading: false, error: null });
    });
    return unlisten;
  },
}));
