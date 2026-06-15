mod commands;
mod db;
mod models;

use commands::AppState;
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            db_path: Mutex::new(None),
            refresh_interval: Mutex::new(15),
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_overview,
            commands::get_sessions,
            commands::get_session_detail,
            commands::get_token_trends,
            commands::get_model_usage,
            commands::get_project_stats,
            commands::get_daily_activity,
            commands::get_cost_breakdown,
            commands::get_models_list,
            commands::get_projects_list,
            commands::set_db_path,
            commands::set_refresh_interval,
            commands::get_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
