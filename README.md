# STorry

A small Windows 11 writing app using Tauri 2, Rust, Svelte 5, TypeScript, and SvelteKit's static adapter.

## Run

Use the **STorry** desktop shortcut or double-click `Launch-STorry.cmd` in this repository to open the locally built app without a development server. Both use `Launch-STorry.ps1`. After changing application code, refresh that desktop build with `npm run tauri build -- --debug --no-bundle`.

For development with live updates:

```powershell
cd C:\GIT\STorry
npm install
npm run tauri dev
```

Requires Node.js, Rust's MSVC toolchain, Visual Studio C++ Build Tools, and WebView2. The first Rust build takes longer while dependencies compile.

Ollama is installed separately at `C:\AI\Ollama\ollama.exe` on this machine. STorry connects to `http://127.0.0.1:11434`; it does not move models or change Ollama settings. If the server is stopped, run the following in a separate terminal:

```powershell
& 'C:\AI\Ollama\ollama.exe' serve
```

STorry connects automatically on launch. Select a model, write a prompt, and click **Send** (or press Ctrl+Enter). Replies append directly to the story conversation. Ask for the next passage or a change in tone, events, or characters; each request includes the previous conversation. The initial environment has `gemma3:4b` installed.

## Current skeleton

- A persistent story conversation with user prompts and assistant replies.
- An editable **Opening draft**, title, word count, and story notes.
- Conversation, draft, current prompt, memory size, and connection settings autosaved in the webview's local storage.
- Rust-side Ollama model discovery and chat requests, with connection and generation errors.
- Full ordered conversation history sent to Ollama, alongside the current opening draft and notes.
- 8K / 16K / 32K context settings (16K default), with a conservative byte-based input budget and a 2K-token output allowance. Oversized input is rejected with an actionable error instead of silently dropping old turns. This is not exact token counting; it intentionally errs on the side of keeping history intact. The model must support the chosen context size.
- Markdown export of the opening draft and conversation, with speaker labels.
- Failed requests keep the prompt ready to retry without adding a false assistant reply.

This is a development foundation. It does not yet implement SQLite projects, chapter management, durable revision history, streaming, cancellation, automatic memory summaries, or selection-based rewriting. Generation waits for a complete response, with a five-minute timeout. Local storage is not a project file or backup; use export for copies. Browser preview supports editing, but connecting to Ollama requires the desktop app.

Existing `storry.draft.v1` data migrates to `storry.draft.v2` without deleting the original. The old scene remains editable in **Opening draft** and is shown at the start of the conversation. If saved data is invalid, autosave pauses rather than overwriting it. Story notes are useful for lasting facts and changes that must remain explicit; model recall still depends on the chosen model.

## Layout

```text
src/routes/+page.svelte    Writing workspace and draft state
src/lib/ollama.ts          Typed Tauri command bridge
src/lib/draft.ts           Saved conversation validation and v1 migration
src/app.css                Shared workspace styles
src-tauri/src/ollama.rs    Ollama HTTP client and commands
src-tauri/src/lib.rs       Desktop application entry point
```

## Checks and builds

```powershell
npm run check
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
# Optional: exercises the real local model and consumes inference resources
cargo test --manifest-path src-tauri/Cargo.toml live_ollama_roundtrip -- --ignored
npm run tauri build
```

`npm run dev` is a browser-only preview. `npm run tauri dev` starts the desktop app. Lockfiles are committed for reproducible dependency resolution.

## Next steps

1. SQLite projects, scenes, story notes, and revision snapshots.
2. Streaming generation with cancellation.
3. Selection-based rewrites and explicit context selection.
4. Windows installer branding and release pipeline.
