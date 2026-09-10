import { useEffect, useState } from "react";
import { toast } from "sonner";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { useGarage } from "@/hooks/useGarage";
import { api } from "@/services/tauri";
import { todayIso } from "@/utils/format";
import { t } from "@/utils/i18n";
import type { DailyReport } from "@/types";

export function ReportsPage() {
  const { locale } = useGarage();
  const [date, setDate] = useState(todayIso());
  const [report, setReport] = useState<DailyReport | null>(null);

  async function load(value = date) {
    try {
      setReport(await api.report(value));
    } catch (error) {
      toast.error(String(error));
    }
  }

  useEffect(() => {
    void load();
  }, []);

  const cards = report
    ? [
        [t(locale, "todayTotal"), report.total],
        [t(locale, "todayDone"), report.completed],
        [t(locale, "todayCancel"), report.cancelled],
        [t(locale, "todayWait"), report.waiting],
        [t(locale, "todayServed"), report.served],
      ]
    : [];

  return (
    <div className="space-y-6">
      <Input
        type="date"
        className="max-w-xs"
        value={date}
        onChange={(e) => {
          setDate(e.target.value);
          void load(e.target.value);
        }}
      />
      <div className="grid grid-cols-2 gap-4 xl:grid-cols-5">
        {cards.map(([label, value]) => (
          <Card key={label}>
            <CardHeader>
              <CardTitle className="text-base">{label}</CardTitle>
            </CardHeader>
            <CardContent className="text-4xl font-black">{value}</CardContent>
          </Card>
        ))}
      </div>
    </div>
  );
}
