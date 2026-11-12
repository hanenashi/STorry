import { invoke, isTauri } from '@tauri-apps/api/core';
import type { Draft } from './draft';

export interface Story {
  id: string;
  revision: number;
  updatedAt: number;
  data: Draft;
}
export interface StorySummary {
  id: string;
  revision: number;
  title: string;
  updatedAt: number;
  words: number;
  turns: number;
  excerpt: string;
}

function command<T>(name: string, args: Record<string, unknown> = {}): Promise<T> {
  if (!isTauri()) return Promise.reject(new Error('Open the STorry desktop app to use your story library.'));
  return invoke(name, args);
}
export const initializeLibrary = (legacy: Draft | null) => command<StorySummary[]>('initialize_library', { legacy });
export const listStories = () => command<StorySummary[]>('list_stories');
export const loadStory = (id: string) => command<Story>('load_story', { id });
export const createStory = (data: Draft) => command<Story>('create_story', { id: crypto.randomUUID(), data });
export const saveStory = (id: string, revision: number, data: Draft) => command<Story>('save_story', { id, revision, data });
export const deleteStory = (id: string, revision: number) => command<void>('delete_story', { id, revision });
