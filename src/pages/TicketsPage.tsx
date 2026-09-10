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

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t(locale, "tickets")}</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="flex gap-2">
          <Input
            placeholder="026"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") void load();
            }}
          />
          <Button onClick={() => void load()}>{t(locale, "search")}</Button>
        </div>
        <div className="space-y-2">
          {items.map((ticket) => (
            <div key={ticket.id} className="flex items-center justify-between rounded-lg border p-3">
              <div>
                <div className="text-2xl font-black">{ticket.ticketNumber}</div>
                <div className="text-sm text-muted-foreground">{ticket.createdAt}</div>
              </div>
              <div className="flex items-center gap-3">
                <Badge variant="muted">{ticket.status}</Badge>
                <Button
                  variant="outline"
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
          ))}
        </div>
      </CardContent>
    </Card>
  );
}
