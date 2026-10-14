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
    /// v2 slug (e.g. "brave-mountain"). None on v1 DBs.
    pub slug: Option<String>,
    /// v2 parent session id for forks. None on v1 DBs / root sessions.
    pub parent_id: Option<String>,
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
    /// v2 slug. None on v1 DBs.
    pub slug: Option<String>,
    /// v2 parent session id for forks.
    pub parent_id: Option<String>,
    pub messages: Vec<MessageInfo>,
}

#[derive(Debug, Serialize)]
pub struct MessageInfo {
    pub id: String,
    pub time_created: i64,
    pub parts_count: i64,
    /// v2 `session_message.type` (assistant/user/system/synthetic/idle/...).
    /// Empty string when sourced from legacy tables.
    #[serde(default)]
    pub message_type: String,
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
    /// Full worktree / directory path for display. Frontend can show this instead of deriving from id.
    pub directory: Option<String>,
    pub session_count: i64,
    pub total_tokens: i64,
    pub total_cost: f64,
}

pub(crate) fn extract_project_name(id: &str, dir: Option<&str>) -> String {
    // Prefer directory / worktree basename if available (new DB has proper project.worktree)
    if let Some(d) = dir {
        let dn = d.replace('\\', "/");
        let dp: Vec<&str> = dn.split('/').filter(|s| !s.is_empty()).collect();
        if let Some(last) = dp.last() {
            if !last.is_empty() {
                return last.to_string();
            }
        }
    }
    let normalized = id.replace('\\', "/");
    let parts: Vec<&str> = normalized.split('/').filter(|s| !s.is_empty()).collect();
    if parts.len() > 1 {
        return parts[parts.len() - 1].to_string();
    }
    let trimmed = id.trim();
    if trimmed.len() <= 16 {
        return trimmed.to_string();
    }
    let truncated: String = trimmed.chars().take(12).collect();
    format!("{}…", truncated)
}

/// Extract readable model id from raw DB value.
/// Handles:
/// - v2 JSON `{"id":"...","providerID":"opencode","variant":"xhigh"}`
/// - v2 string ref `provider/model#variant` (e.g. `opencode/muse-spark-1.3#xhigh`)
/// - bare `model#variant`
/// - legacy plain string + `provider/model` slash form + `modelID`/`modelId` keys.
/// The returned id strips any `#variant` suffix so variants collapse for grouping.
pub(crate) fn extract_model_id(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return trimmed.to_string();
    }
    // Try JSON parse
    if (trimmed.starts_with('{') && trimmed.ends_with('}')) || trimmed.starts_with('"') {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(trimmed) {
            if let Some(obj) = v.as_object() {
                if let Some(id) = obj.get("id").and_then(|x| x.as_str()) {
                    return strip_variant(id);
                }
                if let Some(mid) = obj.get("modelID").and_then(|x| x.as_str()) {
                    return strip_variant(mid);
                }
                if let Some(mid) = obj.get("modelId").and_then(|x| x.as_str()) {
                    return strip_variant(mid);
                }
            } else if let Some(s) = v.as_str() {
                return strip_variant(s);
            }
        }
    }
    strip_variant(trimmed)
}

/// Strip `#variant` suffix then `provider/` prefix.
/// `opencode/muse-spark-1.3#xhigh` -> `muse-spark-1.3`
fn strip_variant(s: &str) -> String {
    let no_variant = match s.find('#') {
        Some(idx) => &s[..idx],
        None => s,
    };
    // Handle slash provider prefix like "opencode/deepseek-v4-flash-free"
    if let Some(idx) = no_variant.rfind('/') {
        let left = &no_variant[..idx];
        if !left.contains(' ') && !left.contains('{') {
            return no_variant[idx + 1..].to_string();
        }
    }
    no_variant.to_string()
}

/// Extract `variant` (`#suffix` or JSON `variant` field) for display. Returns None when absent.
#[allow(dead_code)]
pub(crate) fn extract_model_variant(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.starts_with('{') {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(trimmed) {
            if let Some(obj) = v.as_object() {
                if let Some(var) = obj.get("variant").and_then(|x| x.as_str()) {
                    if !var.is_empty() {
                        return Some(var.to_string());
                    }
                }
            }
        }
        // JSON may still carry #variant inside id; fall through to string handling on id?
        // Fall through: check raw for trailing # (unlikely in JSON, but harmless).
    }
    match trimmed.find('#') {
        Some(idx) if idx + 1 < trimmed.len() => Some(trimmed[idx + 1..].to_string()),
        _ => None,
    }
}

pub(crate) fn normalize_model_filter(input: &str) -> String {
    extract_model_id(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_id_json_v2() {
        assert_eq!(
            extract_model_id(r#"{"id":"muse-spark-1.3-contributor-free","providerID":"opencode","variant":"xhigh"}"#),
            "muse-spark-1.3-contributor-free"
        );
        assert_eq!(
            extract_model_id(r#"{"id":"deepseek-v4-flash-free","providerID":"opencode","variant":"max"}"#),
            "deepseek-v4-flash-free"
        );
    }

    #[test]
    fn model_id_string_ref_v2() {
        // provider/model#variant — variant stripped for grouping.
        assert_eq!(
            extract_model_id("opencode/muse-spark-1.3-contributor-free#xhigh"),
            "muse-spark-1.3-contributor-free"
        );
        assert_eq!(
            extract_model_id("anthropic/claude-sonnet-4-5#high"),
            "claude-sonnet-4-5"
        );
        assert_eq!(extract_model_id("opencode/deepseek-v4-flash-free"), "deepseek-v4-flash-free");
        assert_eq!(extract_model_id("plain-model"), "plain-model");
    }

    #[test]
    fn model_variant_extraction() {
        assert_eq!(
            extract_model_variant(r#"{"id":"x","providerID":"opencode","variant":"xhigh"}"#),
            Some("xhigh".to_string())
        );
        assert_eq!(
            extract_model_variant("opencode/foo#xhigh"),
            Some("xhigh".to_string())
        );
        assert_eq!(extract_model_variant("plain"), None);
    }

    #[test]
    fn project_name_prefers_worktree_basename() {
        assert_eq!(
            extract_project_name("6c2b812d78103429", Some("E:/Developments/Nodejs/opencode-monitor")),
            "opencode-monitor"
        );
        assert_eq!(
            extract_project_name("6c2b812d78103429", Some("C:/Users/Chandupa")),
            "Chandupa"
        );
    }
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
    /// Detected schema: "v2" (session_v2), "v1" (session), "empty" or "unknown".
    #[serde(default)]
    pub schema: String,
}

#[derive(Debug, Serialize)]
pub struct DbInfo {
    pub path: Option<String>,
    pub exists: bool,
    /// "v2" | "v1" | "empty" | "unknown"
    pub schema: String,
    /// Active session table ("session_v2" or "session").
    pub session_table: Option<String>,
    pub has_session_v2: bool,
    pub has_legacy_session: bool,
    pub has_session_message: bool,
    pub has_message: bool,
    pub has_part: bool,
    pub session_count: i64,
    pub message_count: i64,
}
