import { useState } from "react";
import { toast } from "sonner";
import { open } from "@tauri-apps/plugin-dialog";
import { convertFileSrc } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { api } from "@/services/tauri";
import { t } from "@/utils/i18n";
import type { AppSettings, Locale, PrinterInfo } from "@/types";

type Props = {
  initialSettings: AppSettings;
  locale: Locale;
};

const STEP_KEYS = [
  "setupStepIdentity",
  "setupStepLogo",
  "setupStepPrinting",
  "setupStepReview",
] as const;

export function SetupWizard({ initialSettings, locale }: Props) {
  const [draft, setDraft] = useState<AppSettings>(initialSettings);
  const [step, setStep] = useState(0);
  const [saving, setSaving] = useState(false);
  const [printers, setPrinters] = useState<PrinterInfo[]>([]);
  const [printersLoaded, setPrintersLoaded] = useState(false);
  const [nameError, setNameError] = useState(false);

  function patch<K extends keyof AppSettings>(key: K, value: AppSettings[K]) {
    setDraft((prev) => ({ ...prev, [key]: value }));
  }

  async function loadPrintersOnce() {
    if (printersLoaded) return;
    try {
      setPrinters(await api.printers());
    } catch {
      // Printing setup step is optional — a failure to list printers is not fatal.
    } finally {
      setPrintersLoaded(true);
    }
  }

  function goNext() {
    if (step === 0) {
      if (!draft.garageName.trim()) {
        setNameError(true);
        return;
      }
      setNameError(false);
    }
    if (step === 1) {
      void loadPrintersOnce();
    }
    setStep((s) => Math.min(s + 1, STEP_KEYS.length - 1));
  }

  function goBack() {
    setStep((s) => Math.max(s - 1, 0));
  }

  async function finish() {
    if (!draft.garageName.trim()) {
      setStep(0);
      setNameError(true);
      return;
    }
    setSaving(true);
    try {
      await api.saveSettings({ ...draft, setupCompleted: true });
      toast.success(t(locale, "setupFinish"));
    } catch (error) {
      toast.error(String(error));
    } finally {
      setSaving(false);
    }
  }

  return (
    <div className="flex min-h-screen items-center justify-center bg-slate-100 p-6">
      <Card className="w-full max-w-lg shadow-lg">
        <CardHeader>
          <div className="mb-1 flex items-center justify-between text-xs font-bold text-muted-foreground">
            <span>{t(locale, STEP_KEYS[step])}</span>
            <span>
              {step + 1} / {STEP_KEYS.length}
            </span>
          </div>
          <div className="flex gap-1.5">
            {STEP_KEYS.map((key, i) => (
              <div
                key={key}
                className={`h-1.5 flex-1 rounded-full ${i <= step ? "bg-sky-600" : "bg-slate-200"}`}
              />
            ))}
          </div>
          {step === 0 && (
            <>
              <CardTitle className="pt-3 text-xl font-black">
                {t(locale, "setupWelcomeTitle")}
              </CardTitle>
              <p className="text-sm text-muted-foreground">{t(locale, "setupWelcomeBody")}</p>
            </>
          )}
        </CardHeader>

        <CardContent className="space-y-4">
          {step === 0 && (
            <div className="space-y-3">
              <Label>{t(locale, "garageName")}</Label>
              <Input
                autoFocus
                value={draft.garageName}
                onChange={(e) => {
                  const value = e.target.value;
                  setDraft((prev) => ({
                    ...prev,
                    garageName: value,
                    printHeader: prev.printHeader === prev.garageName ? value : prev.printHeader,
                  }));
                  if (value.trim()) setNameError(false);
                }}
              />
              {nameError && (
                <p className="text-xs font-bold text-red-600">{t(locale, "setupNameRequired")}</p>
              )}
              <Label>{t(locale, "printHeader")}</Label>
              <Input
                value={draft.printHeader}
                onChange={(e) => patch("printHeader", e.target.value)}
              />
            </div>
          )}

          {step === 1 && (
            <div className="space-y-3">
              <p className="text-xs text-muted-foreground">{t(locale, "logoHint")}</p>
              {draft.logoPath ? (
                <img
                  src={convertFileSrc(draft.logoPath)}
                  alt="logo"
                  className="h-20 max-w-full rounded border bg-white object-contain p-2"
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
                {draft.logoPath ? (
                  <Button variant="destructive" onClick={() => patch("logoPath", "")}>
                    {t(locale, "removeLogo")}
                  </Button>
                ) : null}
              </div>
            </div>
          )}

          {step === 2 && (
            <div className="space-y-3">
              <Label>الطابعة</Label>
              <select
                className="h-11 w-full rounded-md border bg-background px-3"
                value={draft.printerName}
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
                  variant={draft.paperWidthMm === 58 ? "default" : "outline"}
                  onClick={() => patch("paperWidthMm", 58)}
                >
                  58mm
                </Button>
                <Button
                  variant={draft.paperWidthMm === 80 ? "default" : "outline"}
                  onClick={() => patch("paperWidthMm", 80)}
                >
                  80mm
                </Button>
              </div>
              <Label>{t(locale, "prefix")}</Label>
              <Input
                value={draft.ticketPrefix}
                onChange={(e) => patch("ticketPrefix", e.target.value)}
              />
            </div>
          )}

          {step === 3 && (
            <div className="space-y-3">
              <p className="text-sm text-muted-foreground">{t(locale, "setupReviewIntro")}</p>
              <div className="space-y-2 rounded-lg border p-3 text-sm">
                <div className="flex justify-between">
                  <span className="text-muted-foreground">{t(locale, "garageName")}</span>
                  <span className="font-bold">{draft.garageName}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">{t(locale, "printHeader")}</span>
                  <span className="font-bold">{draft.printHeader || "—"}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">{t(locale, "logoSettings")}</span>
                  <span className="font-bold">{draft.logoPath ? "✓" : "—"}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">حجم الورق</span>
                  <span className="font-bold">{draft.paperWidthMm}mm</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">{t(locale, "prefix")}</span>
                  <span className="font-bold">{draft.ticketPrefix || "—"}</span>
                </div>
              </div>
            </div>
          )}

          <div className="flex items-center justify-between pt-2">
            <Button variant="ghost" onClick={goBack} disabled={step === 0}>
              {t(locale, "setupBack")}
            </Button>
            <div className="flex gap-2">
              {step < STEP_KEYS.length - 1 ? (
                <Button onClick={goNext}>{t(locale, "setupNext")}</Button>
              ) : (
                <Button onClick={() => void finish()} disabled={saving}>
                  {t(locale, "setupFinish")}
                </Button>
              )}
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
