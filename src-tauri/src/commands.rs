use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::time::Instant;
use tauri::State;
use tracing::instrument;

use crate::db;
use crate::models::*;

pub struct AppState {
    pub db_path: Mutex<Option<String>>,
    pub refresh_interval: Mutex<u64>,
}

impl AppState {
    pub fn get_db_path(&self) -> Option<String> {
        self.db_path.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    pub fn get_refresh_interval(&self) -> u64 {
        *self.refresh_interval.lock().unwrap_or_else(|e| e.into_inner())
    }
}

static START_TIME: Lazy<Instant> = Lazy::new(Instant::now);

#[tauri::command]
#[instrument(skip(state))]
pub fn get_overview(
    state: State<AppState>,
    date_from: Option<i64>,
    date_to: Option<i64>,
) -> Result<Overview, String> {
    let path = state.get_db_path();
    db::get_overview(path.as_deref(), date_from, date_to)
}

#[tauri::command]
#[instrument(skip(state))]
pub fn get_sessions(
    state: State<AppState>,
    limit: Option<i64>,
    offset: Option<i64>,
    model_filter: Option<String>,
    project_filter: Option<String>,
    date_from: Option<i64>,
    date_to: Option<i64>,
) -> Result<Vec<SessionRow>, String> {
    let path = state.get_db_path();
    db::get_sessions(
        path.as_deref(),
        limit.unwrap_or(50),
        offset.unwrap_or(0),
        model_filter,
        project_filter,
        date_from,
        date_to,
    )
}

#[tauri::command]
#[instrument(skip(state))]
pub fn get_session_detail(
    state: State<AppState>,
    session_id: String,
) -> Result<SessionDetail, String> {
    let path = state.get_db_path();
    db::get_session_detail(path.as_deref(), &session_id)
}

#[tauri::command]
#[instrument(skip(state))]
pub fn get_token_trends(
    state: State<AppState>,
    days: Option<i64>,
) -> Result<Vec<TokenTrend>, String> {
    let path = state.get_db_path();
    db::get_token_trends(path.as_deref(), days.unwrap_or(30))
}

#[tauri::command]
#[instrument(skip(state))]
pub fn get_model_usage(
    state: State<AppState>,
    date_from: Option<i64>,
    date_to: Option<i64>,
) -> Result<Vec<ModelStat>, String> {
    let path = state.get_db_path();
    db::get_model_usage(path.as_deref(), date_from, date_to)
}

#[tauri::command]
#[instrument(skip(state))]
pub fn get_project_stats(
    state: State<AppState>,
    date_from: Option<i64>,
    date_to: Option<i64>,
) -> Result<Vec<ProjectStat>, String> {
    let path = state.get_db_path();
    db::get_project_stats(path.as_deref(), date_from, date_to)
}

#[tauri::command]
#[instrument(skip(state))]
pub fn get_daily_activity(
    state: State<AppState>,
    months: Option<i64>,
) -> Result<Vec<DayActivity>, String> {
    let path = state.get_db_path();
    db::get_daily_activity(path.as_deref(), months.unwrap_or(12))
}

#[tauri::command]
#[instrument(skip(state))]
pub fn get_cost_breakdown(
    state: State<AppState>,
    date_from: Option<i64>,
    date_to: Option<i64>,
) -> Result<CostBreakdown, String> {
    let path = state.get_db_path();
    db::get_cost_breakdown(path.as_deref(), date_from, date_to)
}

#[tauri::command]
#[instrument(skip(state))]
pub fn get_models_list(state: State<AppState>) -> Result<Vec<String>, String> {
    let path = state.get_db_path();
    db::get_models_list(path.as_deref())
}

#[tauri::command]
#[instrument(skip(state))]
pub fn get_projects_list(state: State<AppState>) -> Result<Vec<String>, String> {
    let path = state.get_db_path();
    db::get_projects_list(path.as_deref())
}

#[tauri::command]
#[instrument(skip(state))]
pub fn set_db_path(state: State<AppState>, path: String) -> Result<(), String> {
    tracing::info!("Setting DB path to: {path}");
    *state.db_path.lock().unwrap_or_else(|e| e.into_inner()) = Some(path);
    Ok(())
}

#[tauri::command]
#[instrument(skip(state))]
pub fn set_refresh_interval(state: State<AppState>, interval: u64) -> Result<(), String> {
    tracing::info!("Setting refresh interval to: {interval}s");
    *state.refresh_interval.lock().unwrap_or_else(|e| e.into_inner()) = interval;
    Ok(())
}

#[tauri::command]
#[instrument(skip(state))]
pub fn get_settings(state: State<AppState>) -> Result<AppSettings, String> {
    Ok(AppSettings {
        db_path: state.get_db_path(),
        refresh_interval: state.get_refresh_interval(),
        theme: "system".to_string(),
    })
}

#[tauri::command]
#[instrument(skip(state))]
pub fn health_check(state: State<AppState>) -> Result<HealthStatus, String> {
    let path = state.get_db_path();
    let uptime = format!("{}s", START_TIME.elapsed().as_secs());
    let db_ok = path.as_deref().is_some_and(|p| {
        std::path::Path::new(p).exists() && db::verify_connection(p)
    });

    Ok(HealthStatus {
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime,
        db_connected: db_ok,
        db_path: path,
    })
}
