import React, { useEffect, useRef } from "react";
import { Activity, BarChart3, Database, FolderOpen, Settings, RefreshCw, Terminal, Zap } from "lucide-react";
import { useStore } from "../store/useStore";
import { useAutoRefresh } from "../hooks/useAutoRefresh";
import { useDb } from "../hooks/useDb";
import { Titlebar } from "./Titlebar";
import { AppLoader } from "./Skeleton";

const tabs = [
  { id: "dashboard", label: "Dashboard", icon: Activity, desc: "Overview & KPIs" },
  { id: "sessions", label: "Sessions", icon: Database, desc: "Session history & details" },
  { id: "analytics", label: "Analytics", icon: BarChart3, desc: "Charts & breakdowns" },
  { id: "projects", label: "Projects", icon: FolderOpen, desc: "Per-project stats" },
  { id: "settings", label: "Settings", icon: Settings, desc: "Configuration" },
];

export function Layout({ children }: { children: React.ReactNode }) {
  const currentTab = useStore((s) => s.currentTab);
  const setCurrentTab = useStore((s) => s.setCurrentTab);
  const isLoading = useStore((s) => s.isLoading);
  const error = useStore((s) => s.error);
  const refreshInterval = useStore((s) => s.refreshInterval);
  const overview = useStore((s) => s.overview);
  const { refreshAll } = useDb();
  const [autoRefresh, setAutoRefresh] = React.useState(true);

  useAutoRefresh(autoRefresh);

  const dateFrom = useStore((s) => s.dateFrom);
  const dateTo = useStore((s) => s.dateTo);
  const prevFilter = useRef({ dateFrom: "", dateTo: "" });
  useEffect(() => {
    const prev = prevFilter.current;
    if (prev.dateFrom !== dateFrom || prev.dateTo !== dateTo) {
      prevFilter.current = { dateFrom, dateTo };
      refreshAll();
    }
  }, [dateFrom, dateTo, refreshAll]);

  const currentTabMeta = tabs.find((t) => t.id === currentTab);

  const initialLoading = isLoading && !overview;

  return (
    <div className="flex flex-col h-screen bg-surface-900 text-gray-100">
      {initialLoading && <AppLoader />}
      <Titlebar />
      <div className="flex flex-1 min-h-0">
        <aside className="w-58 flex-shrink-0 flex flex-col border-r border-border bg-surface-800 relative z-10">
          <div className="flex items-center gap-3 px-4 h-12 border-b border-border">
            <div className="w-7 h-7 rounded-lg bg-gradient-to-br from-accent to-cyan-600 flex items-center justify-center flex-shrink-0">
              <Terminal className="w-3.5 h-3.5 text-white" />
            </div>
            <div className="min-w-0">
              <h1 className="text-xs font-bold leading-tight text-gray-200">OC Monitor</h1>
              <p className="text-[9px] text-gray-600 leading-tight">Usage Dashboard</p>
            </div>
          </div>

          <nav className="flex-1 p-2 space-y-0.5">
            {tabs.map((tab) => {
              const Icon = tab.icon;
              const isActive = currentTab === tab.id;
              return (
                <button
                  key={tab.id}
                  onClick={() => setCurrentTab(tab.id)}
                  className={`w-full flex items-center gap-3 px-3 py-2 rounded-lg text-xs transition-all ${
                    isActive
                      ? "bg-accent-glow text-accent border border-accent/20"
                      : "text-gray-500 hover:text-gray-300 hover:bg-surface-700/50 border border-transparent"
                  }`}
                >
                  <Icon className={`w-3.5 h-3.5 flex-shrink-0 ${isActive ? "text-accent" : ""}`} />
                  <div className="text-left min-w-0">
                    <div className="text-xs font-medium truncate">{tab.label}</div>
                    <div className="text-[9px] text-gray-600 truncate">{tab.desc}</div>
                  </div>
                </button>
              );
            })}
          </nav>

          <div className="p-2 border-t border-border space-y-1.5">
            <div className="px-2 py-1.5">
              <label className="flex items-center gap-2 cursor-pointer">
                <div className={`relative w-7 h-3.5 rounded-full transition-colors ${autoRefresh ? "bg-accent/60" : "bg-surface-500"}`}>
                  <div
                    className={`absolute top-0.5 w-2.5 h-2.5 bg-white rounded-full transition-all ${autoRefresh ? "left-3.5" : "left-0.5"}`}
                    onClick={() => setAutoRefresh(!autoRefresh)}
                  />
                  <input type="checkbox" checked={autoRefresh} onChange={(e) => setAutoRefresh(e.target.checked)} className="hidden" />
                </div>
                <span className="text-[10px] text-gray-600">
                  Auto <span className="text-gray-600">{refreshInterval}s</span>
                </span>
              </label>
            </div>
            <button
              onClick={refreshAll}
              disabled={isLoading}
              className="w-full flex items-center justify-center gap-2 px-3 py-1.5 text-[10px] bg-surface-700/50 hover:bg-surface-600/50 rounded-lg transition-all disabled:opacity-40 border border-border"
            >
              <RefreshCw className={`w-3 h-3 ${isLoading ? "animate-spin" : ""}`} />
              {isLoading ? "Refreshing..." : "Refresh"}
            </button>
          </div>
        </aside>

        <main className="flex-1 flex flex-col min-w-0">
          <header className="h-10 flex items-center justify-between px-5 border-b border-border bg-surface-800">
            <div className="flex items-center gap-3">
              <h2 className="text-xs font-semibold text-gray-300">{currentTabMeta?.label}</h2>
              <span className="text-[10px] text-gray-600 hidden sm:inline">{currentTabMeta?.desc}</span>
            </div>
            {overview && (
              <div className="flex items-center gap-3 text-[10px] text-gray-600">
                <span className="flex items-center gap-1">
                  <Zap className="w-2.5 h-2.5 text-accent" />
                  {overview.total_sessions} sessions
                </span>
                <span className="hidden sm:flex items-center gap-1">
                  <Activity className="w-2.5 h-2.5 text-green-accent" />
                  {overview.active_sessions} active
                </span>
              </div>
            )}
          </header>

          {error && (
            <div className="bg-red-950/60 border-b border-red-900/50 px-5 py-2 text-xs text-red-300 flex items-center gap-2">
              <span className="w-1.5 h-1.5 rounded-full bg-red-accent animate-pulse-dot" />
              {error}
            </div>
          )}

          <div className="flex-1 overflow-auto p-5">{children}</div>
        </main>
      </div>
    </div>
  );
}
