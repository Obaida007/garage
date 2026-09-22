import { useEffect, useState } from "react";
import { LayoutGrid, Table2, Volume2 } from "lucide-react";
import { convertFileSrc } from "@tauri-apps/api/core";
import type { Bay, Locale, WaitingBoard } from "@/types";
import { cn } from "@/lib/utils";
import { speakTicketCall } from "@/utils/audio";
import { t } from "@/utils/i18n";

type Props = {
  board: WaitingBoard;
  bays: Bay[];
  locale: Locale;
  logoPath?: string;
  layout?: "cards" | "table";
  onLayoutChange?: (layout: "cards" | "table") => void;
};

/** عدد الأعمدة الأنسب لعدد الحفر بحيث تملأ البطاقات الشاشة كاملة دون تمرير. */
function gridColumns(count: number) {
  if (count <= 3) return Math.max(count, 1);
  if (count === 4) return 2;
  if (count <= 6) return 3;
  if (count <= 8) return 4;
  if (count === 9) return 3;
  if (count <= 12) return 4;
  return Math.ceil(Math.sqrt(count * 1.6));
}

/** حجم خط رقم الدور: يكبر مع ارتفاع البطاقة ويتقلّص تلقائياً إن كان الرقم طويلاً. */
function ticketFontSize(ticketNumber: string) {
  const widthCap = 86 / (Math.max(ticketNumber.length, 2) * 0.68);
  return `min(50cqh, ${widthCap.toFixed(1)}cqw)`;
}

function Stat({ label, value }: { label: string; value: string | number }) {
  return (
    <div className="flex items-baseline gap-3 rounded-2xl bg-slate-100 px-5 py-2">
      <span className="text-lg font-bold text-slate-500">{label}</span>
      <span className="text-4xl font-black text-slate-900">{value}</span>
    </div>
  );
}

function BayTile({ bay, locale }: { bay: Bay; locale: Locale }) {
  const ticket = bay.currentTicket?.ticketNumber;
  const busy = bay.status === "BUSY" && !!ticket;
  const outOfService = bay.status === "OUT_OF_SERVICE";
  const name = bay.name || `${t(locale, "bay")} ${bay.id}`;

  return (
    <div
      className={cn(
        "flex min-h-0 min-w-0 flex-col overflow-hidden rounded-3xl border-4 bg-white [container-type:size]",
        busy
          ? "border-primary shadow-xl"
          : outOfService
            ? "border-red-200 bg-red-50/60"
            : "border-slate-200",
      )}
    >
      <div
        className={cn(
          "flex items-center justify-center px-4 text-center font-black leading-tight",
          busy
            ? "bg-primary text-primary-foreground"
            : outOfService
              ? "bg-red-100 text-red-800"
              : "bg-slate-100 text-slate-500",
        )}
        style={{ fontSize: "min(8cqh, 6cqw)", minHeight: "24cqh" }}
      >
        <span className="line-clamp-2">{name}</span>
      </div>

      <div className="flex min-h-0 flex-1 items-center justify-center px-2">
        {busy && ticket ? (
          <span
            className="font-black leading-none tracking-wide text-slate-900"
            style={{ fontSize: ticketFontSize(ticket) }}
          >
            {ticket}
          </span>
        ) : (
          <span
            className={cn(
              "font-bold",
              outOfService ? "text-red-600" : "text-slate-400",
            )}
            style={{ fontSize: "min(12cqh, 10cqw)" }}
          >
            {outOfService ? t(locale, "outOfService") : t(locale, "ready")}
          </span>
        )}
      </div>
    </div>
  );
}

function BayTable({ bays, locale }: { bays: Bay[]; locale: Locale }) {
  return (
    <div className="flex min-h-0 flex-1 flex-col overflow-hidden rounded-3xl border-2 border-slate-200 bg-white shadow-md">
      <div className="grid grid-cols-[2fr_3fr] bg-primary px-8 py-3 text-xl font-black text-primary-foreground">
        <span>{t(locale, "assignedBayColumn").trim()}</span>
        <span className="text-center">{t(locale, "ticketNumberColumn")}</span>
      </div>
      <div
        className="grid min-h-0 flex-1"
        style={{ gridTemplateRows: `repeat(${bays.length}, minmax(0, 1fr))` }}
      >
        {bays.map((bay, index) => {
          const ticket = bay.currentTicket?.ticketNumber;
          const busy = bay.status === "BUSY" && !!ticket;
          const outOfService = bay.status === "OUT_OF_SERVICE";
          const name = bay.name || `${t(locale, "bay")} ${bay.id}`;
          const widthCap = 50 / (Math.max(ticket?.length ?? 3, 2) * 0.68);

          return (
            <div
              key={bay.id}
              className={cn(
                "grid min-h-0 grid-cols-[2fr_3fr] items-center border-s-8 px-8 [container-type:size]",
                index > 0 && "border-t border-t-slate-200",
                index % 2 === 1 ? "bg-slate-50" : "bg-white",
                busy ? "border-s-primary" : "border-s-transparent",
              )}
            >
              <span
                className={cn("truncate font-bold", busy ? "text-slate-900" : "text-slate-500")}
                style={{ fontSize: "min(38cqh, 3.2cqw)" }}
              >
                {name}
              </span>
              {busy && ticket ? (
                <span
                  className="text-center font-black leading-none tracking-wide text-slate-900"
                  style={{ fontSize: `min(72cqh, ${widthCap.toFixed(1)}cqw)` }}
                >
                  {ticket}
                </span>
              ) : (
                <span
                  className={cn(
                    "text-center font-bold",
                    outOfService ? "text-red-500" : "text-slate-300",
                  )}
                  style={{ fontSize: "min(38cqh, 3cqw)" }}
                >
                  {outOfService ? t(locale, "outOfService") : t(locale, "ready")}
                </span>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}

export function WaitingBoardView({
  board,
  bays,
  locale,
  logoPath,
  layout: layoutProp = "cards",
  onLayoutChange,
}: Props) {
  const [layout, setLayout] = useState(layoutProp);
  useEffect(() => setLayout(layoutProp), [layoutProp]);

  function toggleLayout() {
    const next = layout === "cards" ? "table" : "cards";
    setLayout(next);
    onLayoutChange?.(next);
  }

  const activeBays = bays.filter((bay) => bay.active);
  const columns = gridColumns(activeBays.length);
  const rows = Math.max(1, Math.ceil(activeBays.length / columns));
  const logoSrc = logoPath ? convertFileSrc(logoPath) : null;

  function testAudio() {
    const calledBay = activeBays.find((bay) => bay.id === board.currentBay) ?? activeBays[0];
    const bayLabel = calledBay?.name || t(locale, "bay");
    speakTicketCall(board.currentTicket ?? "001", bayLabel);
  }

  return (
    <div className="dir-rtl flex h-screen flex-col overflow-hidden bg-slate-100 font-sans text-slate-900">
      <header className="flex items-center justify-between gap-6 border-b border-slate-200 bg-white px-8 py-4 shadow-sm">
        <div className="flex min-w-0 items-center gap-4">
          {logoSrc && (
            <img
              src={logoSrc}
              alt=""
              className="h-14 w-auto max-w-[220px] object-contain"
              onError={(e) => {
                e.currentTarget.style.display = "none";
              }}
            />
          )}
          <h1 className="truncate text-4xl font-black tracking-wide text-slate-900">
            {board.garageName || t(locale, "appName")}
          </h1>
        </div>

        <div className="flex shrink-0 items-center gap-4">
          {board.nextTicket && <Stat label={t(locale, "nextTicket")} value={board.nextTicket} />}
          <Stat label={t(locale, "waiting")} value={board.waitingCount} />
          <button
            type="button"
            onClick={toggleLayout}
            title={t(locale, layout === "cards" ? "layoutTable" : "layoutCards")}
            aria-label={t(locale, layout === "cards" ? "layoutTable" : "layoutCards")}
            className="flex h-12 w-12 items-center justify-center rounded-full border border-slate-200 bg-white text-slate-400 transition-colors hover:bg-slate-100 hover:text-slate-700"
          >
            {layout === "cards" ? <Table2 className="h-5 w-5" /> : <LayoutGrid className="h-5 w-5" />}
          </button>
          <button
            type="button"
            onClick={testAudio}
            title={t(locale, "testAudio")}
            aria-label={t(locale, "testAudio")}
            className="flex h-12 w-12 items-center justify-center rounded-full border border-slate-200 bg-white text-slate-400 transition-colors hover:bg-slate-100 hover:text-slate-700"
          >
            <Volume2 className="h-5 w-5" />
          </button>
        </div>
      </header>

      <main className="flex min-h-0 flex-1 p-6">
        {activeBays.length === 0 ? (
          <div className="flex flex-1 items-center justify-center text-3xl font-bold text-slate-400">
            {t(locale, "noInServiceTickets")}
          </div>
        ) : layout === "table" ? (
          <BayTable bays={activeBays} locale={locale} />
        ) : (
          <div
            className="grid flex-1 gap-5"
            style={{
              gridTemplateColumns: `repeat(${columns}, minmax(0, 1fr))`,
              gridTemplateRows: `repeat(${rows}, minmax(0, 1fr))`,
            }}
          >
            {activeBays.map((bay) => (
              <BayTile key={bay.id} bay={bay} locale={locale} />
            ))}
          </div>
        )}
      </main>
    </div>
  );
}
