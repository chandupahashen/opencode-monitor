import { useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useStore } from "../store/useStore";
import type { Overview, SessionRow, SessionDetail, TokenTrend, ModelStat, ProjectStat, DayActivity, CostBreakdown, HealthStatus } from "../types";

export function useDb() {
  async function fetchOverview(opts?: { dateFrom?: number; dateTo?: number }): Promise<Overview> {
    return invoke<Overview>("get_overview", { dateFrom: opts?.dateFrom ?? null, dateTo: opts?.dateTo ?? null });
  }

  async function fetchSessions(opts?: {
    limit?: number;
    offset?: number;
    model?: string;
    project?: string;
    dateFrom?: number;
    dateTo?: number;
  }): Promise<SessionRow[]> {
    return invoke<SessionRow[]>("get_sessions", {
      limit: opts?.limit ?? 50,
      offset: opts?.offset ?? 0,
      modelFilter: opts?.model ?? null,
      projectFilter: opts?.project ?? null,
      dateFrom: opts?.dateFrom ?? null,
      dateTo: opts?.dateTo ?? null,
    });
  }

  async function fetchSessionDetail(sessionId: string): Promise<SessionDetail> {
    return invoke<SessionDetail>("get_session_detail", { sessionId });
  }

  async function fetchTokenTrends(days?: number): Promise<TokenTrend[]> {
    return invoke<TokenTrend[]>("get_token_trends", { days: days ?? 30 });
  }

  async function fetchModelUsage(opts?: { dateFrom?: number; dateTo?: number }): Promise<ModelStat[]> {
    return invoke<ModelStat[]>("get_model_usage", { dateFrom: opts?.dateFrom ?? null, dateTo: opts?.dateTo ?? null });
  }

  async function fetchProjectStats(opts?: { dateFrom?: number; dateTo?: number }): Promise<ProjectStat[]> {
    return invoke<ProjectStat[]>("get_project_stats", { dateFrom: opts?.dateFrom ?? null, dateTo: opts?.dateTo ?? null });
  }

  async function fetchDailyActivity(months?: number): Promise<DayActivity[]> {
    return invoke<DayActivity[]>("get_daily_activity", { months: months ?? 12 });
  }

  async function fetchCostBreakdown(opts?: { dateFrom?: number; dateTo?: number }): Promise<CostBreakdown> {
    return invoke<CostBreakdown>("get_cost_breakdown", { dateFrom: opts?.dateFrom ?? null, dateTo: opts?.dateTo ?? null });
  }

  async function fetchModelsList(): Promise<string[]> {
    return invoke<string[]>("get_models_list");
  }

  async function fetchProjectsList(): Promise<string[]> {
    return invoke<string[]>("get_projects_list");
  }

  async function fetchHealthStatus(): Promise<HealthStatus> {
    return invoke<HealthStatus>("health_check");
  }

  async function setDbPath(path: string): Promise<void> {
    return invoke("set_db_path", { path });
  }

  async function setRefreshInterval(interval: number): Promise<void> {
    return invoke("set_refresh_interval", { interval });
  }

  function dateToMs(dateStr: string): number | undefined {
    if (!dateStr) return undefined;
    const d = new Date(dateStr + "T00:00:00");
    return d.getTime();
  }

  const refreshAll = useCallback(async () => {
    const store = useStore.getState();
    store.setLoading(true);
    store.setError(null);
    try {
      const df = dateToMs(store.dateFrom);
      const dt = dateToMs(store.dateTo);
      const opts = df || dt ? { dateFrom: df, dateTo: dt ? dt + 86_400_000 : undefined } : undefined;

      const [overview, sessions, tokenTrends, modelUsage, projectStats, dailyActivity, costBreakdown] =
        await Promise.all([
          fetchOverview(opts),
          fetchSessions(opts),
          fetchTokenTrends(),
          fetchModelUsage(opts),
          fetchProjectStats(opts),
          fetchDailyActivity(),
          fetchCostBreakdown(opts),
        ]);
      useStore.getState().setOverview(overview);
      useStore.getState().setSessions(sessions);
      useStore.getState().setTokenTrends(tokenTrends);
      useStore.getState().setModelUsage(modelUsage);
      useStore.getState().setProjectStats(projectStats);
      useStore.getState().setDailyActivity(dailyActivity);
      useStore.getState().setCostBreakdown(costBreakdown);
    } catch (e: unknown) {
      useStore.getState().setError(String(e));
    } finally {
      useStore.getState().setLoading(false);
    }
  }, []);

  return {
    fetchOverview,
    fetchSessions,
    fetchSessionDetail,
    fetchTokenTrends,
    fetchModelUsage,
    fetchProjectStats,
    fetchDailyActivity,
    fetchCostBreakdown,
    fetchModelsList,
    fetchProjectsList,
    fetchHealthStatus,
    setDbPath,
    setRefreshInterval,
    refreshAll,
  };
}
