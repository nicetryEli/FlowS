pub mod commands;
pub mod database;
pub mod models;
pub mod platform;
pub mod services;

use tauri::Manager;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| std::io::Error::other(e.to_string()))?;
            let db_path = app_data_dir.join("flows.sqlite");
            database::init_db(&db_path)
                .map_err(|e| std::io::Error::other(e.to_string()))?;
            app.manage(database::DbPath(db_path.clone()));

            // Local in-app scheduler foundation: every 5 hours while app runtime is active.
            // Concrete TDTU fetch/credentials are configured in later tasks; this loop maintains
            // sync scheduling state without performing network crawling when not configured.
            let scheduler_db_path = db_path.clone();
            tauri::async_runtime::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(5 * 60 * 60));
                interval.tick().await; // initial immediate tick
                loop {
                    interval.tick().await;
                    if let Ok(conn) = database::open_connection(&scheduler_db_path) {
                        let _ = services::sync_service::ensure_sync_state_row(&conn, "tdtu");
                        let next_run = (chrono::Utc::now() + chrono::Duration::hours(5)).to_rfc3339();
                        let _ = conn.execute(
                            "UPDATE sync_state SET next_run_at = COALESCE(next_run_at, ?1), updated_at = ?2 WHERE source = 'tdtu'",
                            rusqlite::params![next_run, chrono::Utc::now().to_rfc3339()],
                        );
                    }
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::tasks::create_task,
            commands::tasks::get_task,
            commands::tasks::list_tasks,
            commands::tasks::update_task,
            commands::tasks::delete_task,
            commands::tasks::complete_task,
            commands::tasks::set_task_focus,
            commands::tasks::list_focused_tasks,
            commands::tags::create_tag,
            commands::tags::get_tag,
            commands::tags::list_tags,
            commands::tags::update_tag,
            commands::tags::delete_tag,
            commands::tags::attach_tag_to_task,
            commands::tags::detach_tag_from_task,
            commands::tags::list_tags_for_task,
            commands::stats::get_daily_task_summary,
            commands::stats::list_completion_counts_by_date,
            commands::stats::calculate_streak,
            commands::stats::list_monthly_completion_counts,
            commands::calendar::create_calendar_event,
            commands::calendar::get_calendar_event,
            commands::calendar::list_calendar_events,
            commands::calendar::list_calendar_events_in_range,
            commands::calendar::update_calendar_event,
            commands::calendar::delete_calendar_event,
            commands::sync::get_sync_state,
            commands::sync::run_tdtu_sync_with_dataset,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
