"""FastAPI backend — local-only, single user.

Contract served to the Flutter Web app (see docs/erd.md and
docs/superpowers/plans/2026-11-06-sqlite-fastapi-backend.md Task 4):

  GET  /api/health                 -> {"status": "ok"}
  GET  /api/events?from&to         -> unified calendar feed (calendar_events view)
  GET  /api/personal_events        -> personal rows
  POST /api/personal_events        -> 201 + row (extra 'kind' rejected 422)
  PATCH /api/personal_events/{id}  -> 200 row | 404
  DELETE /api/personal_events/{id} -> 204 | 404 (never crawled rows)
  GET  /api/sync_runs              -> last 10 runs
  POST /api/crawl                  -> 202 (background thread) or 409 while running
  GET  /                           -> static frontend/build/web (if built)

SQLite (docs/erd.md v1.0) is the ONLY source of truth.
"""

import datetime as dt
import json
import logging
import threading
from pathlib import Path

from fastapi import FastAPI, HTTPException, Query
from fastapi.responses import JSONResponse
from fastapi.staticfiles import StaticFiles
from pydantic import BaseModel, ConfigDict, Field

import run_hour
from db import get_db, migrate

logger = logging.getLogger(__name__)

STATIC_DIR = Path(__file__).resolve().parent / "frontend" / "build" / "web"
_crawl_lock = threading.Lock()
_crawl_running = False


def _iso_local(raw: str | None) -> str | None:
    return raw


def create_app() -> FastAPI:
    migrate()  # resolves LOCAL_DB_PATH dynamically (or data/app.db)
    app = FastAPI(title="TDTU Calendar API", version="1.0.0")

    # ------------------------------------------------------------------
    # Health / meta
    # ------------------------------------------------------------------

    @app.get("/api/health")
    def health():
        return {"status": "ok"}

    @app.get("/api/sync_runs")
    def sync_runs():
        conn = get_db()
        try:
            rows = conn.execute(
                "SELECT id, started_at, finished_at, status, detail "
                "FROM sync_runs ORDER BY id DESC LIMIT 10"
            ).fetchall()
        finally:
            conn.close()
        return [dict(r) for r in rows]

    # ------------------------------------------------------------------
    # Unified events feed
    # ------------------------------------------------------------------

    @app.get("/api/events")
    def events(
        start: str = Query(..., alias="from", pattern=r"^\d{4}-\d{2}-\d{2}$"),
        end: str = Query(..., alias="to", pattern=r"^\d{4}-\d{2}-\d{2}$"),
    ):
        end_bound = f"{end}T23:59:59.999"
        conn = get_db()
        try:
            rows = conn.execute(
                """SELECT kind, ref_id, title, location, start_dt, end_dt, all_day, tag,
                          notes, activity_url, course_name, exam_type, color
                   FROM calendar_events
                   WHERE start_dt <= ? AND end_dt >= ?
                   ORDER BY start_dt""",
                (end_bound, f"{start}T00:00:00"),
            ).fetchall()
        finally:
            conn.close()
        return [
            {
                "id": f"{row['kind']}:{row['ref_id']}",
                "kind": row["kind"],
                "title": row["title"],
                "location": row["location"],
                "start": row["start_dt"],
                "end": row["end_dt"],
                "all_day": row["all_day"],
                "tag": row["tag"],
                "notes": row["notes"],
                "activity_url": row["activity_url"],
                "course_name": row["course_name"],
                "exam_type": row["exam_type"],
                "color": row["color"],
            }
            for row in rows
        ]

    # ------------------------------------------------------------------
    # Personal events CRUD
    # ------------------------------------------------------------------

    class PersonalEventIn(BaseModel):
        model_config = ConfigDict(extra="forbid")  # smuggle guard (e.g. "kind")

        title: str = Field(min_length=1, max_length=500)
        notes: str | None = None
        all_day: int = Field(default=0, ge=0, le=1)
        start_datetime: str
        end_datetime: str
        color: str | None = None

    class PersonalEventPatch(BaseModel):
        model_config = ConfigDict(extra="forbid")

        title: str | None = Field(default=None, min_length=1, max_length=500)
        notes: str | None = None
        all_day: int | None = Field(default=None, ge=0, le=1)
        start_datetime: str | None = None
        end_datetime: str | None = None
        color: str | None = None

    def _row_to_dict(row) -> dict:
        return {
            "id": row["id"],
            "title": row["title"],
            "notes": row["notes"],
            "all_day": row["all_day"],
            "start_datetime": row["start_datetime"],
            "end_datetime": row["end_datetime"],
            "color": row["color"],
        }

    @app.get("/api/personal_events")
    def list_personal():
        conn = get_db()
        try:
            rows = conn.execute(
                "SELECT id, title, notes, all_day, start_datetime, end_datetime, color "
                "FROM personal_events ORDER BY start_datetime"
            ).fetchall()
        finally:
            conn.close()
        return [_row_to_dict(r) for r in rows]

    @app.post("/api/personal_events", status_code=201)
    def create_personal(event: PersonalEventIn):
        if not _iso_dt(event.start_datetime) or not _iso_dt(event.end_datetime):
            raise HTTPException(422, "start_datetime/end_datetime must be ISO 8601")
        if event.end_datetime < event.start_datetime:
            raise HTTPException(422, "end_datetime must be >= start_datetime")
        now = dt.datetime.now(dt.timezone.utc).isoformat()
        conn = get_db()
        try:
            cur = conn.execute(
                """INSERT INTO personal_events
                   (title, notes, all_day, start_datetime, end_datetime, color, created_at, updated_at)
                   VALUES (?,?,?,?,?,?,?,?)""",
                (event.title, event.notes, event.all_day, event.start_datetime,
                 event.end_datetime, event.color, now, now),
            )
            conn.commit()
            row = conn.execute(
                "SELECT id, title, notes, all_day, start_datetime, end_datetime, color "
                "FROM personal_events WHERE id = ?",
                (cur.lastrowid,),
            ).fetchone()
        finally:
            conn.close()
        return _row_to_dict(row)

    @app.patch("/api/personal_events/{event_id}")
    def patch_personal(event_id: int, event: PersonalEventPatch):
        if event.start_datetime and not _iso_dt(event.start_datetime):
            raise HTTPException(422, "start_datetime must be ISO 8601")
        if event.end_datetime and not _iso_dt(event.end_datetime):
            raise HTTPException(422, "end_datetime must be ISO 8601")
        fields = event.model_dump(exclude_unset=True)
        if not fields:
            raise HTTPException(422, "no fields to update")
        conn = get_db()
        try:
            existing = conn.execute(
                "SELECT * FROM personal_events WHERE id = ?", (event_id,)
            ).fetchone()
            if existing is None:
                raise HTTPException(404, "personal event not found")
            merged = {**dict(existing), **fields}
            if merged["end_datetime"] < merged["start_datetime"]:
                raise HTTPException(422, "end_datetime must be >= start_datetime")
            merged["updated_at"] = dt.datetime.now(dt.timezone.utc).isoformat()
            conn.execute(
                """UPDATE personal_events
                   SET title=?, notes=?, all_day=?, start_datetime=?, end_datetime=?,
                       color=?, updated_at=?
                   WHERE id=?""",
                (merged["title"], merged["notes"], merged["all_day"],
                 merged["start_datetime"], merged["end_datetime"], merged["color"],
                 merged["updated_at"], event_id),
            )
            conn.commit()
            row = conn.execute(
                "SELECT id, title, notes, all_day, start_datetime, end_datetime, color "
                "FROM personal_events WHERE id = ?",
                (event_id,),
            ).fetchone()
        finally:
            conn.close()
        return _row_to_dict(row)

    @app.delete("/api/personal_events/{event_id}", status_code=204)
    def delete_personal(event_id: int):
        conn = get_db()
        try:
            cur = conn.execute("DELETE FROM personal_events WHERE id = ?", (event_id,))
            conn.commit()
            if cur.rowcount == 0:
                raise HTTPException(404, "personal event not found")
        finally:
            conn.close()
        return None

    # ------------------------------------------------------------------
    # Trigger crawl → ingest in the background
    # ------------------------------------------------------------------

    @app.post("/api/crawl", status_code=202)
    def crawl():
        global _crawl_running
        with _crawl_lock:
            if _crawl_running:
                raise HTTPException(409, "a crawl is already running")
            _crawl_running = True

        def _run():
            global _crawl_running
            try:
                run_hour.run_hourly_sync()
            except SystemExit:
                # run_hour exits non-zero on failure; the sync_runs row records it
                logger.warning("Background crawl finished with a non-zero exit")
            except Exception:
                logger.exception("Background crawl failed")
            finally:
                with _crawl_lock:
                    _crawl_running = False

        threading.Thread(target=_run, daemon=True, name="tdtu-crawl").start()
        return {"status": "accepted"}

    # ------------------------------------------------------------------
    # Static frontend (API routes win because they were registered first)
    # ------------------------------------------------------------------

    if STATIC_DIR.is_dir():
        app.mount("/", StaticFiles(directory=STATIC_DIR, html=True), name="frontend")
    else:
        @app.get("/")
        def frontend_missing():
            raise HTTPException(404, "frontend not built")

    return app


def _iso_dt(value: str) -> bool:
    try:
        dt.datetime.fromisoformat(value.replace("Z", "+00:00"))
        return True
    except ValueError:
        return False


app = create_app()
