import type { WaitingBoard } from "@/types";

export function WaitingBoardView({ board }: { board: WaitingBoard }) {
  return (
    <div className="flex min-h-screen flex-col bg-slate-950 text-white">
      <header className="border-b border-slate-800 py-6 text-center text-4xl font-black tracking-wide text-amber-300">
        {board.garageName}
      </header>
      <div className="grid flex-1 grid-cols-1 gap-8 p-10 xl:grid-cols-2">
        <section className="flex flex-col items-center justify-center rounded-3xl bg-slate-900 p-10 text-center">
          <div className="text-4xl font-bold text-slate-300">الدور الحالي</div>
          <div className="mt-4 text-[10rem] font-black leading-none text-amber-400">
            {board.currentTicket ?? "--"}
          </div>
          <div className="mt-8 text-4xl text-slate-300">توجه إلى الحفرة</div>
          <div className="mt-2 text-[8rem] font-black text-sky-400">{board.currentBay ?? "-"}</div>
        </section>
        <section className="flex flex-col items-center justify-center rounded-3xl bg-slate-900 p-10 text-center">
          <div className="text-4xl font-bold text-slate-300">الدور التالي</div>
          <div className="mt-4 text-[9rem] font-black leading-none text-white">
            {board.nextTicket ?? "--"}
          </div>
          <div className="mt-10 text-3xl text-slate-400">المنتظرون: {board.waitingCount}</div>
        </section>
      </div>
    </div>
  );
}
