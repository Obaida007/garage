import { useMemo, useState } from "react";
import { toast } from "sonner";
import { AlertTriangle, Zap } from "lucide-react";
import { BayGrid } from "@/features/cashier/BayGrid";
import { QueuePanel } from "@/features/cashier/QueuePanel";
import { RecoveryDialog } from "@/features/cashier/RecoveryDialog";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
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
import { useGarageStore } from "@/stores/garageStore";
import { api } from "@/services/tauri";
import { t } from "@/utils/i18n";

export function CashierPage() {
  const snapshot = useGarageStore((s) => s.snapshot);
  const locale = useGarageStore((s) => s.locale);
  const [recoveryOpen, setRecoveryOpen] = useState(true);
  const [assignTicketId, setAssignTicketId] = useState<number | null>(null);
  const [confirm, setConfirm] = useState<{
    type: "cancel" | "complete";
    id: number;
  } | null>(null);
  const [togglingMode, setTogglingMode] = useState(false);

  const readyBays = useMemo(
    () => snapshot?.bays.filter((b) => b.status === "READY" && b.active) ?? [],
    [snapshot],
  );

  if (!snapshot) {
    return <div className="p-8 text-lg">جاري التحميل...</div>;
  }

  const autoAssign = snapshot.settings.autoAssign;

  async function toggleAssignMode() {
    if (!snapshot) return;
    setTogglingMode(true);
    try {
      await api.saveSettings({ ...snapshot.settings, autoAssign: !autoAssign });
    } catch (error) {
      toast.error(String(error));
    } finally {
      setTogglingMode(false);
    }
  }

  async function createTicket() {
    try {
      const result = await api.createTicket();
      if (result.printError) {
        toast.error(
          `${t(locale, "printFailed")} ${result.ticket.ticketNumber}: ${result.printError}`,
        );
      } else {
        toast.success(
          `${t(locale, "ticketCreated")} ${result.ticket.ticketNumber}`,
        );
      }
      // Announcement fired automatically by garageStore on snapshot diff
    } catch (error) {
      toast.error(String(error));
    }
  }

  async function createPriorityTicket() {
    try {
      const result = await api.createPriorityTicket();
      if (result.printError) {
        toast.error(
          `${t(locale, "printFailed")} ${result.ticket.ticketNumber}: ${result.printError}`,
        );
      } else {
        toast.success(
          `${t(locale, "ticketCreated")} ${result.ticket.ticketNumber}`,
        );
      }
    } catch (error) {
      toast.error(String(error));
    }
  }

  function bayLabel(bayId: number | null) {
    const found = snapshot?.bays.find((b) => b.id === bayId);
    return found?.name || `${t(locale, "bay")} ${bayId}`;
  }

  async function callNext(bayId?: number) {
    try {
      const ticket = await api.callNext(bayId);
      toast.success(`${ticket.ticketNumber} → ${bayLabel(ticket.bayId)}`);
      // Announcement fired automatically by garageStore on snapshot diff
    } catch (error) {
      toast.error(String(error));
    }
  }

  async function toggleOutOfService(bayId: number, outOfService: boolean) {
    try {
      await api.setBayOutOfService(bayId, outOfService);
      toast.success(
        outOfService
          ? `تم تعطيل ${bayLabel(bayId)} (خارج الخدمة)`
          : `تم تنشيط ${bayLabel(bayId)} (جاهزة لاستقبال الأدوار)`,
      );
    } catch (error) {
      toast.error(String(error));
    }
  }

  async function doComplete(bayId: number) {
    try {
      await api.completeBay(bayId);
      // Announcement fired automatically by garageStore on snapshot diff
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
      {/* ── Top action bar ── */}
      <div className="flex flex-wrap items-center gap-3">
        <Button size="xl" onClick={() => void createTicket()}>
          {t(locale, "newTicket")}
        </Button>

        {snapshot.settings.priorityEnabled && (
          <Button
            size="xl"
            variant="warning"
            onClick={() => void createPriorityTicket()}
          >
            <Zap className="h-5 w-5" />
            {t(locale, "newPriorityTicket")}
          </Button>
        )}

        {/* In manual mode the global "call next" button is the primary action */}
        {!autoAssign && (
          <Button size="xl" variant="success" onClick={() => void callNext()}>
            {t(locale, "callNext")}
          </Button>
        )}

        {/* ── Mode toggle ── */}
        <div
          className="flex items-center gap-3 rounded-lg border bg-white px-4 py-2.5 shadow-sm"
          title={
            autoAssign
              ? t(locale, "autoAssignDesc")
              : t(locale, "manualAssignDesc")
          }
        >
          <Label className="text-sm font-bold text-muted-foreground">
            {t(locale, "callMode")}
          </Label>
          <span className="text-sm font-extrabold">
            {autoAssign ? t(locale, "autoAssignAuto") : t(locale, "autoAssignManual")}
          </span>
          <Switch
            checked={autoAssign}
            disabled={togglingMode}
            onCheckedChange={() => void toggleAssignMode()}
          />
        </div>

        <div className="ms-auto rounded-xl bg-white px-5 py-3 text-xl font-black shadow">
          {t(locale, "waiting")}: {snapshot.waitingCount}
        </div>
      </div>

      {/* ── Manual mode warning banner ── */}
      {!autoAssign && (
        <div className="flex items-center gap-3 rounded-xl border border-amber-300 bg-amber-50 px-4 py-3 text-sm text-amber-800">
          <AlertTriangle className="h-5 w-5 shrink-0" />
          <div>
            <span className="font-bold">وضع يدوي: </span>
            {t(locale, "manualAssignDesc")}
          </div>
        </div>
      )}

      <div className="flex gap-4 items-start flex-wrap lg:flex-nowrap">
        <div className="min-w-0 flex-1">
          <BayGrid
            bays={snapshot.bays.filter((b) => b.active)}
            locale={locale}
            autoAssign={autoAssign}
            onComplete={(id) => setConfirm({ type: "complete", id })}
            onCallHere={(id) => void callNext(id)}
            onCancel={(id) => setConfirm({ type: "cancel", id })}
            onToggleOutOfService={(id, val) => void toggleOutOfService(id, val)}
          />
        </div>
        <div className="w-full shrink-0 lg:w-[380px]">
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
      </div>

      <RecoveryDialog
        open={recoveryOpen && snapshot.needsRecovery}
        tickets={snapshot.inService}
        bays={snapshot.bays}
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

      {/* ── Manual assign from queue dialog ── */}
      <Dialog
        open={assignTicketId !== null}
        onOpenChange={() => setAssignTicketId(null)}
      >
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
                      const updatedTicket = await api.assign(
                        assignTicketId,
                        bay.id,
                      );
                      setAssignTicketId(null);
                      // Announcement fired automatically by garageStore on snapshot diff
                      void updatedTicket;
                    } catch (error) {
                      toast.error(String(error));
                    }
                  }}
                >
                  {bay.name || `${t(locale, "bay")} ${bay.id}`}
                </Button>
              ))
            )}
          </div>
        </DialogContent>
      </Dialog>

      <AlertDialog
        open={confirm !== null}
        onOpenChange={() => setConfirm(null)}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t(locale, "confirm")}</AlertDialogTitle>
            <AlertDialogDescription>
              {confirm?.type === "cancel"
                ? t(locale, "confirmCancel")
                : t(locale, "confirmComplete")}
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
