use tauri::State;

use crate::database::{self, DbPath};
use crate::services::credentials_service::{self, TdtuSyncConfig};

fn with_conn<T>(
    db: &State<'_, DbPath>,
    f: impl FnOnce(&rusqlite::Connection) -> crate::database::DbResult<T>,
) -> Result<T, String> {
    let conn = database::open_connection(&db.0).map_err(|e| e.to_string())?;
    f(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_tdtu_sync_config(db: State<'_, DbPath>) -> Result<TdtuSyncConfig, String> {
    with_conn(&db, |conn| credentials_service::get_tdtu_sync_config(conn))
}

#[tauri::command]
pub fn set_tdtu_student_id(db: State<'_, DbPath>, student_id: Option<String>) -> Result<(), String> {
    with_conn(&db, |conn| credentials_service::set_student_id(conn, student_id))
}

#[tauri::command]
pub fn set_tdtu_sync_enabled(db: State<'_, DbPath>, enabled: bool) -> Result<(), String> {
    with_conn(&db, |conn| credentials_service::set_sync_enabled(conn, enabled))
}

#[tauri::command]
pub fn set_tdtu_password(password: Option<String>) -> Result<(), String> {
    credentials_service::set_password(password)
}

// Intentionally no command to read plaintext password back to frontend.
