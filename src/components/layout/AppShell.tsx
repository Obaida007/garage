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
  const { locale, snapshot } = useGarage();
  return (
    <div className="flex min-h-screen bg-slate-100">
      <aside className="flex w-56 flex-col border-l bg-slate-900 p-4 text-white">
        <div className="mb-8 px-2">
          <div className="text-lg font-black">{snapshot?.settings.garageName ?? t(locale, "appName")}</div>
          <div className="text-xs text-slate-300">Al-Sahil Garage</div>
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
                  isActive ? "bg-sky-600 text-white" : "text-slate-200 hover:bg-slate-800",
                )
              }
            >
              {t(locale, link.key)}
            </NavLink>
          ))}
        </nav>
      </aside>
      <main className="flex-1 overflow-auto p-6">
        <Outlet />
      </main>
    </div>
  );
}
