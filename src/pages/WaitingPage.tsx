import { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { convertFileSrc } from "@tauri-apps/api/core";
import { useGarage } from "@/hooks/useGarage";
import { api } from "@/services/tauri";
import { WaitingBoardView } from "@/features/waiting/WaitingBoard";
import type { Ad } from "@/types";
import { cn } from "@/lib/utils";

// ─── Ad slide view ────────────────────────────────────────────────────────────

function AdSlide({ ad, visible }: { ad: Ad; visible: boolean }) {
  const src = convertFileSrc(ad.filePath);
  return (
    <div
      className={cn(
        "absolute inset-0 flex items-center justify-center bg-black transition-opacity duration-700",
        visible ? "opacity-100" : "opacity-0 pointer-events-none",
      )}
    >
      <img
        src={src}
        alt=""
        className="h-full w-full object-contain"
        draggable={false}
      />
    </div>
  );
}

// ─── Cycle state ─────────────────────────────────────────────────────────────

type Phase = { kind: "board" } | { kind: "ad"; index: number };

function nextPhase(current: Phase, adsLen: number): Phase {
  if (current.kind === "board") {
    return adsLen > 0 ? { kind: "ad", index: 0 } : { kind: "board" };
  }
  const next = current.index + 1;
  return next >= adsLen ? { kind: "board" } : { kind: "ad", index: next };
}

// ─── Page ────────────────────────────────────────────────────────────────────

export function WaitingPage() {
  const { snapshot, loading, error, locale } = useGarage();
  const [ads, setAds] = useState<Ad[]>([]);
  const [phase, setPhase] = useState<Phase>({ kind: "board" });
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Load ads on mount and listen for updates
  useEffect(() => {
    void api.listAds().then(setAds).catch(() => {});
    const unlisten = listen<Ad[]>("ads-updated", (e) => setAds(e.payload));
    return () => { void unlisten.then((f) => f()); };
  }, []);

  // Cycle timer
  useEffect(() => {
    if (timerRef.current) clearTimeout(timerRef.current);

    const settings = snapshot?.settings;
    const enabled = settings?.adsEnabled && ads.length > 0;
    if (!enabled) {
      setPhase({ kind: "board" });
      return;
    }

    // Duration for current phase
    let duration: number;
    if (phase.kind === "board") {
      duration = (settings?.boardDurationSecs ?? 8) * 1000;
    } else {
      duration = (ads[phase.index]?.durationSecs ?? 10) * 1000;
    }

    timerRef.current = setTimeout(() => {
      setPhase((p) => nextPhase(p, ads.length));
    }, duration);

    return () => { if (timerRef.current) clearTimeout(timerRef.current); };
  }, [phase, ads, snapshot?.settings?.adsEnabled, snapshot?.settings?.boardDurationSecs]);

  if (error) {
    return (
      <div className="dir-rtl flex h-screen items-center justify-center bg-slate-100 p-8 text-center text-red-600">
        <div className="text-xl font-bold">
          حدث خطأ أثناء تحميل البيانات:
          <br />
          {error}
        </div>
      </div>
    );
  }

  if (loading || !snapshot) {
    return (
      <div className="dir-rtl flex h-screen items-center justify-center bg-slate-100 text-slate-500">
        <div className="text-3xl font-black">جاري التحميل...</div>
      </div>
    );
  }

  const adsEnabled = snapshot.settings.adsEnabled && ads.length > 0;

  return (
    <div className="relative h-screen w-screen overflow-hidden">
      {/* Waiting board — always rendered, fades in/out */}
      <div
        className={cn(
          "absolute inset-0 transition-opacity duration-700",
          !adsEnabled || phase.kind === "board" ? "opacity-100" : "opacity-0 pointer-events-none",
        )}
      >
        <WaitingBoardView
          board={snapshot.board}
          bays={snapshot.bays}
          locale={locale}
          logoPath={snapshot.settings.logoPath}
          layout={snapshot.settings.waitingLayout}
          onLayoutChange={(waitingLayout) => {
            void api.saveSettings({ ...snapshot.settings, waitingLayout }).catch(() => {});
          }}
        />
      </div>

      {/* Ad slides */}
      {adsEnabled &&
        ads.map((ad, i) => (
          <AdSlide key={ad.id} ad={ad} visible={phase.kind === "ad" && phase.index === i} />
        ))}
    </div>
  );
}
