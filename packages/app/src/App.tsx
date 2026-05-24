import { Routes, Route, Outlet, useLocation, Navigate } from "react-router-dom";
import { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import Sidebar from "./components/Sidebar";
import { ErrorBoundary } from "./components/ErrorBoundary";
import { useAppStore } from "./stores/appStore";
import { OnboardingWizard } from "@me/ui";
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
  useEffect(() => {
    if (currentPage !== location.pathname) {
      navigateStore(location.pathname);
    }
  }, [location.pathname, currentPage, navigateStore]);

  return (
    <div
      style={{
        display: "flex",
        height: "100vh",
        background: "#0C0C0C",
        color: "#D4D4D4",
        fontFamily: `"Inter", -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif`,
      }}
    >
      <Sidebar />
      <Outlet />
    </div>
  );
}

export default function App() {
  const [showOnboarding, setShowOnboarding] = useState(false);
  const [demoLoading, setDemoLoading] = useState(false);
  const navigate = useAppStore((s) => s.navigate);

  useEffect(() => {
    const done = localStorage.getItem("has_completed_onboarding");
    if (done !== "true") {
      setShowOnboarding(true);
    }
  }, []);

  const handleLoadDemo = useCallback(async () => {
    setDemoLoading(true);
    try {
      // Demo data is auto-seeded by migrations on first run
      // If no stocks exist, trigger download for demo stocks
      const summary = await invoke<{ total_stocks: number }>("get_data_summary");
      if (summary.total_stocks === 0) {
        await invoke("download_stock_data", { code: "600519", name: "贵州茅台" });
        await invoke("download_stock_data", { code: "300750", name: "宁德时代" });
        await invoke("download_stock_data", { code: "600036", name: "招商银行" });
      }
      setDemoLoading(false);
    } catch (e) {
      console.error("Demo load failed:", e);
      setDemoLoading(false);
      throw e;
    }
  }, []);

  const handleImportTdx = useCallback(() => {
    navigate("/chart");
    // Trigger import dialog — the ChartPage has an ImportDialog component
    setTimeout(() => {
      const event = new CustomEvent("open-import-dialog");
      window.dispatchEvent(event);
    }, 500);
  }, [navigate]);

  const handleComplete = useCallback(() => {
    localStorage.setItem("has_completed_onboarding", "true");
    setShowOnboarding(false);
    navigate("/chart");
  }, [navigate]);

  return (
    <ErrorBoundary>
      {showOnboarding && (
        <OnboardingWizard
          onComplete={handleComplete}
          onLoadDemo={handleLoadDemo}
          onImportTdx={handleImportTdx}
          demoLoading={demoLoading}
        />
      )}
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
    </ErrorBoundary>
  );
}
