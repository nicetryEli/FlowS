# FlowS Backend Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Establish a local-first FlowS backend foundation in Rust/Tauri with SQLite, delivering task management first and then calendar/sync backend foundations for TDTU data.

**Architecture:** Implement a layered Rust backend inside a Tauri 2 app (`commands/` for IPC boundary, `services/` for domain/sync logic, `database/` for migrations and queries, `platform/` for OS-specific integrations later) backed by a single local SQLite database. Persist durable task/calendar data in SQLite and derive presentation-oriented/daily-reset state from stored timestamps and task fields; integrate TDTU crawling as an in-app scheduled sync (every 5 hours, 4-week horizon) without any GitHub/background-cloud dependency.

**Tech Stack:** Tauri 2, Rust, SQLite, Vue 3 + TypeScript frontend shell (minimal for backend wiring), Pinia/vue-router (planned), migration tooling TBD (`sqlx` or `rusqlite`), OS keyring for credentials (planned), TDTU crawl integration informed by `tool-check-calender/` (Python/Playwright reference).

**Spec:** `CONTEXT.md`, `plan.md`, `SRS - Ứng dụng Quản lý Công việc Cá nhân (FlowS).md` (with conflicts resolved by confirmed decisions in `CONTEXT.md`), `tool-check-calender/` (TDTU crawl reference; do not expose `.env` secrets).

## Global Constraints

- Local-first: no backend server, no GitHub background jobs; SQLite is the sole local persistent data store.
- Default gamification concept is Fish Tank (Lake of Task); post-v1 multi-theme support via theme pack folders (theme pack format/spec TBD, do not overdesign now).
- Backend-first focus for this plan; defer undecided UI/theme/visual implementation details (NES.css vs Geist, rendering approach, etc.).
- Task management is implemented before calendar/sync feature completion.
- Confirmed task/lake rules: focused/dragged-down fish state persists across app opens/restarts; overdue incomplete tasks display fully black; tasks with no deadline display white; school-synced tasks appear as fish in the tank.
- Calendar must show school events, deadlines, and personal events together.
- School sync must run locally in-app only, every 5 hours, covering 4 weeks ahead (exact runtime behavior when app is closed/tray TBD).
- Crawl target is TDTU (`old-stdportal.tdtu.edu.vn`, `lichhoc-lichthi.tdtu.edu.vn`, `elearning.tdtu.edu.vn`) with credentials `STUDENT_ID` and `PASSWORD` (reference: `tool-check-calender/`).
- Store credentials securely (OS keyring preferred per project guidance); never commit or log secrets.
- Daily reset/lifecycle semantics for incomplete vs completed tasks across days are TBD except where already implied; derive from stored data (`completed_at`, dates) rather than destructive overwrites.

## Review Focus

- Input/time edge cases likely to break behavior: tasks with no deadline (must remain valid/white-state), overdue tasks (fully black state), and timezone/date-boundary handling for "today", completions, streaks, and sync windows.
- Persistence correctness: focused/dragged-down state (max 3 in focus) survives app restart and cannot corrupt task records.
- Idempotent sync/import: repeated 5-hour TDTU sync runs for overlapping 4-week windows must not create duplicate school events/tasks/deadlines (stable source identifiers/hashes required).
- Partial/failed external sync: crawler/login/network failures must not erase previously stored school data and must surface sync status for notifications/UI later.
- Data integrity across sources: personal tasks/events and school-synced records coexist cleanly in one local store with clear source provenance.

---

## Scope & Assumptions (for approval)

- **In scope now (backend planning/implementation target):** app/backend scaffolding, SQLite schema + migrations, task/tag/completion/focus persistence, Rust commands/services for task management, calendar/event data model foundations, TDTU sync orchestration foundations (scheduler, source mapping, dedupe, credential storage boundary), and testable domain/sync logic.
- **Deferred/TBD (do not block this plan):** exact daily reset lifecycle details, tray/background behavior when app closed, full UI flows, theme pack format/spec, final TDTU crawler porting strategy details (Rust-native vs WebView-assisted vs subprocess bridge) pending spike/design choice, and all undecided visual/UX details.
- **Assumption for first implementation:** prioritize a working backend core for tasks; calendar/sync can land as backend foundations and incremental integration after task core is stable. If direct Rust Playwright-equivalent crawling is costly, a temporary adapter around reference tooling may be used behind a service boundary (explicitly documented) as long as it remains local and secure.

## File Structure (target)

Planned structure (to be created during implementation):

- `frontend/` — Vue 3 + TypeScript shell for invoking backend commands (minimal wiring in backend-first phase)
- `src-tauri/` — Tauri 2 app
  - `src-tauri/src/main.rs` — app entry, plugin/setup wiring
  - `src-tauri/src/lib.rs` — module exports / command registration
  - `src-tauri/src/commands/` — Tauri command handlers (thin IPC layer)
    - `tasks.rs`, `tags.rs`, `calendar.rs`, `sync.rs`, `settings.rs` (as needed)
  - `src-tauri/src/services/` — domain and orchestration logic
    - `task_service.rs`, `tag_service.rs`, `calendar_service.rs`, `sync_service.rs`, `scheduler.rs`
  - `src-tauri/src/database/` — DB connection, migrations, repositories/queries
    - `mod.rs`, `migrations.rs` (or migration runner), query modules
  - `src-tauri/src/platform/` — OS-specific abstractions (`linux.rs` stubs as needed)
  - `src-tauri/src/models/` (optional) — shared domain structs/enums/DTOs
  - `src-tauri/migrations/` — SQL migrations
  - `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, capabilities/config
- `docs/superpowers/plans/` — implementation plans
- `docs/superpowers/specs/` — approved specs/design docs (future)
- `CONTEXT.md` — authoritative session/project context (keep updated)

Responsibility boundaries:
- Commands validate/translate IPC inputs and call services; no complex business rules.
- Services own domain rules (task lifecycle, focus limits, source mapping, sync idempotency decisions).
- Database layer owns schema/migrations and persistence queries; keep SQL/query details out of commands.
- Sync crawler/adapters are isolated behind interfaces so TDTU scraping implementation can evolve without rewriting domain logic.

---

## Phase A — Backend Scaffolding & Persistence Foundation

### Task 1: Scaffold Tauri 2 + Rust + Vue 3 workspace baseline

**Files:**
- Create/Modify: `frontend/` (generated scaffold), `src-tauri/` (generated scaffold), root config files (`package.json`, lockfile, `README.md` updates as needed)
- Test: manual build/dev verification commands (and add automated Rust tests in later tasks)

**Interfaces:**
- Consumes: project decisions in `CONTEXT.md`/`plan.md`
- Produces: runnable Tauri app skeleton with Rust backend and frontend shell ready for command wiring

- [ ] **Step 1: Generate Tauri 2 + Vue 3 + TypeScript scaffold**
  - Use the official Tauri scaffolding flow for the chosen package manager/tooling (document exact command used in commit/notes).

- [ ] **Step 2: Verify scaffold runs in dev mode**
  - Run the project’s dev command (e.g. `pnpm tauri dev` / `npm run tauri dev` depending on scaffold) and confirm window launches.

- [ ] **Step 3: Align repository structure with planned backend layering**
  - Create `src-tauri/src/commands/`, `services/`, `database/`, `platform/` modules and register them in `lib.rs`/`main.rs`.

- [ ] **Step 4: Add minimal frontend route/shell placeholders (optional but useful)**
  - Ensure app can navigate/load without implementing UI features; keep backend-first.

- [ ] **Step 5: Commit**
  - Commit scaffold and structural baseline.

### Task 2: Add SQLite persistence foundation and migration runner

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Create/Modify: `src-tauri/src/database/mod.rs`, `src-tauri/src/database/migrations.rs` (or equivalent), `src-tauri/migrations/*.sql`, `src-tauri/src/main.rs` or app setup module
- Test: Rust integration/unit tests for migration/bootstrap behavior (path TBD once test harness exists)

**Interfaces:**
- Consumes: Task 1 scaffold
- Produces: initialized local SQLite database on app startup with versioned migrations applied

- [ ] **Step 1: Choose and add SQLite/migration dependencies**
  - Select `sqlx` or `rusqlite` (+ migration support) and record rationale in plan/commit message; prefer consistency with async/runtime needs.

- [ ] **Step 2: Implement DB path/config resolution**
  - Store DB under app data/local config path appropriate for Tauri; keep path injectable for tests.

- [ ] **Step 3: Implement migration application on startup**
  - Ensure migrations run safely/idempotently and failures are surfaced clearly.

- [ ] **Step 4: Write and run verification**
  - Add a test or dev check proving a fresh DB initializes and migrations apply cleanly.

- [ ] **Step 5: Commit**

### Task 3: Define initial domain schema (tasks first, calendar/sync-ready)

**Files:**
- Create: `src-tauri/migrations/0001_initial_schema.sql` (name/numbering per chosen tool)
- Modify: database query/repository modules as needed
- Test: schema/migration tests

**Interfaces:**
- Consumes: Task 2 migration infrastructure
- Produces: core tables and indexes supporting task management now and calendar/sync dedupe/provenance later

- [ ] **Step 1: Model task-management core tables**
  - `tasks`: id, title/name, description/notes (optional), created_at, deadline_at (nullable), importance, source (`personal`/`school`/etc.), external_id/source_key (nullable), status fields needed for persistence, timestamps.
  - `tags`: id, name, color (optional), timestamps.
  - `task_tags`: many-to-many join.
  - `completions`: completion log entries linked to task and completion date/time (supports streak/heatmap/history).
  - `task_focus_state` (or equivalent): persisted focus/dragged-down state and ordering/slot metadata, enforcing max 3 focused tasks in service logic (and/or constraints where practical).

- [ ] **Step 2: Add calendar/sync foundation tables (minimal, backend-ready)**
  - `calendar_events` (or normalized equivalent) with fields for title, start/end/all-day, source, external_id/source_key, source_hash, related task link (nullable), sync metadata (`last_seen_at`, etc.).
  - `sync_runs`/`sync_state` (or equivalent) to track last/next sync, status, errors, window coverage.
  - Unique/index strategy for dedupe on `(source, external_id/source_key)` and/or hash.

- [ ] **Step 3: Encode confirmed domain rules in schema where appropriate**
  - Nullable deadline for "no deadline" tasks; allow school-sourced tasks; preserve completion history.

- [ ] **Step 4: Verify migration forward from empty DB**
  - Run migration tests/checks.

- [ ] **Step 5: Commit**

---

## Phase B — Task Management Backend (Priority 1)

### Task 4: Implement task domain models and repository/service layer

**Files:**
- Create/Modify: `src-tauri/src/models/*.rs` (or equivalent), `src-tauri/src/database/*task*.rs`, `src-tauri/src/services/task_service.rs`
- Test: Rust unit/integration tests for task CRUD and state transitions

**Interfaces:**
- Consumes: Task 3 schema
- Produces: service APIs for create/list/update/delete/complete tasks, including focus-state persistence and source-aware tasks

- [ ] **Step 1: Define core task types/enums**
  - Importance levels (extreme/high/mid/low/simple), task source, derived/explicit status fields as needed.

- [ ] **Step 2: Implement CRUD and listing queries**
  - Support filters/sorts needed by task list and dashboard queries later (by date, completion, source, tag) without building UI.

- [ ] **Step 3: Implement completion flow and completion logging**
  - Mark complete in task state as designed and append `completions` record with timestamp; keep history durable.

- [ ] **Step 4: Implement focus-state persistence with constraints**
  - Persist dragged-down/focused tasks across restarts; enforce max 3 focused tasks and safe add/remove/reorder operations.

- [ ] **Step 5: Add tests for confirmed rules and edge cases**
  - No-deadline tasks allowed; overdue detection inputs supported; school-sourced tasks valid; focus state persists.

- [ ] **Step 6: Commit**

### Task 5: Implement tag backend and task-tag relationships

**Files:**
- Create/Modify: `src-tauri/src/database/*tag*.rs`, `src-tauri/src/services/tag_service.rs`, command modules later
- Test: Rust tests for tag CRUD and task-tag linking

**Interfaces:**
- Consumes: Task 3 schema, Task 4 task model/service
- Produces: tag CRUD, multi-tag assignment/removal, tag-filter query support

- [ ] **Step 1: Implement tag CRUD persistence/service methods**
- [ ] **Step 2: Implement attach/detach/list tags for tasks**
- [ ] **Step 3: Add tests (including multi-tag and delete/rename behavior)**
- [ ] **Step 4: Commit**

### Task 6: Expose Tauri commands for task/tag management

**Files:**
- Create/Modify: `src-tauri/src/commands/tasks.rs`, `src-tauri/src/commands/tags.rs`, `src-tauri/src/lib.rs` (command registration), frontend invoke wrappers/types (minimal)
- Test: command-level tests where practical; manual invoke smoke checks from frontend/devtools

**Interfaces:**
- Consumes: Task 4/5 services
- Produces: stable IPC surface for task and tag operations (create/update/delete/list/complete/focus/tag ops)

- [ ] **Step 1: Define command DTOs/inputs/outputs**
  - Keep serialization explicit and frontend-friendly.

- [ ] **Step 2: Implement thin command handlers calling services**
  - Map service/domain errors to clear command errors.

- [ ] **Step 3: Register commands in Tauri builder**
- [ ] **Step 4: Add minimal frontend API wrappers (backend-wiring only)**
  - No UI polish required; enough to exercise backend.

- [ ] **Step 5: Verify end-to-end task CRUD/focus/complete via invoke**
- [ ] **Step 6: Commit**

### Task 7: Add dashboard/history-supporting task queries (backend side)

**Files:**
- Create/Modify: task/completion query modules and services, commands as needed
- Test: Rust tests for streak/count/heatmap/history aggregations

**Interfaces:**
- Consumes: Task 4 completion/task data
- Produces: backend queries for remaining/completed today, streak inputs, heatmap data, and monthly completion counts/history inputs

- [ ] **Step 1: Implement completion/today summary queries**
- [ ] **Step 2: Implement streak and activity aggregation queries** (derived from `completions`)
- [ ] **Step 3: Implement monthly history aggregation queries** (counts/completed items by month)
- [ ] **Step 4: Verify with seeded test data across day/month boundaries**
- [ ] **Step 5: Commit**

---

## Phase C — Calendar & TDTU Sync Backend Foundations (Priority 2)

### Task 8: Implement calendar event domain and persistence (personal + school-ready)

**Files:**
- Create/Modify: `src-tauri/src/models/*calendar*.rs`, `src-tauri/src/database/*calendar*.rs`, `src-tauri/src/services/calendar_service.rs`, `src-tauri/src/commands/calendar.rs`
- Test: Rust tests for event CRUD/listing and source provenance

**Interfaces:**
- Consumes: Task 3 schema
- Produces: backend support for storing/querying calendar events from personal, school, and deadline sources in one unified model

- [ ] **Step 1: Define unified calendar/event models and source enums**
  - Include school schedule events, exam events, deadlines, and personal events; preserve provenance and external identifiers.

- [ ] **Step 2: Implement persistence/query APIs**
  - List by date range (week/month), by source, and combined views for calendar consumption.

- [ ] **Step 3: Implement personal event CRUD commands (backend-first)**
- [ ] **Step 4: Add tests for mixed-source querying and dedupe key constraints**
- [ ] **Step 5: Commit**

### Task 9: Design and implement TDTU sync adapter boundary

**Files:**
- Create/Modify: `src-tauri/src/services/sync_service.rs`, `src-tauri/src/services/tdtu_sync_adapter.rs` (or equivalent), docs/notes as needed
- Reference (read-only): `tool-check-calender/crawler.py`, `tool-check-calender/docs/dataset-structure.md`
- Test: adapter contract tests with fixture data

**Interfaces:**
- Consumes: TDTU credentials/config and sync window parameters
- Produces: normalized schedule/exam/deadline records mapped into FlowS domain/calendar/task-linked structures with stable source keys/hashes

- [ ] **Step 1: Define normalized sync DTOs and mapping rules from TDTU reference data shapes**
  - Cover class sessions, exams, deadlines; map into calendar events and/or school-sourced tasks as required.

- [ ] **Step 2: Choose initial adapter strategy (documented)**
  - Rust-native implementation, WebView-assisted flow, or temporary local bridge to reference tooling; pick simplest secure local option for MVP backend and isolate behind trait/interface.

- [ ] **Step 3: Implement adapter skeleton and deterministic mapping/dedupe helpers**
  - Generate stable `source_key`/`source_hash` for idempotent re-syncs.

- [ ] **Step 4: Add fixture-based tests for parsing/mapping/dedupe behavior**
  - Include updates vs unchanged records and overlapping windows.

- [ ] **Step 5: Commit**

### Task 10: Implement local sync orchestration, scheduling, and status tracking

**Files:**
- Create/Modify: `src-tauri/src/services/scheduler.rs`, `src-tauri/src/services/sync_service.rs`, `src-tauri/src/commands/sync.rs`, `src-tauri/src/database/*sync*.rs`
- Test: service tests for scheduling window/idempotency/state transitions

**Interfaces:**
- Consumes: Task 8 calendar persistence, Task 9 sync adapter
- Produces: in-app local sync runs every 5 hours for a 4-week-ahead window, with persisted sync status/history and manual-trigger command

- [ ] **Step 1: Implement sync window calculation (now -> +4 weeks) and run metadata persistence**
- [ ] **Step 2: Implement idempotent upsert pipeline for synced records**
  - Never duplicate on repeated runs; preserve previously stored data on partial source failure.

- [ ] **Step 3: Implement scheduler trigger (every 5 hours while app runtime is active)**
  - Design API so future tray/closed-app behavior can be added without rewriting sync core (behavior TBD).

- [ ] **Step 4: Add sync status/error reporting commands and logs (no secrets)**
- [ ] **Step 5: Test repeated runs, partial failures, and overlap handling**
- [ ] **Step 6: Commit**

### Task 11: Credential/config storage boundary for TDTU sync

**Files:**
- Create/Modify: `src-tauri/src/services/settings.rs` or `credentials.rs`, platform/keyring integration modules, sync/config commands
- Test: unit/integration tests with mocked keyring/storage where possible

**Interfaces:**
- Consumes: TDTU credential variables (`STUDENT_ID`, `PASSWORD`) and sync config
- Produces: secure local credential/config access for sync without hardcoded secrets or plaintext persistence in app DB

- [ ] **Step 1: Define config model (student ID, sync enabled/interval defaults if needed, target semester/options TBD)**
- [ ] **Step 2: Integrate OS keyring (preferred) or secure abstraction boundary**
  - If keyring integration is deferred, isolate a temporary secure interface and document risk/TODO explicitly (do not store plaintext secrets).

- [ ] **Step 3: Wire sync service to credential/config provider**
- [ ] **Step 4: Verify secrets are not logged or serialized in error paths**
- [ ] **Step 5: Commit**

---

## Verification & Exit Criteria (backend-first milestone)

- [ ] App scaffold builds and runs locally with Rust backend modules in place.
- [ ] SQLite migrations initialize a fresh database and are repeatable.
- [ ] Task/tag/completion/focus backend supports core CRUD/complete flows and persists focus state across restarts (max 3 focused).
- [ ] Confirmed domain cases covered by tests/checks: no-deadline tasks, overdue task state inputs, school-sourced tasks representable, completion history recorded.
- [ ] Calendar backend stores/queries mixed-source events (school/deadline/personal) with dedupe-capable identifiers.
- [ ] Sync backend has a local in-app scheduler/orchestration path for 5-hour runs over a 4-week horizon and idempotent upsert behavior (using fixtures/mocked adapter where live TDTU access is unavailable).
- [ ] Credentials/config handling follows secure boundary (no secrets in repo/logs).

## Execution Handoff

After saving and self-reviewing this plan, backend implementation should proceed only after explicit approval of this plan as the backend-first description.

**Recommended execution approach (for this plan): Subagent-driven development**, because tasks span scaffolding, schema, domain logic, and sync orchestration with meaningful interface boundaries and a high cost of subtle persistence/sync mistakes. Native execution is acceptable if preferred for speed in a single session.

If approved, implement in order: **Phase A → Phase B (tasks first) → Phase C (calendar/sync foundations)**, keeping deferred/TBD items explicit rather than guessed.
