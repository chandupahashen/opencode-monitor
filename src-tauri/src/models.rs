use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Overview {
    pub total_sessions: i64,
    pub total_tokens_input: i64,
    pub total_tokens_output: i64,
    pub total_tokens_reasoning: i64,
    pub total_tokens_cache_read: i64,
    pub total_tokens_cache_write: i64,
    pub total_cost: f64,
    pub total_projects: i64,
    pub active_sessions: i64,
}

#[derive(Debug, Serialize)]
pub struct SessionRow {
    pub id: String,
    pub title: Option<String>,
    pub model: Option<String>,
    pub agent: Option<String>,
    pub tokens_input: i64,
    pub tokens_output: i64,
    pub tokens_reasoning: i64,
    pub tokens_cache_read: i64,
    pub tokens_cache_write: i64,
    pub cost: f64,
    pub summary_additions: i64,
    pub summary_deletions: i64,
    pub summary_files: i64,
    pub time_created: i64,
    pub time_updated: i64,
    pub project_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SessionDetail {
    pub id: String,
    pub title: Option<String>,
    pub model: Option<String>,
    pub agent: Option<String>,
    pub cost: f64,
    pub tokens_input: i64,
    pub tokens_output: i64,
    pub tokens_reasoning: i64,
    pub tokens_cache_read: i64,
    pub tokens_cache_write: i64,
    pub summary_additions: i64,
    pub summary_deletions: i64,
    pub summary_files: i64,
    pub time_created: i64,
    pub time_updated: i64,
    pub time_archived: Option<i64>,
    pub directory: Option<String>,
    pub project_id: Option<String>,
    pub messages: Vec<MessageInfo>,
}

#[derive(Debug, Serialize)]
pub struct MessageInfo {
    pub id: String,
    pub time_created: i64,
    pub parts_count: i64,
}

#[derive(Debug, Serialize)]
pub struct TokenTrend {
    pub date: String,
    pub tokens_input: i64,
    pub tokens_output: i64,
    pub tokens_reasoning: i64,
    pub tokens_cache_read: i64,
    pub tokens_cache_write: i64,
    pub cost: f64,
    pub session_count: i64,
}

#[derive(Debug, Serialize)]
pub struct ModelStat {
    pub model: String,
    pub session_count: i64,
    pub total_tokens: i64,
    pub total_cost: f64,
}

#[derive(Debug, Serialize)]
pub struct ProjectStat {
    pub project_id: String,
    pub session_count: i64,
    pub total_tokens: i64,
    pub total_cost: f64,
}

#[derive(Debug, Serialize)]
pub struct DayActivity {
    pub date: String,
    pub session_count: i64,
    pub total_tokens: i64,
}

#[derive(Debug, Serialize)]
pub struct CostBreakdown {
    pub by_model: Vec<ModelStat>,
    pub by_project: Vec<ProjectStat>,
}

#[derive(Debug, Serialize)]
pub struct AppSettings {
    pub db_path: Option<String>,
    pub refresh_interval: u64,
    pub theme: String,
}
