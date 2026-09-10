import { HashRouter, Navigate, Route, Routes } from "react-router-dom";
import { AppShell } from "@/components/layout/AppShell";
import { CashierPage } from "@/pages/CashierPage";
import { WaitingPage } from "@/pages/WaitingPage";
import { SettingsPage } from "@/pages/SettingsPage";
import { ReportsPage } from "@/pages/ReportsPage";
import { TicketsPage } from "@/pages/TicketsPage";

export default function App() {
  return (
    <HashRouter>
      <Routes>
        <Route path="/waiting" element={<WaitingPage />} />
        <Route path="/" element={<AppShell />}>
          <Route index element={<CashierPage />} />
          <Route path="tickets" element={<TicketsPage />} />
          <Route path="reports" element={<ReportsPage />} />
          <Route path="settings" element={<SettingsPage />} />
        </Route>
        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
    </HashRouter>
  );
}
