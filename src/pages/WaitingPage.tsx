import { useGarage } from "@/hooks/useGarage";
import { api } from "@/services/tauri";
import { WaitingBoardView } from "@/features/waiting/WaitingBoard";

export function WaitingPage() {
  const { snapshot, loading, error, locale } = useGarage();

  if (error) {
    return (
      <div className="dir-rtl flex h-screen items-center justify-center bg-slate-100 p-8 text-center text-red-600">
        <div className="text-xl font-bold">
          حدث خطأ أثناء تحميل البيانات:
          <br />
          {error}
        </div>
      </div>
    );
  }

  if (loading || !snapshot) {
    return (
      <div className="dir-rtl flex h-screen items-center justify-center bg-slate-100 text-slate-500">
        <div className="text-3xl font-black">جاري التحميل...</div>
      </div>
    );
  }

  return (
    <WaitingBoardView
      board={snapshot.board}
      bays={snapshot.bays}
      locale={locale}
      logoPath={snapshot.settings.logoPath}
      layout={snapshot.settings.waitingLayout}
      onLayoutChange={(waitingLayout) => {
        void api.saveSettings({ ...snapshot.settings, waitingLayout }).catch(() => {});
      }}
    />
  );
}
