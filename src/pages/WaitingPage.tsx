import { useGarage } from "@/hooks/useGarage";
import { WaitingBoardView } from "@/features/waiting/WaitingBoard";

export function WaitingPage() {
  const { snapshot } = useGarage();
  if (!snapshot) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-slate-950 text-6xl text-white">
        ...
      </div>
    );
  }
  return <WaitingBoardView board={snapshot.board} />;
}
