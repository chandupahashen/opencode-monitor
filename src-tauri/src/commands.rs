use std::sync::Mutex;
use tauri::State;

use crate::db;
use crate::models::*;

pub struct AppState {
    pub db_path: Mutex<Option<String>>,
    pub refresh_interval: Mutex<u64>,
}

#[tauri::command]
pub fn get_overview(state: State<AppState>) -> Result<Overview, String> {
    let path = state.db_path.lock().unwrap().clone();
    db::get_overview(path.as_deref())
}

#[tauri::command]
pub fn get_sessions(
    state: State<AppState>,
    limit: Option<i64>,
    offset: Option<i64>,
    model_filter: Option<String>,
    project_filter: Option<String>,
    date_from: Option<i64>,
    date_to: Option<i64>,
) -> Result<Vec<SessionRow>, String> {
    let path = state.db_path.lock().unwrap().clone();
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
pub fn get_session_detail(
    state: State<AppState>,
    session_id: String,
) -> Result<SessionDetail, String> {
    let path = state.db_path.lock().unwrap().clone();
    db::get_session_detail(path.as_deref(), &session_id)
}

#[tauri::command]
pub fn get_token_trends(
    state: State<AppState>,
    days: Option<i64>,
) -> Result<Vec<TokenTrend>, String> {
    let path = state.db_path.lock().unwrap().clone();
    db::get_token_trends(path.as_deref(), days.unwrap_or(30))
}

#[tauri::command]
pub fn get_model_usage(state: State<AppState>) -> Result<Vec<ModelStat>, String> {
    let path = state.db_path.lock().unwrap().clone();
    db::get_model_usage(path.as_deref())
}

#[tauri::command]
pub fn get_project_stats(state: State<AppState>) -> Result<Vec<ProjectStat>, String> {
    let path = state.db_path.lock().unwrap().clone();
    db::get_project_stats(path.as_deref())
}

#[tauri::command]
pub fn get_daily_activity(
    state: State<AppState>,
    months: Option<i64>,
) -> Result<Vec<DayActivity>, String> {
    let path = state.db_path.lock().unwrap().clone();
    db::get_daily_activity(path.as_deref(), months.unwrap_or(12))
}

#[tauri::command]
pub fn get_cost_breakdown(state: State<AppState>) -> Result<CostBreakdown, String> {
    let path = state.db_path.lock().unwrap().clone();
    db::get_cost_breakdown(path.as_deref())
}

#[tauri::command]
pub fn get_models_list(state: State<AppState>) -> Result<Vec<String>, String> {
    let path = state.db_path.lock().unwrap().clone();
    db::get_models_list(path.as_deref())
}

#[tauri::command]
pub fn get_projects_list(state: State<AppState>) -> Result<Vec<String>, String> {
    let path = state.db_path.lock().unwrap().clone();
    db::get_projects_list(path.as_deref())
}

#[tauri::command]
pub fn set_db_path(state: State<AppState>, path: String) -> Result<(), String> {
    *state.db_path.lock().unwrap() = Some(path);
    Ok(())
}

#[tauri::command]
pub fn set_refresh_interval(state: State<AppState>, interval: u64) -> Result<(), String> {
    *state.refresh_interval.lock().unwrap() = interval;
    Ok(())
}

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Result<AppSettings, String> {
    Ok(AppSettings {
        db_path: state.db_path.lock().unwrap().clone(),
        refresh_interval: *state.refresh_interval.lock().unwrap(),
        theme: "system".to_string(),
    })
}
