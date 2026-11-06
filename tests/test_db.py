import sqlite3, pytest
from db import migrate, get_db

def test_fresh_db_migrates_and_is_idempotent(tmp_path):
    db = tmp_path / "app.db"
    assert migrate(db) == 1
    assert migrate(db) == 0  # re-run applies nothing

def test_calendar_events_view_derives_session_times(tmp_path):
    db = tmp_path / "app.db"; migrate(db)
    conn = get_db(db)
    conn.execute("""INSERT INTO class_sessions
        (course_id, subject_name, room, session_date, day_of_week,
         start_period, end_period, status, source_key, source_hash,
         first_seen_at, last_seen_at, created_at, updated_at)
        VALUES (NULL, 'X', 'B403', '2026-11-05', 'Thursday',
                7, 9, 'scheduled', 'class:1', 'h', 't', 't', 't', 't')""")
    row = conn.execute("SELECT * FROM calendar_events WHERE kind='class'").fetchone()
    assert row["start_dt"] == "2026-11-05T12:45"
    assert row["end_dt"] == "2026-11-05T15:15"   # period 9 (14:25) + 50 min
    assert row["all_day"] == 0

def test_calendar_events_view_handles_all_day_exam(tmp_path):
    db = tmp_path / "app.db"; migrate(db)
    conn = get_db(db)
    conn.execute("""INSERT INTO exams
        (course_id, subject_name, exam_date, start_time, end_time, exam_room,
         exam_type, notes, source_key, source_hash,
         first_seen_at, last_seen_at, created_at, updated_at)
        VALUES (NULL, 'CNPM', '2026-09-30', NULL, NULL, 'A507',
                'Giữa kỳ', NULL, 'exam:1', 'h', 't', 't', 't', 't')""")
    row = conn.execute("SELECT * FROM calendar_events WHERE kind='exam'").fetchone()
    assert row["all_day"] == 1
    assert row["start_dt"] == "2026-09-30T00:00"

def test_wal_and_foreign_keys_enabled(tmp_path):
    db = tmp_path / "app.db"; migrate(db)
    conn = get_db(db)
    assert conn.execute("PRAGMA journal_mode").fetchone()[0] == "wal"
    assert conn.execute("PRAGMA foreign_keys").fetchone()[0] == 1
