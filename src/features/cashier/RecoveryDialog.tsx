import type { Ticket } from "@/types";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogTitle } from "@/components/ui/dialog";
import { t } from "@/utils/i18n";
import type { Locale } from "@/types";

type Props = {
  open: boolean;
  tickets: Ticket[];
  locale: Locale;
  onComplete: (id: number) => void;
  onReturn: (id: number) => void;
  onCancel: (id: number) => void;
  onLater: () => void;
};

export function RecoveryDialog({
  open,
  tickets,
  locale,
  onComplete,
  onReturn,
  onCancel,
  onLater,
}: Props) {
  return (
    <Dialog open={open}>
      <DialogContent>
        <DialogTitle className="text-xl font-black">{t(locale, "recoveryTitle")}</DialogTitle>
        <p className="mt-2 text-muted-foreground">{t(locale, "recoveryBody")}</p>
        <div className="mt-4 space-y-3">
          {tickets.map((ticket) => (
            <div key={ticket.id} className="rounded-lg border p-3">
              <div className="mb-2 text-2xl font-black">
                {ticket.ticketNumber} — {t(locale, "bay")} {ticket.bayId}
              </div>
              <div className="flex flex-wrap gap-2">
                <Button onClick={() => onComplete(ticket.id)}>{t(locale, "complete")}</Button>
                <Button variant="secondary" onClick={() => onReturn(ticket.id)}>
                  {t(locale, "returnQueue")}
                </Button>
                <Button variant="destructive" onClick={() => onCancel(ticket.id)}>
                  {t(locale, "cancel")}
                </Button>
              </div>
            </div>
          ))}
        </div>
        <Button className="mt-4" variant="outline" onClick={onLater}>
          {t(locale, "later")}
        </Button>
      </DialogContent>
    </Dialog>
  );
}
