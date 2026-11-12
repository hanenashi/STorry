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

export function parseDraft(value: unknown): Partial<Draft> {
  if (!value || typeof value !== 'object' || Array.isArray(value)) throw new Error('Invalid saved draft');
  const draft = value as Record<string, unknown>;
  const result: Partial<Draft> = {};
  for (const key of ['title', 'scene', 'notes', 'baseUrl', 'model', 'prompt'] as const) {
    if (draft[key] !== undefined) {
      if (typeof draft[key] !== 'string') throw new Error(`Invalid saved ${key}`);
      result[key] = draft[key];
    }
  }
  if (draft.messages !== undefined) {
    if (!Array.isArray(draft.messages) || draft.messages.length % 2 !== 0 || !draft.messages.every(
      (message: unknown, index: number) => {
        if (!message || typeof message !== 'object') return false;
        const item = message as ChatMessage;
        return item.role === (index % 2 === 0 ? 'user' : 'assistant') && typeof item.content === 'string' && item.content.trim();
      }
    )) throw new Error('Invalid saved conversation');
    result.messages = draft.messages;
  }
  if (draft.contextTokens !== undefined) {
    if (typeof draft.contextTokens !== 'number' || ![8192, 16384, 32768].includes(draft.contextTokens)) throw new Error('Invalid saved memory size');
    result.contextTokens = draft.contextTokens;
  }
  return result;
}

export function loadDraft(storage: Storage): Partial<Draft> {
  const saved = storage.getItem(STORAGE_KEY) ?? storage.getItem('storry.draft.v1');
  return saved ? parseDraft(JSON.parse(saved)) : {};
}

export function blankDraft(overrides: Partial<Draft> = {}): Draft {
  return { title: 'Untitled story', scene: '', notes: '', baseUrl: 'http://127.0.0.1:11434', model: '', prompt: '', messages: [], contextTokens: 16384, ...overrides };
}
