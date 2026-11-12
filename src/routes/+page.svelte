<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { isTauri } from '@tauri-apps/api/core';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { generate, listModels, type Model, type ChatMessage } from '$lib/ollama';
  import { blankDraft, loadDraft, parseDraft, type Draft } from '$lib/draft';
  import { initializeLibrary, listStories, loadStory, createStory, saveStory, deleteStory, type Story, type StorySummary } from '$lib/library';
  import StoryLibrary from '$lib/StoryLibrary.svelte';
  import StoryActionDialog from '$lib/StoryActionDialog.svelte';
  import '../app.css';
  let title = $state('Untitled story');
  let scene = $state('');
  let notes = $state('');
  let baseUrl = $state('http://127.0.0.1:11434');
  let model = $state('');
  let prompt = $state('');
  let messages = $state<ChatMessage[]>([]);
  let pendingPrompt = $state('');
  let contextTokens = $state(16384);
  let models = $state<Model[]>([]);
  let status = $state('Connecting to Ollama…');
  let error = $state('');
  let connecting = $state(false);
  let generating = $state(false);
  let ready = $state(false);
  let canSave = $state(true);
  let saveStatus = $state('Saved to library');
  let screen = $state<'library' | 'editor'>('library');
  let stories = $state<StorySummary[]>([]);
  let libraryError = $state('');
  let transitioning = $state(false);
  let activeId = $state<string | null>(null);
  let revision = 0;
  let savedBaseline = $state('');
  let savePromise: Promise<boolean> | undefined;
  let dialog = $state<{ mode: 'new' | 'rename' | 'delete'; story?: StorySummary } | null>(null);
  let dialogError = $state('');
  let closing = false;
  const busy = $derived(generating || transitioning);
  let view = $state<'chat' | 'draft'>('chat');
  let chatEnd = $state<HTMLDivElement>();
  const words = $derived([scene, ...messages.filter(m => m.role === 'assistant').map(m => m.content)].join(' ').trim().split(/\s+/).filter(Boolean).length);

  function currentDraft(): Draft { return { title, scene, notes, baseUrl, model, prompt, messages, contextTokens }; }

  onMount(() => {
    void bootstrap();
    let disposed = false;
    let unlisten: (() => void) | undefined;
    if (isTauri()) {
      void getCurrentWindow().onCloseRequested(async (event) => {
        event.preventDefault();
        if (closing) return;
        closing = true; transitioning = true;
        try {
          if (activeId && !await saveActive()) return;
          await getCurrentWindow().destroy();
        } catch (cause) { error = String(cause); }
        finally { closing = false; transitioning = false; }
      }).then(stop => { if (disposed) stop(); else unlisten = stop; })
        .catch(cause => { libraryError = `Could not enable save-on-close: ${String(cause)}`; });
    }
    return () => { disposed = true; unlisten?.(); };
  });
  async function bootstrap() {
    transitioning = true;
    try {
      const draft = loadDraft(localStorage);
      baseUrl = draft.baseUrl ?? baseUrl;
      model = draft.model ?? model;
      contextTokens = draft.contextTokens ?? contextTokens;
      const hasLegacy = !!(localStorage.getItem('storry.draft.v2') ?? localStorage.getItem('storry.draft.v1'));
      stories = await initializeLibrary(hasLegacy ? blankDraft(draft) : null);
      ready = true;
    } catch (cause) { libraryError = `Could not load your library. Existing data has been kept intact. ${String(cause)}`; }
    finally { transitioning = false; }
    void connect();
  }

  function applyStory(story: Story) {
    const data = story.data;
    activeId = story.id; revision = story.revision;
    title = data.title; scene = data.scene; notes = data.notes; prompt = data.prompt;
    messages = data.messages; baseUrl = data.baseUrl; model = data.model; contextTokens = data.contextTokens;
    savedBaseline = JSON.stringify(currentDraft());
    saveStatus = 'Saved to library'; error = ''; view = 'chat'; screen = 'editor';
  }
  function saveActive(): Promise<boolean> {
    if (savePromise) return savePromise.then(ok => ok ? saveActive() : false);
    if (!activeId || !ready || !canSave) return Promise.resolve(false);
    const id = activeId;
    const snapshot = JSON.stringify(currentDraft());
    if (snapshot === savedBaseline) return Promise.resolve(true);
    saveStatus = 'Saving…';
    savePromise = (async () => {
      try {
        const saved = await saveStory(id, revision, JSON.parse(snapshot));
        revision = saved.revision; savedBaseline = snapshot; saveStatus = 'Saved to library';
        return true;
      } catch (cause) {
        saveStatus = `Could not save: ${String(cause)} Use “Save a copy” before closing.`;
        return false;
      }
    })();
    return savePromise.finally(() => { savePromise = undefined; });
  }
  $effect(() => {
    const snapshot = JSON.stringify(currentDraft());
    if (ready && activeId && canSave && snapshot !== savedBaseline) {
      saveStatus = 'Unsaved changes';
      const timer = setTimeout(() => { void saveActive(); }, 400);
      return () => clearTimeout(timer);
    }
  });

  async function openStory(id: string) {
    if (busy) return;
    transitioning = true; libraryError = '';
    try { applyStory(await loadStory(id)); void connect(); await scrollToLatest(); }
    catch (cause) { libraryError = String(cause); }
    finally { transitioning = false; }
  }
  async function showLibrary() {
    if (busy) return;
    transitioning = true;
    try {
      if (!await saveActive()) return;
      stories = await listStories(); activeId = null; screen = 'library'; libraryError = '';
    } catch (cause) { error = String(cause); }
    finally { transitioning = false; }
  }
  async function refreshLibrary() {
    if (busy) return;
    if (!ready) { await bootstrap(); return; }
    transitioning = true;
    try { stories = await listStories(); libraryError = ''; }
    catch (cause) { libraryError = String(cause); }
    finally { transitioning = false; }
  }
  function showDialog(mode: 'new' | 'rename' | 'delete', story?: StorySummary) { dialogError = ''; dialog = { mode, story }; }
  async function confirmDialog(name: string) {
    if (!dialog || busy) return;
    transitioning = true; dialogError = '';
    try {
      if (dialog.mode === 'new') {
        const created = await createStory(blankDraft({ title: name.trim(), baseUrl, model, contextTokens }));
        applyStory(created); dialog = null; await scrollToLatest();
      } else if (dialog.story) {
        if (dialog.mode === 'delete') await deleteStory(dialog.story.id, dialog.story.revision);
        else {
          const existing = await loadStory(dialog.story.id);
          await saveStory(existing.id, existing.revision, { ...existing.data, title: name.trim() });
        }
        stories = await listStories(); dialog = null;
      }
    } catch (cause) { dialogError = String(cause); }
    finally { transitioning = false; }
  }
  function download(content: string, suffix: string, type: string) {
    const url = URL.createObjectURL(new Blob([content], { type }));
    const link = document.createElement('a'); link.href = url;
    link.download = `${title.replace(/[^a-zA-Z0-9 _-]/g, '_').trim() || 'story'}${suffix}`;
    link.click(); setTimeout(() => URL.revokeObjectURL(url), 1000);
  }
  function saveCopy() { download(JSON.stringify({ format: 'storry.story', version: 1, data: currentDraft() }, null, 2), '.storry.json', 'application/json'); }
  async function importStory(file: File) {
    if (busy) return;
    transitioning = true; libraryError = '';
    try {
      if (file.size > 10 * 1024 * 1024) throw new Error('Story files must be smaller than 10 MB.');
      const value = JSON.parse(await file.text());
      if (value?.format !== 'storry.story' || value.version !== 1) throw new Error('Choose a STorry story file (.storry.json).');
      const parsed = parseDraft(value.data);
      if (!parsed.title?.trim() || !Array.isArray(parsed.messages)) throw new Error('This story file is incomplete.');
      applyStory(await createStory(blankDraft(parsed)));
      void connect(); await scrollToLatest();
    } catch (cause) { libraryError = String(cause); }
    finally { transitioning = false; }
  }

  async function connect() {
    connecting = true; error = ''; models = [];
    try {
      models = await listModels(baseUrl);
      if (!models.some((entry) => entry.name === model)) model = models[0]?.name ?? '';
      status = models.length ? 'Connected to Ollama' : 'Connected. Download a model in Ollama, then reconnect.';
    } catch (cause) { model = ''; status = 'Ollama unavailable'; error = String(cause); }
    finally { connecting = false; }
  }

  async function scrollToLatest() {
    await tick();
    chatEnd?.scrollIntoView({ block: 'nearest', behavior: 'instant' });
  }
  async function ask() {
    if (busy || connecting || !model || !prompt.trim() || !canSave) return;
    const request = prompt.trim();
    generating = true; error = ''; pendingPrompt = request;
    view = 'chat';
    await scrollToLatest();
    try {
      if (!await saveActive()) { error = 'Save your current story before generating. See the save status below.'; return; }
      const reply = await generate({ baseUrl, model, prompt: request, scene, notes, history: messages, contextTokens });
      if (closing) return;
      messages = [...messages, { role: 'user', content: request }, { role: 'assistant', content: reply }];
      prompt = '';
      await saveActive();
    } catch (cause) { error = String(cause); }
    finally { generating = false; pendingPrompt = ''; await scrollToLatest(); }
  }

  function exportStory() {
    const conversation = messages.map(m => `## ${m.role === 'user' ? 'You' : 'STorry'}\n\n${m.content}`).join('\n\n');
    const content = [`# ${title}`, scene, conversation].filter(Boolean).join('\n\n');
    download(content + '\n', '.md', 'text/markdown;charset=utf-8');
  }
</script>

<svelte:head><title>STorry — your writing space</title></svelte:head>
<header class="app-header">
  <strong class="brand">STorry<span>your writing space</span></strong>
  {#if screen === 'editor'}<nav class="header-actions" aria-label="Story actions"><button onclick={showLibrary} disabled={busy}>Stories</button><button onclick={() => { void saveActive(); }} disabled={busy}>Save</button><button onclick={saveCopy}>Save a copy</button><button onclick={exportStory}>Export Markdown</button></nav>{/if}
</header>
{#if screen === 'library'}
  <StoryLibrary {stories} {busy} available={ready} error={libraryError} onopen={openStory} onnew={() => showDialog('new')} onrename={(story) => showDialog('rename', story)} ondelete={(story) => showDialog('delete', story)} onimport={importStory} onrefresh={refreshLibrary} />
{:else}
<main>
  <aside class="notes-panel">
    <h2>Story</h2>
    <label for="title">Title</label><input id="title" bind:value={title} maxlength="250" disabled={busy || !canSave} />
    <label for="notes">Story notes</label>
    <p class="hint">Keep characters, setting, and lasting rules here. Included with every message.</p>
    <textarea id="notes" bind:value={notes} disabled={busy || !canSave} placeholder="Who is this story about?" class="notes"></textarea>
    <p class="hint">This story’s conversation, notes, and opening draft save together in your library.</p>
  </aside>

  <section class="workspace" aria-label="Story workspace">
    <div class="section-header">
      <div class="view-tabs" aria-label="Workspace view">
        <button class:active={view === 'chat'} aria-pressed={view === 'chat'} onclick={() => { view = 'chat'; void scrollToLatest(); }}>Story conversation</button>
        <button class:active={view === 'draft'} aria-pressed={view === 'draft'} onclick={() => view = 'draft'}>Opening draft</button>
      </div>
      <span>{words} words</span>
    </div>
    {#if view === 'draft'}
      <p class="hint">Your existing writing stays editable here and is included in the conversation.</p>
      <textarea aria-label="Scene text" class="scene" bind:value={scene} disabled={busy || !canSave} placeholder="Every story starts somewhere…"></textarea>
    {:else}
      <div class="chat-history" role="log" aria-label="Story conversation" aria-busy={generating}>
        {#if scene}<article class="chat-message opening"><h3>Opening draft</h3><div class="prose">{scene}</div></article>{/if}
        {#if !messages.length && !generating}
          {#if scene}
            <p class="hint">Your draft is ready. Ask for the next passage or tell STorry what to change.</p>
          {:else}
            <div class="chat-empty"><h1>Where does your story begin?</h1><p>Start a scene or explore an idea. Then ask for twists, changes, or the next passage.</p></div>
          {/if}
        {/if}
        {#each messages as message, index (index)}
          <article class="chat-message" class:user-message={message.role === 'user'}>
            <h3>{message.role === 'user' ? 'You' : 'STorry'}</h3><div class="prose">{message.content}</div>
          </article>
        {/each}
        {#if generating}
          <article class="chat-message user-message"><h3>You</h3><div class="prose">{pendingPrompt}</div></article>
          <p class="writing-status" role="status">STorry is writing…</p>
        {/if}
        <div bind:this={chatEnd}></div>
      </div>
      <form class="composer" onsubmit={(event) => { event.preventDefault(); void ask(); }}>
        <label for="prompt">Continue the story or suggest a change</label>
        <textarea id="prompt" bind:value={prompt} disabled={busy || !canSave} placeholder="Continue, but have her recognize the stranger. Keep it understated." rows="3" onkeydown={(event) => {
          if (event.key === 'Enter' && (event.ctrlKey || event.metaKey) && !event.isComposing) { event.preventDefault(); void ask(); }
        }}></textarea>
        <div class="composer-actions"><span class="hint">Remembers earlier turns · Ctrl+Enter to send</span><button type="submit" class="primary" disabled={busy || connecting || !model || !prompt.trim() || !canSave}>{generating ? 'Writing…' : 'Send'}</button></div>
        {#if error}<p class="error" role="alert">{error}</p>{/if}
      </form>
    {/if}
    <footer><span role="status">{saveStatus}</span></footer>
  </section>

  <aside class="assistant-panel">
    <h2>Ollama</h2>
    <label for="server">Server</label>
    <div class="connection"><input id="server" bind:value={baseUrl} disabled={connecting || generating} /><button onclick={connect} disabled={connecting || generating}>{connecting ? 'Connecting…' : 'Connect'}</button></div>
    <p class="hint" role="status">{status}</p>
    <label for="model">Model</label>
    <select id="model" bind:value={model} disabled={!models.length || generating}>
      {#if !models.length}<option value="">Connect to load models</option>{/if}
      {#each models as entry}<option value={entry.name}>{entry.name}</option>{/each}
    </select>
    <label for="memory">Memory size</label>
    <select id="memory" bind:value={contextTokens} disabled={generating}>
      <option value={8192}>8K tokens</option><option value={16384}>16K tokens</option><option value={32768}>32K tokens</option>
    </select>
    <p class="hint">Larger memory fits more story but uses more RAM or VRAM. Earlier turns are sent with each request; if they no longer fit, STorry asks you to increase memory.</p>
    {#if view === 'draft' && error}<p class="error" role="alert">{error}</p>{/if}
  </aside>
</main>
{/if}
{#if dialog}
  <StoryActionDialog mode={dialog.mode} title={dialog.story?.title ?? ''} {busy} error={dialogError} onconfirm={confirmDialog} onclose={() => dialog = null} />
{/if}
