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
    () => snapshot?.bays.filter((b) => b.status === "READY") ?? [],
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

  async function callNext(bayId?: number) {
    try {
      const ticket = await api.callNext(bayId);
      toast.success(
        `${ticket.ticketNumber} → ${t(locale, "bay")} ${ticket.bayId}`,
      );
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
          ? `تم تعطيل حفرة ${bayId} (خارج الخدمة)`
          : `تم تنشيط حفرة ${bayId} (جاهزة لاستقبال الأدوار)`,
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

        {/* In manual mode the global "call next" button is the primary action */}
        {!autoAssign && (
          <Button size="xl" variant="success" onClick={() => void callNext()}>
            {t(locale, "callNext")}
          </Button>
        )}

        {/* ── Mode toggle pill ── */}
        <button
          onClick={() => void toggleAssignMode()}
          disabled={togglingMode}
          title={
            autoAssign
              ? t(locale, "autoAssignDesc")
              : t(locale, "manualAssignDesc")
          }
          className={[
            "flex items-center gap-2 rounded-full border px-4 py-2 text-sm font-bold shadow transition-all select-none",
            autoAssign
              ? "border-emerald-400 bg-emerald-50 text-emerald-700 hover:bg-emerald-100"
              : "border-amber-400 bg-amber-50 text-amber-700 hover:bg-amber-100",
            togglingMode ? "opacity-60 cursor-not-allowed" : "cursor-pointer",
          ].join(" ")}
        >
          <span
            className={[
              "inline-block h-2.5 w-2.5 rounded-full",
              autoAssign ? "bg-emerald-500" : "bg-amber-500",
            ].join(" ")}
          />
          <span>{t(locale, "callMode")}:</span>
          <span className="font-extrabold">
            {autoAssign
              ? t(locale, "autoAssignAuto")
              : t(locale, "autoAssignManual")}
          </span>
          <span className="opacity-40">⇄</span>
        </button>

        <div className="ms-auto rounded-xl bg-white px-5 py-3 text-xl font-black shadow">
          {t(locale, "waiting")}: {snapshot.waitingCount}
        </div>
      </div>

      {/* ── Manual mode warning banner ── */}
      {!autoAssign && (
        <div className="flex items-center gap-3 rounded-xl border border-amber-300 bg-amber-50 px-4 py-3 text-sm text-amber-800">
          <span className="text-lg">⚠️</span>
          <div>
            <span className="font-bold">وضع يدوي: </span>
            {t(locale, "manualAssignDesc")}
          </div>
        </div>
      )}

      <div className="flex gap-2 flex-wrap">
        <BayGrid
          bays={snapshot.bays}
          locale={locale}
          autoAssign={autoAssign}
          onComplete={(id) => setConfirm({ type: "complete", id })}
          onCallHere={(id) => void callNext(id)}
          onCancel={(id) => setConfirm({ type: "cancel", id })}
          onToggleOutOfService={(id, val) => void toggleOutOfService(id, val)}
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
                  {t(locale, "bay")} {bay.id}
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
