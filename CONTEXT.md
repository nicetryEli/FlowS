# CONTEXT.md — TDTU Local Calendar Dashboard

> **Purpose of this file:** my working memory for this project. It is updated
> every time something important happens. Read it first before any task.

## What we are building

A **local personal calendar dashboard** for a TDTU student that:

1. Crawls the student's **class schedule** (TDTU portal), **exam schedule**, and
   **eLearning deadlines** (elearning.tdtu.edu.vn).
2. Stores everything in a **SQLite database** on disk — the *only* source of truth.
3. Serves the data via a small **FastAPI** local server.
4. Renders it in a dark-mode **Flutter Web** calendar UI where the user can also
   **add personal events**, persisted to the same SQLite DB.

Google Calendar and ntfy notifications are **fully removed**.

## Agreed architecture

```text
crawler.py (Playwright, TDTU portal + eLearning)
   └─ raw rows (dicts / JSON-shaped)  ──►  Step 3: identify exact JSON structure
        └─ SQLite DB (tables per Step 4 ERD)  ← THE ONLY SOURCE OF TRUTH
             └─ FastAPI (REST API, read/write, serves Flutter Web build)
                  └─ Flutter Web dashboard (dark slate-teal calendar UI)
```

- **No Google anything** (service account, OAuth, Calendar API, Tasks API, ntfy).
- **No full database server** — SQLite file only.
- Web = **Flutter Web first** (not desktop/mobile yet, not necessarily final).
- Personal events are created in the UI and saved via FastAPI → SQLite.
- Backups/migration = copy the SQLite file.

## Step plan (user-approved order)

- **Step 1:** ✅ Reuse the existing calendar-checking tool.
- **Step 2:** ✅ Remove all components related to Google.
- **Step 3:** ✅ Identify the JSON structure returned by the dataset.
- **Step 4:** ✅ Create the ERD based on the objects found in that JSON.
- **Step 5:** Build the visually appealing Flutter Web calendar frontend aligned with the ERD.

Progress markers live at the bottom of this file and MUST be kept current.

## UI spec (user-provided mockup description)

General: dark mode, modern calendar dashboard; subtle greenish-teal/grey palette;
clean grid layout; soft rounded corners.

1. **Top nav bar**
   - Left tabs (muted grey/white text): Activity, Tasks, **Scheduled** (active,
     underlined light grey), Analytics, Documents. (Only Scheduled/calendar is in
     scope now; other tabs are future work.)
   - Right controls: view options — e.g. WEEK / MONTH (highlighted dark pill
     button) / YEAR; month/year selector dropdown ("February 2016", dropdown arrow
     far right).
2. **Calendar grid sub-header**: days of week in small uppercase muted text
   SUN…SAT.
3. **Monthly view grid**: 7×N grid of days.
   - Previous/next month days greyed with **diagonal stripe pattern**.
   - Active "Today" day: blue badge "Today", highlighted light blue-teal block.
   - **Event badges**: e.g. "10:30AM New task to c…"; multi-item days show a list
     with **colored accent indicators** (green, orange, cyan, purple, magenta).
   - **Multi-day span banner** across a row (e.g. "Vacation" banner, emoji ok).
4. **Palette**
   - Background: dark slate teal / charcoal grey (`#1e292b`-ish).
   - Grid cells: semi-transparent darker blocks, subtle spacing.
   - Accents: electric cyan/blue (Today), pastel green, orange, purple, magenta
     (tags / event indicators).

## Scope decisions (from brainstorming)

- **Calendar only, first.** Extra tabs/features come later — do NOT build them now.
- **SQLite from the start**, not a bare JSON file. JSON is only the crawler's
  in-memory/intermediate shape; it is identified (Step 3), modelled (Step 4),
  and persisted in SQLite.
- **ntfy notifications: DROPPED.** (daily_notify.py goes away with Google stuff.)
- **JSON file storage: REJECTED** in favour of SQLite as sole source of truth.
- Flutter Web first; native/desktop later if desired.
- Backend: **FastAPI** (user's choice). Everything Python, matching existing code.
- Personal events: title/date/time/color-ish minimum; exact fields TBD in Steps 3–5.

## Existing codebase state (as found 2026-11-05)

Python project `tool-check-calender` (folder name has this spelling — leave it).

| File | Role | Fate after refactor |
|------|------|---------------------|
| `crawler.py` | Playwright scraper: TDTU schedule, exams, eLearning deadlines | **KEEP & REUSE** (Step 1), feed SQLite instead of Google |
| `calendar_sync.py` | Google Calendar sync + reconciliation (source_hash/source_key dedup) | **REMOVE / GUT** (Step 2); hashing/reconciliation ideas may be reused for DB upserts |
| `run_hour.py` | Hourly orchestrator: crawl → sync to Google | Keep skeleton, replace Step-2 Google sync with DB ingest |
| `daily_notify.py` | ntfy morning notification (Google Tasks + Calendar) | **REMOVE** (ntfy dropped) |
| `auth_google_tasks.py` | Google Tasks OAuth helper | **REMOVE** |
| `time_utils.py` | `local_today()` etc. | KEEP |
| `tests/` | pytest suite | Update to match refactor |
| `requirements.txt` | playwright, requests, google-* libs, dotenv, pytest | Drop google-*/ntfy deps; add fastapi, uvicorn, sqlite (stdlib) |
| `.github/workflows/daily_notify.yml`, `hourly_sync.yml` | CI cron | Rework or remove Google/ntfy bits |
| `.env` | STUDENT_ID=52400210, PASSWORD=(user's, gitignored) + crawl defaults | KEEP (gitignored ✓) |
| `.env.example` | Documented vars incl. Google/ntfy | Clean to match new reality |
| `README.md` | Docs (Vietnamese), Google-centric | Rewrite for new architecture |

## Key crawler output shapes (to be fully mapped in Step 3)

`fetch_schedule()` → list of class-session dicts. Known keys seen in code:
`id?, subject_name, room, session_date (ISO), start_time, end_time,
start_period/end_period (ints, mapped via PERIOD_START 06:50…19:45), status
(scheduled|absent|makeup|cancelled|moved), day_of_week`.

`fetch_exam_schedule()` → list of exam dicts. Keys seen: `id?, subject_name,
exam_date, start_time, end_time, exam_room, exam_type (midterm|final…), notes`.

`fetch_elearning_deadlines()` → list of deadline dicts. Keys seen:
`course_id?, course_name, activity_name, activity_url, due_date (ISO datetime),
source_signature?`.

`calendar_sync.py` derives: `source_key` (e.g. `class_session:{id}` or
`exam:{date}:{start}:{end}:{subject}:{type}`), `source_hash` (sha256 of canonical
JSON). These concepts map naturally to SQLite UNIQUE constraints / upsert keys.

Exams/deadlines also support **all-day events** (date-only payloads).

## Working agreements

- Update THIS file after every important milestone (step completion, decision,
  schema change, notable failure).
- Never commit `.env` or any secret. `.env` is gitignored (verified).
- Keep the repo runnable: `pytest -q` must pass at each step boundary.
- Step-by-step; don't jump ahead of the user's approved order.
- User's credentials used only for local crawling; never printed/logged by me.

## Step 1 — Reuse plan (DONE 2026-11-05)

Baseline verified in fresh `.venv` (Python 3.14.7): `pytest -q` → **33 passed,
16 skipped**; `py_compile` clean. Git working tree was clean except CONTEXT.md.

**Kept and reused as-is (no Google dependency, safe to build on):**

- `crawler.py` — the core asset. Public entry points for the new pipeline:
  - `fetch_schedule(weeks_ahead=…)` → class-session dict rows
  - `fetch_exam_schedule(weeks_ahead=…)` → exam dict rows
  - `fetch_elearning_deadlines()` → deadline dict rows
  - `get_current_semester(student_id=…)` (optional, for logging)
  - `_deduplicate_*_rows()` helpers (already dedup crawler output)
  - `PERIOD_START` period→"HH:MM" map lives in `calendar_sync.py` — when Google
    code is gutted in Step 2, this constant must be **moved, not lost** (into a
    shared timing module or the new ingest layer).
- `time_utils.py` — `local_now()`, `local_today()` (Asia/Ho_Chi_Minh).
- `tests/test_crawler_parsers.py` — parser tests; must stay green.
- `run_hour.py` — orchestrator skeleton reused; Step 2 replaces Google sync
  with SQLite ingest.

**Pattern to reuse (not the code):** `source_key` (stable identity, e.g.
`class_session:{id}`) and `source_hash` (sha256 of canonical JSON) from
`calendar_sync.py` → becomes SQLite upsert key / change detection in Step 3–4.

**NOT reused / removed in Step 2:** `calendar_sync.py` (kept only as reference
for hashing until Step 3), `daily_notify.py`, `auth_google_tasks.py`,
Google workflows and env vars, ntfy.

## Step 2 — Google/ntfy removal (DONE 2026-11-05)

- **Deleted:** `calendar_sync.py`, `daily_notify.py`, `auth_google_tasks.py`,
  `contact_example.txt` (Google attendee auto-add), `.github/workflows/hourly_sync.yml`
  and `daily_notify.yml` (CI removed intentionally; re-add with DB ingest later),
  Google-era tests (`test_calendar_sync.py`, `test_daily_notify.py`).
- **Moved, not lost** (survive in new modules):
  - `PERIOD_START` + `period_start_time()` / `fallback_period_time()` → `time_utils.py`
  - `sync_hash()` / `normalize_value()` → new `hash_utils.py` (sha256 of canonical
    JSON; becomes SQLite change-detection in Steps 3–4)
- **Rewritten:** `run_hour.py` (crawl-only orchestrator; `collect()` returns
  `{schedule, exams, deadlines}` with `None` = failed source, `[]` = OK-but-empty;
  ingest marked as Step 3–4 TODO), `tests/test_run_hour.py` (Google mocks replaced),
  `requirements.txt` (google-*, requests dropped; playwright + dotenv + pytest),
  `.env.example`, `README.md` (new local-dashboard setup, Vietnamese kept).
- `.env` needed no change (never contained Google vars); confirmed gitignored.
- Requires re-check: fresh `.venv` — Google packages uninstalled; `pytest -q`
  → **8 passed, 16 skipped** (skips are pre-existing Playwright-browser-unavailable
  parser tests, unrelated); `py_compile` clean.
- No Google/ntfy references remain except README notes and code comments.
- Workflow YAMLs were deleted rather than neutered — CI comes back with the
  SQLite ingest step instead of "crawl and throw away".

## Step 3 — Identify JSON structure (DONE 2026-11-05)

- Live crawl succeeded via `.venv` (chromium installed): **13 schedule rows,
  1 exam, 0 deadlines**; full sample at `data/sample_dataset.json`
  (gitignored — personal data).
- Dump tool: `scripts/dump_dataset.py`; output gitignored, UTF-8.
- Full schema documented in **`docs/dataset-structure.md`**: top-level
  `{schedule, exams, deadlines}` (`None` = failed source, `[]` = empty);
  per-entity keys/types/examples; semantics (absent/makeup, PERIOD_START
  derivation, English weekday names, Vietnamese exam_type labels, ISO dates,
  TZ-aware deadlines).
- Findings for Step 4 ERD: no FKs between crawl entities; subjects repeat
  across entities → optional `courses` link, fuzzy match; exams may lack
  times (all-day); `source_key`/`source_hash` needed for idempotent upserts;
  schema must admit `source='personal'` events.
- Deadline schema documented from code (account has no current deadlines).

## Step 4 — ERD v1.0 FINAL (2026-11-05, post-review polish)

**`docs/erd.md`** — complete, self-contained spec. Entities: `settings`
(K/V, single user + feature-flag bus), `courses` (soft normalizer),
`class_sessions`, `exams`, `deadlines`, `personal_events` (reserved
`recurrence_rule`), `sync_runs`, plus **`schema_migrations`** (every future
change = one numbered migration; backup to `data/backups/` before applying).
WAL mode, foreign_keys ON. Contains: mermaid ERD, `calendar_events`
unified read view contract, full dataset→table traceability, and an
**"Extending the schema"** table mapping every likely future feature
(Tasks / Activity / Documents / Analytics / Reminders / Recurrence / Tags /
Multi-user-cost) to additive-only changes — v1 tables never reshaped.
Owner decisions recorded (hard delete on portal-vanish; all-day exams;
day_of_week debug-only).

**`docs/erd.md`** — Mermaid ERD + SQLite DDL sketch. Entities: `settings`
(K/V, single user), `courses` (soft normalizer, fuzzy/optional links),
`class_sessions`, `exams`, `deadlines`, `personal_events`, `sync_runs`.
Decisions: separate tables per kind, FastAPI `UNION ALL`s a normalized
calendar feed for the UI; session clock times never stored (derived from
periods); personal events carry `all_day` + offset ISO datetimes (multi-day
banner); idempotent ingest via `source_key UNIQUE` + `source_hash` (failed
source ⇒ table untouched); portal = truth for crawled rows (vanished ⇒
deleted); personal events never crawler-owned. Non-blocking open questions
at the bottom of erd.md (hard-delete vs stale-flag, future recurrence).

## Current status / progress log

- [x] Step 5a plan written: docs/superpowers/plans/2026-11-06-sqlite-fastapi-backend.md
      (6 tasks, TDD, review-focus tests; scope: SQLite+ingest+FastAPI+API repo swap;
      CI/reminders/recurrences/out-of-scope explicitly excluded)
- [x] Color spec v2 applied (owner-provided jpg pair): canvas = radial
      gradient #242f31 (center-glow) → #1a2224 (corners); tiles = #2d393b
      (~90% opacity, soft teal); out-month tiles = #222b2e + black@25%
      hatch; Today = FILLED #7195a0, ALL text white, white TODAY label
      (no cyan pill); bright cyan #35c5e8 REMOVED from code, active states
      = white / slate #141a1b only; Scheduled tab = white TOP indicator;
      view toggle active = #141a1b pill + white text;
      accent bars: class/CNTT #03a9f4, personal green #4caf50,
      makeup/orange #ff9800, exam+deadline #e91e63; Add-event/CTA buttons
      = slate+white; screenshots: flutter-month/week/year.png
- [x] Out-month tile spec v3 + motion (owner image ref): tiles = #2d393b
      base + 45° hatch in dark blue @35% (#0d1f2b), 1px subtle dark border
      outline, number muted top-right (already); canvas corner shifted to
      darker slate blue #16242c; MINIMAL ANIMATIONS added — view switch
      fade+slide (AnimatedSwitcher keyed by mode+anchor), dialogs
      scale+fade (showGeneralDialog, easeOutCubic 230ms), rail/view-toggle/
      tab-indicator use AnimatedContainer transitions
- [x] Stripe clip fix (owner: "strip line out of day block"): out-month
      hatch is now ClipRRect-clipped to the tile's rounded corners
- [x] YEAR view brought back (owner): 12 mini-month grid (views/year_view.dart)
      with per-day event dots (exam>deadline>class>personal priority),
      cyan TODAY dot; ‹ › navigates years; tap a mini-month header → jumps
      to that month in month view; TopBar has WEEK/MONTH/YEAR again
- [x] Week view: ALL-DAY BANNER ROW DELETED (owner) — spanning/all-day
      events (Vacation) now only appear in the month view

- [x] Recon of repo (structure, README, requirements, main modules)
- [x] Created `.env` with user-provided STUDENT_ID/PASSWORD + crawl defaults
      (verified gitignored)
- [x] Brainstorming: architecture agreed → SQLite-only, FastAPI, Flutter Web,
      Google + ntfy dropped, calendar-only scope first
- [x] Created this CONTEXT.md
- [x] Step 1 — Reuse existing calendar-checking tool: baseline green
      (33 passed / 16 skipped), venv created, reuse plan recorded above
- [x] Step 2 — Remove Google components: all Google/ntfy code, tests, CI, deps
      deleted; reusable bits (PERIOD_START, sync_hash) moved to shared modules;
      pytest 8 passed / 16 skipped; py_compile clean
- [x] Step 3 — Identify JSON structure returned by the dataset: live crawl
      (13 sessions / 1 exam / 0 deadlines), schema in docs/dataset-structure.md,
      sample gitignored
- [x] Step 4 — ERD v1.0 FINAL (user requested post-calendar growth
      readiness): schema_migrations + WAL, unified view, extension-points
      table for Tasks/Documents/etc., owner decisions recorded;
      `recurrence_rule` reserved on personal_events
- [x] Prototype DARK = chosen design (owner: "i like the rectangle one
      style, make it simple and minimum too"): flat & rectangular — ~7px
      radius everywhere (cells 7px, chips 4px), no gradients/shadows/glows/
      blur (flat tonal layers, flat green Vacation banner #2c4a3f w/ muted
      text, flat logo/avatar), no hover lift, out-month days flat tonal,
      keeps rail + Next-up panel + month nav + detail modal + footer strip;
      out-month diagonal stripes KEPT per owner (not minimal-stripped)
- [x] Step 5b — Flutter Web frontend BUILT & VERIFIED (owner: frontend
      first, backend later): Flutter 3.47.9 SDK → ~/flutter; app at
      frontend/ (pubspec, theme, models, mock repository, screens,
      month + week views, rail/topbar/next-up/footer, detail modal,
      add-event dialog); `flutter analyze` clean, `flutter test` passes
      (1600x900 desktop surface), `flutter build web` OK; served locally
      on http://localhost:8734 (python http.server on frontend/build/web);
      screenshots: prototype/flutter-month.png, prototype/flutter-week.png
      (stripes on out-month cells, TODAY badge, all-day Vacation banner,
      week view with lanes/now-line, footer strip); data is MOCKED
      (MockEventsRepository) until the FastAPI backend lands; commit
      pending owner decision
      - 5a FastAPI+SQLite backend deferred; frontend ships first per owner
        ("you just have to done the front end")
      - Flutter SDK 3.47.9 (Dart 3.13.7) installing to ~/flutter
      - Scaffold at frontend/: pubspec (url_launcher only), flat dark theme
        (theme.dart), models/calendar_event.dart (mirrors calendar_events
        view), data/tdtu_periods.dart (PERIOD_START mirror), data/
        mock_events.dart (MockEventsRepository — real crawled rows + demo
        extras; ChangeNotifier; listens for add/delete), screens/
        calendar_screen.dart (rail | topbar+view | NextUpPanel, footer),
        views/month_view.dart (striped out-month cells, TODAY chip,
        spanning banners, +N more dialog), views/week_view.dart (all-day
        banner row, hourly grid 06:00-24:00, lanes for overlapping events,
        deadlines as 30-min blocks, live now-line), widgets/* (top_bar,
        app_rail, next_up_panel, event_detail_dialog, add_event_dialog,
        footer_strip, event_style); YEAR view removed per owner;
        repository interface ready for API swap-in after 5a
      - 5a: FastAPI + SQLite (schema.sql, WAL, migrations, idempotent ingest,
            period_start_time UDF, calendar_events view, REST API)
      - 5b: Flutter Web dark calendar (per prototype/index.html)
- [x] Prototype v3/v4 (owner supplied 2nd mockup + example.png: flat light
      full-bleed, minimal like Google Calendar): off-white #e9ebed bg,
      white flat cells, SMALL radius (6px cells / 4px chips), day numbers
      top-right big, left slot for Today (orange) / Active! (blue) labels,
      full-width pastel chips (time left · title centered), grey flat
      out-month cells, "Scheduled" active grey pill, gold-outlined MONTH,
      no rail/sidebar — calendar is full width; pink "Dubai Business
      Conference" 2-day span, green Vacation banner full-row; month nav
      (‹ › + TODAY) + detail modal kept; fits 100vh; footer strip added
      to match example.png (Privacy · Community · Help · Cookie
      preferences · Do not sell…, dead placeholder links — Google's footer
      text as visual reference only); THROWAWAY PROTOTYPE ribbon removed;
      example.png is the target look; screenshots: prototype/light-preview.png,
      light-modal.png.
      DARK variant (prototype/index.html) kept for comparison.
      OPEN QUESTION: which theme ships — dark / light / toggle?
- [x] Prototype v2/v1 (dark, mockup-faithful): grid JS-rendered from a data
      model mirroring the calendar_events view; ‹ › prev/next month buttons
      + TODAY jump + Nov 2026 default; clicking any chip/banner/aside card
      opens a detail modal (kind chip, date/time/room/status/notes;
      deadline gets "Open in eLearning →" link; Edit/Delete are placeholder
      alerts in prototype); Oct/Nov/Dec 2026 sample data; screenshots:
      prototype/preview-month.png, prototype/preview-modal.png
