use tauri::State;

use crate::database::{self, DbPath};
use crate::models::{NewTask, Task, TaskUpdate};
use crate::services::task_service;

fn with_conn<T>(
    db: &State<'_, DbPath>,
    f: impl FnOnce(&rusqlite::Connection) -> crate::database::DbResult<T>,
) -> Result<T, String> {
    let conn = database::open_connection(&db.0).map_err(|e| e.to_string())?;
    f(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_task(db: State<'_, DbPath>, input: NewTask) -> Result<Task, String> {
    with_conn(&db, |conn| task_service::create_task(conn, input))
}

#[tauri::command]
pub fn get_task(db: State<'_, DbPath>, id: i64) -> Result<Option<Task>, String> {
    with_conn(&db, |conn| task_service::get_task(conn, id))
}

#[tauri::command]
pub fn list_tasks(db: State<'_, DbPath>) -> Result<Vec<Task>, String> {
    with_conn(&db, |conn| task_service::list_tasks(conn))
}

#[tauri::command]
pub fn update_task(db: State<'_, DbPath>, id: i64, patch: TaskUpdate) -> Result<Task, String> {
    with_conn(&db, |conn| task_service::update_task(conn, id, patch))
}

#[tauri::command]
pub fn delete_task(db: State<'_, DbPath>, id: i64) -> Result<(), String> {
    with_conn(&db, |conn| task_service::delete_task(conn, id))
}

#[tauri::command]
pub fn complete_task(
    db: State<'_, DbPath>,
    id: i64,
    completed_at: Option<String>,
) -> Result<Task, String> {
    with_conn(&db, |conn| task_service::complete_task(conn, id, completed_at))
}

#[tauri::command]
pub fn set_task_focus(db: State<'_, DbPath>, id: i64, focused: bool) -> Result<Task, String> {
    with_conn(&db, |conn| task_service::set_task_focus(conn, id, focused))
}

#[tauri::command]
pub fn list_focused_tasks(db: State<'_, DbPath>) -> Result<Vec<Task>, String> {
    with_conn(&db, |conn| task_service::list_focused_tasks(conn))
}
