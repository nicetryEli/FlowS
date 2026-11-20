import { invoke } from "@tauri-apps/api/core";

export interface DailyTaskSummary {
  date: string;
  completed_today: number;
  total_tasks_for_day: number;
  remaining_tasks_for_day: number;
}

export interface DailyActivity {
  date: string;
  completion_count: number;
}

export interface MonthlyCompletionCount {
  month: string;
  completion_count: number;
}

export const getDailyTaskSummary = (date: string) =>
  invoke<DailyTaskSummary>("get_daily_task_summary", { date });

export const listCompletionCountsByDate = (startDate: string, endDate: string) =>
  invoke<DailyActivity[]>("list_completion_counts_by_date", { start_date: startDate, end_date: endDate });

export const calculateStreak = (asOfDate: string) =>
  invoke<number>("calculate_streak", { as_of_date: asOfDate });

export const listMonthlyCompletionCounts = (year: number) =>
  invoke<MonthlyCompletionCount[]>("list_monthly_completion_counts", { year });
