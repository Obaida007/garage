import { useEffect } from "react";
import { NavLink, Outlet } from "react-router-dom";
import { convertFileSrc } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { t } from "@/utils/i18n";
import { useGarage } from "@/hooks/useGarage";
import { cn } from "@/lib/utils";
import { SetupWizard } from "@/features/setup/SetupWizard";

/** يحوّل صورة الشعار (أي صيغة) إلى PNG مربّع بخلفية شفافة ليصلح كأيقونة نافذة/شريط مهام. */
async function logoToIconPng(logoUrl: string): Promise<Uint8Array | null> {
  const img = new Image();
  img.src = logoUrl;
  await new Promise<void>((resolve, reject) => {
    img.onload = () => resolve();
    img.onerror = () => reject(new Error("failed to load logo image"));
  });

  const size = 256;
  const canvas = document.createElement("canvas");
  canvas.width = size;
  canvas.height = size;
  const ctx = canvas.getContext("2d");
  if (!ctx) return null;

  const scale = Math.min(size / img.naturalWidth, size / img.naturalHeight);
  const w = img.naturalWidth * scale;
  const h = img.naturalHeight * scale;
  ctx.clearRect(0, 0, size, size);
  ctx.drawImage(img, (size - w) / 2, (size - h) / 2, w, h);

  const blob = await new Promise<Blob | null>((resolve) => canvas.toBlob(resolve, "image/png"));
  if (!blob) return null;
  return new Uint8Array(await blob.arrayBuffer());
}

const links = [
  { to: "/", key: "cashier" as const },
  { to: "/tickets", key: "tickets" as const },
  { to: "/reports", key: "reports" as const },
  { to: "/settings", key: "settings" as const },
];

export function AppShell() {
  const { locale, snapshot, loading } = useGarage();
  const logoPath = snapshot?.settings.logoPath;
  const logoSrc = logoPath ? convertFileSrc(logoPath) : "/logo.png";

  useEffect(() => {
    if (!logoPath) return;
    let cancelled = false;
    void (async () => {
      try {
        const icon = await logoToIconPng(convertFileSrc(logoPath));
        if (!cancelled && icon) {
          await getCurrentWindow().setIcon(icon);
        }
      } catch (error) {
        console.warn("تعذر تحديث أيقونة النافذة من الشعار", error);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [logoPath]);

  if (loading) {
    return (
      <div className="flex h-screen items-center justify-center bg-slate-100 text-lg text-slate-500">
        جاري التحميل...
      </div>
    );
  }

  if (snapshot && !snapshot.settings.setupCompleted) {
    return <SetupWizard initialSettings={snapshot.settings} locale={locale} />;
  }

  return (
    <div className="flex h-screen overflow-hidden bg-slate-100">
      <aside className="flex w-56 shrink-0 flex-col overflow-y-auto border-l bg-slate-900 p-4 text-white">
        <div className="mb-8 px-2 flex flex-col items-center gap-3">
          <img
            src={logoSrc}
            alt="Logo"
            className="w-24 h-24 object-contain drop-shadow-md"
          />
          <div className="text-center">
            <div className="text-xl font-black text-amber-400">
              {snapshot?.settings.garageName || t(locale, "appName")}
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
  
        </nav>
        <div className="mt-4 flex flex-col items-center gap-1.5 border-t border-slate-800 pt-4 text-slate-400">
          <img
            src="/OS_logo_transparent.png"
            alt="OS Development"
            className="h-20 w-auto object-contain opacity-80"
            onError={(e) => {
              e.currentTarget.style.display = "none";
            }}
          />
          <div className="text-[11px] font-medium">تم تطويره بواسطة OS </div>
        </div>
      </aside>
      <main className="flex-1 overflow-auto p-6">
        <Outlet />
      </main>
    </div>
  );
}
