# FlowS - Project Context (Session Context Window)

> Generated from all readable files in the repository as of 2026-11-20.
> Purpose: capture the current whole-context understanding of the FlowS project for this and future sessions.

## 1. Project Snapshot

- **Name:** FlowS
- **Type:** Desktop personal task/study management app for students
- **Core idea:** Local-first task management with gamification/visual motivation, combining practical planning with playful progress feedback.
- **Current repo state:** Planning/documentation stage only. No application source code, build config, or database schema files are present yet.
- **Primary development target (per docs):** Arch Linux (Wayland/X11), with planned cross-platform support.
- **Current direction:** Planning first, then implementation. Near-term focus is on backend planning/design before UI/frontend implementation details.

## 2. Repository Contents (all files found)

| Path | Type | Notes |
|---|---|---|
| `.garden.md` | Markdown (doc) | Early/alternative concept: "Vườn hoa (Task Garden)" where tasks are flowers. |
| `plan.md` | Markdown (doc) | Implementation-oriented plan: "Lake of Task" (fish tank) concept, stack, phased roadmap, MVP suggestion, open notes. |
| ` ProjectDiscription.txt` | Text (doc, filename has leading space) | Analysis/review of the SRS: strengths, contradictions, risks, and technical suggestions. |
| `SRS - Ứng dụng Quản lý Công việc Cá nhân (FlowS).md` | Markdown (doc) | Formal Software Requirements Specification (Vietnamese). Contains mixed/partially outdated concept references. |
| `assets/fish_bg.gif` | Image asset (GIF, 400x688) | Likely background/reference art for fish/lake concept. Image content not visually inspected in this session (model image-input limitation). |
| `assets/.garden1.jpg` | Image asset (JPEG, 941x1672, hidden filename) | Likely concept/reference art for garden/flower concept. Image content not visually inspected in this session. |
| `tool-check-calender/` | Directory (reference/prototype project) | Existing TDTU calendar crawl/ingest/dashboard prototype (Python Playwright crawler, SQLite ingest, FastAPI/Flutter components, docs, tests). Contains its own `.git`, `.venv`, `.env`/`.env.example`, and sample/build artifacts; treat as reference material for crawl target and data shapes. **Do not expose `.env` secrets.** |

The FlowS root workspace itself currently has no application source scaffold (`frontend/`, `src-tauri/`, `package.json`, `Cargo.toml`, migrations, packaging) beyond planning docs/assets and the `tool-check-calender/` reference folder.

## 3. Concept Evolution & Key Contradictions

A major theme in the docs is an evolving gamification metaphor:

- **Flower/Garden concept** (`.garden.md`, parts of SRS intro/design notes): tasks are flowers in a rose bush; flowers wilt/brown near deadline; size = importance; drag/cut to focus area; "Đã xong" recolors flower and adds to a daily bouquet; daily reset saves bouquet; monthly history shows bouquets.
- **Fish/Lake concept** (`plan.md`, SRS 5.3/5.4, `fish_bg.gif`): tasks are fish in a tank; fish become gray/black as deadline approaches, colorful when completed; drag up to 3 fish to a focus area; completed fish return to tank coloring it; monthly history shows 12 tanks.

**Current canonical direction (user-confirmed):** Default theme/concept is **Fish Tank (Lake of Task)**. After the first version, the app should support changing to multiple themes via theme pack folders (package folders).

**Documented inconsistencies to resolve / superseded by decisions:**
- SRS mixes metaphors (intro/design notes mention garden/flowers; functional sections 5.3/5.4 specify Lake of Task/fish). Default is now confirmed as fish tank; flower/garden may become a future theme pack.
- SRS 5.5 states school-calendar sync as a periodic background job "trong github" — **superseded:** sync must run locally in-app (no GitHub background job).
- Visual direction tension: NES.css (8-bit/pixel UI) vs Geist/Geist Mono (modern minimalist typography) noted in `ProjectDiscription.txt` (still undecided).
- SRS design notes say "other colors is for flowers" while newer concept centers on fish/colors in the tank.

## 4. Intended Product & Requirements (consolidated)

### 4.1 Goals
- Track daily tasks, deadlines, and priority levels.
- Motivate completion through visual gamification and progress feedback.
- Integrate school timetable/deadlines with personal tasks.
- Support multiple projects/contexts via tags.
- Operate local-first/offline-first (except school crawling/sync).

### 4.2 Planned Main Areas / Screens (from SRS & plan)
1. **Dashboard**
   - Current date/time
   - System media player widget (compact, now playing)
   - Streak (consecutive days with >=1 completed task)
   - Remaining tasks today
   - Completed/total tasks today
   - Activity heatmap (GitHub-contribution style)
2. **Task List**
   - List/filter/sort tasks
   - CRUD tasks
   - Fields: name, created date (auto today), deadline, importance
   - Importance levels mentioned: extreme, high, mid, low, simple (5 levels)
   - Mark complete, edit/delete
3. **Lake of Task (gamified focus view)**
   - Split view: upper area ~3/4, lower focus area ~1/4
   - Each task represented as a fish; size corresponds to importance
   - Color/appearance reflects deadline proximity (gray/black as deadline nears; colorful when done)
   - **Confirmed visual/state rules:** overdue incomplete tasks display as fully black; tasks with no deadline display as white
   - Drag/catch fish into lower area to focus; max 3 fish in focus
   - **Confirmed:** focused/dragged-down fish state persists across app opens/restarts
   - "Đã xong" marks complete, recolors fish, returns to tank and adds color/progress to tank
   - **Confirmed:** tasks synced from school also appear as fish in the tank
   - Daily reset behavior described (completed/colored fish hidden next day; tank returns to gray/black fish) — exact lifecycle rules for incomplete vs completed tasks across days still need clarification
4. **Monthly History**
   - Horizontal scroll view (centered, ~3/5 screen in some descriptions)
   - 12 monthly tanks showing completed/colorized fish per month
   - Display monthly completion count (e.g., "December - 67 tasks")
5. **Calendar**
   - Week/month views combining personal tasks + school schedule + school deadlines
   - **Confirmed:** calendar must show all events from school, deadlines, and personal events together
   - Crawling/sync of timetable and deadlines from school system
   - **Confirmed sync approach:** local in-app sync only (no GitHub background job), periodic sync every 5 hours, fetching/planning 4 weeks ahead; manual sync option may still be desired
   - Map crawled data to tasks/events; avoid duplicates on re-sync
   - Secure local storage of school credentials (encrypted locally per SRS / secure storage approach TBD)
   - **Target/reference (from `tool-check-calender/`):** crawl TDTU systems — student portal (`https://old-stdportal.tdtu.edu.vn/`), schedule/exam pages (`https://lichhoc-lichthi.tdtu.edu.vn/tkb2.aspx`, `https://lichhoc-lichthi.tdtu.edu.vn/xemlichthi.aspx`), and eLearning (`https://elearning.tdtu.edu.vn/`) for class sessions, exams, and deadlines. Credential variables used by reference crawler: `STUDENT_ID` and `PASSWORD` (plus optional settings like semester/weeks-ahead/timezone/DB path).
6. **Project Tags**
   - Multiple tags per task
   - Create/rename/recolor/delete tags
   - Filter/view by tag across list/dashboard/calendar
   - Stats: completed tasks per tag/project
7. **Notifications**
   - Deadline reminders (configurable timing)
   - Sync success/failure notifications
   - Streak-at-risk warning
   - Native OS notifications via Tauri API
   - Per-notification-type enable/disable settings

### 4.3 Non-functional / Architectural Requirements
- **Performance:** lightweight, fast startup (Tauri vs Electron rationale)
- **Offline-first:** full functionality without network except school crawling
- **Security:** school credentials encrypted/stored securely locally
- **Cross-platform:** Linux first (native packaging targets mentioned), later Windows/macOS
- **Extensibility:** layered commands/services/database/platform; theme/modding support via app rules
- **Data store:** SQLite as sole local database, no centralized backend server

### 4.4 Confirmed Decisions (user-provided)
- Default gamification theme/concept is Fish Tank (Lake of Task). After the first version, support multiple themes via theme pack folders (package folders).
- Planning should come before implementation; current focus is backend-first.
- Focused/dragged-down fish state persists across app opens/restarts.
- Overdue incomplete tasks are displayed as fully black fish.
- Tasks synced from school appear as fish in the tank.
- Tasks with no deadline are displayed as white fish.
- School sync runs locally in-app only (no GitHub background job), every 5 hours, covering 4 weeks ahead.
- Calendar shows all events: school events, deadlines, and personal events.
- Crawl target for school data is TDTU (reference: `tool-check-calender/`), using credentials/variables `STUDENT_ID` and `PASSWORD`.

## 5. Planned Tech Stack & Architecture

- **Frontend:** Vue 3 + TypeScript
- **Desktop shell:** Tauri 2
- **Backend/logic:** Rust
- **Database:** SQLite
- **Styling:** Tailwind CSS + NES.css (planned, but style direction under discussion)
- **Fonts mentioned:** Geist, Geist Mono
- **State/routing (per plan):** Pinia, vue-router
- **DB access (per plan):** sqlx or rusqlite + migrations

**Proposed project structure (from SRS):**
```
FlowS/
├── frontend/            Vue 3 + TypeScript
├── src-tauri/
│   ├── src/
│   │   ├── commands/
│   │   ├── services/
│   │   ├── database/
│   │   └── platform/ (linux.rs, windows.rs, macos.rs)
│   └── migrations/
├── packaging/ (arch, debian, fedora, appimage)
└── README.md
```

## 6. Implementation Plan (from `plan.md`)

- **Phase 0 - Foundation:** scaffold Tauri 2 + Vue 3 + TS, Tailwind/NES.css/Geist, Rust structure (`commands/services/database`), router + Pinia, SQLite + migrations, CSS-variable design tokens for future theming.
- **Phase 1 - Core tasks:** schema (`tasks`, `tags`, `task_tags`, `completions`), Rust commands (CRUD/complete/list), task list page, basic tags; persist durable data in DB and derive daily-reset/focus state from `completed_at`.
- **Phase 2 - Dashboard:** streak/remaining/completed queries, heatmap from `completions`, media player via MPRIS on Linux with platform abstraction.
- **Phase 3 - Lake of Task:** render fish (canvas or DOM + pixel sprites), size by importance, color by deadline progress `(now-created)/(deadline-created)`, drag/drop with 3-fish limit, completion flow, daily reset logic; start with simple sprites.
- **Phase 4 - Monthly history:** 12 tanks, horizontal scroll, monthly counts, reuse tank component.
- **Phase 5 - Calendar:** week/month view combining personal + school data; add `source` and `external_id` to prevent sync duplicates.
- **Phase 6 - School crawler:** Rust (`reqwest` + `scraper`), manual sync first then periodic; store credentials in OS keyring (`keyring` crate). Architecture decision needed (in-app scheduler vs GitHub Actions/local-first conflict).
- **Phase 7 - Notifications + settings:** Tauri notification plugin, deadline/streak/sync schedulers and toggles.
- **Phase 8 - Theme, packaging, polish:** theme loading (JSON/CSS token overrides), package for Arch/Debian/Fedora/AppImage, Wayland/X11 testing.

**Suggested MVP:** Phases 0 -> 3 (task core + dashboard + lake) before crawler.

## 7. Noted Design/Technical Suggestions (`ProjectDiscription.txt` and references)

- Use Canvas 2D or PixiJS (e.g. Boids-like movement) for fish rendering/interaction instead of 3D engine.
- Linux media integration via `mpris` (D-Bus); Windows via `windows::Media::Control`; platform-specific code isolated in `platform/`.
- Store credentials in OS keyring rather than custom-encrypted files in app storage.
- Prefer in-app Tauri WebView-assisted login/scraping flow for fragile school portals (SSO/CAPTCHA/HTML changes) instead of pure headless HTTP crawling.
- Theming via CSS variables and user-provided `theme.json` (e.g. under `~/.config/flows/themes/`) plus sprite assets, without source changes.
- Decide and unify visual style (pixel art/8-bit end-to-end vs monochrome minimalist with Geist).
- `tool-check-calender/` provides a concrete TDTU crawl reference (Playwright selectors/flows, parsed data shapes for schedule/exams/deadlines, SQLite-oriented pipeline ideas) that can inform FlowS backend planning; any adaptation should follow FlowS's local-first/in-app scheduling decisions and security practices.

## 8. Open Questions / Ambiguities Captured from Docs

- Exact daily reset/lifecycle semantics for incomplete vs completed tasks across days.
- Theme pack format/specification (folder structure, manifest/rules, asset/theme variable loading) for post-v1 multi-theme support.
- How to port/adapt TDTU crawling from `tool-check-calender/` (Python/Playwright reference) into FlowS backend architecture, including authentication/session handling details and behavior when app is closed (if any background/tray execution is desired beyond in-app runtime).
- Final visual/style system for the default fish-tank theme: NES.css + pixel sprites vs Geist minimalist; definitive color/font rules.
- DB schema details beyond proposed tables, migration/tooling choices (`sqlx` vs `rusqlite`), and app packaging specifics.
- Whether ` ProjectDiscription.txt` (leading space in filename) is intentional.
- Other product/UX/technical decisions explicitly deferred by user for now.

**Resolved (confirmed by user, 2026-11-20):** default theme is Fish Tank (Lake of Task), with post-v1 multi-theme support via theme pack folders; planning before implementation with current backend-first focus; focused/dragged-down fish state persists across app opens/restarts; overdue incomplete tasks display fully black; school-synced tasks appear as fish in the tank; tasks with no deadline display white; school sync is local in-app only (no GitHub background job), every 5 hours, covering 4 weeks ahead; calendar shows school events, deadlines, and personal events; crawl target is TDTU (`old-stdportal.tdtu.edu.vn`, `lichhoc-lichthi.tdtu.edu.vn`, `elearning.tdtu.edu.vn`) with credential variables `STUDENT_ID` and `PASSWORD` (reference: `tool-check-calender/`).

## 9. Session Notes

- This context file is based on repository documents, file metadata, and user-provided clarifications in this session.
- No source code behavior was inspected because no implementation exists in the repo yet.
- Asset image contents were not visually analyzed due to model limitations; only filenames/metadata were recorded.
- User indicated additional decisions will be provided later; current priority is backend-first planning.
- If the user provides additional information (decisions, designs, schemas, external docs, or code), update this file to keep it the authoritative session context.
