import { useEffect, useState } from "react";
import { toast } from "sonner";
import { Zap } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { useGarage } from "@/hooks/useGarage";
import { api } from "@/services/tauri";
import { t } from "@/utils/i18n";
import type { Ticket } from "@/types";

export function TicketsPage() {
  const { locale, snapshot } = useGarage();
  const [query, setQuery] = useState("");
  const [items, setItems] = useState<Ticket[]>([]);

  async function load() {
    try {
      if (query.trim()) {
        const found = await api.searchTicket(query.trim());
        setItems(found ? [found] : []);
      } else {
        setItems(await api.recentTickets(80));
      }
    } catch (error) {
      toast.error(String(error));
    }
  }

  useEffect(() => {
    void load();
  }, []);

  function getStatusBadge(ticket: Ticket) {
    switch (ticket.status) {
      case "WAITING":
        return <Badge variant="warning">{t(locale, "statusWaiting")}</Badge>;
      case "IN_SERVICE":
        return <Badge variant="busy">{t(locale, "statusInService")}</Badge>;
      case "COMPLETED":
        return <Badge variant="ready">{t(locale, "statusCompleted")}</Badge>;
      case "CANCELLED":
        return <Badge variant="muted">{t(locale, "statusCancelled")}</Badge>;
      default:
        return <Badge variant="muted">{ticket.status}</Badge>;
    }
  }

  return (
    <Card className="shadow-md">
      <CardHeader className="border-b bg-slate-50">
        <CardTitle className="text-2xl font-black text-slate-900">
          {t(locale, "tickets")}
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-6 pt-6">
        <div className="flex gap-3 max-w-md">
          <Input
            placeholder={t(locale, "ticketSearchPlaceholder")}
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") void load();
            }}
            className="h-11 text-lg"
          />
          <Button size="lg" onClick={() => void load()}>
            {t(locale, "search")}
          </Button>
        </div>

        {/* Table / List Header Row */}
        <div className="space-y-3">
          <div className="hidden md:flex items-center justify-between rounded-xl bg-slate-800 px-6 py-3 text-white font-bold text-base">
            <span className="w-1/5">{t(locale, "ticketNumberColumn")}</span>
            <span className="w-1/5">{t(locale, "assignedBayColumn")}</span>
            <span className="w-1/4">{t(locale, "createdAtColumn")}</span>
            <span className="w-1/5 text-center">{t(locale, "statusColumn")}</span>
            <span className="w-1/6 text-left">{t(locale, "actionsColumn")}</span>
          </div>

          {items.length === 0 ? (
            <div className="py-12 text-center text-lg text-slate-500 font-medium">
              {t(locale, "noTicketsFound")}
            </div>
          ) : (
            items.map((ticket) => (
              <div
                key={ticket.id}
                className="flex flex-col md:flex-row md:items-center justify-between rounded-xl border bg-white p-4 shadow-sm hover:border-slate-400 transition-all gap-4"
              >
                {/* Ticket Number */}
                <div className="md:w-1/5 flex items-center gap-2">
                  <span className="md:hidden font-bold text-slate-500">
                    {t(locale, "ticketNumberColumn")}:
                  </span>
                  <span className="text-2xl font-black text-slate-900">
                    {ticket.ticketNumber}
                  </span>
                  {ticket.isPriority && (
                    <Badge variant="warning" className="gap-1">
                      <Zap className="h-3.5 w-3.5" />
                      {t(locale, "priorityBadge")}
                    </Badge>
                  )}
                </div>

                {/* Assigned Pit / Bay */}
                <div className="md:w-1/5 flex items-center gap-2">
                  <span className="md:hidden font-bold text-slate-500">
                    {t(locale, "bay")}:
                  </span>
                  {ticket.bayId ? (
                    <span className="inline-flex items-center gap-1.5 rounded-lg bg-sky-50 px-3 py-1 text-base font-bold text-sky-700 border border-sky-200">
                      {snapshot?.bays.find((b) => b.id === ticket.bayId)?.name ??
                        `${t(locale, "bay")} ${ticket.bayId}`}
                    </span>
                  ) : (
                    <span className="text-slate-400 text-sm font-medium">
                      {t(locale, "unassignedInQueue")}
                    </span>
                  )}
                </div>

                {/* Created At */}
                <div className="md:w-1/4 text-sm font-semibold text-slate-600 dir-ltr text-right md:text-right">
                  {ticket.createdAt ? ticket.createdAt.replace("T", " ") : "-"}
                </div>

                {/* Status */}
                <div className="md:w-1/5 flex md:justify-center">
                  {getStatusBadge(ticket)}
                </div>

                {/* Actions */}
                <div className="md:w-1/6 flex justify-end">
                  <Button
                    variant="outline"
                    size="sm"
                    className="font-bold border-slate-300"
                    onClick={async () => {
                      try {
                        await api.reprint(ticket.id);
                        toast.success(t(locale, "reprint"));
                      } catch (error) {
                        toast.error(String(error));
                      }
                    }}
                  >
                    {t(locale, "reprint")}
                  </Button>
                </div>
              </div>
            ))
          )}
        </div>
      </CardContent>
    </Card>
  );
}
