import type { Ticket } from "@/types";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { t } from "@/utils/i18n";
import type { Locale } from "@/types";

type Props = {
  waiting: Ticket[];
  locale: Locale;
  onReprint: (id: number) => void;
  onCancel: (id: number) => void;
  onAssign: (id: number) => void;
};

export function QueuePanel({
  waiting,
  locale,
  onReprint,
  onCancel,
  onAssign,
}: Props) {
  return (
    <Card className="flex flex-col max-h-[calc(100vh-220px)]">
      <CardHeader className="shrink-0">
        <CardTitle>
          {t(locale, "queue")} — {t(locale, "waiting")}: {waiting.length}
        </CardTitle>
      </CardHeader>
      <CardContent className="flex-1 min-h-0 overflow-y-auto space-y-3">
        {waiting.length === 0 ? (
          <div className="text-muted-foreground">لا يوجد منتظرون</div>
        ) : (
          waiting.map((ticket) => (
            <div
              key={ticket.id}
              className="flex flex-col gap-2 rounded-xl border border-slate-200 bg-white p-3 shadow-sm"
            >
              <div className="flex items-center justify-between">
                <span className="text-3xl font-black tracking-wide text-slate-800">
                  {ticket.ticketNumber}
                </span>
              </div>

              <div className="text-xs text-muted-foreground">
                {new Date(ticket.createdAt).toLocaleTimeString("ar-SA", {
                  hour: "2-digit",
                  minute: "2-digit",
                })}
              </div>

              {/* ── أزرار الإجراءات بعرض كامل ── */}
              <div className="grid grid-cols-3 gap-1.5">
                <Button
                  size="sm"
                  className="w-full font-bold"
                  onClick={() => onAssign(ticket.id)}
                >
                  {t(locale, "assign")}
                </Button>
                <Button
                  size="sm"
                  variant="outline"
                  className="w-full"
                  onClick={() => onReprint(ticket.id)}
                >
                  {t(locale, "reprint")}
                </Button>
                <Button
                  size="sm"
                  variant="destructive"
                  className="w-full"
                  onClick={() => onCancel(ticket.id)}
                >
                  {t(locale, "cancel")}
                </Button>
              </div>
            </div>
          ))
        )}
      </CardContent>
    </Card>
  );
}
