use rusqlite::{params, Connection};
use std::path::PathBuf;

use crate::models::*;

/// SQL fragment that extracts a readable model name from the `model` column.
/// v2 stores JSON like {"id":"muse-spark-1.3-contributor-free","providerID":"opencode","variant":"xhigh"}.
/// v1 stored plain string or `provider/model`. Uses json_valid / json_extract when available.
/// Note: `#variant` suffix stripping (provider/model#variant) happens in Rust
/// post-processing via `extract_model_id`, since SQLite string ops are brittle.
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

fn column_exists(conn: &Connection, table: &str, column: &str) -> bool {
    // Quote table name to be safe; column comparison is case-insensitive in SQLite.
    let sql = format!("SELECT {column} FROM \"{table}\" LIMIT 0");
    conn.prepare(&sql).is_ok()
}

/// Resolve the DB path: explicit setting > OPENCODE_DB env > auto-detect.
pub fn resolve_db_path(custom_path: Option<&str>) -> Option<PathBuf> {
    if let Some(p) = custom_path {
        let t = p.trim();
        if !t.is_empty() {
            return Some(PathBuf::from(t));
        }
    }
    if let Ok(env_path) = std::env::var("OPENCODE_DB") {
        let t = env_path.trim().to_string();
        if !t.is_empty() {
            return Some(PathBuf::from(t));
        }
    }
    dirs_next()
}

fn push_candidate(out: &mut Vec<PathBuf>, mut base: PathBuf) {
    base.push("opencode");
    base.push("opencode.db");
    out.push(base);
}

fn dirs_next() -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    // XDG_DATA_HOME override (Linux/macOS + Windows when set).
    if let Some(xdg) = std::env::var_os("XDG_DATA_HOME") {
        let mut p = PathBuf::from(xdg);
        p.push("opencode");
        p.push("opencode.db");
        candidates.push(p);
    }
    // Standard Unix location: $HOME/.local/share/opencode/opencode.db
    if let Some(home) = std::env::var_os("HOME") {
        let mut p = PathBuf::from(home);
        p.push(".local");
        p.push("share");
        push_candidate(&mut candidates, p);
    }
    // Windows: %USERPROFILE%\.local\share\opencode\opencode.db
    // (opencode v2 on Windows uses this, NOT %APPDATA%).
    if let Some(profile) = std::env::var_os("USERPROFILE") {
        let mut p = PathBuf::from(profile);
        p.push(".local");
        p.push("share");
        push_candidate(&mut candidates, p);
    }
    // dirs crate data_dir: %APPDATA% on Windows, ~/.local/share on Linux.
    if let Some(mut data_dir) = dirs::data_dir() {
        data_dir.push("opencode");
        data_dir.push("opencode.db");
        candidates.push(data_dir);
    }
    if let Some(appdata) = std::env::var_os("APPDATA") {
        let mut p = PathBuf::from(appdata);
        p.push("opencode");
        p.push("opencode.db");
        candidates.push(p);
    }
    if let Some(local) = std::env::var_os("LOCALAPPDATA") {
        let mut p = PathBuf::from(&local);
        p.push("opencode");
        p.push("opencode.db");
        candidates.push(p);
    }

    // Return first candidate that exists.
    for c in &candidates {
        if c.exists() {
            return Some(c.clone());
        }
    }
    // No existing file: return the most likely default so callers can show it
    // and produce a clear "not found" error on open.
    candidates.into_iter().next()
}

pub fn verify_connection(path: &str) -> bool {
    // Don't create empty DB files as a side effect of a health check.
    if !std::path::Path::new(path).exists() {
        return false;
    }
    let conn = match Connection::open(path) {
        Ok(c) => c,
        Err(_) => return false,
    };
    // Consider connected only if we can open AND see a session table.
    table_exists(&conn, "session_v2") || table_exists(&conn, "session")
}

/// Which session table should queries use? v2 (`session_v2`) preferred, v1 (`session`) fallback.
fn session_table(conn: &Connection) -> Result<&'static str, String> {
    if table_exists(conn, "session_v2") {
        Ok("session_v2")
    } else if table_exists(conn, "session") {
        Ok("session")
    } else {
        Err("No session table found (expected `session_v2` for opencode v2 or `session` for v1). Is this the right opencode.db?".to_string())
    }
}

pub fn detect_schema(conn: &Connection) -> &'static str {
    if table_exists(conn, "session_v2") {
        "v2"
    } else if table_exists(conn, "session") {
        "v1"
    } else {
        "empty"
    }
}

const SCHEMA_VERSION: i64 = 3;

fn open_db(custom_path: Option<&str>) -> Result<Connection, String> {
    let path = resolve_db_path(custom_path).ok_or_else(|| "opencode.db not found. Please set the path manually in Settings (or set OPENCODE_DB).".to_string())?;
    if !path.exists() {
        return Err(format!(
            "opencode.db not found at {}. Please set the path manually in Settings (or set OPENCODE_DB).",
            path.display()
        ));
    }
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
        // v1 tables (harmless if missing — execute_batch errors are mapped).
        let _ = conn.execute_batch(
            "CREATE INDEX IF NOT EXISTS idx_session_time_created ON session(time_created);
             CREATE INDEX IF NOT EXISTS idx_session_model ON session(model);
             CREATE INDEX IF NOT EXISTS idx_session_project_id ON session(project_id);",
        );
    }

    if current < 2 {
        // Legacy message/part helpers.
        let _ = conn.execute_batch(
            "CREATE INDEX IF NOT EXISTS idx_message_session_id ON message(session_id);
             CREATE INDEX IF NOT EXISTS idx_part_message_id ON part(message_id);
             CREATE INDEX IF NOT EXISTS idx_part_session_id ON part(session_id);",
        );
    }

    if current < 3 {
        // v2 tables: session_v2 + session_message. All best-effort (opencode owns the schema).
        let _ = conn.execute_batch(
            "CREATE INDEX IF NOT EXISTS idx_session_v2_time_created ON session_v2(time_created);
             CREATE INDEX IF NOT EXISTS idx_session_v2_model ON session_v2(model);
             CREATE INDEX IF NOT EXISTS idx_session_v2_project_id ON session_v2(project_id);
             CREATE INDEX IF NOT EXISTS idx_session_message_session_id ON session_message(session_id);
             CREATE INDEX IF NOT EXISTS idx_session_message_type ON session_message(type);
             CREATE INDEX IF NOT EXISTS idx_session_message_seq ON session_message(session_id, seq);",
        );
    }

    // Only bump user_version if we own it (don't clobber opencode's own version numbering
    // blindly — but our indexes are additive so recording is safe).
    // If opencode uses user_version itself, max() preserves the higher value.
    let opencode_version: i64 = conn
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .unwrap_or(current);
    let target = opencode_version.max(SCHEMA_VERSION);
    let _ = conn.pragma_update(None, "user_version", target);

    tracing::info!("DB migration to v{SCHEMA_VERSION} complete");
    Ok(())
}

fn build_date_filter(date_from: Option<i64>, date_to: Option<i64>, param_values: &mut Vec<Box<dyn rusqlite::types::ToSql>>) -> String {
    build_date_filter_for("time_created", date_from, date_to, param_values)
}

fn build_date_filter_for(col: &str, date_from: Option<i64>, date_to: Option<i64>, param_values: &mut Vec<Box<dyn rusqlite::types::ToSql>>) -> String {
    let mut sql = String::new();
    if let Some(f) = date_from {
        sql.push_str(&format!(" AND {col} >= ?{}", param_values.len() + 1));
        param_values.push(Box::new(f));
    }
    if let Some(t) = date_to {
        sql.push_str(&format!(" AND {col} <= ?{}", param_values.len() + 1));
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
    let tbl = session_table(&conn)?;
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
         FROM \"{tbl}\" WHERE 1=1{date_filter}",
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
    let tbl = session_table(&conn)?;
    let is_v2 = tbl == "session_v2";
    // v2 adds slug + parent_id; v1 lacks them — select accordingly.
    let extra_cols = if is_v2 { ", slug, parent_id" } else { "" };
    let mut sql = format!(
        "SELECT id, title, model, agent,
                COALESCE(tokens_input, 0), COALESCE(tokens_output, 0), COALESCE(tokens_reasoning, 0),
                COALESCE(tokens_cache_read, 0), COALESCE(tokens_cache_write, 0),
                COALESCE(cost, 0), COALESCE(summary_additions, 0), COALESCE(summary_deletions, 0), COALESCE(summary_files, 0),
                time_created, time_updated, project_id{extra_cols}
         FROM \"{tbl}\" WHERE 1=1",
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
            let slug: Option<String> = if is_v2 { row.get(16)? } else { None };
            let parent_id: Option<String> = if is_v2 { row.get(17)? } else { None };
            Ok(SessionRow {
                id: row.get(0)?,
                title: row.get(1)?,
                model: row.get(2)?,
                agent: row.get(3)?,
                tokens_input: row.get::<_, Option<i64>>(4)?.unwrap_or(0),
                tokens_output: row.get::<_, Option<i64>>(5)?.unwrap_or(0),
                tokens_reasoning: row.get::<_, Option<i64>>(6)?.unwrap_or(0),
                tokens_cache_read: row.get::<_, Option<i64>>(7)?.unwrap_or(0),
                tokens_cache_write: row.get::<_, Option<i64>>(8)?.unwrap_or(0),
                cost: row.get::<_, Option<f64>>(9)?.unwrap_or(0.0),
                summary_additions: row.get::<_, Option<i64>>(10)?.unwrap_or(0),
                summary_deletions: row.get::<_, Option<i64>>(11)?.unwrap_or(0),
                summary_files: row.get::<_, Option<i64>>(12)?.unwrap_or(0),
                time_created: row.get::<_, Option<i64>>(13)?.unwrap_or(0),
                time_updated: row.get::<_, Option<i64>>(14)?.unwrap_or(0),
                project_id: row.get(15)?,
                slug,
                parent_id,
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
    let tbl = session_table(&conn)?;
    let is_v2 = tbl == "session_v2";
    let extra = if is_v2 { ", slug, parent_id" } else { "" };
    let sql = format!(
        "SELECT id, title, model, agent, COALESCE(cost, 0),
                    COALESCE(tokens_input, 0), COALESCE(tokens_output, 0), COALESCE(tokens_reasoning, 0),
                    COALESCE(tokens_cache_read, 0), COALESCE(tokens_cache_write, 0),
                    COALESCE(summary_additions, 0), COALESCE(summary_deletions, 0), COALESCE(summary_files, 0),
                    time_created, time_updated, time_archived,
                    directory, project_id{extra}
             FROM \"{tbl}\" WHERE id = ?1"
    );
    let mut session = conn
        .query_row(
            &sql,
            params![session_id],
            |row| {
                let slug: Option<String> = if is_v2 { row.get(18)? } else { None };
                let parent_id: Option<String> = if is_v2 { row.get(19)? } else { None };
                Ok(SessionDetail {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    model: row.get(2)?,
                    agent: row.get(3)?,
                    cost: row.get::<_, Option<f64>>(4)?.unwrap_or(0.0),
                    tokens_input: row.get::<_, Option<i64>>(5)?.unwrap_or(0),
                    tokens_output: row.get::<_, Option<i64>>(6)?.unwrap_or(0),
                    tokens_reasoning: row.get::<_, Option<i64>>(7)?.unwrap_or(0),
                    tokens_cache_read: row.get::<_, Option<i64>>(8)?.unwrap_or(0),
                    tokens_cache_write: row.get::<_, Option<i64>>(9)?.unwrap_or(0),
                    summary_additions: row.get::<_, Option<i64>>(10)?.unwrap_or(0),
                    summary_deletions: row.get::<_, Option<i64>>(11)?.unwrap_or(0),
                    summary_files: row.get::<_, Option<i64>>(12)?.unwrap_or(0),
                    time_created: row.get::<_, Option<i64>>(13)?.unwrap_or(0),
                    time_updated: row.get::<_, Option<i64>>(14)?.unwrap_or(0),
                    time_archived: row.get(15)?,
                    directory: row.get(16)?,
                    project_id: row.get(17)?,
                    slug,
                    parent_id,
                    messages: Vec::new(),
                })
            },
        )
        .map_err(|e| e.to_string())?;

    // Fetch messages. v2 uses `session_message` (type/seq/data JSON);
    // v1 used `message` + `part` (or legacy `session_message`).
    let messages = if table_exists(&conn, "session_message") && is_v2 {
        // v2 path: order by seq so transcript order matches opencode.
        // parts_count = content array length when data has $.content, else 1.
        let sql_v2 = "SELECT id, type, time_created, \
                      COALESCE(json_array_length(data, '$.content'), 1) as parts_count \
                      FROM session_message WHERE session_id = ?1 ORDER BY seq, time_created";
        let fallback_sql = "SELECT id, type, time_created, 1 as parts_count \
                      FROM session_message WHERE session_id = ?1 ORDER BY time_created";
        let try_query = |q: &str| -> Vec<MessageInfo> {
            match conn.prepare(q) {
                Ok(mut stmt) => {
                    match stmt.query_map(params![session_id], |row| {
                        Ok(MessageInfo {
                            id: row.get(0)?,
                            message_type: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                            time_created: row.get::<_, Option<i64>>(2)?.unwrap_or(0),
                            parts_count: row.get::<_, Option<i64>>(3)?.unwrap_or(1),
                        })
                    }) {
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
        };
        let mut v = try_query(sql_v2);
        if v.is_empty() {
            // json_array_length may be missing or data shape differs — retry without it.
            // Only retry if the first attempt truly failed to produce rows AND rows exist.
            let count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM session_message WHERE session_id = ?1",
                params![session_id],
                |r| r.get(0),
            ).unwrap_or(0);
            if count > 0 {
                v = try_query(fallback_sql);
            }
        }
        v
    } else if table_exists(&conn, "message") {
        // v1 path: message + optional part counts.
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
                        message_type: String::new(),
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
        // Old beta shape or v1 fallback: session_message without v2 seq/type — best effort.
        match conn.prepare("SELECT id, time_created, 0 as parts_count FROM session_message WHERE session_id = ?1 ORDER BY time_created") {
            Ok(mut stmt) => {
                let rows = stmt.query_map(params![session_id], |row| {
                    Ok(MessageInfo {
                        id: row.get(0)?,
                        time_created: row.get(1)?,
                        parts_count: row.get(2)?,
                        message_type: String::new(),
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
    let tbl = session_table(&conn)?;
    let cutoff = chrono::Utc::now().timestamp_millis() - days * 86_400_000;
    let sql = format!(
        "SELECT DATE(time_created / 1000, 'unixepoch') as day,
                    COALESCE(SUM(tokens_input), 0),
                    COALESCE(SUM(tokens_output), 0),
                    COALESCE(SUM(tokens_reasoning), 0),
                    COALESCE(SUM(tokens_cache_read), 0),
                    COALESCE(SUM(tokens_cache_write), 0),
                    COALESCE(SUM(cost), 0.0),
                    COUNT(*)
             FROM \"{tbl}\"
             WHERE time_created > ?1
             GROUP BY day
             ORDER BY day",
    );
    let mut stmt = conn
        .prepare(&sql)
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
    let tbl = session_table(&conn)?;
    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
    let date_filter = build_date_filter(date_from, date_to, &mut param_values);

    // Group by normalized model id so variants collapse (e.g. deepseek-v4-flash-free max/high/default -> one row).
    // Fallback to raw grouping if json functions not available.
    // Note: `#variant` suffix stripping happens in Rust post-processing (extract_model_id),
    // so SQL groups by JSON id and Rust collapses any remaining `a/b#c` string refs.
    let key_expr = model_key_expr("model");
    let sql = format!(
        "SELECT {key_expr} as model_key, COUNT(*), COALESCE(SUM(tokens_input + tokens_output),0), COALESCE(SUM(cost), 0.0)
         FROM \"{tbl}\" WHERE 1=1{date_filter}
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
                     FROM \"{tbl}\" WHERE 1=1{date_filter}
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
            let raw: String = row.get(0)?;
            Ok((raw, row.get::<_, i64>(1)?, row.get::<_, i64>(2)?, row.get::<_, f64>(3)?))
        })
        .map_err(|e| e.to_string())?;

    // Collapse `provider/model#variant` string refs that SQLite couldn't (JSON path only
    // handles JSON). Aggregate in Rust by normalized id.
    use std::collections::BTreeMap;
    let mut agg: BTreeMap<String, (i64, i64, f64)> = BTreeMap::new();
    for row in rows {
        let (raw, cnt, toks, cost) = row.map_err(|e| e.to_string())?;
        let norm = extract_model_id(&raw);
        if norm.is_empty() || norm == "unknown" {
            continue;
        }
        let e = agg.entry(norm).or_insert((0, 0, 0.0));
        e.0 += cnt;
        e.1 += toks;
        e.2 += cost;
    }
    let mut result: Vec<ModelStat> = agg
        .into_iter()
        .map(|(model, (session_count, total_tokens, total_cost))| ModelStat {
            model,
            session_count,
            total_tokens,
            total_cost,
        })
        .collect();
    result.sort_by(|a, b| b.total_tokens.cmp(&a.total_tokens));
    Ok(result)
}

pub fn get_project_stats(
    custom_path: Option<&str>,
    date_from: Option<i64>,
    date_to: Option<i64>,
) -> Result<Vec<ProjectStat>, String> {
    let conn = open_db(custom_path)?;
    let tbl = session_table(&conn)?;
    let has_project = table_exists(&conn, "project");
    let has_name = has_project && column_exists(&conn, "project", "name");
    let has_worktree = has_project && column_exists(&conn, "project", "worktree");
    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
    // Prefix with s. — v2 `project` also has time_created, so bare would be ambiguous.
    let date_filter = build_date_filter_for("s.time_created", date_from, date_to, &mut param_values);

    // Build SQL depending on whether project table exists.
    let sql = if has_project && has_worktree {
        let name_sel = if has_name { "p.name" } else { "NULL" };
        format!(
            "SELECT s.project_id, COUNT(*), COALESCE(SUM(s.tokens_input + s.tokens_output),0), COALESCE(SUM(s.cost), 0.0),
                    COALESCE(p.worktree, (SELECT s2.directory FROM \"{tbl}\" s2 WHERE s2.project_id = s.project_id AND s2.directory IS NOT NULL LIMIT 1)) as worktree,
                    {name_sel}
             FROM \"{tbl}\" s
             LEFT JOIN project p ON p.id = s.project_id
             WHERE s.project_id IS NOT NULL{date_filter}
             GROUP BY s.project_id
             ORDER BY COALESCE(SUM(s.tokens_input + s.tokens_output),0) DESC"
        )
    } else {
        format!(
            "SELECT s.project_id, COUNT(*), COALESCE(SUM(s.tokens_input + s.tokens_output),0), COALESCE(SUM(s.cost), 0.0),
                    (SELECT s2.directory FROM \"{tbl}\" s2 WHERE s2.project_id = s.project_id AND s2.directory IS NOT NULL LIMIT 1) as worktree
             FROM \"{tbl}\" s
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
            let project_table_name: Option<String> = if has_project && has_worktree {
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
    let tbl = session_table(&conn)?;
    let cutoff = chrono::Utc::now().timestamp_millis() - months * 30 * 86_400_000;
    let sql = format!(
        "SELECT DATE(time_created / 1000, 'unixepoch') as day,
                    COUNT(*),
                    COALESCE(SUM(tokens_input + tokens_output), 0)
             FROM \"{tbl}\"
             WHERE time_created > ?1
             GROUP BY day
             ORDER BY day",
    );
    let mut stmt = conn
        .prepare(&sql)
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
    let tbl = session_table(&conn)?;
    let key_expr = model_key_expr("model");
    let sql = format!(
        "SELECT DISTINCT {key_expr} as mk FROM \"{tbl}\" WHERE model IS NOT NULL AND mk != 'unknown' AND mk != '' ORDER BY mk"
    );
    let mut stmt = match conn.prepare(&sql) {
        Ok(s) => s,
        Err(e) => {
            if e.to_string().contains("no such function") {
                conn.prepare(&format!("SELECT DISTINCT model FROM \"{tbl}\" WHERE model IS NOT NULL ORDER BY model"))
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
        // Normalize once more via Rust to ensure consistent short id (handles provider/modelID/#variant edge).
        result.push(extract_model_id(&raw));
    }
    // Deduplicate after Rust normalization (e.g. two JSON variants that map to same id after Rust parse but SQLite missed)
    result.sort();
    result.dedup();
    result.retain(|s| !s.is_empty() && s != "unknown");
    Ok(result)
}

pub fn get_projects_list(custom_path: Option<&str>) -> Result<Vec<String>, String> {
    let conn = open_db(custom_path)?;
    let tbl = session_table(&conn)?;
    let sql = format!("SELECT DISTINCT project_id FROM \"{tbl}\" WHERE project_id IS NOT NULL ORDER BY project_id");
    let mut stmt = conn
        .prepare(&sql)
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

pub fn get_db_info(custom_path: Option<&str>) -> Result<DbInfo, String> {
    let path = resolve_db_path(custom_path);
    let path_str = path.as_ref().map(|p| p.display().to_string());
    let exists = path.as_ref().is_some_and(|p| p.exists());
    if !exists {
        return Ok(DbInfo {
            path: path_str,
            exists: false,
            schema: "unknown".to_string(),
            session_table: None,
            has_session_v2: false,
            has_legacy_session: false,
            has_session_message: false,
            has_message: false,
            has_part: false,
            session_count: 0,
            message_count: 0,
        });
    }
    let conn = open_db(custom_path)?;
    let has_v2 = table_exists(&conn, "session_v2");
    let has_legacy = table_exists(&conn, "session");
    let has_sm = table_exists(&conn, "session_message");
    let has_msg = table_exists(&conn, "message");
    let has_part = table_exists(&conn, "part");
    let schema = detect_schema(&conn).to_string();
    let tbl = session_table(&conn).ok();
    let session_count: i64 = match tbl {
        Some(t) => conn
            .query_row(&format!("SELECT COUNT(*) FROM \"{t}\""), [], |r| r.get(0))
            .unwrap_or(0),
        None => 0,
    };
    let message_count: i64 = if has_sm && has_v2 {
        conn.query_row("SELECT COUNT(*) FROM session_message", [], |r| r.get(0))
            .unwrap_or(0)
    } else if has_msg {
        conn.query_row("SELECT COUNT(*) FROM message", [], |r| r.get(0))
            .unwrap_or(0)
    } else {
        0
    };
    Ok(DbInfo {
        path: path_str,
        exists: true,
        schema,
        session_table: tbl.map(|s| s.to_string()),
        has_session_v2: has_v2,
        has_legacy_session: has_legacy,
        has_session_message: has_sm,
        has_message: has_msg,
        has_part,
        session_count,
        message_count,
    })
}

/// Resolved DB path for health/settings display (custom > OPENCODE_DB > auto-detect).
pub fn resolved_path_string(custom_path: Option<&str>) -> Option<String> {
    resolve_db_path(custom_path).map(|p| p.display().to_string())
}
