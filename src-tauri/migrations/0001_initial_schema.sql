-- Initial FlowS schema (tasks-first, calendar/sync-ready)

PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    description TEXT,
    created_at TEXT NOT NULL,
    deadline_at TEXT,
    importance TEXT NOT NULL CHECK (importance IN ('extreme', 'high', 'mid', 'low', 'simple')),
    source TEXT NOT NULL DEFAULT 'personal' CHECK (source IN ('personal', 'school')),
    external_source TEXT,
    external_id TEXT,
    source_key TEXT,
    source_hash TEXT,
    completed_at TEXT,
    is_focused INTEGER NOT NULL DEFAULT 0 CHECK (is_focused IN (0, 1)),
    focus_order INTEGER,
    created_source TEXT NOT NULL DEFAULT 'local' CHECK (created_source IN ('local', 'sync')),
    updated_at TEXT NOT NULL,
    last_seen_at TEXT,
    UNIQUE (source, source_key)
);

CREATE INDEX IF NOT EXISTS idx_tasks_deadline_at ON tasks(deadline_at);
CREATE INDEX IF NOT EXISTS idx_tasks_source ON tasks(source);
CREATE INDEX IF NOT EXISTS idx_tasks_completed_at ON tasks(completed_at);
CREATE INDEX IF NOT EXISTS idx_tasks_focus ON tasks(is_focused, focus_order);

CREATE TABLE IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    color TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS task_tags (
    task_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    PRIMARY KEY (task_id, tag_id),
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_task_tags_tag_id ON task_tags(tag_id);

CREATE TABLE IF NOT EXISTS completions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id INTEGER NOT NULL,
    completed_at TEXT NOT NULL,
    completion_date TEXT NOT NULL,
    source TEXT NOT NULL DEFAULT 'local' CHECK (source IN ('local', 'sync')),
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_completions_task_id ON completions(task_id);
CREATE INDEX IF NOT EXISTS idx_completions_completed_at ON completions(completed_at);
CREATE INDEX IF NOT EXISTS idx_completions_completion_date ON completions(completion_date);

CREATE TABLE IF NOT EXISTS task_focus_state (
    task_id INTEGER PRIMARY KEY,
    is_focused INTEGER NOT NULL CHECK (is_focused IN (0, 1)),
    focus_order INTEGER,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_task_focus_state_active ON task_focus_state(is_focused, focus_order);

CREATE TABLE IF NOT EXISTS calendar_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    description TEXT,
    start_at TEXT,
    end_at TEXT,
    all_day INTEGER NOT NULL DEFAULT 0 CHECK (all_day IN (0, 1)),
    timezone TEXT,
    source TEXT NOT NULL CHECK (source IN ('personal', 'school', 'deadline')),
    event_kind TEXT,
    external_source TEXT,
    external_id TEXT,
    source_key TEXT,
    source_hash TEXT,
    related_task_id INTEGER,
    status TEXT,
    metadata_json TEXT,
    last_seen_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE (source, source_key),
    FOREIGN KEY (related_task_id) REFERENCES tasks(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_calendar_events_start_at ON calendar_events(start_at);
CREATE INDEX IF NOT EXISTS idx_calendar_events_source ON calendar_events(source);
CREATE INDEX IF NOT EXISTS idx_calendar_events_related_task_id ON calendar_events(related_task_id);

CREATE TABLE IF NOT EXISTS sync_state (
    source TEXT PRIMARY KEY,
    enabled INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0, 1)),
    last_attempt_at TEXT,
    last_success_at TEXT,
    last_error TEXT,
    next_run_at TEXT,
    window_start_at TEXT,
    window_end_at TEXT,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS sync_runs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('running', 'success', 'partial', 'failed')),
    started_at TEXT NOT NULL,
    finished_at TEXT,
    window_start_at TEXT,
    window_end_at TEXT,
    records_seen INTEGER NOT NULL DEFAULT 0,
    records_upserted INTEGER NOT NULL DEFAULT 0,
    error_message TEXT,
    metadata_json TEXT
);

CREATE INDEX IF NOT EXISTS idx_sync_runs_source_started_at ON sync_runs(source, started_at);
