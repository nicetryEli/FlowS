# Step 5a — SQLite + FastAPI Backend Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn the current mocked Flutter calendar into the real app: crawled TDTU/eLearning data is stored idempotently in a local SQLite database, served by a local FastAPI backend, and the Flutter frontend reads/writes through that API.

**Architecture:** SQLite is the single source of truth (schema from `docs/erd.md` v1.0). Crawled rows are upserted via stable `source_key` + `source_hash` (failure of a source preserves its existing rows; a successful crawl reconciles its table to portal truth). A single SQL view `calendar_events` unifies the four event kinds for the frontend. FastAPI (stdlib `sqlite3`, no ORM) serves `/api/events` and personal-event CRUD plus static hosting of the built Flutter app. The Flutter repository becomes an interface with two implementations: `MockEventsRepository` (tests) and `ApiEventsRepository` (runtime).

**Tech Stack:** Python 3.14 (venv at `.venv/`), stdlib `sqlite3` (WAL mode), FastAPI + uvicorn, pytest, Flutter 3.47.9 / Dart 3.13.7, package `http: ^1.2.0` for the Flutter API client.

**Spec:** `docs/erd.md` (v1.0, FINAL — entity design, DDL, `calendar_events` view contract, idempotency rules) and `docs/dataset-structure.md` (exact crawler output shapes; the plan maps from these, not from guesswork).

## Global Constraints

- **SQLite only** — no external DB server, no ORM (`sqlite3` stdlib).
- **Single user** — no `users` table; `settings(key, value)` K/V only.
- **Time/timezone** — `session_date` = plain `YYYY-MM-DD`; `deadlines.due_date` keeps `+07:00` offset; `exams` store raw `HH:MM` or `NULL` (empty string from crawler → `NULL`); session clock times are **never stored** (derived from periods).
- **Idempotent ingest** — `source_key` UNIQUE per crawled table; `source_hash` = `hash_utils.sync_hash()` (sha256 of canonical JSON). Successful crawl reconciles its kind only: insert new → update changed → hard-delete vanished. Failed crawl (`None` from `collect()`) leaves that table **untouched** and `sync_runs` records `partial`.
- **Personal events are never crawler-managed.** Crawled kinds can never be created/edited/deleted via the personal-events API.
- **`calendar_events` is the only read contract** the frontend uses; kind tables stay free to evolve.
- Portal = truth: rows vanishing from a successful crawl are deleted.
- No Google/ntfy/OAuth anywhere.
- `.env` never committed; DB file (`data/app.db*`) and `data/` are gitignored.
- Currency: every step that adds behavior starts with a failing test (TDD), ends with a commit.

## Review Focus

Five failure modes most likely to bite, pinned to the tasks that own them:

1. **Partial crawl silently wipes data** — crawler returns `None` for eLearning but `[]` for schedule. → Task 2 test: `ingest({...})` with one `None` leaves that table untouched and records `partial` in `sync_runs`.
2. **Garbage datetime operands crash whole ingest** — deadline `due_date="every friday :)"` or exam `start_time=""` or `end_period < start_period`. → Task 2 tests: rows with invalid datetimes/CHECK violations are skipped with a warning; valid sibling rows still upsert; run reports `partial`, never throws on parse failure.
3. **View returns wrong times for EXAM with NULL start** — all-day exam must surface as `all_day=1` with `start_dt = exam_dateT00:00`, never `""` or crash. → Task 1 test: seeded all-day exam row → `calendar_events` returns `all_day = 1`, and `00:00` start.
4. **Personal API can deface crawled rows** — `PATCH /api/personal_events/1` targeting a class row, or a body with `kind:"exam"` smuggled in. → Task 4 tests: unknown id → 404; `kind` in body is rejected (422); crawled rows are invisible to personal endpoints.
5. **Duplicate deadlines on re-crawl** — two activities share `activity_url` after cleaning, or hash collision fallback keys differ. → Task 2 test: same `activity_url` twice in one batch → exactly one row, second upsert updates `last_seen_at` only; hash-stable across runs (no churn) — second ingest of identical data performs **zero writes** (asserted via `sync_runs.detail` counters and row `updated_at` unchanged).

## File Structure

| Path | Responsibility |
|------|----------------|
| `db/migrations/001_initial.sql` | DDL: reference/meta tables, `courses`, `class_sessions`, `exams`, `deadlines`, `personal_events`, `sync_runs`, `schema_migrations`, all indexes, `calendar_events` view, `ref_periods` seed (from `time_utils.PERIOD_START` values) |
| `db.py` | Connection factory (WAL, `foreign_keys=ON`), `migrate()` (applies `db/migrations/*.sql` in order, records in `schema_migrations`), `get_db()` |
| `ingest.py` | `compute_source_key(row, kind)`, `ingest(data: dict) -> dict` (counts + status), per-kind reconcile inside a single transaction; links `courses` on confident name match |
| `server.py` | FastAPI app: `/api/health`, `/api/events`, `/api/personal_events` CRUD, `/api/sync_runs`, `/api/crawl` (background thread), static mount of `frontend/build/web` at `/` (only when built) |
| `run_hour.py` | Modify: Step 2 calls `ingest.ingest(collect())` and records outcome via `sync_runs` |
| `requirements.txt` | Add `fastapi`, `uvicorn[standard]` |
| `frontend/lib/data/events_repository.dart` | Abstract `CalendarEventsRepository` interface |
| `frontend/lib/data/mock_events.dart` | Implements the interface (unchanged behavior) |
| `frontend/lib/data/api_events_repository.dart` | HTTP impl (`package:http`), base URL from `--dart-define=API_BASE` (default `''` same-origin) |
| `frontend/lib/main.dart` | Repo selection: `MockEventsRepository` when `USE_MOCK=true`, else `ApiEventsRepository` |
| `tests/test_db.py`, `tests/test_ingest.py`, `tests/test_server.py` | pytest suites (tmp SQLite in `tmp_path`) |
| `scripts/smoke_e2e.py` | Offline end-to-end: migrate → ingest `data/sample_dataset.json` if present → start uvicorn → assert `/api/events` shape |

---

### Task 1: Database layer — schema + migrations

**Files:**
- Create: `db/migrations/001_initial.sql`
- Create: `db.py`
- Test: `tests/test_db.py`

**Interfaces:**
- Consumes: `time_utils.PERIOD_START` (period → `"HH:MM"`), `hash_utils` (not needed here)
- Produces: `db.migrate(db_path: str | Path) -> int` (applies pending migrations, returns count applied); `db.get_db(db_path: str | Path | None) -> sqlite3.Connection` (WAL, `row_factory=sqlite3.Row`, `foreign_keys=ON`); `db.DEFAULT_DB_PATH` = `os.environ.get("LOCAL_DB_PATH", "data/app.db")`

- [ ] **Step 1: Write the failing tests** (`tests/test_db.py`)

```python
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
```

- [ ] **Step 2: Run to verify failure** — `.venv/bin/pytest tests/test_db.py -v` → FAIL (`No module named 'db'`)

- [ ] **Step 3: Implement** — `db/migrations/001_initial.sql` (paste the v1.0 DDL from `docs/erd.md` verbatim, then adjust):
  - add `PRAGMA journal_mode = WAL;` and `PRAGMA foreign_keys = ON;` at top
  - add `CREATE TABLE ref_periods (period INTEGER PRIMARY KEY, start_time TEXT NOT NULL);` and seed rows 1–15 with the exact values from `time_utils.PERIOD_START` (1:"06:50",…,15:"19:45")
  - `class_sessions` stores NO clock times; view joins `ref_periods`:
    `start_dt = session_date || 'T' || rp1.start_time`, `end_dt = datetime(session_date || 'T' || rp2.start_time, '+50 minutes')` formatted `strftime('%Y-%m-%dT%H:%M', ...)`
  - exams: `all_day = (start_time IS NULL)`, `start_dt = exam_date || 'T' || COALESCE(start_time,'00:00')`, `end_dt = exam_date || 'T' || COALESCE(end_time, start_time, '00:00')`
  - deadlines: instant row (`start_dt = due_date`, `end_dt = due_date`, `all_day = 0`, `tag = 'deadline'`)
  - personal: `kind='personal'`, `all_day` passthrough, `tag = color`
  - indexes exactly as in spec (`idx_class_sessions_date`, `idx_exams_date`, `idx_deadlines_due`, `idx_personal_events_start`)
  - `schema_migrations(version INTEGER PRIMARY KEY, name TEXT NOT NULL, applied_at TEXT NOT NULL)`

  `db.py`: `migrate()` reads `db/migrations/*.sql` sorted by filename, applies unapplied files inside one transaction each, inserts into `schema_migrations`; `get_db()` creates parent dir of db_path, sets pragmas + `row_factory`.

- [ ] **Step 4: Verify tests pass** — `.venv/bin/pytest tests/test_db.py -v` → PASS (4 tests)

- [ ] **Step 5: Commit** — `git add db/ db.py tests/test_db.py && git commit -m "feat(db): SQLite schema + migration runner (calendar_events view)"`

---

### Task 2: Idempotent ingest

**Files:**
- Create: `ingest.py`
- Test: `tests/test_ingest.py`

**Interfaces:**
- Consumes: `db.get_db`, `db.migrate`, `hash_utils.sync_hash`, `hash_utils.normalize_value`
- Produces: `ingest.ingest(data: dict, db_path: str | Path | None = None) -> dict` returning `{"status": "ok"|"partial"|"failed", "inserted": int, "updated": int, "deleted": int, "skipped": int}`; `ingest.source_key_for(kind: str, row: dict) -> str`; `ingest.source_hash_for(row: dict) -> str`

- [ ] **Step 1: Write the failing tests** covering every Review Focus line owned by this task:

```python
# success path: schedule inserted, re-ingest is a no-op
# changed content -> UPDATE in place (same id), vanished row -> hard delete
# failed source (schedule=None) -> table untouched, status=partial
# garbage rows skipped: bad due_date, end_period < start_period, "" start_time exam coerced to NULL;
#   siblings still insert; skipped counter increments
# duplicate activity_url in one batch -> one row only
# two identical ingests -> second reports inserted=updated=deleted=0 and row.updated_at unchanged
# course link: class row subject 'Công nghệ phần mềm' links courses.id after upsert; mismatched
#   names do NOT crash and leave course_id NULL
```

Concretely, minimum test list (one function each, `tmp_path` DB):
`test_ingest_inserts_new_rows`, `test_reingest_is_noop`, `test_changed_row_updates_in_place`, `test_vanished_row_is_deleted`, `test_failed_source_leaves_table_untouched`, `test_invalid_deadline_is_skipped_and_reported`, `test_all_day_exam_inserted_with_null_times`, `test_duplicate_activity_url_collapses`, `test_course_link_best_effort`, `test_sync_runs_records_partial`.

- [ ] **Step 2: Run to verify failure** — `.venv/bin/pytest tests/test_ingest.py -v` → FAIL

- [ ] **Step 3: Implement `ingest.py`** — per kind, inside ONE transaction:

```python
# pseudocode contract (signatures fixed)
def ingest(data: dict, db_path=None) -> dict:
    """data = {"schedule": list[dict]|None, "exams": ...|None, "deadlines": ...|None}"""
    ...
def source_key_for(kind, row):
    # class: f"class:{session date}|{subject}|{start}-{end}|{status}"
    # exam:  f"exam:{exam_date}|{subject}|{start_time or 'allday'}|{exam_type or ''}"
    # deadline: activity_url if present else "deadline:" + sync_hash({course_id, activity_name, due_date})
```

  Rules: `None` kind ⇒ skip silently except flag `partial=True`. Empty list ⇒ reconcile to zero (all vanished rows deleted) — this is the "portal truth" path, not a failure. Hash mismatch ⇒ UPDATE. Do not touch `personal_events`, ever. `sync_runs` insert: `status='ok'|'partial'|'failed'`, `started_at`/`finished_at` (UTC ISO), `detail` = JSON of counters. Courses linking: normalized cleaned name equality vs existing `courses.name` only; else leave NULL (never create duplicates of near-identical names).

- [ ] **Step 4: Verify tests pass** — `.venv/bin/pytest tests/test_ingest.py -v` → PASS

- [ ] **Step 5: Commit** — `git add ingest.py tests/test_ingest.py && git commit -m "feat(ingest): idempotent crawler → SQLite upsert with source_key/hash"`

---

### Task 3: Wire `run_hour.py` Step 2 to the database

**Files:**
- Modify: `run_hour.py` (Step 2 body only)
- Modify: `.gitignore`, `.env.example`
- Test: `tests/test_run_hour.py` (extend)

**Interfaces:**
- Consumes: `ingest.ingest`, `db.get_db`
- Produces: `run_hour.run_hourly_sync()` still callable; Step 2 logs `"Ingested: X inserted, Y updated, Z deleted (status=…)"` and exits non-zero only on total failure

- [ ] **Step 1: Extend the failing tests** in `tests/test_run_hour.py`: full-crawl mock now also asserts `sync_runs` row exists with `status='ok'`; deadline-crawl-failure mock → `status='partial'`, `class_sessions` table still holds pre-seeded row (seeded via direct SQL in the test); DB path overridden per-test via `LOCAL_DB_PATH` env var + `get_db()`.

- [ ] **Step 2: Run to verify failure** — `.venv/bin/pytest tests/test_run_hour.py -v` → FAIL

- [ ] **Step 3: Implement** — replace the "Step 2 TODO" block in `run_hour.py` with: `report = ingest.ingest(data)`; `logger.info` the counters; `sys.exit(1)` only when `data` == `{'schedule': None, 'exams': None, 'deadlines': None}`. Add to `.gitignore`: `data/app.db*`, `data/backups/`. Add to `.env.example`: `LOCAL_DB_PATH=data/app.db` and comment "SQLite file; back it up by copying it".

- [ ] **Step 4: Verify** — `.venv/bin/pytest tests/test_run_hour.py -v` → PASS, plus `LOCAL_DB_PATH=/tmp/smoke.db .venv/bin/python run_hour.py` prints the new ingest line (crawls network — may fail offline; log must still show partial/failed, not a traceback).

- [ ] **Step 5: Commit** — `git add run_hour.py tests/test_run_hour.py .gitignore .env.example && git commit -m "feat(run): wire hourly crawl into SQLite ingest"`

---

### Task 4: FastAPI server (events read + personal CRUD + static hosting)

**Files:**
- Create: `server.py`
- Modify: `requirements.txt` (add `fastapi>=0.115`, `uvicorn[standard]>=0.30`)
- Test: `tests/test_server.py`

**Interfaces:**
- Consumes: `db.get_db`, `db.migrate` (server migrates on startup)
- Produces: HTTP contract the Flutter app depends on —
  - `GET /api/health` → `{"status":"ok"}`
  - `GET /api/events?from=YYYY-MM-DD&to=YYYY-MM-DD` → `[{"id":"<kind>:<ref_id>","kind":"class|exam|deadline|personal","title":str,"location":str|null,"start":ISO-local,"end":ISO-local,"all_day":0|1,"tag":str}]`
  - `GET /api/personal_events` → rows (id int, title, notes, all_day, start_datetime, end_datetime, color)
  - `POST /api/personal_events` body `{title*, notes, all_day, start_datetime*, end_datetime*, color}` → 201 + created row; **422 if body contains `kind`** (smuggle guard)
  - `PATCH /api/personal_events/{id}` partial → 200 updated row; 404 otherwise
  - `DELETE /api/personal_events/{id}` → 204; 404 otherwise; **never deletes crawled rows** (personal table only)
  - `GET /api/sync_runs` → last 10 runs
  - `POST /api/crawl` → 202 `{"status":"accepted"}` fires background thread; without crawler creds, thread logs and records `failed` run
  - `GET /` → serves `frontend/build/web/index.html` when built (mount), else `{"detail":"frontend not built"}` 404 — API routes take precedence

- [ ] **Step 1: Write failing tests** (`tests/test_server.py`, `TestClient`, tmp `LOCAL_DB_PATH`): `health`, `events_returns_unified_feed_with_class_times` (seed via ingest of one class row; assert start/end), `events_filters_by_range`, `personal_crud_roundtrip`, `post_rejects_extra_kind_field`, `patch_unknown_id_404`, `delete_only_personal`, `crawl_endpoint_returns_202`, `static_mount_serves_index_when_built`.

- [ ] **Step 2: Verify failure** — `.venv/bin/pytest tests/test_server.py -v` → FAIL

- [ ] **Step 3: Implement `server.py`** — module-level `create_app()`; DB path from `LOCAL_DB_PATH`; `migrate` at import/startup; routes per contract; static mount guarded by `Path("frontend/build/web").is_dir()`; background crawl uses `threading.Thread(target=run_hour.run_hourly_sync, daemon=True)` guarded by a simple lock + `sync_runs` row; CORS not needed (same origin).

- [ ] **Step 4: Verify** — `.venv/bin/pytest tests/test_server.py -v` → PASS

- [ ] **Step 5: Commit** — `git add server.py requirements.txt tests/test_server.py && git commit -m "feat(api): FastAPI /api/events + personal CRUD + static hosting"`

---

### Task 5: Flutter frontend switches to the API

**Files:**
- Modify: `frontend/pubspec.yaml` (add `http: ^1.2.0`)
- Create: `frontend/lib/data/events_repository.dart` (abstract interface)
- Modify: `frontend/lib/data/mock_events.dart` (implement interface)
- Create: `frontend/lib/data/api_events_repository.dart` (HTTP impl)
- Modify: `frontend/lib/views/month_view.dart`, `week_view.dart`, `year_view.dart`, `widgets/next_up_panel.dart`, `widgets/event_detail_dialog.dart`, `widgets/add_event_dialog.dart`, `screens/calendar_screen.dart` — depend on the abstract type
- Modify: `frontend/lib/main.dart` — repo select via `const bool.fromEnvironment('USE_MOCK', defaultValue: false)`; API base = `String.fromEnvironment('API_BASE', defaultValue: '')`
- Test: `frontend/test/widget_test.dart` (unchanged — uses mock via direct injection if repo is injected; otherwise keep `USE_MOCK=true` for tests)

**Interfaces:**
- Consumes: Task 4 HTTP contract (`/api/events`, `/api/personal_events`)
- Produces: `abstract class CalendarEventsRepository` with `List<CalendarEvent> get events`, `List<CalendarEvent> forRange(DateTime from, DateTime to)`, `void add(CalendarEvent)`, `void remove(String id)`, plus `ChangeNotifier` mixin (widgets listen via `ListenableBuilder` — no change needed)

- [ ] **Step 1: Refactor** — extract interface from `MockEventsRepository` verbatim; widget files just swap the type.

- [ ] **Step 2: Implement `ApiEventsRepository`** — `forRange` = `GET {base}/api/events?from&to`; maps JSON → `CalendarEvent` (`kind` → enum, `start`/`end` naive ISO → `DateTime.parse`, `all_day` → bool, exam/deadline metadata passthrough, `color` for personal); `add` = `POST /api/personal_events` then insert returned row locally; `remove(id)` = `DELETE /api/personal_events/{id}` with `kind:personal` prefix stripped; failure → `SnackBar` via thrown `ApiException` surfaced in UI by repository callers (widgets already handle errors minimally — show a SnackBar, no silent swallow).

- [ ] **Step 3: Verify** — `flutter analyze` clean; `flutter test` passes (mock path); manual smoke: `LOCAL_DB_PATH=/tmp/app.db .venv/bin/uvicorn server:app --port 8735` + `flutter build web --dart-define=API_BASE=http://localhost:8735` + browser check: month view loads real rows from DB; add-event persists after reload (`crud roundtrip visually`).

- [ ] **Step 4: Commit** — `git add frontend/ && git commit -m "feat(web): API-backed repository replaces mock (USE_MOCK to revert)"`

---

### Task 6: End-to-end offline smoke + docs

**Files:**
- Create: `scripts/smoke_e2e.py`
- Modify: `README.md` (short "Run it" section: migrate → run hourly once → start server → open http://localhost:8000)
- Modify: `CONTEXT.md` (Step 5a complete), `docs/erd.md` §"To be implemented" → mark view implemented

**Interfaces:**
- Consumes: Tasks 1–5
- Produces: single command `python scripts/smoke_e2e.py` that exits 0 and prints `SMOKE OK (events=N)`; CI-friendly

- [ ] **Step 1: Implement** — script: fresh `LOCAL_DB_PATH=$(mktemp -d)/app.db`; call `migrate()`; if `data/sample_dataset.json` exists → `ingest.ingest(json.load(...))` else ingest a hand-made two-row payload (one class + one deadline); launch uvicorn in-thread on port 0 (ephemeral) or subprocess on 8741; `GET /api/events?from=2026-11-01&to=2026-12-01`; assert JSON list, assert one row has `kind=personal` after `POST`; cleanup; print `SMOKE OK`.

- [ ] **Step 2: Verify** — `.venv/bin/python scripts/smoke_e2e.py` → `SMOKE OK (events=3)`

- [ ] **Step 3: Docs** — README "Run the app" block (≤15 lines), CONTEXT status 5a ✅, ERD view note.

- [ ] **Step 4: Commit** — `git commit -m "chore: e2e smoke script + docs for the local stack"`

---

## Self-review checklist (ran)

- Spec coverage: ERD §ddl → Task 1; §idempotent ingest + §deletion semantics → Task 2; §calendar_events view → Task 1 (view) + Task 4 (API on top); §extendedProperties-style tagging → view `tag` column consumed by UI (existing widgets) — no change; §extension story → explicitly out of scope (no Tasks/Documents now). dataset-structure.md mapping → Tasks 1–2. ✅
- Review Focus: lines 1–5 each map to named tests in Task 2 (lines 1, 2, 5), Task 1 (line 3), Task 4 (line 4). ✅
- Type consistency: `ingest() -> dict` consumed in Tasks 3, 4, 6; `get_db()` everywhere; endpoint JSON field names match Flutter mapping in Task 5 (`id`, `kind`, `start`, `end`, `all_day`). `source_key` format fixed in Task 2 interface; Task 3 only logs counters. ✅
- Proportion: 6 tasks, code paths are thin by design (stdlib sqlite3, no ORM). ✅
- Out of scope: CI reintroduction, reminders/notifications, recurring events, Tasks/Documents tabs, auth.
