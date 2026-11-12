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

Click **Connect**, select an installed model, write a prompt, and generate a suggestion. Suggestions only enter the manuscript when you click **Insert at end**. The initial environment has `gemma3:4b` installed.

## Current skeleton

- One editable scene, title, word count, and story notes.
- Draft and connection settings autosaved in the webview's local storage (250 ms debounce).
- Rust-side Ollama model discovery and chat requests, with connection and generation errors.
- Separate suggestion preview, discard, and one-level undo of insertion.
- Markdown export through the webview download mechanism.

This is a development foundation. It does not yet implement SQLite projects, chapter management, durable revision history, streaming, cancellation, or selection-based rewriting. Generation waits for a complete response, with a five-minute timeout. The prompt contains the entire scene and story notes; there is no automatic context budgeting yet. Local storage is not a project file or backup; use export for copies. Suggestions and prompts are not persisted. Browser preview supports editing, but connecting to Ollama requires the desktop app.

## Layout

```text
src/routes/+page.svelte    Writing workspace and draft state
src/lib/ollama.ts          Typed Tauri command bridge
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
