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
            app.manage(database::DbPath(db_path));
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
