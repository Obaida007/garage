import { useEffect, useState } from "react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { useGarage } from "@/hooks/useGarage";
import { api } from "@/services/tauri";
import { t } from "@/utils/i18n";
import type { Ticket } from "@/types";

export function TicketsPage() {
  const { locale } = useGarage();
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
        return (
          <Badge
            variant="muted"
            className="bg-amber-100 text-amber-800 border-amber-300 font-bold px-3 py-1 text-sm"
          >
            {t(locale, "statusWaiting")}
          </Badge>
        );
      case "IN_SERVICE":
        return (
          <Badge
            variant="default"
            className="bg-sky-600 text-white font-bold px-3 py-1 text-sm"
          >
            {t(locale, "statusInService")}
          </Badge>
        );
      case "COMPLETED":
        return (
          <Badge
            variant="default"
            className="bg-emerald-100 text-emerald-800 border-emerald-300 font-bold px-3 py-1 text-sm"
          >
            {t(locale, "statusCompleted")}
          </Badge>
        );
      case "CANCELLED":
        return (
          <Badge
            variant="busy"
            className="bg-red-100 text-red-800 border-red-300 font-bold px-3 py-1 text-sm"
          >
            {t(locale, "statusCancelled")}
          </Badge>
        );
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
            placeholder="بحث برقم الدور (مثال: 001)..."
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
            <span className="w-1/5">رقم الدور</span>
            <span className="w-1/5">الحفرة التابعة</span>
            <span className="w-1/4">وقت وتاريخ الإنشاء</span>
            <span className="w-1/5 text-center">حالة الدور</span>
            <span className="w-1/6 text-left">الإجراءات</span>
          </div>

          {items.length === 0 ? (
            <div className="py-12 text-center text-lg text-slate-500 font-medium">
              لا توجد أدوار مسجلة تطابق البحث
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
                    رقم الدور:
                  </span>
                  <span className="text-2xl font-black text-slate-900">
                    {ticket.ticketNumber}
                  </span>
                </div>

                {/* Assigned Pit / Bay */}
                <div className="md:w-1/5 flex items-center gap-2">
                  <span className="md:hidden font-bold text-slate-500">
                    الحفرة:
                  </span>
                  {ticket.bayId ? (
                    <span className="inline-flex items-center gap-1.5 rounded-lg bg-sky-50 px-3 py-1 text-base font-bold text-sky-700 border border-sky-200">
                      حفرة {ticket.bayId}
                    </span>
                  ) : (
                    <span className="text-slate-400 text-sm font-medium">
                      غير محددة (في الطابور)
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
