# Dataset structure — crawler output (Step 3)

Captured 2026-11-05 by running `scripts/dump_dataset.py` against the live TDTU
portal + eLearning. Real rows: **13 schedule sessions, 1 exam, 0 deadlines**
(the eLearning account currently has no incomplete deadlines — the deadline
schema below is taken from the crawler code, not from a sample).

> `data/sample_dataset.json` is gitignored (personal data). Do not commit it.

## Top level

In memory (`run_hour.collect()`) and in the dump, the dataset is one object
with three keys:

| Key         | Type              | Meaning                                  |
|-------------|-------------------|------------------------------------------|
| `schedule`  | `list[Session] \| None` | Class sessions (timetable)              |
| `exams`     | `list[Exam] \| None`    | Exam sessions                           |
| `deadlines` | `list[Deadline] \| None`| eLearning deadlines (incomplete only)   |

`None` = that source **failed to crawl** (partial outage); `[]` = crawled OK,
genuinely empty. The ingest layer (Steps 3–4) must treat these differently and
never wipe previously stored rows for a failed source.

## `schedule` — Session rows

| Key            | Type  | Notes / example                                             |
|----------------|-------|-------------------------------------------------------------|
| `student_id`   | str   | Always the same student for this personal dashboard (`"52400210"`) |
| `subject_name` | str   | e.g. `"Công nghệ phần mềm"`                                 |
| `room`         | str   | e.g. `"B403"`, `"HOCTRUCTUYEN-5"`, `"D0205-B"`               |
| `day_of_week`  | str   | **English** weekday name, e.g. `"Tuesday"`                  |
| `session_date` | str   | ISO date `YYYY-MM-DD`, e.g. `"2026-11-03"`                  |
| `start_period` | int   | TDTU period number 1–15 (see `PERIOD_START` in `time_utils.py`) |
| `end_period`   | int   | Same numbering; inclusive                                   |
| `status`       | str   | `scheduled` (thường), `absent` (báo vắng), `makeup` (học bù), `cancelled`, `moved` |

Example (real):

```json
{
  "student_id": "52400210",
  "subject_name": "Công nghệ phần mềm",
  "room": "D0205-B",
  "day_of_week": "Thursday",
  "session_date": "2026-11-05",
  "start_period": 7,
  "end_period": 9,
  "status": "scheduled"
}
```

Field semantics inferred from portal UI:
- `absent` = student reported absent ("báo vắng") for that session.
- `makeup` = rescheduled/makeup session ("học bù").
- `day_of_week` is **redundant** with `session_date` (derivable via weekday()) —
  kept by the portal; the DB may store it or drop it (decision for Step 4).
- Start/end **clock times are not stored** — they are derived from periods via
  `PERIOD_START` (50-min standard periods) + 50-minute overrun for the end.

## `exams` — Exam rows

| Key            | Type            | Notes / example                                  |
|----------------|-----------------|--------------------------------------------------|
| `subject_name` | str             | e.g. `"Công nghệ phần mềm"`                      |
| `exam_date`    | str             | ISO date `YYYY-MM-DD`, e.g. `"2026-09-30"`       |
| `start_time`   | str             | `"HH:MM"` (24h, zero-padded) or `""` if unknown  |
| `end_time`     | str             | `"HH:MM"` or `""` (sample has empty; duration unknown) |
| `exam_room`    | str             | e.g. `"A507"`                                    |
| `exam_type`    | str \| None     | Raw label as shown, e.g. `"Giữa kỳ"`, `"Cuối kỳ"` (note: **Vietnamese**, unlike `TARGET_EXAM_TYPES=midterm,final` config keys) |
| `notes`        | str \| None     | e.g. `"Crawled from exam grid"`                  |

Example (real):

```json
{
  "subject_name": "Công nghệ phần mềm",
  "exam_date": "2026-09-30",
  "start_time": "07:30",
  "end_time": "",
  "exam_room": "A507",
  "exam_type": "Giữa kỳ",
  "notes": "Crawled from exam grid"
}
```

No `student_id` and no stable id — identity must be derived (hash).

## `deadlines` — Deadline rows

From the crawler code (`_parse_elearning_dashboard_deadlines`,
`_parse_elearning_deadlines_on_course`); account has none right now.

| Key                | Type | Notes / example                                        |
|--------------------|------|--------------------------------------------------------|
| `course_id`        | str  | Moodle course id, e.g. `"12345"`; may fall back to a sha256-derived pseudo-id or be empty |
| `course_name`      | str  | Cleaned course label, e.g. `"Công nghệ phần mềm"`      |
| `activity_name`    | str  | e.g. `"Bài tập 3 – Chương 5"`                         |
| `activity_url`     | str  | Absolute URL to the Moodle activity (`/mod/...`) — best stable key |
| `due_date`         | str  | ISO datetime with tz, e.g. `"2026-11-12T23:59:00+07:00"` |
| `completion_status`| str  | Always `"incomplete"` (crawler only collects incomplete) |

## Cross-entity observations (input for Step 4 ERD)

1. **Three independent entity types** share no foreign keys in the raw dataset.
   They are joined only by the student (single-user personal tool).
2. **Subjects repeat across entities** — the same course appears as class
   sessions *and* as exams *and* as eLearning deadlines (e.g. "Công nghệ phần
   mềm"). A `courses` table can normalize this, but names are not guaranteed to
   match exactly between portal and eLearning (cleaning may differ), so treat
   `course` linkage as **fuzzy/optional**, not a hard FK, unless we verify.
3. **Derived values** to persist vs. compute:
   - Session start/end clock times ← periods (`PERIOD_START`) — compute on read.
   - Session `weekday` ← `session_date` — redundant, store or drop.
   - Exam end time when empty + reminder offsets — UI concern.
4. **Identity / upsert keys** (from removed-Google-sync logic, reused):
   - Session: `(subject_name, session_date, start_period, end_period, status?)`
     or hash of canonical payload.
   - Exam: `(subject_name, exam_date, start_time, exam_type)`.
   - Deadline: `activity_url` is best; else hash of `(course_id, activity_name, due_date)`.
   Each stored row should carry a `source_key` + `source_hash` for idempotent
   re-crawls and change detection (hash_utils.py).
5. **All-day vs timed**: exams may arrive with empty times (all-day); deadlines
   are instants; sessions are timed via periods.
6. **Personal events** (added later in the UI) are a *fourth* entity kind with
   no crawler source — the schema must leave room for `source = 'personal'`.
7. `student_id` is constant in practice — either a `settings`/`meta` table or
   dropped entirely (decision for Step 4).
8. Statuses/labels: `status` (session) and `exam_type` (raw Vietnamese label)
   need either CHECK constraints with the full enum or a loose TEXT + docs.

## Proposed implications for the ERD (Step 4)

- Entities: `courses?`, `class_sessions`, `exams`, `deadlines`,
  `personal_events`, plus optional `sync_meta`/`settings`.
- Relationships: all events belong to the (single) user; optional link from
  sessions/exams/deadlines to `courses`.
- Common columns everywhere: `id PK`, `source` (`portal|elearning|personal`),
  `source_key UNIQUE`, `source_hash`, `created_at`, `updated_at`,
  `last_seen_at`.
- Date/time: store ISO strings or SQLite `TEXT` in UTC + separate local
  display, or keep `+07:00` strings — decide in Step 4.
