PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

-- Enrich the unified feed with the metadata the UI needs for the detail
-- dialog (notes, exam type, course link for eLearning deadlines, color).
DROP VIEW IF EXISTS calendar_events;

CREATE VIEW calendar_events AS
  SELECT 'class'                  AS kind,
         id                       AS ref_id,
         subject_name             AS title,
         room                     AS location,
         session_date || 'T' || rp1.start_time AS start_dt,
         strftime('%Y-%m-%dT%H:%M', session_date || 'T' || rp2.start_time, '+50 minutes') AS end_dt,
         0                        AS all_day,
         status                   AS tag,
         NULL                     AS notes,
         NULL                     AS activity_url,
         NULL                     AS course_name,
         NULL                     AS exam_type,
         NULL                     AS color
  FROM class_sessions
  JOIN ref_periods rp1 ON rp1.period = class_sessions.start_period
  JOIN ref_periods rp2 ON rp2.period = class_sessions.end_period
  UNION ALL
  SELECT 'exam', id,
         subject_name, exam_room,
         exam_date || 'T' || COALESCE(start_time, '00:00'),
         exam_date || 'T' || COALESCE(end_time, start_time, '00:00'),
         CASE WHEN start_time IS NULL THEN 1 ELSE 0 END,
         COALESCE(exam_type, 'exam'),
         notes, NULL, NULL, exam_type, NULL
  FROM exams
  UNION ALL
  SELECT 'deadline', id,
         activity_name, course_name,
         due_date,
         due_date,
         0,
         'deadline',
         NULL, activity_url, course_name, NULL, NULL
  FROM deadlines
  UNION ALL
  SELECT 'personal', id,
         title, NULL,
         start_datetime, end_datetime,
         all_day,
         COALESCE(color, 'green'),
         notes, NULL, NULL, NULL, color
  FROM personal_events;
