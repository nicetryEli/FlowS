"""Idempotent crawler → SQLite ingest.

Implements the contract in docs/erd.md §Idempotent ingest:
- `source_key` (stable identity) UNIQUE per crawled table;
- `source_hash` (sha256 of canonical JSON) detects changes;
- a successful crawl reconciles its kind: insert new → update changed →
  hard-delete vanished (portal is truth);
- a failed source (`None` from collect()) leaves that table untouched;
- `personal_events` is never crawler-managed.
"""

from __future__ import annotations

import datetime as dt
import json
import logging
import re
from pathlib import Path

from db import DEFAULT_DB_PATH, get_db, migrate
from hash_utils import normalize_value, sync_hash

logger = logging.getLogger(__name__)

ALLOWED_STATUS = {"scheduled", "absent", "makeup", "cancelled", "moved"}
_HHMM = re.compile(r"^([01]\d|2[0-3]):([0-5]\d)$")

KIND_SCHEDULE = "schedule"
KIND_EXAM = "exams"
KIND_DEADLINE = "deadlines"


def source_key_for(kind: str, row: dict) -> str:
    if kind == KIND_SCHEDULE:
        date = str(row.get("session_date") or "").strip()
        subject = normalize_value(row.get("subject_name"))
        sp, ep = _to_int(row.get("start_period")), _to_int(row.get("end_period"))
        status = normalize_value(row.get("status")) or "scheduled"
        return f"class:{date}|{subject}|{sp}-{ep}|{status}"
    if kind == KIND_EXAM:
        date = str(row.get("exam_date") or "").strip()
        subject = normalize_value(row.get("subject_name"))
        start = str(row.get("start_time") or "").strip() or "allday"
        etype = normalize_value(row.get("exam_type"))
        return f"exam:{date}|{subject}|{start}|{etype}"
    if kind == KIND_DEADLINE:
        url = str(row.get("activity_url") or "").strip()
        if url:
            return f"deadline:{url}"
        sig = {
            "course_id": normalize_value(row.get("course_id")),
            "activity_name": normalize_value(row.get("activity_name")),
            "due_date": str(row.get("due_date") or "").strip(),
        }
        return f"deadline:{sync_hash(sig)}"
    raise ValueError(f"unknown kind: {kind}")


def source_hash_for(row: dict) -> str:
    return sync_hash(row)


def ingest(data: dict, db_path: str | Path | None = None) -> dict:
    """Upsert crawled rows into SQLite. Never raises for bad rows/sources."""
    migrate(db_path)
    started = dt.datetime.now(dt.timezone.utc)
    totals = {"inserted": 0, "updated": 0, "deleted": 0, "skipped": 0}
    partial = False

    conn = get_db(db_path)
    try:
        conn.execute("BEGIN")
        for kind in (KIND_SCHEDULE, KIND_EXAM, KIND_DEADLINE):
            rows = data.get(kind)
            if rows is None:
                partial = True
                continue
            counters = _reconcile(conn, kind, rows)
            for k, v in counters.items():
                totals[k] += v
        status = "partial" if partial else "ok"
        _record_run(conn, started, status, totals)
        conn.commit()
    except Exception as exc:
        conn.rollback()
        logger.exception("Ingest failed: %s", exc)
        status = "failed"
        try:
            _record_run(conn, started, "failed", totals, error=str(exc))
            conn.commit()
        except Exception:
            conn.rollback()
        raise
    finally:
        conn.close()

    return {"status": status, **totals}


# ---------------------------------------------------------------------------
# Reconcile per kind
# ---------------------------------------------------------------------------

def _reconcile(conn, kind: str, rows: list[dict]) -> dict:
    counters = {"inserted": 0, "updated": 0, "deleted": 0, "skipped": 0}
    normalizer = {"schedule": _norm_class, "exams": _norm_exam, "deadlines": _norm_deadline}[kind]
    table = {"schedule": "class_sessions", "exams": "exams", "deadlines": "deadlines"}[kind]
    now = dt.datetime.now(dt.timezone.utc).isoformat()

    seen: set[str] = set()
    for raw in rows:
        row = normalizer(raw)
        if row is None:
            counters["skipped"] += 1
            continue
        key = source_key_for(kind, row)
        if key in seen:          # duplicate within the same batch (e.g. same activity_url)
            continue
        seen.add(key)
        existing = conn.execute(
            f"SELECT id, source_hash FROM {table} WHERE source_key = ?", (key,)
        ).fetchone()
        if existing is None:
            _insert(conn, kind, row, key, now)
            counters["inserted"] += 1
        elif existing["source_hash"] != source_hash_for(row):
            _update(conn, kind, row, existing["id"], now)
            counters["updated"] += 1
        else:
            conn.execute(
                f"UPDATE {table} SET last_seen_at = ? WHERE id = ?", (now, existing["id"])
            )

    # Vanished rows are deleted (portal is truth). Empty list ⇒ zero rows remain.
    if seen:
        placeholders = ",".join("?" for _ in seen)
        gone = conn.execute(
            f"SELECT id FROM {table} WHERE source_key NOT IN ({placeholders})", tuple(seen)
        ).fetchall()
    else:
        gone = conn.execute(f"SELECT id FROM {table}").fetchall()
    for row in gone:
        conn.execute(f"DELETE FROM {table} WHERE id = ?", (row["id"],))
        counters["deleted"] += 1
    return counters


# ---------------------------------------------------------------------------
# Normalizers / validators
# ---------------------------------------------------------------------------

def _norm_class(raw: dict) -> dict | None:
    subject = str(raw.get("subject_name") or "").strip()
    if not subject:
        return None
    date = _iso_date(raw.get("session_date"))
    if date is None:
        return None
    start, end = _to_int(raw.get("start_period")), _to_int(raw.get("end_period"))
    if start is None or end is None or not (1 <= start <= 15 and 1 <= end <= 15 and end >= start):
        return None
    status = normalize_value(raw.get("status")) or "scheduled"
    if status not in ALLOWED_STATUS:
        status = "scheduled"
    return {
        "subject_name": subject,
        "room": str(raw.get("room") or "").strip() or None,
        "session_date": date.isoformat(),
        "day_of_week": str(raw.get("day_of_week") or "").strip() or None,
        "start_period": start,
        "end_period": end,
        "status": status,
    }


def _norm_exam(raw: dict) -> dict | None:
    subject = str(raw.get("subject_name") or "").strip()
    if not subject:
        return None
    date = _iso_date(raw.get("exam_date"))
    if date is None:
        return None
    start = _hhmm(raw.get("start_time"))
    end = _hhmm(raw.get("end_time"))
    return {
        "subject_name": subject,
        "exam_date": date.isoformat(),
        "start_time": start,   # None ⇒ all-day
        "end_time": end,
        "exam_room": str(raw.get("exam_room") or "").strip() or None,
        "exam_type": str(raw.get("exam_type") or "").strip() or None,
        "notes": str(raw.get("notes") or "").strip() or None,
    }


def _norm_deadline(raw: dict) -> dict | None:
    activity = str(raw.get("activity_name") or "").strip()
    if not activity:
        return None
    due = _iso_datetime(raw.get("due_date"))
    if due is None:
        return None
    return {
        "course_id": str(raw.get("course_id") or "").strip() or None,
        "course_name": str(raw.get("course_name") or "").strip() or None,
        "activity_name": activity,
        "activity_url": str(raw.get("activity_url") or "").strip() or f"deadline:{sync_hash(activity)}",  # keep UNIQUE constraint happy when missing
        "due_date": due,
        "completion_status": str(raw.get("completion_status") or "incomplete").strip() or "incomplete",
    }


def _iso_date(value) -> dt.date | None:
    text = str(value or "").strip()
    try:
        return dt.date.fromisoformat(text)
    except ValueError:
        return None


def _iso_datetime(value) -> str | None:
    text = str(value or "").strip()
    if not text:
        return None
    try:
        parsed = dt.datetime.fromisoformat(text.replace("Z", "+00:00"))
    except ValueError:
        return None
    if parsed.tzinfo is None:
        parsed = parsed.replace(tzinfo=dt.timezone(dt.timedelta(hours=7)))
    return parsed.isoformat()


def _hhmm(value) -> str | None:
    text = str(value or "").strip()
    if not text:
        return None
    match = _HHMM.match(text)
    return f"{match.group(1)}:{match.group(2)}" if match else None


def _to_int(value) -> int | None:
    try:
        return int(value)  # type: ignore[arg-type]
    except (TypeError, ValueError):
        return None


# ---------------------------------------------------------------------------
# Inserts / updates
# ---------------------------------------------------------------------------

def _insert(conn, kind: str, row: dict, key: str, now: str) -> None:
    if kind == KIND_SCHEDULE:
        course_id = _link_course(conn, row["subject_name"], None)
        conn.execute(
            """INSERT INTO class_sessions
               (course_id, subject_name, room, session_date, day_of_week,
                start_period, end_period, status, source_key, source_hash,
                first_seen_at, last_seen_at, created_at, updated_at)
               VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?)""",
            (course_id, row["subject_name"], row["room"], row["session_date"],
             row["day_of_week"], row["start_period"], row["end_period"],
             row["status"], key, source_hash_for(row), now, now, now, now),
        )
    elif kind == KIND_EXAM:
        course_id = _link_course(conn, row["subject_name"], None)
        conn.execute(
            """INSERT INTO exams
               (course_id, subject_name, exam_date, start_time, end_time,
                exam_room, exam_type, notes, source_key, source_hash,
                first_seen_at, last_seen_at, created_at, updated_at)
               VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?)""",
            (course_id, row["subject_name"], row["exam_date"], row["start_time"],
             row["end_time"], row["exam_room"], row["exam_type"], row["notes"],
             key, source_hash_for(row), now, now, now, now),
        )
    else:  # deadline
        course_id = _link_course(conn, row["course_name"], row["course_id"])
        conn.execute(
            """INSERT INTO deadlines
               (course_id, course_name, activity_name, activity_url, due_date,
                completion_status, source_key, source_hash,
                first_seen_at, last_seen_at, created_at, updated_at)
               VALUES (?,?,?,?,?,?,?,?,?,?,?,?)""",
            (course_id, row["course_name"], row["activity_name"], row["activity_url"],
             row["due_date"], row["completion_status"], key, source_hash_for(row),
             now, now, now, now),
        )


def _update(conn, kind: str, row: dict, row_id: int, now: str) -> None:
    if kind == KIND_SCHEDULE:
        course_id = _link_course(conn, row["subject_name"], None)
        conn.execute(
            """UPDATE class_sessions
               SET course_id=?, subject_name=?, room=?, session_date=?, day_of_week=?,
                   start_period=?, end_period=?, status=?, source_hash=?,
                   last_seen_at=?, updated_at=?
               WHERE id=?""",
            (course_id, row["subject_name"], row["room"], row["session_date"],
             row["day_of_week"], row["start_period"], row["end_period"],
             row["status"], source_hash_for(row), now, now, row_id),
        )
    elif kind == KIND_EXAM:
        course_id = _link_course(conn, row["subject_name"], None)
        conn.execute(
            """UPDATE exams
               SET course_id=?, subject_name=?, exam_date=?, start_time=?, end_time=?,
                   exam_room=?, exam_type=?, notes=?, source_hash=?,
                   last_seen_at=?, updated_at=?
               WHERE id=?""",
            (course_id, row["subject_name"], row["exam_date"], row["start_time"],
             row["end_time"], row["exam_room"], row["exam_type"], row["notes"],
             source_hash_for(row), now, now, row_id),
        )
    else:
        course_id = _link_course(conn, row["course_name"], row["course_id"])
        conn.execute(
            """UPDATE deadlines
               SET course_id=?, course_name=?, activity_name=?, activity_url=?, due_date=?,
                   completion_status=?, source_hash=?, last_seen_at=?, updated_at=?
               WHERE id=?""",
            (course_id, row["course_name"], row["activity_name"], row["activity_url"],
             row["due_date"], row["completion_status"], source_hash_for(row),
             now, now, row_id),
        )


def _link_course(conn, name: str | None, moodle_id: str | None) -> int | None:
    """Best-effort course link — never creates duplicates of near-identical
    names, never raises. Links only confident matches on exact (cleaned,
    lowercased) name or exact Moodle id."""
    clean = str(name or "").strip()
    if not clean:
        return None
    key = normalize_value(clean)
    row = conn.execute(
        "SELECT id FROM courses WHERE lower(trim(name)) = ? OR (? IS NOT NULL AND moodle_course_id = ?)",
        (key, moodle_id, moodle_id),
    ).fetchone()
    if row:
        return int(row["id"])
    if moodle_id:
        now = dt.datetime.now(dt.timezone.utc).isoformat()
        cur = conn.execute(
            "INSERT OR IGNORE INTO courses (name, moodle_course_id, created_at, updated_at) VALUES (?,?,?,?)",
            (clean, str(moodle_id).strip(), now, now),
        )
        if cur.rowcount == 1:
            return int(cur.lastrowid)
        row = conn.execute("SELECT id FROM courses WHERE moodle_course_id = ?", (str(moodle_id).strip(),)).fetchone()
        return int(row["id"]) if row else None
    return None


def _record_run(conn, started, status: str, totals: dict, error: str | None = None) -> None:
    detail = {**totals}
    if error:
        detail["error"] = error[:500]
    conn.execute(
        """INSERT INTO sync_runs (started_at, finished_at, status, detail)
           VALUES (?,?,?,?)""",
        (started.isoformat(), dt.datetime.now(dt.timezone.utc).isoformat(),
         status, json.dumps(detail, ensure_ascii=False)),
    )



