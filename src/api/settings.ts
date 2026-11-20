import { invoke } from "@tauri-apps/api/core";

export interface TdtuSyncConfig {
  student_id: string | null;
  has_password: boolean;
  sync_enabled: boolean;
}

export const getTdtuSyncConfig = () => invoke<TdtuSyncConfig>("get_tdtu_sync_config");
export const setTdtuStudentId = (studentId: string | null) =>
  invoke<void>("set_tdtu_student_id", { student_id: studentId });
export const setTdtuSyncEnabled = (enabled: boolean) =>
  invoke<void>("set_tdtu_sync_enabled", { enabled });
export const setTdtuPassword = (password: string | null) =>
  invoke<void>("set_tdtu_password", { password });
