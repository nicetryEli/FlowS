import { invoke } from "@tauri-apps/api/core";

export interface Tag {
  id: number;
  name: string;
  color: string | null;
  created_at: string;
  updated_at: string;
}

export interface NewTag {
  name: string;
  color?: string | null;
}

export interface TagUpdate {
  name?: string;
  color?: string | null;
}

export const createTag = (input: NewTag) => invoke<Tag>("create_tag", { input });
export const getTag = (id: number) => invoke<Tag | null>("get_tag", { id });
export const listTags = () => invoke<Tag[]>("list_tags");
export const updateTag = (id: number, patch: TagUpdate) => invoke<Tag>("update_tag", { id, patch });
export const deleteTag = (id: number) => invoke<void>("delete_tag", { id });
export const attachTagToTask = (taskId: number, tagId: number) =>
  invoke<void>("attach_tag_to_task", { task_id: taskId, tag_id: tagId });
export const detachTagFromTask = (taskId: number, tagId: number) =>
  invoke<void>("detach_tag_from_task", { task_id: taskId, tag_id: tagId });
export const listTagsForTask = (taskId: number) => invoke<Tag[]>("list_tags_for_task", { task_id: taskId });
