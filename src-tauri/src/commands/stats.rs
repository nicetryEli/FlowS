use tauri::State;

use crate::database::{self, DbPath};
use crate::services::stats_service::{self, DailyActivity, DailyTaskSummary, MonthlyCompletionCount};

fn with_conn<T>(
    db: &State<'_, DbPath>,
    f: impl FnOnce(&rusqlite::Connection) -> crate::database::DbResult<T>,
) -> Result<T, String> {
    let conn = database::open_connection(&db.0).map_err(|e| e.to_string())?;
    f(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_daily_task_summary(db: State<'_, DbPath>, date: String) -> Result<DailyTaskSummary, String> {
    with_conn(&db, |conn| stats_service::get_daily_task_summary(conn, &date))
}

#[tauri::command]
pub fn list_completion_counts_by_date(
    db: State<'_, DbPath>,
    start_date: String,
    end_date: String,
) -> Result<Vec<DailyActivity>, String> {
    with_conn(&db, |conn| stats_service::list_completion_counts_by_date(conn, &start_date, &end_date))
}

#[tauri::command]
pub fn calculate_streak(db: State<'_, DbPath>, as_of_date: String) -> Result<i64, String> {
    with_conn(&db, |conn| stats_service::calculate_streak(conn, &as_of_date))
}

#[tauri::command]
pub fn list_monthly_completion_counts(db: State<'_, DbPath>, year: i32) -> Result<Vec<MonthlyCompletionCount>, String> {
    with_conn(&db, |conn| stats_service::list_monthly_completion_counts(conn, year))
}
