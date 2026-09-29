import { useEffect, useState } from "react";
import QRCode from "qrcode";
import { api } from "@/services/tauri";

type Props = {
  /** رسالة تظهر فوق الـ QR */
  hint?: string;
};

export function MobileQrPanel({ hint }: Props) {
  const [urls, setUrls] = useState<{ url: string; qrSrc: string }[]>([]);
  const [current, setCurrent] = useState(0);

  useEffect(() => {
    let cancelled = false;
    void (async () => {
      try {
        const [ips, port] = await Promise.all([api.localIps(), api.mobilePort()]);
        const built: { url: string; qrSrc: string }[] = [];
        for (const ip of ips) {
          const url = `http://${ip}:${port}`;
          const qrSrc = await QRCode.toDataURL(url, {
            width: 200,
            margin: 1,
            color: { dark: "#0f172a", light: "#ffffff" },
          });
          built.push({ url, qrSrc });
        }
        if (!cancelled) setUrls(built);
      } catch {
        // demo mode or no network
      }
    })();
    return () => { cancelled = true; };
  }, []);

  if (urls.length === 0) return null;

  const entry = urls[current];

  return (
    <div className="flex flex-col items-center gap-2 rounded-xl border bg-white p-4 shadow-sm">
      {hint && <p className="text-center text-xs text-muted-foreground">{hint}</p>}
      <img
        src={entry.qrSrc}
        alt="QR"
        className="h-[200px] w-[200px] rounded-lg"
        draggable={false}
      />
      <p className="select-all font-mono text-xs text-slate-600">{entry.url}</p>
      {urls.length > 1 && (
        <div className="flex items-center gap-3">
          <button
            onClick={() => setCurrent((c) => (c - 1 + urls.length) % urls.length)}
            className="rounded-full border border-slate-200 bg-white px-3 py-1 text-lg font-bold text-slate-600 shadow-sm hover:bg-slate-50 active:bg-slate-100"
            aria-label="السابق"
          >
            ‹
          </button>
          <span className="text-xs text-slate-500">
            {current + 1} / {urls.length}
          </span>
          <button
            onClick={() => setCurrent((c) => (c + 1) % urls.length)}
            className="rounded-full border border-slate-200 bg-white px-3 py-1 text-lg font-bold text-slate-600 shadow-sm hover:bg-slate-50 active:bg-slate-100"
            aria-label="التالي"
          >
            ›
          </button>
        </div>
      )}
    </div>
  );
}
