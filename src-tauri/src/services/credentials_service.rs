use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::database::DbResult;

const KEYRING_SERVICE: &str = "flows.tdtu";
const KEYRING_PASSWORD_USER: &str = "password";
const SETTINGS_STUDENT_ID: &str = "tdtu.student_id";
const SETTINGS_SYNC_ENABLED: &str = "tdtu.sync_enabled";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TdtuSyncConfig {
    pub student_id: Option<String>,
    pub has_password: bool,
    pub sync_enabled: bool,
}

fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn get_setting(conn: &Connection, key: &str) -> DbResult<Option<String>> {
    let value = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = ?1",
            params![key],
            |row| row.get(0),
        )
        .optional()?;
    Ok(value)
}

fn set_setting(conn: &Connection, key: &str, value: Option<&str>) -> DbResult<()> {
    let now = now_rfc3339();
    match value {
        Some(v) => {
            conn.execute(
                "INSERT INTO app_settings (key, value, updated_at) VALUES (?1, ?2, ?3)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
                params![key, v, now],
            )?;
        }
        None => {
            conn.execute("DELETE FROM app_settings WHERE key = ?1", params![key])?;
        }
    }
    Ok(())
}

pub fn set_student_id(conn: &Connection, student_id: Option<String>) -> DbResult<()> {
    set_setting(conn, SETTINGS_STUDENT_ID, student_id.as_deref())
}

pub fn get_student_id(conn: &Connection) -> DbResult<Option<String>> {
    get_setting(conn, SETTINGS_STUDENT_ID)
}

pub fn set_sync_enabled(conn: &Connection, enabled: bool) -> DbResult<()> {
    set_setting(conn, SETTINGS_SYNC_ENABLED, Some(if enabled { "1" } else { "0" }))
}

pub fn get_sync_enabled(conn: &Connection) -> DbResult<bool> {
    Ok(get_setting(conn, SETTINGS_SYNC_ENABLED)?.as_deref() != Some("0"))
}

pub fn set_password(password: Option<String>) -> Result<(), String> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_PASSWORD_USER)
        .map_err(|e| format!("keyring unavailable: {e}"))?;
    match password {
        Some(pw) if !pw.is_empty() => entry.set_password(&pw).map_err(|e| format!("failed to store password in keyring: {e}"))?,
        _ => {
            let _ = entry.delete_credential();
        }
    }
    Ok(())
}

pub fn has_password() -> bool {
    let Ok(entry) = keyring::Entry::new(KEYRING_SERVICE, KEYRING_PASSWORD_USER) else {
        return false;
    };
    entry.get_password().is_ok()
}

pub fn get_password() -> Result<Option<String>, String> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_PASSWORD_USER)
        .map_err(|e| format!("keyring unavailable: {e}"))?;
    match entry.get_password() {
        Ok(pw) => Ok(Some(pw)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(format!("failed to read password from keyring: {e}")),
    }
}

pub fn get_tdtu_sync_config(conn: &Connection) -> DbResult<TdtuSyncConfig> {
    Ok(TdtuSyncConfig {
        student_id: get_student_id(conn)?,
        has_password: has_password(),
        sync_enabled: get_sync_enabled(conn)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::init_test_db;

    #[test]
    fn settings_roundtrip_for_student_id_and_enabled_flag() {
        let conn = init_test_db().unwrap();
        set_student_id(&conn, Some("52400210".into())).unwrap();
        set_sync_enabled(&conn, false).unwrap();

        let cfg = get_tdtu_sync_config(&conn).unwrap();
        assert_eq!(cfg.student_id.as_deref(), Some("52400210"));
        assert!(!cfg.sync_enabled);
        // has_password depends on system keyring; do not assert secret state here
    }
}
