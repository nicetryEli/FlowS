import { invoke } from "@tauri-apps/api/core";

export interface TdtuDataset {
  schedule: unknown[];
  exams: unknown[];
  deadlines: unknown[];
}

export interface SyncRunResult {
  source: string;
  status: string;
  records_seen: number;
  events_upserted: number;
  tasks_upserted: number;
  window_start_at: string;
  window_end_at: string;
  error_message: string | null;
}

export const getSyncState = (source: string) =>
  invoke<Record<string, unknown> | null>("get_sync_state", { source });

export const runTdtuSyncWithDataset = (dataset: TdtuDataset) =>
  invoke<SyncRunResult>("run_tdtu_sync_with_dataset", { dataset });
