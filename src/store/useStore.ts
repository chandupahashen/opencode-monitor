import { create } from "zustand";
import type { Overview, SessionRow, TokenTrend, ModelStat, ProjectStat, DayActivity, CostBreakdown } from "../types";

interface AppStore {
  overview: Overview | null;
  sessions: SessionRow[];
  tokenTrends: TokenTrend[];
  modelUsage: ModelStat[];
  projectStats: ProjectStat[];
  dailyActivity: DayActivity[];
  costBreakdown: CostBreakdown | null;
  isLoading: boolean;
  error: string | null;
  refreshInterval: number;
  currentTab: string;

  setOverview: (data: Overview) => void;
  setSessions: (data: SessionRow[]) => void;
  setTokenTrends: (data: TokenTrend[]) => void;
  setModelUsage: (data: ModelStat[]) => void;
  setProjectStats: (data: ProjectStat[]) => void;
  setDailyActivity: (data: DayActivity[]) => void;
  setCostBreakdown: (data: CostBreakdown) => void;
  setLoading: (v: boolean) => void;
  setError: (e: string | null) => void;
  setRefreshInterval: (v: number) => void;
  setCurrentTab: (t: string) => void;
}

export const useStore = create<AppStore>((set) => ({
  overview: null,
  sessions: [],
  tokenTrends: [],
  modelUsage: [],
  projectStats: [],
  dailyActivity: [],
  costBreakdown: null,
  isLoading: false,
  error: null,
  refreshInterval: 15,
  currentTab: "dashboard",

  setOverview: (data) => set({ overview: data }),
  setSessions: (data) => set({ sessions: data }),
  setTokenTrends: (data) => set({ tokenTrends: data }),
  setModelUsage: (data) => set({ modelUsage: data }),
  setProjectStats: (data) => set({ projectStats: data }),
  setDailyActivity: (data) => set({ dailyActivity: data }),
  setCostBreakdown: (data) => set({ costBreakdown: data }),
  setLoading: (v) => set({ isLoading: v }),
  setError: (e) => set({ error: e }),
  setRefreshInterval: (v) => set({ refreshInterval: v }),
  setCurrentTab: (t) => set({ currentTab: t }),
}));
