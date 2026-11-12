# STorry development

- Stack: Tauri 2 + Rust + Svelte 5 + TypeScript. SvelteKit builds a static SPA; no Node server is shipped.
- Keep Ollama networking in Rust and the typed frontend bridge in `src/lib/ollama.ts`.
- Never silently replace manuscript text with generated content. Keep suggestions separate until accepted.
- Preserve existing user drafts. Schema changes require migration or an explicit recovery path.
- Keep this app local by default. Do not change Ollama's installation, model directory, or service settings.
- Run `npm run check`, `npm run build`, and relevant Rust checks for code changes.
- Do not require a running model for the default test suite. Live inference checks must be opt-in.
