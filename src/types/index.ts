export interface Overview {
  total_sessions: number;
  total_tokens_input: number;
  total_tokens_output: number;
  total_tokens_reasoning: number;
  total_tokens_cache_read: number;
  total_tokens_cache_write: number;
  total_cost: number;
  total_projects: number;
  active_sessions: number;
}

export interface SessionRow {
  id: string;
  title: string | null;
  model: string | null;
  agent: string | null;
  tokens_input: number;
  tokens_output: number;
  tokens_reasoning: number;
  tokens_cache_read: number;
  tokens_cache_write: number;
  cost: number;
  summary_additions: number;
  summary_deletions: number;
  summary_files: number;
  time_created: number;
  time_updated: number;
  project_id: string | null;
}

export interface SessionDetail {
  id: string;
  title: string | null;
  model: string | null;
  agent: string | null;
  cost: number;
  tokens_input: number;
  tokens_output: number;
  tokens_reasoning: number;
  tokens_cache_read: number;
  tokens_cache_write: number;
  summary_additions: number;
  summary_deletions: number;
  summary_files: number;
  time_created: number;
  time_updated: number;
  time_archived: number | null;
  directory: string | null;
  project_id: string | null;
  messages: MessageInfo[];
}

export interface MessageInfo {
  id: string;
  time_created: number;
  parts_count: number;
}

export interface TokenTrend {
  date: string;
  tokens_input: number;
  tokens_output: number;
  tokens_reasoning: number;
  tokens_cache_read: number;
  tokens_cache_write: number;
  cost: number;
  session_count: number;
}

export interface ModelStat {
  model: string;
  session_count: number;
  total_tokens: number;
  total_cost: number;
}

export interface ProjectStat {
  project_id: string;
  project_name: string;
  session_count: number;
  total_tokens: number;
  total_cost: number;
}

export interface DayActivity {
  date: string;
  session_count: number;
  total_tokens: number;
}

export interface CostBreakdown {
  by_model: ModelStat[];
  by_project: ProjectStat[];
}

export interface AppSettings {
  db_path: string | null;
  refresh_interval: number;
  theme: string;
}

export interface HealthStatus {
  version: string;
  uptime: string;
  db_connected: boolean;
  db_path: string | null;
}
