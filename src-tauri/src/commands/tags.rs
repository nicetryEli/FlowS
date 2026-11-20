use tauri::State;

use crate::database::{self, DbPath};
use crate::models::{NewTag, Tag, TagUpdate};
use crate::services::tag_service;

fn with_conn<T>(
    db: &State<'_, DbPath>,
    f: impl FnOnce(&rusqlite::Connection) -> crate::database::DbResult<T>,
) -> Result<T, String> {
    let conn = database::open_connection(&db.0).map_err(|e| e.to_string())?;
    f(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_tag(db: State<'_, DbPath>, input: NewTag) -> Result<Tag, String> {
    with_conn(&db, |conn| tag_service::create_tag(conn, input))
}

#[tauri::command]
pub fn get_tag(db: State<'_, DbPath>, id: i64) -> Result<Option<Tag>, String> {
    with_conn(&db, |conn| tag_service::get_tag(conn, id))
}

#[tauri::command]
pub fn list_tags(db: State<'_, DbPath>) -> Result<Vec<Tag>, String> {
    with_conn(&db, |conn| tag_service::list_tags(conn))
}

#[tauri::command]
pub fn update_tag(db: State<'_, DbPath>, id: i64, patch: TagUpdate) -> Result<Tag, String> {
    with_conn(&db, |conn| tag_service::update_tag(conn, id, patch))
}

#[tauri::command]
pub fn delete_tag(db: State<'_, DbPath>, id: i64) -> Result<(), String> {
    with_conn(&db, |conn| tag_service::delete_tag(conn, id))
}

#[tauri::command]
pub fn attach_tag_to_task(db: State<'_, DbPath>, task_id: i64, tag_id: i64) -> Result<(), String> {
    with_conn(&db, |conn| tag_service::attach_tag_to_task(conn, task_id, tag_id))
}

#[tauri::command]
pub fn detach_tag_from_task(db: State<'_, DbPath>, task_id: i64, tag_id: i64) -> Result<(), String> {
    with_conn(&db, |conn| tag_service::detach_tag_from_task(conn, task_id, tag_id))
}

#[tauri::command]
pub fn list_tags_for_task(db: State<'_, DbPath>, task_id: i64) -> Result<Vec<Tag>, String> {
    with_conn(&db, |conn| tag_service::list_tags_for_task(conn, task_id))
}
