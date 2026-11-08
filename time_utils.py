"""Utilities for timezone-aware current date/time handling."""

from __future__ import annotations

import datetime as dt
import os
from functools import lru_cache
from zoneinfo import ZoneInfo, ZoneInfoNotFoundError

DEFAULT_TIMEZONE = "Asia/Ho_Chi_Minh"


@lru_cache(maxsize=1)
def _resolve_timezone() -> ZoneInfo:
    timezone_name = os.environ.get("APP_TIMEZONE", DEFAULT_TIMEZONE).strip() or DEFAULT_TIMEZONE
    try:
        return ZoneInfo(timezone_name)
    except ZoneInfoNotFoundError:
        return ZoneInfo(DEFAULT_TIMEZONE)


def local_now() -> dt.datetime:
    """Return current datetime in application timezone."""
    return dt.datetime.now(_resolve_timezone())


def local_today() -> dt.date:
    """Return current local date in application timezone."""
    return local_now().date()


# ---------------------------------------------------------------------------
# TDTU period → clock time mapping
# (moved from the removed Google Calendar sync module in Step 2)
# ---------------------------------------------------------------------------

# Official TDTU period start times from the provided timetable image.
PERIOD_START: dict[int, str] = {
    1: "06:50",
    2: "07:40",
    3: "08:30",
    4: "09:30",
    5: "10:20",
    6: "11:10",
    7: "12:45",
    8: "13:35",
    9: "14:25",
    10: "15:25",
    11: "16:15",
    12: "17:05",
    13: "18:05",
    14: "18:55",
    15: "19:45",
}


def fallback_period_time(period: int) -> str:
    """Approximate a period start time for periods outside ``PERIOD_START``."""
    baseline = dt.datetime.combine(dt.date.today(), dt.time(hour=7, minute=0))
    computed = baseline + dt.timedelta(minutes=max(period - 1, 0) * 50)
    return computed.strftime("%H:%M")


def period_start_time(period: object) -> str:
    """Resolve the start time for a TDTU period number (fallback-aware)."""
    try:
        number = int(period)  # type: ignore[arg-type]
    except (TypeError, ValueError):
        number = 1
    return PERIOD_START.get(number, fallback_period_time(number))
