import json
import sqlite3
from pathlib import Path

import pytest

from db import migrate, get_db
from ingest import ingest


def sample_schedule():
    return [
        {"student_id": "52400210", "subject_name": "Công nghệ phần mềm",
         "room": "D0205-B", "day_of_week": "Thursday", "session_date": "2026-11-05",
         "start_period": 7, "end_period": 9, "status": "scheduled"},
        {"student_id": "52400210", "subject_name": "Nhập môn Học máy",
         "room": "B403", "day_of_week": "Tuesday", "session_date": "2026-11-10",
         "start_period": 4, "end_period": 6, "status": "absent"},
    ]


def sample_exams():
    return [
        {"subject_name": "Công nghệ phần mềm", "exam_date": "2026-09-30",
         "start_time": "07:30", "end_time": "", "exam_room": "A507",
         "exam_type": "Giữa kỳ", "notes": "Crawled from exam grid"},
    ]


def sample_deadlines():
    return [
        {"course_id": "12345", "course_name": "Công nghệ phần mềm",
         "activity_name": "Bài tập 3 — Chương 5",
         "activity_url": "https://elearning.tdtu.edu.vn/mod/assign/view.php?id=123",
         "due_date": "2026-11-12T23:59:00+07:00",
         "completion_status": "incomplete"},
    ]


def rows(db_path, table):
    conn = get_db(db_path)
    try:
        return conn.execute(f"SELECT * FROM {table}").fetchall()
    finally:
        conn.close()


def test_ingest_inserts_new_rows(tmp_path):
    db = tmp_path / "app.db"
    migrate(db)
    report = ingest({"schedule": sample_schedule(), "exams": sample_exams(),
                     "deadlines": sample_deadlines()}, db)
    assert report["status"] == "ok"
    assert report["inserted"] == 2 + 1 + 1
    assert len(rows(db, "class_sessions")) == 2
    assert len(rows(db, "exams")) == 1
    assert len(rows(db, "deadlines")) == 1


def test_reingest_is_noop(tmp_path):
    db = tmp_path / "app.db"
    data = {"schedule": sample_schedule(), "exams": sample_exams(),
            "deadlines": sample_deadlines()}
    first = ingest(data, db)
    conn = get_db(db)
    try:
        before = [dict(r) for r in conn.execute("SELECT * FROM class_sessions").fetchall()]
    finally:
        conn.close()
    second = ingest(data, db)
    assert (second["inserted"], second["updated"], second["deleted"]) == (0, 0, 0)
    conn = get_db(db)
    try:
        after = [dict(r) for r in conn.execute("SELECT * FROM class_sessions").fetchall()]
    finally:
        conn.close()
    assert {r["id"]: r["updated_at"] for r in after} == {r["id"]: r["updated_at"] for r in before}


def test_changed_row_updates_in_place(tmp_path):
    db = tmp_path / "app.db"
    ingest({"schedule": sample_schedule(), "exams": None, "deadlines": None}, db)
    changed = sample_schedule()
    changed[0]["room"] = "D0205-B (đổi phòng)"
    report = ingest({"schedule": changed, "exams": None, "deadlines": None}, db)
    assert report["updated"] == 1 and report["inserted"] == 0
    cs = rows(db, "class_sessions")
    assert any(r["room"] == "D0205-B (đổi phòng)" for r in cs)


def test_vanished_row_is_deleted(tmp_path):
    db = tmp_path / "app.db"
    ingest({"schedule": sample_schedule(), "exams": None, "deadlines": None}, db)
    report = ingest({"schedule": [sample_schedule()[0]], "exams": None, "deadlines": None}, db)
    assert report["deleted"] == 1
    assert len(rows(db, "class_sessions")) == 1
    assert rows(db, "class_sessions")[0]["subject_name"] == "Công nghệ phần mềm"


def test_failed_source_leaves_table_untouched(tmp_path):
    db = tmp_path / "app.db"
    ingest({"schedule": sample_schedule(), "exams": None, "deadlines": None}, db)
    report = ingest({"schedule": None, "exams": [], "deadlines": []}, db)
    assert report["status"] == "partial"
    assert len(rows(db, "class_sessions")) == 2  # untouched
    assert len(rows(db, "exams")) == 0


def test_invalid_deadline_is_skipped_and_reported(tmp_path):
    db = tmp_path / "app.db"
    bad = {"course_id": "1", "course_name": "X",
           "activity_name": "Broken",
           "activity_url": "https://elearning.tdtu.edu.vn/mod/assign/view.php?id=999",
           "due_date": "every friday :)", "completion_status": "incomplete"}
    report = ingest({"schedule": [], "exams": [], "deadlines": [bad]}, db)
    assert report["skipped"] == 1
    assert len(rows(db, "deadlines")) == 0


def test_all_day_exam_inserted_with_null_times(tmp_path):
    db = tmp_path / "app.db"
    all_day = dict(sample_exams()[0])
    all_day["start_time"] = ""
    ingest({"schedule": [], "exams": [all_day], "deadlines": []}, db)
    ex = rows(db, "exams")[0]
    assert ex["start_time"] is None and ex["end_time"] is None
    conn = get_db(db)
    view_row = conn.execute("SELECT all_day, start_dt FROM calendar_events WHERE kind='exam'").fetchone()
    conn.close()
    assert view_row["all_day"] == 1
    assert view_row["start_dt"] == "2026-09-30T00:00"


def test_duplicate_activity_url_collapses(tmp_path):
    db = tmp_path / "app.db"
    dup = sample_deadlines() + sample_deadlines()
    ingest({"schedule": [], "exams": [], "deadlines": dup}, db)
    assert len(rows(db, "deadlines")) == 1


def test_course_link_best_effort(tmp_path):
    db = tmp_path / "app.db"
    ingest({"schedule": [], "exams": [], "deadlines": sample_deadlines()}, db)
    report = ingest({"schedule": sample_schedule(), "exams": sample_exams(), "deadlines": None}, db)
    cs = rows(db, "class_sessions")
    linked = [r for r in cs if r["subject_name"] == "Công nghệ phần mềm"][0]
    assert linked["course_id"] is not None
    other = [r for r in cs if r["subject_name"] == "Nhập môn Học máy"][0]
    assert other["course_id"] is None  # no matching course — not an error


def test_sync_runs_records_partial(tmp_path):
    db = tmp_path / "app.db"
    ingest({"schedule": None, "exams": sample_exams(), "deadlines": []}, db)
    conn = get_db(db)
    run = conn.execute("SELECT * FROM sync_runs ORDER BY id DESC LIMIT 1").fetchone()
    conn.close()
    assert run["status"] == "partial"
    detail = json.loads(run["detail"])
    assert detail["inserted"] == 1
