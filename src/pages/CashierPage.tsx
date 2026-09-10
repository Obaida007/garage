import { useMemo, useState } from "react";
import { toast } from "sonner";
import { BayGrid } from "@/features/cashier/BayGrid";
import { QueuePanel } from "@/features/cashier/QueuePanel";
import { RecoveryDialog } from "@/features/cashier/RecoveryDialog";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogTitle } from "@/components/ui/dialog";
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
import { t } from "@/utils/i18n";

export function CashierPage() {
  const { snapshot, locale } = useGarage();
  const [recoveryOpen, setRecoveryOpen] = useState(true);
  const [assignTicketId, setAssignTicketId] = useState<number | null>(null);
  const [confirm, setConfirm] = useState<{ type: "cancel" | "complete"; id: number } | null>(null);

  const readyBays = useMemo(
    () => snapshot?.bays.filter((b) => b.status === "READY") ?? [],
    [snapshot],
  );

  if (!snapshot) {
    return <div className="p-8 text-lg">جاري التحميل...</div>;
  }

  async function createTicket() {
    try {
      const result = await api.createTicket();
      if (result.printError) {
        toast.error(`${t(locale, "printFailed")} ${result.ticket.ticketNumber}: ${result.printError}`);
      } else {
        toast.success(`${t(locale, "ticketCreated")} ${result.ticket.ticketNumber}`);
      }
    } catch (error) {
      toast.error(String(error));
    }
  }

  async function callNext(bayId?: number) {
    try {
      const ticket = await api.callNext(bayId);
      toast.success(`${ticket.ticketNumber} → ${t(locale, "bay")} ${ticket.bayId}`);
    } catch (error) {
      toast.error(String(error));
    }
  }

  async function doComplete(bayId: number) {
    try {
      await api.completeBay(bayId);
    } catch (error) {
      toast.error(String(error));
    }
  }

  async function doCancel(ticketId: number) {
    try {
      await api.cancelTicket(ticketId);
    } catch (error) {
      toast.error(String(error));
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex flex-wrap items-center gap-3">
        <Button size="xl" onClick={() => void createTicket()}>
          {t(locale, "newTicket")}
        </Button>
        <Button size="xl" variant="success" onClick={() => void callNext()}>
          {t(locale, "callNext")}
        </Button>
        <div className="ms-auto rounded-xl bg-white px-5 py-3 text-xl font-black shadow">
          {t(locale, "waiting")}: {snapshot.waitingCount}
        </div>
      </div>

      <div className="grid grid-cols-1 gap-6 xl:grid-cols-[1fr_360px]">
        <BayGrid
          bays={snapshot.bays}
          locale={locale}
          onComplete={(id) => setConfirm({ type: "complete", id })}
          onCallHere={(id) => void callNext(id)}
          onCancel={(id) => setConfirm({ type: "cancel", id })}
        />
        <QueuePanel
          waiting={snapshot.waiting}
          locale={locale}
          onReprint={async (id) => {
            try {
              await api.reprint(id);
              toast.success(t(locale, "reprint"));
            } catch (error) {
              toast.error(String(error));
            }
          }}
          onCancel={(id) => setConfirm({ type: "cancel", id })}
          onAssign={setAssignTicketId}
        />
      </div>

      <RecoveryDialog
        open={recoveryOpen && snapshot.needsRecovery}
        tickets={snapshot.inService}
        locale={locale}
        onComplete={async (id) => {
          try {
            await api.completeTicket(id);
          } catch (error) {
            toast.error(String(error));
          }
        }}
        onReturn={async (id) => {
          try {
            await api.returnToQueue(id);
          } catch (error) {
            toast.error(String(error));
          }
        }}
        onCancel={(id) => setConfirm({ type: "cancel", id })}
        onLater={() => setRecoveryOpen(false)}
      />

      <Dialog open={assignTicketId !== null} onOpenChange={() => setAssignTicketId(null)}>
        <DialogContent>
          <DialogTitle>{t(locale, "assign")}</DialogTitle>
          <div className="mt-4 grid grid-cols-2 gap-3">
            {readyBays.length === 0 ? (
              <div>لا توجد حفرة جاهزة</div>
            ) : (
              readyBays.map((bay) => (
                <Button
                  key={bay.id}
                  size="lg"
                  onClick={async () => {
                    if (assignTicketId == null) return;
                    try {
                      await api.assign(assignTicketId, bay.id);
                      setAssignTicketId(null);
                    } catch (error) {
                      toast.error(String(error));
                    }
                  }}
                >
                  {t(locale, "bay")} {bay.id}
                </Button>
              ))
            )}
          </div>
        </DialogContent>
      </Dialog>

      <AlertDialog open={confirm !== null} onOpenChange={() => setConfirm(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t(locale, "confirm")}</AlertDialogTitle>
            <AlertDialogDescription>
              {confirm?.type === "cancel" ? t(locale, "confirmCancel") : t(locale, "confirmComplete")}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t(locale, "close")}</AlertDialogCancel>
            <AlertDialogAction
              onClick={() => {
                if (!confirm) return;
                if (confirm.type === "cancel") void doCancel(confirm.id);
                else void doComplete(confirm.id);
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
