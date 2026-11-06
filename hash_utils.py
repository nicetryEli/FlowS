"""Stable hashing helpers shared by crawler ingest and storage layers.

Moved from the removed Google Calendar sync module in Step 2. Used to detect
whether a crawled row has changed since it was last stored (``source_hash``).
"""

from __future__ import annotations

import hashlib
import json


def sync_hash(payload: dict) -> str:
    encoded = json.dumps(payload, sort_keys=True, ensure_ascii=False, separators=(",", ":"))
    return hashlib.sha256(encoded.encode("utf-8")).hexdigest()


def normalize_value(value: object) -> str:
    return str(value or "").strip().lower()
