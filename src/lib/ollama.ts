import { invoke, isTauri } from '@tauri-apps/api/core';

export interface Model { name: string }
export interface ChatMessage { role: 'user' | 'assistant'; content: string }
export interface GenerateRequest {
  baseUrl: string;
  model: string;
  prompt: string;
  scene: string;
  notes: string;
  history: ChatMessage[];
  contextTokens: number;
}
function requireDesktop() {
  if (!isTauri()) throw new Error('Open the desktop app with npm run tauri dev to connect to Ollama.');
}
export async function listModels(baseUrl: string): Promise<Model[]> {
  requireDesktop();
  return invoke('list_models', { baseUrl });
}
export async function generate(request: GenerateRequest): Promise<string> {
  requireDesktop();
  return invoke('generate', { ...request });
}
