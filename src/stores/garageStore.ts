import { create } from "zustand";
import { listen } from "@tauri-apps/api/event";
import { api } from "@/services/tauri";
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
      set({
        loading: false,
        error: error instanceof Error ? error.message : String(error),
      });
    }
  },
  start: async () => {
    await get().refresh();
    const unlisten = await listen<GarageSnapshot>("garage-updated", (event) => {
      set({ snapshot: event.payload, loading: false, error: null });
    });
    return unlisten;
  },
}));
