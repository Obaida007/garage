import { useEffect, useState } from "react";
import { toast } from "sonner";
import { save, open } from "@tauri-apps/plugin-dialog";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { useGarage } from "@/hooks/useGarage";
import { api } from "@/services/tauri";
import { backupFileName } from "@/utils/format";
import { t } from "@/utils/i18n";
import { useGarageStore } from "@/stores/garageStore";
import type { AppSettings, MonitorInfo, PrinterInfo } from "@/types";

export function SettingsPage() {
  const { locale, snapshot } = useGarage();
  const setLocale = useGarageStore((s) => s.setLocale);
  const [settings, setSettings] = useState<AppSettings | null>(snapshot?.settings ?? null);
  const [printers, setPrinters] = useState<PrinterInfo[]>([]);
  const [monitors, setMonitors] = useState<MonitorInfo[]>([]);
  const [autostart, setAutostart] = useState(false);
  const [confirm, setConfirm] = useState<"reset" | "restore" | null>(null);
  const [restorePath, setRestorePath] = useState<string | null>(null);

  useEffect(() => {
    void (async () => {
      try {
        setSettings(await api.getSettings());
        setPrinters(await api.printers());
        setMonitors(await api.monitors());
        setAutostart(await api.isAutostart());
      } catch (error) {
        toast.error(String(error));
      }
    })();
  }, []);

  if (!settings) return <div>جاري التحميل...</div>;

  function patch<K extends keyof AppSettings>(key: K, value: AppSettings[K]) {
    setSettings((prev) => (prev ? { ...prev, [key]: value } : prev));
  }

  return (
    <div className="grid gap-6 xl:grid-cols-2">
      <Card>
        <CardHeader>
          <CardTitle>{t(locale, "garageSettings")}</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <Label>{t(locale, "garageName")}</Label>
          <Input value={settings.garageName} onChange={(e) => patch("garageName", e.target.value)} />
          <Label>{t(locale, "printHeader")}</Label>
          <Input value={settings.printHeader} onChange={(e) => patch("printHeader", e.target.value)} />
          <div className="flex items-center justify-between pt-2">
            <Label>English UI</Label>
            <Switch checked={locale === "en"} onCheckedChange={(v) => setLocale(v ? "en" : "ar")} />
          </div>
          <div className="flex items-center justify-between rounded-lg border p-3 bg-muted/40">
            <div>
              <div className="font-semibold text-sm">{t(locale, "autoAssignMode")}</div>
              <div className="text-xs text-muted-foreground mt-0.5">
                {settings.autoAssign ? t(locale, "autoAssignDesc") : t(locale, "manualAssignDesc")}
              </div>
            </div>
            <Switch
              checked={settings.autoAssign}
              onCheckedChange={(v) => patch("autoAssign", v)}
            />
          </div>
        </CardContent>
      </Card>


      <Card>
        <CardHeader>
          <CardTitle>{t(locale, "printerSettings")}</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <Label>الطابعة</Label>
          <select
            className="h-11 w-full rounded-md border bg-background px-3"
            value={settings.printerName}
            onChange={(e) => patch("printerName", e.target.value)}
          >
            <option value="">افتراضية</option>
            {printers.map((printer) => (
              <option key={printer.name} value={printer.name}>
                {printer.name}
                {printer.isDefault ? " (default)" : ""}
              </option>
            ))}
          </select>
          <Label>حجم الورق</Label>
          <div className="flex gap-2">
            <Button
              variant={settings.paperWidthMm === 58 ? "default" : "outline"}
              onClick={() => patch("paperWidthMm", 58)}
            >
              58mm
            </Button>
            <Button
              variant={settings.paperWidthMm === 80 ? "default" : "outline"}
              onClick={() => patch("paperWidthMm", 80)}
            >
              80mm
            </Button>
          </div>
          <div className="flex gap-2">
            <Button variant="outline" onClick={() => api.testPrint().catch((e) => toast.error(String(e)))}>
              {t(locale, "testPrint")}
            </Button>
            <Button variant="outline" onClick={() => api.reprintLast().catch((e) => toast.error(String(e)))}>
              {t(locale, "reprintLast")}
            </Button>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>{t(locale, "ticketSettings")}</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <Label>{t(locale, "prefix")}</Label>
          <Input value={settings.ticketPrefix} onChange={(e) => patch("ticketPrefix", e.target.value)} />
          <Label>{t(locale, "nextNumber")}</Label>
          <Input
            type="number"
            value={settings.nextSequence}
            onChange={(e) => patch("nextSequence", Number(e.target.value))}
          />
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>{t(locale, "displaySettings")}</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <Label>شاشة الانتظار</Label>
          <select
            className="h-11 w-full rounded-md border bg-background px-3"
            value={settings.waitingMonitorId}
            onChange={(e) => patch("waitingMonitorId", e.target.value)}
          >
            <option value="">تلقائي (الشاشة الثانية)</option>
            {monitors.map((monitor) => (
              <option key={monitor.id} value={monitor.id}>
                {monitor.name} {monitor.isPrimary ? "(رئيسية)" : ""} {monitor.width}x{monitor.height}
              </option>
            ))}
          </select>
          <div className="flex items-center justify-between">
            <Label>{t(locale, "fullscreen")}</Label>
            <Switch
              checked={settings.waitingFullscreen}
              onCheckedChange={(v) => patch("waitingFullscreen", v)}
            />
          </div>
          <Button variant="outline" onClick={() => api.testDisplay().catch((e) => toast.error(String(e)))}>
            {t(locale, "testDisplay")}
          </Button>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Windows</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <Label>{t(locale, "autostart")}</Label>
            <Switch
              checked={autostart}
              onCheckedChange={async (v) => {
                try {
                  setAutostart(await api.setAutostart(v));
                } catch (error) {
                  toast.error(String(error));
                }
              }}
            />
          </div>
          <div className="flex flex-wrap gap-2">
            <Button
              onClick={async () => {
                const dest = await save({
                  defaultPath: backupFileName(),
                  filters: [{ name: "SQLite", extensions: ["db"] }],
                });
                if (typeof dest === "string") {
                  try {
                    await api.backup(dest);
                    toast.success(t(locale, "backup"));
                  } catch (error) {
                    toast.error(String(error));
                  }
                }
              }}
            >
              {t(locale, "backup")}
            </Button>
            <Button
              variant="outline"
              onClick={async () => {
                const source = await open({
                  filters: [{ name: "SQLite", extensions: ["db"] }],
                });
                if (typeof source === "string") {
                  setRestorePath(source);
                  setConfirm("restore");
                }
              }}
            >
              {t(locale, "restore")}
            </Button>
            <Button variant="destructive" onClick={() => setConfirm("reset")}>
              {t(locale, "resetQueue")}
            </Button>
          </div>
        </CardContent>
      </Card>

      <div className="xl:col-span-2">
        <Button
          size="lg"
          onClick={async () => {
            try {
              await api.saveSettings(settings);
              toast.success(t(locale, "save"));
            } catch (error) {
              toast.error(String(error));
            }
          }}
        >
          {t(locale, "save")}
        </Button>
      </div>

      <AlertDialog open={confirm !== null} onOpenChange={() => setConfirm(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t(locale, "confirm")}</AlertDialogTitle>
            <AlertDialogDescription>
              {confirm === "reset" ? t(locale, "confirmReset") : t(locale, "confirmRestore")}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t(locale, "close")}</AlertDialogCancel>
            <AlertDialogAction
              onClick={async () => {
                try {
                  if (confirm === "reset") await api.resetQueue();
                  if (confirm === "restore" && restorePath) await api.restore(restorePath);
                } catch (error) {
                  toast.error(String(error));
                }
                setConfirm(null);
              }}
            >
              {t(locale, "confirm")}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}
