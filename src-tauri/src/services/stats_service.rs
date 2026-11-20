use chrono::{Duration, NaiveDate};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::database::DbResult;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DailyTaskSummary {
    pub date: String,
    pub completed_today: i64,
    pub total_tasks_for_day: i64,
    pub remaining_tasks_for_day: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DailyActivity {
    pub date: String,
    pub completion_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MonthlyCompletionCount {
    pub month: String, // YYYY-MM
    pub completion_count: i64,
}

pub fn count_completions_on_date(conn: &Connection, date: &str) -> DbResult<i64> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM completions WHERE completion_date = ?1",
        params![date],
        |row| row.get(0),
    )?;
    Ok(count)
}

/// Tasks considered "for day" are tasks whose deadline falls on that date (local date string YYYY-MM-DD).
/// Completed-today is derived from completions on that date for those tasks (and in general may include other tasks;
/// here we return completion count for tasks-for-day completed on that date as completed_today, and also total completions on date separately if needed).
pub fn get_daily_task_summary(conn: &Connection, date: &str) -> DbResult<DailyTaskSummary> {
    let total_tasks_for_day: i64 = conn.query_row(
        "SELECT COUNT(*) FROM tasks WHERE substr(deadline_at, 1, 10) = ?1",
        params![date],
        |row| row.get(0),
    )?;

    // completed today among tasks-for-day (completed on that date and task deadline is that date)
    let completed_tasks_for_day: i64 = conn.query_row(
        "SELECT COUNT(DISTINCT t.id)
         FROM tasks t
         INNER JOIN completions c ON c.task_id = t.id
         WHERE substr(t.deadline_at, 1, 10) = ?1 AND c.completion_date = ?1",
        params![date],
        |row| row.get(0),
    )?;

    let remaining = if total_tasks_for_day >= completed_tasks_for_day {
        total_tasks_for_day - completed_tasks_for_day
    } else {
        0
    };

    Ok(DailyTaskSummary {
        date: date.to_string(),
        completed_today: completed_tasks_for_day,
        total_tasks_for_day,
        remaining_tasks_for_day: remaining,
    })
}

pub fn list_completion_counts_by_date(
    conn: &Connection,
    start_date: &str,
    end_date: &str,
) -> DbResult<Vec<DailyActivity>> {
    let mut stmt = conn.prepare(
        "SELECT completion_date, COUNT(*) as cnt
         FROM completions
         WHERE completion_date >= ?1 AND completion_date <= ?2
         GROUP BY completion_date
         ORDER BY completion_date ASC",
    )?;
    let rows = stmt.query_map(params![start_date, end_date], |row| {
        Ok(DailyActivity {
            date: row.get(0)?,
            completion_count: row.get(1)?,
        })
    })?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

/// Streak = number of consecutive days ending at `as_of_date` with >= 1 completion each day.
pub fn calculate_streak(conn: &Connection, as_of_date: &str) -> DbResult<i64> {
    let mut date = NaiveDate::parse_from_str(as_of_date, "%Y-%m-%d")
        .map_err(|e| crate::database::DatabaseError::InvalidInput(format!("invalid date {as_of_date}: {e}")))?;
    let mut streak = 0i64;
    loop {
        let ds = date.format("%Y-%m-%d").to_string();
        let count = count_completions_on_date(conn, &ds)?;
        if count <= 0 {
            break;
        }
        streak += 1;
        date -= Duration::days(1);
    }
    Ok(streak)
}

pub fn list_monthly_completion_counts(conn: &Connection, year: i32) -> DbResult<Vec<MonthlyCompletionCount>> {
    let year_prefix = format!("{year:04}-");
    let mut stmt = conn.prepare(
        "SELECT substr(completion_date, 1, 7) as ym, COUNT(*) as cnt
         FROM completions
         WHERE substr(completion_date, 1, 5) = ?1
         GROUP BY ym
         ORDER BY ym ASC",
    )?;
    // prefix like "2026-"
    let rows = stmt.query_map(params![year_prefix], |row| {
        Ok(MonthlyCompletionCount {
            month: row.get(0)?,
            completion_count: row.get(1)?,
        })
    })?;
    let mut map: BTreeMap<String, i64> = BTreeMap::new();
    for row in rows {
        let item = row?;
        map.insert(item.month, item.completion_count);
    }
    // ensure all 12 months present? useful for history view; include zeros
    let mut out = Vec::with_capacity(12);
    for m in 1..=12 {
        let ym = format!("{year:04}-{m:02}");
        out.push(MonthlyCompletionCount {
            completion_count: *map.get(&ym).unwrap_or(&0),
            month: ym,
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::init_test_db;
    use crate::models::{NewTask, TaskImportance, TaskSource};
    use crate::services::task_service;

    fn task(title: &str, deadline: Option<&str>) -> NewTask {
        NewTask {
            title: title.to_string(),
            description: None,
            deadline_at: deadline.map(|s| s.to_string()),
            importance: TaskImportance::Mid,
            source: TaskSource::Personal,
            external_source: None,
            external_id: None,
            source_key: None,
            source_hash: None,
            last_seen_at: None,
        }
    }

    #[test]
    fn daily_summary_and_heatmap_counts() {
        let conn = init_test_db().unwrap();
        let t1 = task_service::create_task(&conn, task("A", Some("2026-11-20T09:00:00Z"))).unwrap();
        let _t2 = task_service::create_task(&conn, task("B", Some("2026-11-20T12:00:00Z"))).unwrap();
        let t3 = task_service::create_task(&conn, task("C", Some("2026-11-21T09:00:00Z"))).unwrap();

        task_service::complete_task(&conn, t1.id, Some("2026-11-20T18:00:00Z".into())).unwrap();
        task_service::complete_task(&conn, t3.id, Some("2026-11-21T10:00:00Z".into())).unwrap();
        task_service::complete_task(&conn, t3.id, Some("2026-11-21T11:00:00Z".into())).unwrap(); // another completion record same day/task

        let summary = get_daily_task_summary(&conn, "2026-11-20").unwrap();
        assert_eq!(summary.total_tasks_for_day, 2);
        assert_eq!(summary.completed_today, 1);
        assert_eq!(summary.remaining_tasks_for_day, 1);

        let heatmap = list_completion_counts_by_date(&conn, "2026-11-19", "2026-11-21").unwrap();
        assert_eq!(heatmap.len(), 2);
        assert_eq!(heatmap[0], DailyActivity { date: "2026-11-20".into(), completion_count: 1 });
        assert_eq!(heatmap[1], DailyActivity { date: "2026-11-21".into(), completion_count: 2 });
    }

    #[test]
    fn streak_and_monthly_counts_across_boundaries() {
        let conn = init_test_db().unwrap();
        let t = task_service::create_task(&conn, task("A", None)).unwrap();
        // completions on consecutive days ending 2026-11-20
        task_service::complete_task(&conn, t.id, Some("2026-11-18T10:00:00Z".into())).unwrap();
        task_service::complete_task(&conn, t.id, Some("2026-11-19T10:00:00Z".into())).unwrap();
        task_service::complete_task(&conn, t.id, Some("2026-11-20T10:00:00Z".into())).unwrap();
        // gap breaks earlier streak
        task_service::complete_task(&conn, t.id, Some("2026-11-16T10:00:00Z".into())).unwrap();
        // month boundary / previous month
        task_service::complete_task(&conn, t.id, Some("2026-10-31T10:00:00Z".into())).unwrap();
        task_service::complete_task(&conn, t.id, Some("2026-12-01T10:00:00Z".into())).unwrap();

        let streak = calculate_streak(&conn, "2026-11-20").unwrap();
        assert_eq!(streak, 3);
        let broken = calculate_streak(&conn, "2026-11-17").unwrap();
        assert_eq!(broken, 0);
        let earlier = calculate_streak(&conn, "2026-11-16").unwrap();
        assert_eq!(earlier, 1);

        let months = list_monthly_completion_counts(&conn, 2026).unwrap();
        assert_eq!(months.len(), 12);
        assert_eq!(months[9].month, "2026-10");
        assert_eq!(months[9].completion_count, 1);
        assert_eq!(months[10].month, "2026-11");
        assert_eq!(months[10].completion_count, 4); // 16,18,19,20
        assert_eq!(months[11].month, "2026-12");
        assert_eq!(months[11].completion_count, 1);
    }
}
