import { useEffect, useState } from "react";
import { toast } from "sonner";
import { save, open } from "@tauri-apps/plugin-dialog";
import { convertFileSrc } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Separator } from "@/components/ui/separator";
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

function SectionHeading({ children }: { children: React.ReactNode }) {
  return (
    <div className="space-y-2">
      <h2 className="text-xs font-bold uppercase tracking-wider text-muted-foreground">
        {children}
      </h2>
      <Separator />
    </div>
  );
}

export function SettingsPage() {
  const { locale, snapshot } = useGarage();
  const settingsUnlocked = useGarageStore((s) => s.settingsUnlocked);
  const unlockSettings = useGarageStore((s) => s.unlockSettings);
  const lockSettings = useGarageStore((s) => s.lockSettings);
  const [settings, setSettings] = useState<AppSettings | null>(snapshot?.settings ?? null);
  const [printers, setPrinters] = useState<PrinterInfo[]>([]);
  const [monitors, setMonitors] = useState<MonitorInfo[]>([]);
  const [autostart, setAutostart] = useState(false);
  const [dbPath, setDbPath] = useState("");
  const [confirm, setConfirm] = useState<"reset" | "restore" | "numbering" | null>(null);
  const [restorePath, setRestorePath] = useState<string | null>(null);
  const [bayNames, setBayNames] = useState<Record<number, string>>({});
  const [passwordInput, setPasswordInput] = useState("");
  const [newBayName, setNewBayName] = useState("");
  const [localIps, setLocalIps] = useState<string[]>([]);
  const [mobilePort, setMobilePort] = useState(7878);
  const [ads, setAds] = useState<import("@/types").Ad[]>([]);
  const [adUploading, setAdUploading] = useState(false);

  // تُقفل الإعدادات من جديد عند مغادرة الصفحة، فيُطلب إدخال كلمة المرور في كل مرة تُفتح فيها.
  useEffect(() => lockSettings, [lockSettings]);

  useEffect(() => {
    void (async () => {
      try {
        setSettings(await api.getSettings());
        setPrinters(await api.printers());
        setMonitors(await api.monitors());
        setAutostart(await api.isAutostart());
        setDbPath(await api.dbPath());
        setLocalIps(await api.localIps().catch(() => []));
        setMobilePort(await api.mobilePort().catch(() => 7878));
        setAds(await api.listAds().catch(() => []));
      } catch (error) {
        toast.error(String(error));
      }
    })();
  }, []);

  if (!settings) return <div>جاري التحميل...</div>;

  function patch<K extends keyof AppSettings>(key: K, value: AppSettings[K]) {
    setSettings((prev) => (prev ? { ...prev, [key]: value } : prev));
  }

  function tryUnlock() {
    if (!settings) return;
    if (passwordInput === settings.settingsPassword) {
      setPasswordInput("");
      unlockSettings();
    } else {
      setPasswordInput("");
      toast.error(t(locale, "wrongPassword"));
    }
  }

  if (settings.settingsPassword && !settingsUnlocked) {
    return (
      <div className="mx-auto max-w-sm pt-24">
        <Card>
          <CardHeader>
            <CardTitle>{t(locale, "securitySettings")}</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            <p className="text-sm text-muted-foreground">{t(locale, "enterSettingsPassword")}</p>
            <Input
              type="password"
              autoFocus
              value={passwordInput}
              onChange={(e) => setPasswordInput(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") tryUnlock();
              }}
            />
            <Button className="w-full" onClick={tryUnlock}>
              {t(locale, "unlock")}
            </Button>
          </CardContent>
        </Card>
      </div>
    );
  }

  async function saveSettings() {
    if (!settings) return;
    if (settings.priorityEnabled && !settings.prioritySuffix.trim()) {
      toast.error(t(locale, "priorityEmptySuffixError"));
      return;
    }
    try {
      await api.saveSettings(settings);
      toast.success(t(locale, "save"));
    } catch (error) {
      toast.error(String(error));
    }
  }

  return (
    <div className="space-y-8 pb-4">
      {/* ── شريط علوي ثابت: العنوان + لغة الواجهة + زر الحفظ ── */}
      <div className="sticky top-0 z-10 -mx-6 -mt-6 flex items-center justify-between border-b bg-slate-100/95 px-6 py-4 backdrop-blur">
        <h1 className="text-2xl font-black text-slate-900">{t(locale, "settings")}</h1>
        
          <Button size="lg" onClick={() => void saveSettings()}>
            {t(locale, "save")}
          </Button>
      </div>

      {/* ── الهوية والعلامة التجارية ── */}
      <section className="space-y-4">
        <SectionHeading>{t(locale, "sectionIdentity")}</SectionHeading>
        <Card>
          <CardHeader>
            <CardTitle>{t(locale, "garageSettings")}</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            <Label>{t(locale, "garageName")}</Label>
            <Input value={settings.garageName} onChange={(e) => patch("garageName", e.target.value)} />
            <Label>{t(locale, "printHeader")}</Label>
            <Input value={settings.printHeader} onChange={(e) => patch("printHeader", e.target.value)} />

            <Separator className="my-1" />

            <Label>{t(locale, "logoSettings")}</Label>
            <p className="text-xs text-muted-foreground">{t(locale, "logoHint")}</p>
            {settings.logoPath ? (
              <img
                src={convertFileSrc(settings.logoPath)}
                alt="logo"
                className="h-36 max-w-full rounded border bg-white object-contain p-2"
              />
            ) : null}
            <div className="flex gap-2">
              <Button
                variant="outline"
                onClick={async () => {
                  const source = await open({
                    filters: [{ name: "Image", extensions: ["png", "jpg", "jpeg"] }],
                  });
                  if (typeof source === "string") {
                    try {
                      const path = await api.uploadLogo(source);
                      patch("logoPath", path);
                    } catch (error) {
                      toast.error(String(error));
                    }
                  }
                }}
              >
                {t(locale, "chooseLogo")}
              </Button>
              {settings.logoPath ? (
                <Button variant="destructive" onClick={() => patch("logoPath", "")}>
                  {t(locale, "removeLogo")}
                </Button>
              ) : null}
            </div>
          </CardContent>
        </Card>
      </section>

      {/* ── الطابور والأدوار ── */}
      <section className="space-y-4">
        <SectionHeading>{t(locale, "sectionQueue")}</SectionHeading>

        <Card>
          <CardHeader>
            <CardTitle>{t(locale, "ticketSettings")}</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
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

            <div className="grid gap-4 md:grid-cols-2">
              <div className="space-y-3 rounded-lg border p-3">
                <Label className="text-xs font-bold uppercase text-muted-foreground">
                  {t(locale, "regularNumberingTitle")}
                </Label>
                <Label>{t(locale, "prefix")}</Label>
                <Input
                  value={settings.ticketPrefix}
                  onChange={(e) => patch("ticketPrefix", e.target.value)}
                />
                <Label>{t(locale, "nextNumber")}</Label>
                <Input
                  type="number"
                  value={settings.nextSequence}
                  onChange={(e) => patch("nextSequence", Number(e.target.value))}
                />
              </div>

              <div className="space-y-3 rounded-lg border p-3">
                <div className="flex items-center justify-between">
                  <Label className="text-xs font-bold uppercase text-muted-foreground">
                    {t(locale, "priorityNumberingTitle")}
                  </Label>
                  <Switch
                    checked={settings.priorityEnabled}
                    onCheckedChange={(v) => patch("priorityEnabled", v)}
                  />
                </div>
                <p className="text-xs text-muted-foreground">{t(locale, "priorityEnabledHint")}</p>
                <Label>{t(locale, "prioritySuffixLabel")}</Label>
                <Input
                  value={settings.prioritySuffix}
                  onChange={(e) => patch("prioritySuffix", e.target.value)}
                />
                <Label>{t(locale, "nextPriorityNumber")}</Label>
                <Input
                  type="number"
                  value={settings.nextPrioritySequence}
                  onChange={(e) => patch("nextPrioritySequence", Number(e.target.value))}
                />
              </div>
            </div>

            <div className="flex items-center justify-between gap-4 rounded-lg border p-3">
              <p className="text-xs text-muted-foreground">{t(locale, "resetNumberingHint")}</p>
              <Button variant="destructive" onClick={() => setConfirm("numbering")}>
                {t(locale, "resetNumbering")}
              </Button>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>{t(locale, "bayNamesTitle")}</CardTitle>
          </CardHeader>
          <CardContent className="space-y-2">
            {(snapshot?.bays ?? [])
              .filter((bay) => bay.active)
              .map((bay) => (
                <div key={bay.id} className="flex items-center gap-2">
                  <Input
                    value={bayNames[bay.id] ?? bay.name}
                    onChange={(e) =>
                      setBayNames((prev) => ({ ...prev, [bay.id]: e.target.value }))
                    }
                  />
                  <Button
                    size="sm"
                    variant="outline"
                    onClick={async () => {
                      try {
                        await api.renameBay(bay.id, bayNames[bay.id] ?? bay.name);
                        toast.success(t(locale, "save"));
                      } catch (error) {
                        toast.error(String(error));
                      }
                    }}
                  >
                    {t(locale, "saveBayName")}
                  </Button>
                  <Button
                    size="sm"
                    variant="destructive"
                    onClick={async () => {
                      try {
                        await api.setBayActive(bay.id, false);
                      } catch (error) {
                        toast.error(String(error));
                      }
                    }}
                  >
                    {t(locale, "deleteBay")}
                  </Button>
                </div>
              ))}

            <div className="flex items-center gap-2 pt-2">
              <Input
                placeholder={t(locale, "addBayPlaceholder")}
                value={newBayName}
                onChange={(e) => setNewBayName(e.target.value)}
              />
              <Button
                size="sm"
                onClick={async () => {
                  if (!newBayName.trim()) return;
                  try {
                    await api.addBay(newBayName.trim());
                    setNewBayName("");
                  } catch (error) {
                    toast.error(String(error));
                  }
                }}
              >
                {t(locale, "addBayButton")}
              </Button>
            </div>

            {(snapshot?.bays ?? []).some((bay) => !bay.active) ? (
              <div className="space-y-2 border-t pt-3 mt-2">
                <Label>{t(locale, "removedBaysTitle")}</Label>
                {(snapshot?.bays ?? [])
                  .filter((bay) => !bay.active)
                  .map((bay) => (
                    <div
                      key={bay.id}
                      className="flex items-center justify-between rounded-md bg-muted/40 px-3 py-2"
                    >
                      <span className="text-sm text-muted-foreground">{bay.name}</span>
                      <Button
                        size="sm"
                        variant="outline"
                        onClick={async () => {
                          try {
                            await api.setBayActive(bay.id, true);
                          } catch (error) {
                            toast.error(String(error));
                          }
                        }}
                      >
                        {t(locale, "restoreBay")}
                      </Button>
                    </div>
                  ))}
              </div>
            ) : null}
          </CardContent>
        </Card>
      </section>

      {/* ── الطباعة ── */}
      <section className="space-y-4">
        <SectionHeading>{t(locale, "printerSettings")}</SectionHeading>
        <Card>
          <CardContent className="space-y-3 pt-6">
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
            <Label>{t(locale, "numberFormatSettings")}</Label>
            <div className="flex gap-2">
              <Button
                variant={settings.numberFormat === "en" ? "default" : "outline"}
                onClick={() => patch("numberFormat", "en")}
              >
                {t(locale, "englishDigits")}
              </Button>
              <Button
                variant={settings.numberFormat === "ar" ? "default" : "outline"}
                onClick={() => patch("numberFormat", "ar")}
              >
                {t(locale, "arabicDigits")}
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
      </section>

      {/* ── الشاشات ── */}
      <section className="space-y-4">
        <SectionHeading>{t(locale, "displaySettings")}</SectionHeading>
        <Card>
          <CardContent className="space-y-3 pt-6">
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
            <Label>{t(locale, "waitingLayoutLabel")}</Label>
            <div className="flex gap-2">
              <Button
                variant={settings.waitingLayout === "cards" ? "default" : "outline"}
                onClick={() => patch("waitingLayout", "cards")}
              >
                {t(locale, "layoutCards")}
              </Button>
              <Button
                variant={settings.waitingLayout === "table" ? "default" : "outline"}
                onClick={() => patch("waitingLayout", "table")}
              >
                {t(locale, "layoutTable")}
              </Button>
            </div>
            <div className="flex items-center justify-between">
              <Label>{t(locale, "fullscreen")}</Label>
              <Switch
                checked={settings.waitingFullscreen}
                onCheckedChange={(v) => patch("waitingFullscreen", v)}
              />
            </div>
            <div className="flex gap-2">
              <Button variant="outline" onClick={() => api.testDisplay().catch((e) => toast.error(String(e)))}>
                {t(locale, "testDisplay")}
              </Button>
              <Button
                variant="outline"
                onClick={() => api.refreshDisplay().catch((e) => toast.error(String(e)))}
              >
                {t(locale, "refreshDisplay")}
              </Button>
            </div>
          </CardContent>
        </Card>
      </section>

      {/* ── الإعلانات ── */}
      <section className="space-y-4">
        <SectionHeading>{t(locale, "sectionAds")}</SectionHeading>
        <Card>
          <CardContent className="space-y-4 pt-6">
            <div className="flex items-center justify-between">
              <Label>{t(locale, "adsEnabled")}</Label>
              <Switch
                checked={settings.adsEnabled}
                onCheckedChange={(v) => patch("adsEnabled", v)}
              />
            </div>

            {settings.adsEnabled && (
              <>
                <div className="flex items-center gap-3">
                  <Label className="shrink-0">{t(locale, "boardDurationSecs")}</Label>
                  <Input
                    type="number"
                    min={1}
                    max={300}
                    className="w-24 text-center"
                    value={settings.boardDurationSecs}
                    onChange={(e) => patch("boardDurationSecs", Math.max(1, Number(e.target.value)))}
                  />
                </div>

                <Separator />

                {/* قائمة الإعلانات */}
                <div className="space-y-2">
                  {ads.length === 0 ? (
                    <p className="text-sm text-muted-foreground">{t(locale, "noAds")}</p>
                  ) : (
                    ads.map((ad, i) => (
                      <div key={ad.id} className="flex items-center gap-2 rounded-lg border p-2">
                        <img
                          src={convertFileSrc(ad.filePath)}
                          alt=""
                          className="h-12 w-20 rounded object-cover bg-slate-100"
                        />
                        <div className="flex flex-1 items-center gap-2 min-w-0">
                          <Input
                            type="number"
                            min={1}
                            max={600}
                            className="w-20 text-center"
                            value={ad.durationSecs}
                            onChange={async (e) => {
                              const dur = Math.max(1, Number(e.target.value));
                              const updated = await api.updateAdDuration(ad.id, dur).catch(() => null);
                              if (updated) setAds((prev) => prev.map((a) => (a.id === ad.id ? updated : a)));
                            }}
                          />
                          <span className="text-xs text-muted-foreground">{t(locale, "adDurationSecsUnit")}</span>
                        </div>
                        <div className="flex gap-1">
                          <Button
                            size="sm"
                            variant="outline"
                            disabled={i === 0}
                            onClick={async () => {
                              const newIds = ads.map((a) => a.id);
                              [newIds[i - 1], newIds[i]] = [newIds[i], newIds[i - 1]];
                              await api.reorderAds(newIds).catch(() => {});
                              setAds(await api.listAds().catch(() => ads));
                            }}
                          >
                            {t(locale, "moveUp")}
                          </Button>
                          <Button
                            size="sm"
                            variant="outline"
                            disabled={i === ads.length - 1}
                            onClick={async () => {
                              const newIds = ads.map((a) => a.id);
                              [newIds[i], newIds[i + 1]] = [newIds[i + 1], newIds[i]];
                              await api.reorderAds(newIds).catch(() => {});
                              setAds(await api.listAds().catch(() => ads));
                            }}
                          >
                            {t(locale, "moveDown")}
                          </Button>
                          <Button
                            size="sm"
                            variant="destructive"
                            onClick={async () => {
                              await api.removeAd(ad.id).catch((e) => toast.error(String(e)));
                              setAds(await api.listAds().catch(() => []));
                            }}
                          >
                            {t(locale, "removeAd")}
                          </Button>
                        </div>
                      </div>
                    ))
                  )}
                </div>

                <Button
                  variant="outline"
                  disabled={adUploading}
                  onClick={async () => {
                    const file = await open({
                      multiple: false,
                      filters: [{ name: "صور", extensions: ["png", "jpg", "jpeg", "webp", "gif"] }],
                    }).catch(() => null);
                    if (!file || typeof file !== "string") return;
                    setAdUploading(true);
                    try {
                      await api.addAd(file, 10);
                      setAds(await api.listAds());
                    } catch (e) {
                      toast.error(String(e));
                    } finally {
                      setAdUploading(false);
                    }
                  }}
                >
                  {adUploading ? "جاري الرفع..." : t(locale, "addAd")}
                </Button>
              </>
            )}
          </CardContent>
        </Card>
      </section>

      {/* ── الأمان والنظام ── */}
      <section className="space-y-4">
        <SectionHeading>{t(locale, "sectionSecuritySystem")}</SectionHeading>
        <Card>
          <CardContent className="space-y-4 pt-6">
            <div>
              <Label>{t(locale, "settingsPasswordLabel")}</Label>
              <Input
                type="password"
                value={settings.settingsPassword}
                onChange={(e) => patch("settingsPassword", e.target.value)}
              />
              <p className="mt-1 text-xs text-muted-foreground">{t(locale, "settingsPasswordHint")}</p>
            </div>

            <Separator />

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

            {dbPath ? (
              <p className="text-xs text-muted-foreground break-all">
                {t(locale, "dbPathLabel")}: {dbPath}
              </p>
            ) : null}
          </CardContent>
        </Card>
      </section>

      {/* ── قسم الموبايل ── */}
      <section className="space-y-4">
        <SectionHeading>{t(locale, "sectionMobile")}</SectionHeading>
        <Card>
          <CardContent className="space-y-3 pt-6">
            <p className="text-sm text-muted-foreground">{t(locale, "mobileHint")}</p>
            {localIps.length === 0 ? (
              <p className="text-sm text-amber-600">{t(locale, "mobileNoIp")}</p>
            ) : (
              <div className="space-y-2">
                {localIps.map((ip) => {
                  const url = `http://${ip}:${mobilePort}`;
                  return (
                    <div key={ip} className="flex items-center gap-2 rounded-lg border bg-muted/40 px-3 py-2">
                      <span className="flex-1 select-all font-mono text-sm">{url}</span>
                      <Button
                        size="sm"
                        variant="outline"
                        onClick={() => { void navigator.clipboard.writeText(url); toast.success(t(locale, "copied")); }}
                      >
                        {t(locale, "copy")}
                      </Button>
                    </div>
                  );
                })}
              </div>
            )}
          </CardContent>
        </Card>
      </section>

      <AlertDialog open={confirm !== null} onOpenChange={() => setConfirm(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t(locale, "confirm")}</AlertDialogTitle>
            <AlertDialogDescription>
              {confirm === "reset"
                ? t(locale, "confirmReset")
                : confirm === "numbering"
                  ? t(locale, "confirmResetNumbering")
                  : t(locale, "confirmRestore")}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t(locale, "close")}</AlertDialogCancel>
            <AlertDialogAction
              onClick={async () => {
                try {
                  if (confirm === "reset") await api.resetQueue();
                  if (confirm === "numbering") {
                    await api.resetNumbering();
                    patch("nextSequence", 1);
                    patch("nextPrioritySequence", 1);
                    toast.success(t(locale, "resetNumberingDone"));
                  }
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
