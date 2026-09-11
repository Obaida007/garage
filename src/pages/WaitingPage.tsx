import { useGarage } from "@/hooks/useGarage";
import { WaitingBoardView } from "@/features/waiting/WaitingBoard";

export function WaitingPage() {
  const { snapshot, loading, error } = useGarage();

  if (error) {
    return (
      <div className="flex h-screen items-center justify-center bg-slate-900 text-red-500 dir-rtl p-8 text-center">
        <div className="text-xl font-bold">حدث خطأ أثناء تحميل البيانات:<br/>{error}</div>
      </div>
    );
  }

  if (loading || !snapshot) {
    return (
      <div className="flex h-screen items-center justify-center bg-slate-900 text-amber-500 dir-rtl">
        <div className="text-3xl font-black">جاري التحميل...</div>
      </div>
    );
  }

  return <WaitingBoardView board={snapshot.board} bays={snapshot.bays} />;
}
