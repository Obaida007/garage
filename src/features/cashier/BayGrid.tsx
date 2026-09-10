import type { Bay } from "@/types";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { t } from "@/utils/i18n";
import type { Locale } from "@/types";

type Props = {
  bays: Bay[];
  locale: Locale;
  onComplete: (bayId: number) => void;
  onCallHere: (bayId: number) => void;
  onCancel: (ticketId: number) => void;
};

export function BayGrid({ bays, locale, onComplete, onCallHere, onCancel }: Props) {
  return (
    <div className="grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-3">
      {bays.map((bay) => {
        const busy = bay.status === "BUSY" && bay.currentTicket;
        return (
          <Card key={bay.id} className={busy ? "border-red-300" : "border-emerald-300"}>
            <CardHeader className="flex-row items-center justify-between">
              <CardTitle>
                {t(locale, "bay")} {bay.id}
              </CardTitle>
              <Badge variant={busy ? "busy" : "ready"}>
                {busy ? t(locale, "busy") : t(locale, "ready")}
              </Badge>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="text-4xl font-black tracking-wider">
                {bay.currentTicket?.ticketNumber ?? t(locale, "empty")}
              </div>
              {busy ? (
                <div className="flex gap-2">
                  <Button className="flex-1" size="lg" onClick={() => onComplete(bay.id)}>
                    {t(locale, "complete")}
                  </Button>
                  <Button
                    className="flex-1"
                    size="lg"
                    variant="destructive"
                    onClick={() => bay.currentTicket && onCancel(bay.currentTicket.id)}
                  >
                    {t(locale, "cancel")}
                  </Button>
                </div>
              ) : (
                <Button className="w-full" size="lg" variant="success" onClick={() => onCallHere(bay.id)}>
                  {t(locale, "callNext")}
                </Button>
              )}
            </CardContent>
          </Card>
        );
      })}
    </div>
  );
}
