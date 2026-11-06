PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS settings (
  key TEXT PRIMARY KEY,
  value TEXT,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS schema_migrations (
  version INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  applied_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS ref_periods (
  period INTEGER PRIMARY KEY,
  start_time TEXT NOT NULL
);

INSERT OR IGNORE INTO ref_periods (period, start_time) VALUES
  (1, '06:50'), (2, '07:40'), (3, '08:30'), (4, '09:30'), (5, '10:20'),
  (6, '11:10'), (7, '12:45'), (8, '13:35'), (9, '14:25'), (10, '15:25'),
  (11, '16:15'), (12, '17:05'), (13, '18:05'), (14, '18:55'), (15, '19:45');

CREATE TABLE IF NOT EXISTS courses (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  moodle_course_id TEXT UNIQUE,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS class_sessions (
  id INTEGER PRIMARY KEY,
  course_id INTEGER REFERENCES courses(id) ON DELETE SET NULL,
  subject_name TEXT NOT NULL,
  room TEXT,
  session_date TEXT NOT NULL,
  day_of_week TEXT,
  start_period INTEGER NOT NULL CHECK (start_period BETWEEN 1 AND 15),
  end_period   INTEGER NOT NULL CHECK (end_period   BETWEEN 1 AND 15),
  status TEXT NOT NULL DEFAULT 'scheduled'
    CHECK (status IN ('scheduled','absent','makeup','cancelled','moved')),
  source_key TEXT NOT NULL UNIQUE,
  source_hash TEXT NOT NULL,
  first_seen_at TEXT NOT NULL,
  last_seen_at  TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  CHECK (end_period >= start_period)
);
CREATE INDEX IF NOT EXISTS idx_class_sessions_date ON class_sessions(session_date);

CREATE TABLE IF NOT EXISTS exams (
  id INTEGER PRIMARY KEY,
  course_id INTEGER REFERENCES courses(id) ON DELETE SET NULL,
  subject_name TEXT NOT NULL,
  exam_date TEXT NOT NULL,
  start_time TEXT,
  end_time TEXT,
  exam_room TEXT,
  exam_type TEXT,
  notes TEXT,
  source_key TEXT NOT NULL UNIQUE,
  source_hash TEXT NOT NULL,
  first_seen_at TEXT NOT NULL,
  last_seen_at  TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_exams_date ON exams(exam_date);

CREATE TABLE IF NOT EXISTS deadlines (
  id INTEGER PRIMARY KEY,
  course_id INTEGER REFERENCES courses(id) ON DELETE SET NULL,
  course_name TEXT,
  activity_name TEXT NOT NULL,
  activity_url TEXT NOT NULL UNIQUE,
  due_date TEXT NOT NULL,
  completion_status TEXT NOT NULL DEFAULT 'incomplete',
  source_key TEXT NOT NULL UNIQUE,
  source_hash TEXT NOT NULL,
  first_seen_at TEXT NOT NULL,
  last_seen_at  TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_deadlines_due ON deadlines(due_date);

CREATE TABLE IF NOT EXISTS personal_events (
  id INTEGER PRIMARY KEY,
  title TEXT NOT NULL,
  notes TEXT,
  all_day INTEGER NOT NULL DEFAULT 0 CHECK (all_day IN (0,1)),
  start_datetime TEXT NOT NULL,
  end_datetime   TEXT NOT NULL,
  color TEXT,
  recurrence_rule TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  CHECK (end_datetime >= start_datetime)
);
CREATE INDEX IF NOT EXISTS idx_personal_events_start ON personal_events(start_datetime);

CREATE TABLE IF NOT EXISTS sync_runs (
  id INTEGER PRIMARY KEY,
  started_at TEXT NOT NULL,
  finished_at TEXT,
  status TEXT NOT NULL DEFAULT 'running'
    CHECK (status IN ('running','ok','partial','failed')),
  detail TEXT
);

CREATE VIEW IF NOT EXISTS calendar_events AS
  SELECT 'class'                  AS kind,
         id                       AS ref_id,
         subject_name             AS title,
         room                     AS location,
         session_date || 'T' || rp1.start_time AS start_dt,
         strftime('%Y-%m-%dT%H:%M', session_date || 'T' || rp2.start_time, '+50 minutes') AS end_dt,
         0                        AS all_day,
         status                   AS tag
  FROM class_sessions
  JOIN ref_periods rp1 ON rp1.period = class_sessions.start_period
  JOIN ref_periods rp2 ON rp2.period = class_sessions.end_period
  UNION ALL
  SELECT 'exam', id,
         subject_name, exam_room,
         exam_date || 'T' || COALESCE(start_time, '00:00'),
         exam_date || 'T' || COALESCE(end_time, start_time, '00:00'),
         CASE WHEN start_time IS NULL THEN 1 ELSE 0 END,
         COALESCE(exam_type, 'exam')
  FROM exams
  UNION ALL
  SELECT 'deadline', id,
         activity_name, course_name,
         due_date,
         due_date,
         0,
         'deadline'
  FROM deadlines
  UNION ALL
  SELECT 'personal', id,
         title, NULL,
         start_datetime, end_datetime,
         all_day,
         COALESCE(color, 'green')
  FROM personal_events;
