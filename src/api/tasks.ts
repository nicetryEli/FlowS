import { invoke } from "@tauri-apps/api/core";

export type TaskImportance = "extreme" | "high" | "mid" | "low" | "simple";
export type TaskSource = "personal" | "school";

export interface Task {
  id: number;
  title: string;
  description: string | null;
  created_at: string;
  deadline_at: string | null;
  importance: TaskImportance;
  source: TaskSource;
  external_source: string | null;
  external_id: string | null;
  source_key: string | null;
  source_hash: string | null;
  completed_at: string | null;
  is_focused: boolean;
  focus_order: number | null;
  updated_at: string;
  last_seen_at: string | null;
}

export interface NewTask {
  title: string;
  description?: string | null;
  deadline_at?: string | null;
  importance: TaskImportance;
  source: TaskSource;
  external_source?: string | null;
  external_id?: string | null;
  source_key?: string | null;
  source_hash?: string | null;
  last_seen_at?: string | null;
}

export interface TaskUpdate {
  title?: string;
  description?: string | null;
  deadline_at?: string | null;
  importance?: TaskImportance;
  source?: TaskSource;
  external_source?: string | null;
  external_id?: string | null;
  source_key?: string | null;
  source_hash?: string | null;
  last_seen_at?: string | null;
}

export const createTask = (input: NewTask) => invoke<Task>("create_task", { input });
export const getTask = (id: number) => invoke<Task | null>("get_task", { id });
export const listTasks = () => invoke<Task[]>("list_tasks");
export const updateTask = (id: number, patch: TaskUpdate) => invoke<Task>("update_task", { id, patch });
export const deleteTask = (id: number) => invoke<void>("delete_task", { id });
export const completeTask = (id: number, completedAt?: string | null) =>
  invoke<Task>("complete_task", { id, completed_at: completedAt ?? null });
export const setTaskFocus = (id: number, focused: boolean) => invoke<Task>("set_task_focus", { id, focused });
export const listFocusedTasks = () => invoke<Task[]>("list_focused_tasks");
