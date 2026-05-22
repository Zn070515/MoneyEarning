import { Routes, Route, Outlet, useLocation, Navigate } from "react-router-dom";
import Sidebar from "./components/Sidebar";
import { useAppStore } from "./stores/appStore";
import DashboardPage from "./pages/DashboardPage";
import ChartPage from "./pages/ChartPage";
import BacktestPage from "./pages/BacktestPage";
import ScannerPage from "./pages/ScannerPage";
import PortfolioPage from "./pages/PortfolioPage";
import ReviewPage from "./pages/ReviewPage";
import MEScriptPage from "./pages/MEScriptPage";
import SettingsPage from "./pages/SettingsPage";

function Layout() {
  const location = useLocation();
  const navigateStore = useAppStore((s) => s.navigate);

  // Sync react-router location → zustand store
  const currentPage = useAppStore((s) => s.currentPage);
  if (currentPage !== location.pathname) {
    navigateStore(location.pathname);
  }

  return (
    <div
      style={{
        display: "flex",
        height: "100vh",
        background: "#0f0f23",
        color: "#ccc",
      }}
    >
      <Sidebar />
      <Outlet />
    </div>
  );
}

export default function App() {
  return (
    <Routes>
      <Route element={<Layout />}>
        <Route path="/" element={<DashboardPage />} />
        <Route path="/chart" element={<ChartPage />} />
        <Route path="/chart/:stockId" element={<ChartPage />} />
        <Route path="/backtest" element={<BacktestPage />} />
        <Route path="/scanner" element={<ScannerPage />} />
        <Route path="/portfolio" element={<PortfolioPage />} />
        <Route path="/review" element={<ReviewPage />} />
        <Route path="/editor" element={<MEScriptPage />} />
        <Route path="/settings" element={<SettingsPage />} />
        <Route path="*" element={<Navigate to="/" replace />} />
      </Route>
    </Routes>
  );
}
