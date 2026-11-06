import os
import tempfile
import unittest

from fastapi.testclient import TestClient

from db import get_db, migrate
from ingest import ingest
from server import create_app


def sample_schedule():
    return [
        {"student_id": "52400210", "subject_name": "Công nghệ phần mềm",
         "room": "D0205-B", "day_of_week": "Thursday", "session_date": "2026-11-05",
         "start_period": 7, "end_period": 9, "status": "scheduled"},
    ]


def personal_payload(**overrides):
    base = {
        "title": "Gym · legs",
        "notes": "bring towel",
        "all_day": 0,
        "start_datetime": "2026-11-24T07:40:00+07:00",
        "end_datetime": "2026-11-24T08:40:00+07:00",
        "color": "green",
    }
    base.update(overrides)
    return base


class ServerTests(unittest.TestCase):
    def setUp(self) -> None:
        self._tmp = tempfile.TemporaryDirectory()
        self.db_path = os.path.join(self._tmp.name, "app.db")
        self._old = os.environ.get("LOCAL_DB_PATH")
        os.environ["LOCAL_DB_PATH"] = self.db_path
        self.client = TestClient(create_app())

    def tearDown(self) -> None:
        if self._old is None:
            os.environ.pop("LOCAL_DB_PATH", None)
        else:
            os.environ["LOCAL_DB_PATH"] = self._old
        self._tmp.cleanup()

    # --- contract from the plan -------------------------------------------

    def test_health(self) -> None:
        r = self.client.get("/api/health")
        self.assertEqual(r.status_code, 200)
        self.assertEqual(r.json(), {"status": "ok"})

    def test_events_returns_unified_feed_with_class_times(self) -> None:
        ingest({"schedule": sample_schedule(), "exams": None, "deadlines": None}, self.db_path)
        r = self.client.get("/api/events?from=2026-11-01&to=2026-11-30")
        self.assertEqual(r.status_code, 200)
        feed = r.json()
        self.assertEqual(len(feed), 1)
        self.assertEqual(feed[0]["kind"], "class")
        self.assertEqual(feed[0]["start"], "2026-11-05T12:45")
        self.assertEqual(feed[0]["end"], "2026-11-05T15:15")
        self.assertTrue(feed[0]["id"].startswith("class:"))
        self.assertEqual(feed[0]["all_day"], 0)

    def test_events_filters_by_range(self) -> None:
        ingest({"schedule": sample_schedule(), "exams": None, "deadlines": None}, self.db_path)
        in_range = self.client.get("/api/events?from=2026-11-01&to=2026-11-30").json()
        out_range = self.client.get("/api/events?from=2026-12-01&to=2026-12-31").json()
        self.assertEqual(len(in_range), 1)
        self.assertEqual(len(out_range), 0)

    def test_personal_crud_roundtrip(self) -> None:
        created = self.client.post("/api/personal_events", json=personal_payload())
        self.assertEqual(created.status_code, 201)
        body = created.json()
        pid = body["id"]
        listed = self.client.get("/api/personal_events").json()
        self.assertEqual([e["id"] for e in listed], [pid])

        patched = self.client.patch(f"/api/personal_events/{pid}", json={"color": "blue"})
        self.assertEqual(patched.status_code, 200)
        self.assertEqual(patched.json()["color"], "blue")

        deleted = self.client.delete(f"/api/personal_events/{pid}")
        self.assertEqual(deleted.status_code, 204)
        self.assertEqual(self.client.get("/api/personal_events").json(), [])

    def test_post_rejects_extra_kind_field(self) -> None:
        payload = personal_payload()
        payload["kind"] = "exam"          # smuggle attempt
        r = self.client.post("/api/personal_events", json=payload)
        self.assertEqual(r.status_code, 422)

    def test_patch_unknown_id_404(self) -> None:
        r = self.client.patch("/api/personal_events/9999", json={"title": "nope"})
        self.assertEqual(r.status_code, 404)

    def test_delete_only_personal(self) -> None:
        ingest({"schedule": sample_schedule(), "exams": None, "deadlines": None}, self.db_path)
        # crawled rows live in class_sessions, not personal_events — attempting
        # to delete "class:1" as a personal id must 404 and leave the class intact.
        r = self.client.delete("/api/personal_events/1")
        self.assertEqual(r.status_code, 404)
        feed = self.client.get("/api/events?from=2026-11-01&to=2026-11-30").json()
        self.assertEqual(len(feed), 1)

    def test_crawl_endpoint_returns_202(self) -> None:
        r = self.client.post("/api/crawl")
        self.assertIn(r.status_code, (202, 409))

    @unittest.skipUnless(
        __import__("pathlib").Path("frontend/build/web").is_dir(),
        "frontend/build/web not built (CI runs backend tests without the web build)",
    )
    def test_static_mount_serves_index_when_built(self) -> None:
        r = self.client.get("/")
        self.assertEqual(r.status_code, 200)
        self.assertIn("html", r.headers.get("content-type", "").lower())

    def test_sync_runs_returns_runs(self) -> None:
        ingest({"schedule": sample_schedule(), "exams": None, "deadlines": None}, self.db_path)
        r = self.client.get("/api/sync_runs")
        self.assertEqual(r.status_code, 200)
        self.assertGreaterEqual(len(r.json()), 1)


if __name__ == "__main__":
    unittest.main()
