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
    pub project_name: String,
    pub session_count: i64,
    pub total_tokens: i64,
    pub total_cost: f64,
}

pub(crate) fn extract_project_name(id: &str, dir: Option<&str>) -> String {
    let normalized = id.replace('\\', "/");
    let parts: Vec<&str> = normalized.split('/').filter(|s| !s.is_empty()).collect();
    if parts.len() > 1 {
        return parts[parts.len() - 1].to_string();
    }
    if let Some(d) = dir {
        let dn = d.replace('\\', "/");
        let dp: Vec<&str> = dn.split('/').filter(|s| !s.is_empty()).collect();
        if let Some(last) = dp.last() {
            if !last.is_empty() {
                return last.to_string();
            }
        }
    }
    let trimmed = id.trim();
    if trimmed.len() <= 16 {
        return trimmed.to_string();
    }
    let truncated: String = trimmed.chars().take(12).collect();
    format!("{}…", truncated)
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

#[derive(Debug, Serialize)]
pub struct HealthStatus {
    pub version: String,
    pub uptime: String,
    pub db_connected: bool,
    pub db_path: Option<String>,
}
