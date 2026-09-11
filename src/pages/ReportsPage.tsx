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
          <Card key={label as string}>
            <CardHeader>
              <CardTitle className="text-base">{label}</CardTitle>
            </CardHeader>
            <CardContent className="text-4xl font-black">{value}</CardContent>
          </Card>
        ))}
      </div>

      {report?.bayStats && report.bayStats.length > 0 && (
        <div className="mt-8">
          <h2 className="text-2xl font-bold mb-4 text-slate-800">أداء الحفر (السيارات المخدومة)</h2>
          <div className="grid grid-cols-2 gap-4 md:grid-cols-3 lg:grid-cols-6">
            {report.bayStats.map((stat) => (
              <Card key={stat.bayId} className="border-sky-200 bg-sky-50 shadow-sm">
                <CardHeader className="pb-2">
                  <CardTitle className="text-sm font-bold text-sky-800">حفرة {stat.bayId}</CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="text-3xl font-black text-sky-600">{stat.completed}</div>
                  <div className="text-xs text-sky-700/70 mt-1 font-bold">سيارة</div>
                </CardContent>
              </Card>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
