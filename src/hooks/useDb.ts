import { invoke } from "@tauri-apps/api/core";
import { useStore } from "../store/useStore";
import type { Overview, SessionRow, SessionDetail, TokenTrend, ModelStat, ProjectStat, DayActivity, CostBreakdown } from "../types";

export function useDb() {
  async function fetchOverview(): Promise<Overview> {
    return invoke<Overview>("get_overview");
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

  async function fetchModelUsage(): Promise<ModelStat[]> {
    return invoke<ModelStat[]>("get_model_usage");
  }

  async function fetchProjectStats(): Promise<ProjectStat[]> {
    return invoke<ProjectStat[]>("get_project_stats");
  }

  async function fetchDailyActivity(months?: number): Promise<DayActivity[]> {
    return invoke<DayActivity[]>("get_daily_activity", { months: months ?? 12 });
  }

  async function fetchCostBreakdown(): Promise<CostBreakdown> {
    return invoke<CostBreakdown>("get_cost_breakdown");
  }

  async function fetchModelsList(): Promise<string[]> {
    return invoke<string[]>("get_models_list");
  }

  async function fetchProjectsList(): Promise<string[]> {
    return invoke<string[]>("get_projects_list");
  }

  async function setDbPath(path: string): Promise<void> {
    return invoke("set_db_path", { path });
  }

  async function setRefreshInterval(interval: number): Promise<void> {
    return invoke("set_refresh_interval", { interval });
  }

  async function refreshAll() {
    useStore.getState().setLoading(true);
    useStore.getState().setError(null);
    try {
      const [overview, sessions, tokenTrends, modelUsage, projectStats, dailyActivity, costBreakdown] =
        await Promise.all([
          fetchOverview(),
          fetchSessions(),
          fetchTokenTrends(),
          fetchModelUsage(),
          fetchProjectStats(),
          fetchDailyActivity(),
          fetchCostBreakdown(),
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
  }

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
    setDbPath,
    setRefreshInterval,
    refreshAll,
  };
}
