import type { ChatMessage } from './ollama';

export const STORAGE_KEY = 'storry.draft.v2';
export interface Draft {
  title: string;
  scene: string;
  notes: string;
  baseUrl: string;
  model: string;
  prompt: string;
  messages: ChatMessage[];
  contextTokens: number;
}

export function loadDraft(storage: Storage): Partial<Draft> {
  const saved = storage.getItem(STORAGE_KEY) ?? storage.getItem('storry.draft.v1');
  if (!saved) return {};
  const value = JSON.parse(saved);
  if (!value || typeof value !== 'object' || Array.isArray(value)) throw new Error('Invalid saved draft');
  const result: Partial<Draft> = {};
  for (const key of ['title', 'scene', 'notes', 'baseUrl', 'model', 'prompt'] as const) {
    if (value[key] !== undefined) {
      if (typeof value[key] !== 'string') throw new Error(`Invalid saved ${key}`);
      result[key] = value[key];
    }
  }
  if (value.messages !== undefined) {
    if (!Array.isArray(value.messages) || value.messages.length % 2 !== 0 || !value.messages.every(
      (message: unknown, index: number) => {
        if (!message || typeof message !== 'object') return false;
        const item = message as ChatMessage;
        return item.role === (index % 2 === 0 ? 'user' : 'assistant') && typeof item.content === 'string' && item.content.trim();
      }
    )) throw new Error('Invalid saved conversation');
    result.messages = value.messages;
  }
  if ([8192, 16384, 32768].includes(value.contextTokens)) result.contextTokens = value.contextTokens;
  return result;
}
