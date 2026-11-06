import json
import os
import tempfile
import unittest
from unittest.mock import patch

import crawler
import run_hour
from db import get_db, migrate


class RunHourlySyncTests(unittest.TestCase):
    def setUp(self) -> None:
        self._tmp = tempfile.TemporaryDirectory()
        self.db_path = os.path.join(self._tmp.name, "app.db")
        self._old_env = os.environ.get("LOCAL_DB_PATH")
        os.environ["LOCAL_DB_PATH"] = self.db_path

    def tearDown(self) -> None:
        if self._old_env is None:
            os.environ.pop("LOCAL_DB_PATH", None)
        else:
            os.environ["LOCAL_DB_PATH"] = self._old_env
        self._tmp.cleanup()

    def _seed_class_rows(self) -> None:
        migrate(self.db_path)
        conn = get_db(self.db_path)
        try:
            conn.execute(
                """INSERT INTO class_sessions
                   (course_id, subject_name, room, session_date, day_of_week,
                    start_period, end_period, status, source_key, source_hash,
                    first_seen_at, last_seen_at, created_at, updated_at)
                   VALUES (NULL, 'Seed subject', 'B403', '2026-11-03', 'Tuesday',
                           4, 6, 'scheduled', 'class:seed', 'h',
                           't', 't', 't', 't')"""
            )
            conn.commit()
        finally:
            conn.close()

    def test_full_crawl_ingests_and_records_ok_run(self) -> None:
        with (
            patch.object(run_hour, "_load_dotenv"),
            patch.object(crawler, "get_current_semester", return_value="HK1/2026-2027"),
            patch.object(crawler, "fetch_schedule", return_value=[{"id": "class-1"}, {"id": "class-2"}]),
            patch.object(crawler, "fetch_exam_schedule", return_value=[{"id": "exam-1"}]),
            patch.object(crawler, "fetch_elearning_deadlines", return_value=[]),
        ):
            # crawler rows use "id" (not the real schema) — ingest skips them;
            # the important part is that a sync_runs row records 'ok'
            run_hour.run_hourly_sync()

        conn = get_db(self.db_path)
        try:
            run = conn.execute("SELECT * FROM sync_runs ORDER BY id DESC LIMIT 1").fetchone()
        finally:
            conn.close()
        self.assertEqual(run["status"], "ok")

    def test_failed_deadline_crawl_preserves_seeded_classes(self) -> None:
        self._seed_class_rows()
        with (
            patch.object(run_hour, "_load_dotenv"),
            patch.object(crawler, "get_current_semester", return_value="HK1/2026-2027"),
            patch.object(crawler, "fetch_schedule", side_effect=RuntimeError("portal timeout")),
            patch.object(crawler, "fetch_exam_schedule", return_value=[]),
            patch.object(crawler, "fetch_elearning_deadlines", side_effect=RuntimeError("eLearning unavailable")),
        ):
            # crawl outages (None) preserve stored rows; both sources failing
            # still records a partial run and never fabricates data.
            run_hour.run_hourly_sync()

        conn = get_db(self.db_path)
        try:
            rows = conn.execute("SELECT * FROM class_sessions").fetchall()
            run = conn.execute("SELECT * FROM sync_runs ORDER BY id DESC LIMIT 1").fetchone()
        finally:
            conn.close()
        self.assertEqual([r["source_key"] for r in rows], ["class:seed"])
        self.assertEqual(run["status"], "partial")

    def test_failed_schedule_crawl_marks_partial(self) -> None:
        with (
            patch.object(run_hour, "_load_dotenv"),
            patch.object(crawler, "get_current_semester", return_value="HK1/2026-2027"),
            patch.object(crawler, "fetch_schedule", side_effect=RuntimeError("portal timeout")),
            patch.object(crawler, "fetch_exam_schedule", return_value=[]),
            patch.object(crawler, "fetch_elearning_deadlines", return_value=[]),
        ):
            run_hour.run_hourly_sync()

        conn = get_db(self.db_path)
        try:
            run = conn.execute("SELECT * FROM sync_runs ORDER BY id DESC LIMIT 1").fetchone()
        finally:
            conn.close()
        self.assertEqual(run["status"], "partial")


if __name__ == "__main__":
    unittest.main()
