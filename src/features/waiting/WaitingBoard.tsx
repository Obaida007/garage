import { useState, useEffect } from "react";
import type { Bay, WaitingBoard } from "@/types";
import { speakTicketCall } from "@/utils/audio";

type Props = {
  board: WaitingBoard;
  bays?: Bay[];
};

export function WaitingBoardView({ board, bays }: Props) {
  // Theme state: 'light' by default, persisted in localStorage
  const [theme, setTheme] = useState<"light" | "dark">(() => {
    return (
      (localStorage.getItem("waiting_theme") as "light" | "dark") || "light"
    );
  });

  useEffect(() => {
    localStorage.setItem("waiting_theme", theme);
  }, [theme]);

  // Combine board.inService or bays to show all active services in clean rows
  const inServiceItems =
    board.inService.length > 0
      ? board.inService
      : (bays
          ?.filter((b) => b.status === "BUSY" && b.currentTicket)
          .map((b) => ({
            ticketNumber: b.currentTicket!.ticketNumber,
            bayId: b.id,
          })) ?? []);

  const isLight = theme === "light";

  return (
    <div
      className={`flex h-screen overflow-hidden flex-col font-sans dir-rtl transition-colors duration-300 ${
        isLight ? "bg-slate-100 text-slate-900" : "bg-slate-950 text-white"
      }`}
    >
      {/* Header */}
      <header
        className={`px-8 py-5 flex items-center justify-between shadow-md border-b transition-colors ${
          isLight
            ? "bg-white border-slate-200"
            : "bg-slate-900/90 border-slate-800"
        }`}
      >
        <div className="flex items-center gap-4">
          <img
            src="/logo.png"
            alt="Logo"
            className="w-16 h-16 object-contain drop-shadow-sm"
          />
          <div>
            <h1
              className={`text-4xl font-black tracking-wide ${
                isLight ? "text-amber-600" : "text-amber-400"
              }`}
            >
              {board.garageName || "كراج البارودي"}
            </h1>
            <p
              className={`text-lg mt-1 font-medium ${isLight ? "text-slate-600" : "text-slate-400"}`}
            >
              شاشة متابعة الأدوار والحفر
            </p>
          </div>
        </div>

        {/* Right Header Controls */}
        <div className="flex items-center gap-4">
          {/* Audio Test Button */}
          <button
            type="button"
            onClick={() => {
              const ticket = board.currentTicket || "004";
              const bay = board.currentBay || 4;
              speakTicketCall(ticket, bay);
            }}
            className={`flex items-center gap-2 px-5 py-2.5 rounded-2xl border font-bold text-base shadow-sm transition-all ${
              isLight
                ? "bg-amber-500 hover:bg-amber-600 border-amber-600 text-slate-950"
                : "bg-amber-500 hover:bg-amber-400 border-amber-400 text-slate-950"
            }`}
          >
            <span>🔊 اختبار الصوت</span>
          </button>

          {/* Theme Toggle Button */}
          <button
            type="button"
            onClick={() => setTheme(isLight ? "dark" : "light")}
            className={`flex items-center gap-2 px-5 py-2.5 rounded-2xl border font-bold text-base shadow-sm transition-all ${
              isLight
                ? "bg-slate-100 hover:bg-slate-200 border-slate-300 text-slate-800"
                : "bg-slate-800 hover:bg-slate-700 border-slate-700 text-amber-300"
            }`}
          >
            {isLight ? (
              <>
                <span>🌙 المظهر الداكن</span>
              </>
            ) : (
              <>
                <span>☀️ المظهر الفاتح</span>
              </>
            )}
          </button>

          {/* Queue Count Badge */}
          <div
            className={`flex items-center gap-3 px-6 py-2.5 rounded-2xl border ${
              isLight
                ? "bg-amber-50 border-amber-200 text-slate-900"
                : "bg-slate-800/80 border-slate-700 text-slate-300"
            }`}
          >
            <span className="text-lg font-bold">المنتظرون في الطابور:</span>
            <span
              className={`text-3xl font-black ${isLight ? "text-amber-600" : "text-amber-400"}`}
            >
              {board.waitingCount}
            </span>
          </div>
        </div>
      </header>

      {/* Main Rows Section: Active Pit Assignments */}
      <div className="flex-1 p-8 flex flex-col min-h-0">
        <div
          className={`rounded-3xl p-6 shadow-xl border transition-all ${
            isLight
              ? "bg-white border-slate-200"
              : "bg-slate-900/90 border-slate-800"
          }`}
        >
          <div
            className={`flex items-center justify-between pb-4 mb-6 border-b ${
              isLight ? "border-slate-200" : "border-slate-800"
            }`}
          >
            <h2
              className={`text-3xl font-black flex items-center gap-3 ${
                isLight ? "text-slate-900" : "text-amber-300"
              }`}
            >
              <span>الأدوار قيد الخدمة حالياً (حسب الحفر)</span>
            </h2>
            <span
              className={`text-lg font-semibold ${isLight ? "text-slate-500" : "text-slate-400"}`}
            >
              توزيع الحفر والأرقام الشغالة
            </span>
          </div>

          {/* Rows List (سطور) */}
          <div className="space-y-4">
            {bays && bays.length > 0 ? (
              bays.map((bay) => {
                const isBusy = bay.status === "BUSY" && bay.currentTicket;
                const isOutOfService = bay.status === "OUT_OF_SERVICE";
                const ticketNum = bay.currentTicket?.ticketNumber;

                return (
                  <div
                    key={bay.id}
                    className={`flex items-center justify-between rounded-2xl border p-5 transition-all ${
                      isBusy
                        ? isLight
                          ? "border-2 border-amber-400 bg-amber-50/80 shadow-md"
                          : "border-amber-500/50 bg-amber-950/20 shadow-md"
                        : isOutOfService
                          ? isLight
                            ? "border-red-200 bg-red-50/50 opacity-75"
                            : "border-red-900/50 bg-red-950/20 opacity-70"
                          : isLight
                            ? "border-slate-200 bg-slate-50/60 opacity-80"
                            : "border-slate-800 bg-slate-950/50 opacity-70"
                    }`}
                  >
                    {/* Pit Label */}
                    <div className="flex items-center gap-4">
                      <div
                        className={`flex h-16 w-32 items-center justify-center rounded-xl font-black text-2xl shadow-sm ${
                          isBusy
                            ? "bg-amber-500 text-slate-950"
                            : isOutOfService
                              ? "bg-slate-700 text-slate-200"
                              : isLight
                                ? "bg-slate-200 text-slate-700"
                                : "bg-slate-800 text-slate-400"
                        }`}
                      >
                        حفرة {bay.id}
                      </div>
                      <div>
                        <div
                          className={`text-xl font-bold ${
                            isLight ? "text-slate-900" : "text-white"
                          }`}
                        >
                          {bay.name || `حفرة ${bay.id}`}
                        </div>
                        <div
                          className={`text-sm mt-1 font-medium ${
                            isBusy
                              ? isLight
                                ? "text-amber-800 font-bold"
                                : "text-amber-400"
                              : isOutOfService
                                ? "text-red-600 font-bold"
                                : isLight
                                  ? "text-slate-500"
                                  : "text-slate-400"
                          }`}
                        >
                          {isBusy
                            ? "الحالة: قيد العمل والصيانة"
                            : isOutOfService
                              ? "الحالة: خارج الخدمة (مغلقة للصيانة)"
                              : "الحالة: جاهزة واستقبال"}
                        </div>
                      </div>
                    </div>

                    {/* Arrow / Connector */}
                    <div
                      className={`hidden md:flex items-center font-bold text-lg ${
                        isBusy
                          ? isLight
                            ? "text-amber-700"
                            : "text-amber-400"
                          : isOutOfService
                            ? "text-red-500"
                            : "text-slate-400"
                      }`}
                    >
                      {isBusy
                        ? "⟸ يخدم حالياً ⟸"
                        : isOutOfService
                          ? "✖ مغلقة للصيانة ✖"
                          : "⟸ متوفرة ⟸"}
                    </div>

                    {/* Ticket Number Assigned */}
                    <div className="flex items-center gap-4">
                      {isBusy && ticketNum ? (
                        <div
                          className={`flex items-center gap-3 px-6 py-3 rounded-xl border ${
                            isLight
                              ? "bg-white border-2 border-amber-400 text-slate-900 shadow-sm"
                              : "bg-slate-900 border border-amber-500/40 text-white"
                          }`}
                        >
                          <span
                            className={`text-lg font-bold ${isLight ? "text-slate-600" : "text-slate-400"}`}
                          >
                            رقم الدور:
                          </span>
                          <span
                            className={`text-4xl font-black tracking-wider ${
                              isLight ? "text-amber-600" : "text-amber-400"
                            }`}
                          >
                            {ticketNum}
                          </span>
                        </div>
                      ) : isOutOfService ? (
                        <div
                          className={`px-6 py-3 rounded-xl border text-xl font-bold ${
                            isLight
                              ? "bg-red-100 border-red-200 text-red-700"
                              : "bg-red-950/40 border-red-900 text-red-400"
                          }`}
                        >
                          خارج الخدمة
                        </div>
                      ) : (
                        <div
                          className={`px-6 py-3 rounded-xl border text-xl font-bold ${
                            isLight
                              ? "bg-slate-100 border-slate-200 text-slate-400"
                              : "bg-slate-900/60 border-slate-800 text-slate-500"
                          }`}
                        >
                          فارغة (جاهزة)
                        </div>
                      )}
                    </div>
                  </div>
                );
              })
            ) : inServiceItems.length > 0 ? (
              inServiceItems.map((item) => (
                <div
                  key={item.ticketNumber}
                  className={`flex items-center justify-between rounded-2xl border p-5 shadow-md ${
                    isLight
                      ? "border-2 border-amber-400 bg-amber-50/80 text-slate-900"
                      : "border-amber-500/50 bg-amber-950/20 text-white"
                  }`}
                >
                  <div className="flex items-center gap-4">
                    <div className="flex h-16 w-32 items-center justify-center rounded-xl bg-amber-500 text-slate-950 font-black text-2xl">
                      حفرة {item.bayId}
                    </div>
                    <span className="text-xl font-bold">
                      قيد الخدمة والصيانة
                    </span>
                  </div>
                  <div
                    className={`flex items-center gap-3 px-6 py-3 rounded-xl border ${
                      isLight
                        ? "bg-white border-amber-400"
                        : "bg-slate-900 border-amber-500/40"
                    }`}
                  >
                    <span className="text-lg font-bold">رقم الدور:</span>
                    <span className="text-4xl font-black text-amber-600 tracking-wider">
                      {item.ticketNumber}
                    </span>
                  </div>
                </div>
              ))
            ) : (
              <div
                className={`py-8 text-center text-2xl font-bold ${isLight ? "text-slate-400" : "text-slate-500"}`}
              >
                لا توجد أدوار قيد الخدمة حالياً
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
