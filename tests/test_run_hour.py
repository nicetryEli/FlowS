import unittest
from unittest.mock import patch

import crawler
import run_hour


class RunHourlySyncTests(unittest.TestCase):
    def test_full_crawl_collects_all_sources(self) -> None:
        with (
            patch.object(run_hour, "_load_dotenv"),
            patch.object(crawler, "get_current_semester", return_value="HK2/2025-2026"),
            patch.object(crawler, "fetch_schedule", return_value=[{"id": "class-1"}]),
            patch.object(crawler, "fetch_exam_schedule", return_value=[{"id": "exam-1"}]),
            patch.object(crawler, "fetch_elearning_deadlines", return_value=[{"activity_name": "Report"}]),
        ):
            data = run_hour.collect()

        self.assertEqual(data, {
            "schedule": [{"id": "class-1"}],
            "exams": [{"id": "exam-1"}],
            "deadlines": [{"activity_name": "Report"}],
        })

    def test_failed_schedule_crawl_yields_none_and_keeps_others(self) -> None:
        with (
            patch.object(run_hour, "_load_dotenv"),
            patch.object(crawler, "get_current_semester", return_value="HK2/2025-2026"),
            patch.object(crawler, "fetch_schedule", side_effect=RuntimeError("portal timeout")),
            patch.object(crawler, "fetch_exam_schedule", return_value=[{"id": "exam-1"}]),
            patch.object(crawler, "fetch_elearning_deadlines", return_value=[]),
        ):
            data = run_hour.collect()

        self.assertIsNone(data["schedule"])
        self.assertEqual(data["exams"], [{"id": "exam-1"}])
        self.assertEqual(data["deadlines"], [])

    def test_failed_deadline_crawl_yields_none_and_keeps_others(self) -> None:
        with (
            patch.object(run_hour, "_load_dotenv"),
            patch.object(crawler, "get_current_semester", return_value="HK2/2025-2026"),
            patch.object(crawler, "fetch_schedule", return_value=[]),
            patch.object(crawler, "fetch_exam_schedule", return_value=[]),
            patch.object(crawler, "fetch_elearning_deadlines", side_effect=RuntimeError("eLearning unavailable")),
        ):
            data = run_hour.collect()

        self.assertEqual(data["schedule"], [])
        self.assertEqual(data["exams"], [])
        self.assertIsNone(data["deadlines"])

    def test_run_hourly_sync_completes_without_google(self) -> None:
        with (
            patch.object(run_hour, "_load_dotenv"),
            patch.object(crawler, "get_current_semester", return_value="HK2/2025-2026"),
            patch.object(crawler, "fetch_schedule", return_value=[]),
            patch.object(crawler, "fetch_exam_schedule", return_value=[]),
            patch.object(crawler, "fetch_elearning_deadlines", return_value=[]),
        ):
            run_hour.run_hourly_sync()  # must not raise


if __name__ == "__main__":
    unittest.main()
