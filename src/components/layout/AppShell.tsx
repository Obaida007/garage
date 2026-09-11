import { NavLink, Outlet } from "react-router-dom";
import { t } from "@/utils/i18n";
import { useGarage } from "@/hooks/useGarage";
import { cn } from "@/lib/utils";

const links = [
  { to: "/", key: "cashier" as const },
  { to: "/tickets", key: "tickets" as const },
  { to: "/reports", key: "reports" as const },
  { to: "/settings", key: "settings" as const },
];

export function AppShell() {
  const { locale } = useGarage();
  return (
    <div className="flex min-h-screen bg-slate-100">
      <aside className="flex w-56 flex-col border-l bg-slate-900 p-4 text-white">
        <div className="mb-8 px-2 flex flex-col items-center gap-3">
          <img
            src="/logo.png"
            alt="Logo"
            className="w-24 h-24 object-contain drop-shadow-md"
          />
          <div className="text-center">
            {/* <div className="text-xl font-black text-amber-400">{snapshot?.settings.garageName ?? t(locale, "appName")}</div> */}
            <div className="text-xs text-slate-300">
              نظام إدارة الأدوار والخدمة
            </div>
          </div>
        </div>
        <nav className="flex flex-1 flex-col gap-2">
          {links.map((link) => (
            <NavLink
              key={link.to}
              to={link.to}
              end={link.to === "/"}
              className={({ isActive }) =>
                cn(
                  "rounded-lg px-4 py-3 text-right text-base font-bold",
                  isActive
                    ? "bg-sky-600 text-white"
                    : "text-slate-200 hover:bg-slate-800",
                )
              }
            >
              {t(locale, link.key)}
            </NavLink>
          ))}
          <button
            type="button"
            onClick={async () => {
              try {
                const { WebviewWindow } = await import("@tauri-apps/api/webviewWindow");
                const waitingWindow = new WebviewWindow("waiting-board", {
                  url: "/#/waiting",
                  title: "شاشة الانتظار",
                  width: 1280,
                  height: 720,
                  center: true,
                });
                
                waitingWindow.once("tauri://error", (e) => {
                  console.error("Window creation error:", e);
                  // fallback
                  window.open("/#/waiting", "_blank", "width=1280,height=720");
                });
              } catch (e) {
                window.open("/#/waiting", "_blank", "width=1280,height=720");
              }
            }}
            className="mt-4 rounded-lg border border-amber-500/40 bg-amber-950/40 px-4 py-3 text-right text-base font-bold text-amber-300 hover:bg-amber-900/50 transition-all flex items-center justify-between"
          >
            <span>شاشة الانتظار</span>
            <span className="text-xs bg-amber-500 text-slate-950 px-2 py-0.5 rounded font-black">
              فتح ↗
            </span>
          </button>
        </nav>
      </aside>
      <main className="flex-1 overflow-auto p-6">
        <Outlet />
      </main>
    </div>
  );
}
