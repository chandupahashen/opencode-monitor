mod commands;
mod db;
mod models;

use commands::AppState;
use std::sync::Mutex;
use tracing_subscriber::EnvFilter;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize structured logging to both stdout and file
    let log_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("opencode-monitor")
        .join("logs");
    std::fs::create_dir_all(&log_dir).ok();

    let file_appender = tracing_appender::rolling::daily(log_dir, "opencode-monitor.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_writer(non_blocking)
        .with_ansi(false)
        .init();

    tracing::info!("Starting OpenCode Monitor");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_updater::Builder::default().build())
        .manage(AppState {
            db_path: Mutex::new(None),
            refresh_interval: Mutex::new(5),
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
            commands::health_check,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
