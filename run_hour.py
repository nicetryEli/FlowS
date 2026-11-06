"""
run_hour.py – Hourly data collection orchestrator.

This script runs the data-collection pipeline:
    1. Crawl the TDTU portal for the latest timetable, exams, and eLearning deadlines.
    2. Ingest the results into the local SQLite store (Step 3/4 – not implemented yet).

Can be scheduled to run hourly via cron, GitHub Actions, or similar.
"""

import logging
import os
import sys
import time
sys.stdout.reconfigure(encoding='utf-8')
# -----------------------------------------------------------------------
# Logging
# -----------------------------------------------------------------------
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(levelname)s] %(name)s – %(message)s",
    datefmt="%Y-%m-%d %H:%M:%S",
    stream=sys.stdout,
)
logger = logging.getLogger(__name__)


def _load_dotenv() -> None:
    """Load .env file into os.environ when running locally."""
    try:
        from dotenv import load_dotenv  # type: ignore[import]
        loaded = load_dotenv()
        if loaded:
            logger.info(".env file loaded successfully.")
    except ImportError:
        pass


def _resolve_crawler_weeks_ahead() -> int:
    """Read multi-week crawl horizon from env with safe fallback."""
    raw = (os.environ.get("CRAWLER_WEEKS_AHEAD") or "2").strip()
    try:
        weeks = int(raw)
    except ValueError:
        logger.warning("Invalid CRAWLER_WEEKS_AHEAD=%r; using 2.", raw)
        return 2
    if weeks < 0:
        return 0
    return min(weeks, 12)


def _handle_error(context: str, exc: Exception) -> None:
    """Log the error and exit non-zero so the scheduler marks the run failed."""
    logger.exception("%s: %s", context, exc)
    sys.exit(1)


def _log_step_elapsed(step_name: str, started_at: float) -> None:
    """Log how long a step took in seconds."""
    elapsed = time.perf_counter() - started_at
    logger.info("%s finished in %.2fs", step_name, elapsed)


def collect() -> dict:
    """Crawl all sources and return the raw rows.

    ``None`` means that source could not be collected; an empty list means it
    was collected successfully and genuinely has no rows.
    """
    student_id = os.environ.get("STUDENT_ID")

    # -------- Pre-check: log active semester --------
    try:
        from crawler import get_current_semester

        semester = get_current_semester(student_id=student_id)
        logger.info("Active semester on portal: %s", semester)
    except Exception as exc:
        logger.warning("Could not determine active semester: %s", exc)

    from crawler import (
        fetch_elearning_deadlines,
        fetch_exam_schedule,
        fetch_schedule,
    )
    weeks_ahead = _resolve_crawler_weeks_ahead()
    logger.debug("Crawler will fetch current week + %d future week(s).", weeks_ahead)

    schedule = None
    try:
        schedule = fetch_schedule(weeks_ahead=weeks_ahead)
        logger.info("Schedule crawler returned %d row(s).", len(schedule))
    except Exception:
        logger.exception("Schedule crawl failed; continuing without schedule data.")

    exams = None
    try:
        exams = fetch_exam_schedule(weeks_ahead=weeks_ahead)
        logger.info("Exam crawler returned %d row(s).", len(exams))
    except Exception:
        logger.exception("Exam crawl failed; continuing without exam data.")

    elearning_deadlines = None
    try:
        elearning_deadlines = fetch_elearning_deadlines()
        logger.info("eLearning deadline crawler returned %d row(s).", len(elearning_deadlines))
    except Exception:
        logger.exception("eLearning deadline crawl failed; continuing without a deadline update.")

    return {"schedule": schedule, "exams": exams, "deadlines": elearning_deadlines}


def run_hourly_sync() -> None:
    """Execute the crawler and ingest its results."""
    _load_dotenv()

    step_started = time.perf_counter()
    logger.info("Step 1: Crawling schedule from TDTU portal")
    try:
        data = collect()
    except Exception as exc:
        logger.exception("Step 1 failed after %.2fs", time.perf_counter() - step_started)
        _handle_error("Crawler failed", exc)
        return
    _log_step_elapsed("Step 1", step_started)

    step_started = time.perf_counter()
    logger.info("Step 2: Ingesting crawl results into SQLite")
    logger.warning(
        "SQLite ingest is not implemented yet (Steps 3–4). "
        "Crawl results collected but not stored: schedule=%s, exams=%s, deadlines=%s.",
        "FAILED" if data["schedule"] is None else len(data["schedule"]),
        "FAILED" if data["exams"] is None else len(data["exams"]),
        "FAILED" if data["deadlines"] is None else len(data["deadlines"]),
    )
    _log_step_elapsed("Step 2", step_started)

    logger.info("=== Hourly data collection complete. ===")


if __name__ == "__main__":
    run_hourly_sync()
