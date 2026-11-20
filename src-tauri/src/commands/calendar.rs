use tauri::State;

use crate::database::{self, DbPath};
use crate::models::{CalendarEvent, CalendarEventSource, CalendarEventUpdate, NewCalendarEvent};
use crate::services::calendar_service;

fn with_conn<T>(
    db: &State<'_, DbPath>,
    f: impl FnOnce(&rusqlite::Connection) -> crate::database::DbResult<T>,
) -> Result<T, String> {
    let conn = database::open_connection(&db.0).map_err(|e| e.to_string())?;
    f(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_calendar_event(db: State<'_, DbPath>, input: NewCalendarEvent) -> Result<CalendarEvent, String> {
    with_conn(&db, |conn| calendar_service::create_event(conn, input))
}

#[tauri::command]
pub fn get_calendar_event(db: State<'_, DbPath>, id: i64) -> Result<Option<CalendarEvent>, String> {
    with_conn(&db, |conn| calendar_service::get_event(conn, id))
}

#[tauri::command]
pub fn list_calendar_events(db: State<'_, DbPath>, source: Option<CalendarEventSource>) -> Result<Vec<CalendarEvent>, String> {
    with_conn(&db, |conn| calendar_service::list_events(conn, source))
}

#[tauri::command]
pub fn list_calendar_events_in_range(
    db: State<'_, DbPath>,
    start_at: String,
    end_at: String,
) -> Result<Vec<CalendarEvent>, String> {
    with_conn(&db, |conn| calendar_service::list_events_in_range(conn, &start_at, &end_at))
}

#[tauri::command]
pub fn update_calendar_event(
    db: State<'_, DbPath>,
    id: i64,
    patch: CalendarEventUpdate,
) -> Result<CalendarEvent, String> {
    with_conn(&db, |conn| calendar_service::update_event(conn, id, patch))
}

#[tauri::command]
pub fn delete_calendar_event(db: State<'_, DbPath>, id: i64) -> Result<(), String> {
    with_conn(&db, |conn| calendar_service::delete_event(conn, id))
}
