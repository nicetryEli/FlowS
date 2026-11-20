use chrono::Utc;
use tauri::State;

use crate::database::{self, DbPath};
use crate::services::sync_service::{self, SyncRunResult};
use crate::services::tdtu_sync_adapter::TdtuDataset;

fn with_conn<T>(
    db: &State<'_, DbPath>,
    f: impl FnOnce(&rusqlite::Connection) -> crate::database::DbResult<T>,
) -> Result<T, String> {
    let conn = database::open_connection(&db.0).map_err(|e| e.to_string())?;
    f(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_sync_state(db: State<'_, DbPath>, source: String) -> Result<Option<serde_json::Value>, String> {
    with_conn(&db, |conn| sync_service::get_sync_state(conn, &source))
}

#[tauri::command]
pub fn run_tdtu_sync_with_dataset(
    db: State<'_, DbPath>,
    dataset: TdtuDataset,
) -> Result<SyncRunResult, String> {
    with_conn(&db, |conn| {
        sync_service::run_tdtu_sync_with_dataset(conn, "tdtu", &dataset, Utc::now())
    })
}
