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

export function QueuePanel({ waiting, locale, onReprint, onCancel, onAssign }: Props) {
  return (
    <Card className="h-full">
      <CardHeader>
        <CardTitle>
          {t(locale, "queue")} — {t(locale, "waiting")}: {waiting.length}
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-3">
        {waiting.length === 0 ? (
          <div className="text-muted-foreground">لا يوجد منتظرون</div>
        ) : (
          waiting.map((ticket) => (
            <div key={ticket.id} className="flex items-center justify-between rounded-lg border bg-slate-50 p-3">
              <div className="text-2xl font-black">{ticket.ticketNumber}</div>
              <div className="flex gap-2">
                <Button size="sm" onClick={() => onAssign(ticket.id)}>
                  {t(locale, "assign")}
                </Button>
                <Button size="sm" variant="outline" onClick={() => onReprint(ticket.id)}>
                  {t(locale, "reprint")}
                </Button>
                <Button size="sm" variant="destructive" onClick={() => onCancel(ticket.id)}>
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
