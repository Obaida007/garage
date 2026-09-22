import { Ban, CheckCircle2, Play } from "lucide-react";
import type { Bay } from "@/types";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { t } from "@/utils/i18n";
import type { Locale } from "@/types";

type Props = {
  bays: Bay[];
  locale: Locale;
  autoAssign: boolean;
  onComplete: (bayId: number) => void;
  onCallHere: (bayId: number) => void;
  onCancel: (ticketId: number) => void;
  onToggleOutOfService: (bayId: number, outOfService: boolean) => void;
};

/** بادج حالة الحفرة، مبني على مكوّن Badge الموحّد */
function StatusBadge({ status }: { status: "busy" | "ready" | "out" }) {
  if (status === "busy") {
    return (
      <Badge variant="busy" className="gap-1.5">
        <span className="relative flex h-2 w-2">
          <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-red-500 opacity-75" />
          <span className="relative inline-flex h-2 w-2 rounded-full bg-red-600" />
        </span>
        قيد العمل
      </Badge>
    );
  }
  if (status === "out") {
    return (
      <Badge variant="muted" className="gap-1.5">
        <Ban className="h-3.5 w-3.5" />
        خارج الخدمة
      </Badge>
    );
  }
  return (
    <Badge variant="ready" className="gap-1.5">
      <CheckCircle2 className="h-3.5 w-3.5" />
      جاهزة
    </Badge>
  );
}

export function BayGrid({
  bays,
  locale,
  autoAssign,
  onComplete,
  onCallHere,
  onCancel,
  onToggleOutOfService,
}: Props) {
  return (
    <div className="grid grid-cols-1 h-fit gap-4 md:grid-cols-2 xl:grid-cols-3">
      {bays.map((bay) => {
        const busy = bay.status === "BUSY" && bay.currentTicket;
        const outOfService = bay.status === "OUT_OF_SERVICE";
        const isReady = bay.status === "READY";

        return (
          <Card
            key={bay.id}
            className={
              busy
                ? "border-red-300 bg-red-50/20"
                : outOfService
                  ? "border-slate-300 bg-slate-100/80 opacity-75"
                  : "border-emerald-300 bg-emerald-50/20"
            }
          >
            <CardHeader className="flex-row items-center justify-between pb-2">
              <CardTitle className="text-xl font-bold">
                {bay.name || `${t(locale, "bay")} ${bay.id}`}
              </CardTitle>
              <StatusBadge
                status={outOfService ? "out" : busy ? "busy" : "ready"}
              />
            </CardHeader>

            <CardContent className="space-y-4">
              {/* رقم الدور الحالي */}
              <div
                className={`font-black tracking-wider ${
                  outOfService
                    ? "text-2xl text-slate-400"
                    : "text-4xl text-slate-800"
                }`}
              >
                {outOfService
                  ? "خارج الخدمة"
                  : (bay.currentTicket?.ticketNumber ?? t(locale, "empty"))}
              </div>

              {/* وقت بدء الخدمة للحفرة المشغولة */}
              {busy && bay.currentTicket?.startedAt && (
                <div className="text-xs text-muted-foreground">
                  بدأ:{" "}
                  {new Date(bay.currentTicket.startedAt).toLocaleTimeString(
                    "ar-SA",
                    { hour: "2-digit", minute: "2-digit" },
                  )}
                </div>
              )}

              {/* الأزرار */}
              {outOfService ? (
                <Button
                  className="w-full font-bold"
                  size="lg"
                  variant="outline"
                  onClick={() => onToggleOutOfService(bay.id, false)}
                >
                  <CheckCircle2 className="h-4 w-4" />
                  {t(locale, "activateBay")}
                </Button>
              ) : busy ? (
                <div className="flex flex-col gap-2">
                  <Button
                    className="w-full font-bold"
                    size="lg"
                    onClick={() => onComplete(bay.id)}
                  >
                    {t(locale, "complete")}
                  </Button>
                  <Button
                    className="w-full font-bold"
                    size="sm"
                    variant="destructive"
                    onClick={() =>
                      bay.currentTicket && onCancel(bay.currentTicket.id)
                    }
                  >
                    {t(locale, "cancel")}
                  </Button>
                </div>
              ) : isReady ? (
                <div className="flex flex-col gap-2">
                  {/* وضع يدوي: زر استدعاء كبير أخضر — وضع تلقائي: زر استدعاء فوري استثنائي */}
                  <Button
                    className="w-full font-bold"
                    size={autoAssign ? "sm" : "lg"}
                    variant={autoAssign ? "warning" : "success"}
                    onClick={() => onCallHere(bay.id)}
                  >
                    {autoAssign && <Play className="h-4 w-4" />}
                    {t(locale, "callNext")}
                  </Button>
                  <Button
                    className="w-full text-slate-500 hover:text-red-700 hover:bg-red-50"
                    size="sm"
                    variant="ghost"
                    onClick={() => onToggleOutOfService(bay.id, true)}
                  >
                    <Ban className="h-4 w-4" />
                    {t(locale, "deactivateBay")}
                  </Button>
                </div>
              ) : null}
            </CardContent>
          </Card>
        );
      })}
    </div>
  );
}
