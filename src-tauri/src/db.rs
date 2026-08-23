use rusqlite::{params, Connection};
use std::path::PathBuf;

use crate::models::*;

/// SQL fragment that extracts a readable model name from the `model` column.
/// New DB stores JSON like {"id":"deepseek-v4-flash-free","providerID":"opencode","variant":"max"}.
/// Old DB stored plain string. Uses json_valid / json_extract when available.
fn model_key_expr(col: &str) -> String {
    format!(
        "COALESCE(CASE WHEN json_valid({col}) THEN COALESCE(json_extract({col}, '$.id'), json_extract({col}, '$.modelID'), json_extract({col}, '$.modelId')) ELSE {col} END, 'unknown')"
    )
}

/// Check if a table exists in the current connection.
fn table_exists(conn: &Connection, name: &str) -> bool {
    conn.prepare("SELECT 1 FROM sqlite_master WHERE type='table' AND name=?1")
        .and_then(|mut stmt| {
            stmt.query_row(params![name], |_| Ok(()))
                .map(|_| true)
                .or_else(|e| match e {
                    rusqlite::Error::QueryReturnedNoRows => Ok(false),
                    _ => Err(e),
                })
        })
        .unwrap_or(false)
}

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
        p.push("opencode");
        if p.join("opencode.db").exists() {
            return Some(p.join("opencode.db"));
        }
    }
    None
}

pub fn verify_connection(path: &str) -> bool {
    Connection::open(path).is_ok()
}

const SCHEMA_VERSION: i64 = 2;

fn open_db(custom_path: Option<&str>) -> Result<Connection, String> {
    let path = match custom_path {
        Some(p) => PathBuf::from(p),
        None => dirs_next().ok_or_else(|| "opencode.db not found. Please set the path manually in Settings.".to_string())?,
    };
    let conn = Connection::open(&path).map_err(|e| format!("Failed to open database: {e}"))?;
    // Mitigate DB locked errors when opencode is writing concurrently.
    let _ = conn.busy_timeout(std::time::Duration::from_millis(3000));
    // Ensure we don't fail the whole open if migrations attempt to mutate a read-only/locked DB.
    // Run migrations best-effort, but don't block reads.
    if let Err(e) = run_migrations(&conn) {
        tracing::warn!("DB migration warning (non-fatal): {e}");
    }
    Ok(conn)
}

fn run_migrations(conn: &Connection) -> Result<(), String> {
    let current: i64 = conn
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(|e| e.to_string())?;

    if current >= SCHEMA_VERSION {
        return Ok(());
    }

    tracing::info!("Running DB migration: {} -> {}", current, SCHEMA_VERSION);

    if current < 1 {
        conn.execute_batch(
            "CREATE INDEX IF NOT EXISTS idx_session_time_created ON session(time_created);
             CREATE INDEX IF NOT EXISTS idx_session_model ON session(model);
             CREATE INDEX IF NOT EXISTS idx_session_project_id ON session(project_id);",
        )
        .map_err(|e| format!("Migration 1 failed: {e}"))?;
    }

    if current < 2 {
        // New DB already has these indexes, but ensure for older files.
        // Also add indexes that help message/part queries if tables exist.
        let _ = conn.execute_batch(
            "CREATE INDEX IF NOT EXISTS idx_message_session_id ON message(session_id);
             CREATE INDEX IF NOT EXISTS idx_part_message_id ON part(message_id);
             CREATE INDEX IF NOT EXISTS idx_part_session_id ON part(session_id);",
        );
    }

    conn.pragma_update(None, "user_version", SCHEMA_VERSION)
        .map_err(|e| e.to_string())?;

    tracing::info!("DB migration to v{SCHEMA_VERSION} complete");
    Ok(())
}

fn build_date_filter(date_from: Option<i64>, date_to: Option<i64>, param_values: &mut Vec<Box<dyn rusqlite::types::ToSql>>) -> String {
    let mut sql = String::new();
    if let Some(f) = date_from {
        sql.push_str(&format!(" AND time_created >= ?{}", param_values.len() + 1));
        param_values.push(Box::new(f));
    }
    if let Some(t) = date_to {
        sql.push_str(&format!(" AND time_created <= ?{}", param_values.len() + 1));
        param_values.push(Box::new(t));
    }
    sql
}

pub fn get_overview(
    custom_path: Option<&str>,
    date_from: Option<i64>,
    date_to: Option<i64>,
) -> Result<Overview, String> {
    let conn = open_db(custom_path)?;
    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
    let date_filter = build_date_filter(date_from, date_to, &mut param_values);

    let sql = format!(
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
         FROM session WHERE 1=1{date_filter}",
        date_filter = date_filter
    );

    let params_ref: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
    conn.query_row(&sql, params_ref.as_slice(), |row| {
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
    })
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
        // Normalize filter: if caller passed a JSON string, extract its id so short-id filters match JSON storage.
        let normalized = normalize_model_filter(m);
        // Match either raw JSON equality, or extracted id. This supports both old and new callers.
        let expr = model_key_expr("model");
        // We need two placeholders: one for raw equality, one for extracted.
        sql.push_str(&format!(
            " AND (model = ?{} OR {} = ?{})",
            param_values.len() + 1,
            expr,
            param_values.len() + 2
        ));
        param_values.push(Box::new(m.clone()));
        param_values.push(Box::new(normalized));
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
    let mut session = conn
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

    // Fetch messages from the actual message storage.
    // New DB uses `message` + `part` tables (with `session_message` legacy empty).
    // Old DB may have used `message` with different shape, or no table. Handle gracefully.
    let messages = if table_exists(&conn, "message") {
        // Prefer message table
        let has_part = table_exists(&conn, "part");
        let sql = if has_part {
            "SELECT m.id, m.time_created, (SELECT COUNT(*) FROM part p WHERE p.message_id = m.id) as parts_count \
             FROM message m WHERE m.session_id = ?1 ORDER BY m.time_created"
        } else {
            "SELECT id, time_created, 0 as parts_count FROM message WHERE session_id = ?1 ORDER BY time_created"
        };
        match conn.prepare(sql) {
            Ok(mut stmt) => {
                let rows = stmt.query_map(params![session_id], |row| {
                    Ok(MessageInfo {
                        id: row.get(0)?,
                        time_created: row.get(1)?,
                        parts_count: row.get(2)?,
                    })
                });
                match rows {
                    Ok(mapped) => {
                        let mut v = Vec::new();
                        for r in mapped {
                            if let Ok(mi) = r { v.push(mi); }
                        }
                        v
                    }
                    Err(_) => Vec::new(),
                }
            }
            Err(_) => Vec::new(),
        }
    } else if table_exists(&conn, "session_message") {
        match conn.prepare("SELECT id, time_created, 0 as parts_count FROM session_message WHERE session_id = ?1 ORDER BY time_created") {
            Ok(mut stmt) => {
                let rows = stmt.query_map(params![session_id], |row| {
                    Ok(MessageInfo {
                        id: row.get(0)?,
                        time_created: row.get(1)?,
                        parts_count: row.get(2)?,
                    })
                });
                match rows {
                    Ok(mapped) => {
                        let mut v = Vec::new();
                        for r in mapped { if let Ok(mi) = r { v.push(mi); } }
                        v
                    }
                    Err(_) => Vec::new(),
                }
            }
            Err(_) => Vec::new(),
        }
    } else {
        Vec::new()
    };

    session.messages = messages;
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

pub fn get_model_usage(
    custom_path: Option<&str>,
    date_from: Option<i64>,
    date_to: Option<i64>,
) -> Result<Vec<ModelStat>, String> {
    let conn = open_db(custom_path)?;
    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
    let date_filter = build_date_filter(date_from, date_to, &mut param_values);

    // Group by normalized model id so variants collapse (e.g. deepseek-v4-flash-free max/high/default -> one row).
    // Fallback to raw model string if json functions not available.
    let key_expr = model_key_expr("model");
    let sql = format!(
        "SELECT {key_expr} as model_key, COUNT(*), COALESCE(SUM(tokens_input + tokens_output),0), COALESCE(SUM(cost), 0.0)
         FROM session WHERE 1=1{date_filter}
         GROUP BY model_key
         HAVING model_key IS NOT NULL AND model_key != 'unknown' AND model_key != ''
         ORDER BY COALESCE(SUM(tokens_input + tokens_output),0) DESC",
    );

    let params_ref: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
    // Try primary query; if json functions missing, fallback to raw grouping.
    let mut stmt = match conn.prepare(&sql) {
        Ok(s) => s,
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("no such function") {
                let fallback = format!(
                    "SELECT model, COUNT(*), COALESCE(SUM(tokens_input + tokens_output),0), COALESCE(SUM(cost), 0.0)
                     FROM session WHERE 1=1{date_filter}
                     GROUP BY model
                     ORDER BY COALESCE(SUM(tokens_input + tokens_output),0) DESC"
                );
                conn.prepare(&fallback).map_err(|e2| e2.to_string())?
            } else {
                return Err(msg);
            }
        }
    };

    let rows = stmt
        .query_map(params_ref.as_slice(), |row| {
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

pub fn get_project_stats(
    custom_path: Option<&str>,
    date_from: Option<i64>,
    date_to: Option<i64>,
) -> Result<Vec<ProjectStat>, String> {
    let conn = open_db(custom_path)?;
    let has_project = table_exists(&conn, "project");
    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
    let date_filter = build_date_filter(date_from, date_to, &mut param_values);

    // Build SQL depending on whether project table exists.
    let sql = if has_project {
        format!(
            "SELECT s.project_id, COUNT(*), COALESCE(SUM(s.tokens_input + s.tokens_output),0), COALESCE(SUM(s.cost), 0.0),
                    COALESCE(p.worktree, (SELECT s2.directory FROM session s2 WHERE s2.project_id = s.project_id AND s2.directory IS NOT NULL LIMIT 1)) as worktree,
                    p.name
             FROM session s
             LEFT JOIN project p ON p.id = s.project_id
             WHERE s.project_id IS NOT NULL{date_filter}
             GROUP BY s.project_id
             ORDER BY COALESCE(SUM(s.tokens_input + s.tokens_output),0) DESC"
        )
    } else {
        format!(
            "SELECT s.project_id, COUNT(*), COALESCE(SUM(s.tokens_input + s.tokens_output),0), COALESCE(SUM(s.cost), 0.0),
                    (SELECT s2.directory FROM session s2 WHERE s2.project_id = s.project_id AND s2.directory IS NOT NULL LIMIT 1) as worktree
             FROM session s
             WHERE s.project_id IS NOT NULL{date_filter}
             GROUP BY s.project_id
             ORDER BY COALESCE(SUM(s.tokens_input + s.tokens_output),0) DESC"
        )
    };

    let params_ref: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params_ref.as_slice(), |row| {
            let project_id: String = row.get(0)?;
            let worktree: Option<String> = row.get(4)?;
            let project_table_name: Option<String> = if has_project {
                row.get::<_, Option<String>>(5)?
            } else {
                None
            };
            // Prefer explicit project.name from DB, otherwise derive from worktree/directory.
            let worktree_for_name = worktree.clone().or_else(|| project_table_name.clone());
            let project_name = if let Some(ref pn) = project_table_name {
                if !pn.trim().is_empty() {
                    pn.clone()
                } else {
                    extract_project_name(&project_id, worktree_for_name.as_deref())
                }
            } else {
                extract_project_name(&project_id, worktree_for_name.as_deref())
            };
            Ok(ProjectStat {
                project_id,
                project_name,
                directory: worktree,
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

pub fn get_cost_breakdown(
    custom_path: Option<&str>,
    date_from: Option<i64>,
    date_to: Option<i64>,
) -> Result<CostBreakdown, String> {
    let by_model = get_model_usage(custom_path, date_from, date_to)?;
    let by_project = get_project_stats(custom_path, date_from, date_to)?;
    Ok(CostBreakdown { by_model, by_project })
}

pub fn get_models_list(custom_path: Option<&str>) -> Result<Vec<String>, String> {
    let conn = open_db(custom_path)?;
    let key_expr = model_key_expr("model");
    let sql = format!(
        "SELECT DISTINCT {key_expr} as mk FROM session WHERE model IS NOT NULL AND mk != 'unknown' AND mk != '' ORDER BY mk"
    );
    let mut stmt = match conn.prepare(&sql) {
        Ok(s) => s,
        Err(e) => {
            if e.to_string().contains("no such function") {
                conn.prepare("SELECT DISTINCT model FROM session WHERE model IS NOT NULL ORDER BY model")
                    .map_err(|e2| e2.to_string())?
            } else {
                return Err(e.to_string());
            }
        }
    };

    let rows = stmt
        .query_map([], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    let mut result = Vec::new();
    for row in rows {
        let raw: String = row.map_err(|e| e.to_string())?;
        // Normalize once more via Rust to ensure consistent short id (handles provider/modelID edge).
        result.push(extract_model_id(&raw));
    }
    // Deduplicate after Rust normalization (e.g. two JSON variants that map to same id after Rust parse but SQLite missed)
    result.sort();
    result.dedup();
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
