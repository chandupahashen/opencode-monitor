use rusqlite::{params, Connection};
use std::path::PathBuf;

use crate::models::*;

fn dirs_next() -> Option<PathBuf> {
    if let Some(home) = std::env::var_os("HOME") {
        let mut p = PathBuf::from(home);
        p.push(".local");
        p.push("share");
        p.push("opencode");
        if p.join("opencode.db").exists() {
            return Some(p.join("opencode.db"));
        }
    }
    if let Some(appdata) = std::env::var_os("APPDATA") {
        let mut p = PathBuf::from(appdata);
        p.push("opencode");
        if p.join("opencode.db").exists() {
            return Some(p.join("opencode.db"));
        }
    }
    if let Some(local) = std::env::var_os("LOCALAPPDATA") {
        let mut p = PathBuf::from(&local);
        p.pop();
        p.push("Local");
        p.push("share");
        p.push("opencode");
        if p.join("opencode.db").exists() {
            return Some(p.join("opencode.db"));
        }
    }
    None
}

fn open_db(custom_path: Option<&str>) -> Result<Connection, String> {
    let path = match custom_path {
        Some(p) => PathBuf::from(p),
        None => dirs_next().ok_or_else(|| "opencode.db not found. Please set the path manually in Settings.".to_string())?,
    };
    Connection::open(&path).map_err(|e| format!("Failed to open database: {e}"))
}

pub fn get_overview(custom_path: Option<&str>) -> Result<Overview, String> {
    let conn = open_db(custom_path)?;
    conn.query_row(
        "SELECT
            COUNT(*) as total_sessions,
            COALESCE(SUM(tokens_input), 0) as total_tokens_input,
            COALESCE(SUM(tokens_output), 0) as total_tokens_output,
            COALESCE(SUM(tokens_reasoning), 0) as total_tokens_reasoning,
            COALESCE(SUM(tokens_cache_read), 0) as total_tokens_cache_read,
            COALESCE(SUM(tokens_cache_write), 0) as total_tokens_cache_write,
            COALESCE(SUM(cost), 0.0) as total_cost,
            COUNT(DISTINCT project_id) as total_projects,
            COUNT(CASE WHEN time_archived IS NULL THEN 1 END) as active_sessions
         FROM session",
        [],
        |row| {
            Ok(Overview {
                total_sessions: row.get(0)?,
                total_tokens_input: row.get(1)?,
                total_tokens_output: row.get(2)?,
                total_tokens_reasoning: row.get(3)?,
                total_tokens_cache_read: row.get(4)?,
                total_tokens_cache_write: row.get(5)?,
                total_cost: row.get(6)?,
                total_projects: row.get(7)?,
                active_sessions: row.get(8)?,
            })
        },
    )
    .map_err(|e| e.to_string())
}

pub fn get_sessions(
    custom_path: Option<&str>,
    limit: i64,
    offset: i64,
    model_filter: Option<String>,
    project_filter: Option<String>,
    date_from: Option<i64>,
    date_to: Option<i64>,
) -> Result<Vec<SessionRow>, String> {
    let conn = open_db(custom_path)?;
    let mut sql = String::from(
        "SELECT id, title, model, agent,
                tokens_input, tokens_output, tokens_reasoning,
                tokens_cache_read, tokens_cache_write,
                cost, summary_additions, summary_deletions, summary_files,
                time_created, time_updated, project_id
         FROM session WHERE 1=1",
    );
    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(ref m) = model_filter {
        sql.push_str(&format!(" AND model = ?{}", param_values.len() + 1));
        param_values.push(Box::new(m.clone()));
    }
    if let Some(ref p) = project_filter {
        sql.push_str(&format!(" AND project_id = ?{}", param_values.len() + 1));
        param_values.push(Box::new(p.clone()));
    }
    if let Some(f) = date_from {
        sql.push_str(&format!(" AND time_created >= ?{}", param_values.len() + 1));
        param_values.push(Box::new(f));
    }
    if let Some(t) = date_to {
        sql.push_str(&format!(" AND time_created <= ?{}", param_values.len() + 1));
        param_values.push(Box::new(t));
    }

    sql.push_str(" ORDER BY time_created DESC");
    sql.push_str(&format!(" LIMIT ?{}", param_values.len() + 1));
    param_values.push(Box::new(limit));
    sql.push_str(&format!(" OFFSET ?{}", param_values.len() + 1));
    param_values.push(Box::new(offset));

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let params_ref: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();

    let rows = stmt
        .query_map(params_ref.as_slice(), |row| {
            Ok(SessionRow {
                id: row.get(0)?,
                title: row.get(1)?,
                model: row.get(2)?,
                agent: row.get(3)?,
                tokens_input: row.get(4)?,
                tokens_output: row.get(5)?,
                tokens_reasoning: row.get(6)?,
                tokens_cache_read: row.get(7)?,
                tokens_cache_write: row.get(8)?,
                cost: row.get(9)?,
                summary_additions: row.get(10)?,
                summary_deletions: row.get(11)?,
                summary_files: row.get(12)?,
                time_created: row.get(13)?,
                time_updated: row.get(14)?,
                project_id: row.get(15)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(|e| e.to_string())?);
    }
    Ok(result)
}

pub fn get_session_detail(custom_path: Option<&str>, session_id: &str) -> Result<SessionDetail, String> {
    let conn = open_db(custom_path)?;
    let session = conn
        .query_row(
            "SELECT id, title, model, agent, cost,
                    tokens_input, tokens_output, tokens_reasoning,
                    tokens_cache_read, tokens_cache_write,
                    summary_additions, summary_deletions, summary_files,
                    time_created, time_updated, time_archived,
                    directory, project_id
             FROM session WHERE id = ?1",
            params![session_id],
            |row| {
                Ok(SessionDetail {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    model: row.get(2)?,
                    agent: row.get(3)?,
                    cost: row.get(4)?,
                    tokens_input: row.get(5)?,
                    tokens_output: row.get(6)?,
                    tokens_reasoning: row.get(7)?,
                    tokens_cache_read: row.get(8)?,
                    tokens_cache_write: row.get(9)?,
                    summary_additions: row.get(10)?,
                    summary_deletions: row.get(11)?,
                    summary_files: row.get(12)?,
                    time_created: row.get(13)?,
                    time_updated: row.get(14)?,
                    time_archived: row.get(15)?,
                    directory: row.get(16)?,
                    project_id: row.get(17)?,
                    messages: Vec::new(),
                })
            },
        )
        .map_err(|e| e.to_string())?;

    Ok(session)
}

pub fn get_token_trends(
    custom_path: Option<&str>,
    days: i64,
) -> Result<Vec<TokenTrend>, String> {
    let conn = open_db(custom_path)?;
    let cutoff = chrono::Utc::now().timestamp_millis() - days * 86_400_000;
    let mut stmt = conn
        .prepare(
            "SELECT DATE(time_created / 1000, 'unixepoch') as day,
                    COALESCE(SUM(tokens_input), 0),
                    COALESCE(SUM(tokens_output), 0),
                    COALESCE(SUM(tokens_reasoning), 0),
                    COALESCE(SUM(tokens_cache_read), 0),
                    COALESCE(SUM(tokens_cache_write), 0),
                    COALESCE(SUM(cost), 0.0),
                    COUNT(*)
             FROM session
             WHERE time_created > ?1
             GROUP BY day
             ORDER BY day",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![cutoff], |row| {
            Ok(TokenTrend {
                date: row.get(0)?,
                tokens_input: row.get(1)?,
                tokens_output: row.get(2)?,
                tokens_reasoning: row.get(3)?,
                tokens_cache_read: row.get(4)?,
                tokens_cache_write: row.get(5)?,
                cost: row.get(6)?,
                session_count: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(|e| e.to_string())?);
    }
    Ok(result)
}

pub fn get_model_usage(custom_path: Option<&str>) -> Result<Vec<ModelStat>, String> {
    let conn = open_db(custom_path)?;
    let mut stmt = conn
        .prepare(
            "SELECT model, COUNT(*), SUM(tokens_input + tokens_output), COALESCE(SUM(cost), 0.0)
             FROM session
             GROUP BY model
             ORDER BY SUM(tokens_input + tokens_output) DESC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(ModelStat {
                model: row.get(0)?,
                session_count: row.get(1)?,
                total_tokens: row.get(2)?,
                total_cost: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(|e| e.to_string())?);
    }
    Ok(result)
}

pub fn get_project_stats(custom_path: Option<&str>) -> Result<Vec<ProjectStat>, String> {
    let conn = open_db(custom_path)?;
    let mut stmt = conn
        .prepare(
            "SELECT project_id, COUNT(*), SUM(tokens_input + tokens_output), COALESCE(SUM(cost), 0.0)
             FROM session
             WHERE project_id IS NOT NULL
             GROUP BY project_id
             ORDER BY SUM(tokens_input + tokens_output) DESC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(ProjectStat {
                project_id: row.get(0)?,
                session_count: row.get(1)?,
                total_tokens: row.get(2)?,
                total_cost: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(|e| e.to_string())?);
    }
    Ok(result)
}

pub fn get_daily_activity(
    custom_path: Option<&str>,
    months: i64,
) -> Result<Vec<DayActivity>, String> {
    let conn = open_db(custom_path)?;
    let cutoff = chrono::Utc::now().timestamp_millis() - months * 30 * 86_400_000;
    let mut stmt = conn
        .prepare(
            "SELECT DATE(time_created / 1000, 'unixepoch') as day,
                    COUNT(*),
                    COALESCE(SUM(tokens_input + tokens_output), 0)
             FROM session
             WHERE time_created > ?1
             GROUP BY day
             ORDER BY day",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![cutoff], |row| {
            Ok(DayActivity {
                date: row.get(0)?,
                session_count: row.get(1)?,
                total_tokens: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(|e| e.to_string())?);
    }
    Ok(result)
}

pub fn get_cost_breakdown(custom_path: Option<&str>) -> Result<CostBreakdown, String> {
    let by_model = get_model_usage(custom_path)?;
    let by_project = get_project_stats(custom_path)?;
    Ok(CostBreakdown { by_model, by_project })
}

pub fn get_models_list(custom_path: Option<&str>) -> Result<Vec<String>, String> {
    let conn = open_db(custom_path)?;
    let mut stmt = conn
        .prepare("SELECT DISTINCT model FROM session WHERE model IS NOT NULL ORDER BY model")
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(|e| e.to_string())?);
    }
    Ok(result)
}

pub fn get_projects_list(custom_path: Option<&str>) -> Result<Vec<String>, String> {
    let conn = open_db(custom_path)?;
    let mut stmt = conn
        .prepare("SELECT DISTINCT project_id FROM session WHERE project_id IS NOT NULL ORDER BY project_id")
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(|e| e.to_string())?);
    }
    Ok(result)
}
