<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { generate, listModels, type Model, type ChatMessage } from '$lib/ollama';
  import { loadDraft, STORAGE_KEY } from '$lib/draft';
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
  let saveStatus = $state('Saved on this device');
  let view = $state<'chat' | 'draft'>('chat');
  let chatEnd = $state<HTMLDivElement>();
  const words = $derived([scene, ...messages.filter(m => m.role === 'assistant').map(m => m.content)].join(' ').trim().split(/\s+/).filter(Boolean).length);

  onMount(() => {
    try {
      const draft = loadDraft(localStorage);
      title = draft.title ?? title;
      scene = draft.scene ?? scene;
      notes = draft.notes ?? notes;
      baseUrl = draft.baseUrl ?? baseUrl;
      model = draft.model ?? model;
      prompt = draft.prompt ?? prompt;
      messages = draft.messages ?? messages;
      contextTokens = draft.contextTokens ?? contextTokens;
    } catch {
      canSave = false;
      saveStatus = 'Saved data could not be loaded. It has been kept intact; autosave is paused.';
    }
    ready = true;
    void connect();
    void scrollToLatest();
  });

  function persist() {
    if (!canSave) return;
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify({ title, scene, notes, baseUrl, model, prompt, messages, contextTokens }));
      saveStatus = 'Saved on this device';
    } catch { saveStatus = 'Could not save — export a copy before closing'; }
  }
  $effect(() => {
    // Save completed turns and the current prompt. The v1 draft remains as a migration backup.
    if (ready) persist();
  });

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
    if (generating || connecting || !model || !prompt.trim() || !canSave) return;
    const request = prompt.trim();
    generating = true; error = ''; pendingPrompt = request;
    view = 'chat';
    await scrollToLatest();
    try {
      const reply = await generate({ baseUrl, model, prompt: request, scene, notes, history: messages, contextTokens });
      messages = [...messages, { role: 'user', content: request }, { role: 'assistant', content: reply }];
      prompt = '';
      persist();
    } catch (cause) { error = String(cause); }
    finally { generating = false; pendingPrompt = ''; await scrollToLatest(); }
  }

  function exportStory() {
    const conversation = messages.map(m => `## ${m.role === 'user' ? 'You' : 'STorry'}\n\n${m.content}`).join('\n\n');
    const content = [`# ${title}`, scene, conversation].filter(Boolean).join('\n\n');
    const url = URL.createObjectURL(new Blob([content + '\n'], { type: 'text/markdown;charset=utf-8' }));
    const link = document.createElement('a');
    link.href = url;
    link.download = `${title.replace(/[^a-zA-Z0-9 _-]/g, '_').trim() || 'story'}.md`;
    link.click();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
  }
</script>

<svelte:head><title>STorry — your writing space</title></svelte:head>
<header class="app-header">
  <strong class="brand">STorry<span>your writing space</span></strong>
  <button onclick={exportStory}>Export Markdown</button>
</header>
<main>
  <aside class="notes-panel">
    <h2>Story</h2>
    <label for="title">Title</label><input id="title" bind:value={title} disabled={generating || !canSave} />
    <label for="notes">Story notes</label>
    <p class="hint">Keep characters, setting, and lasting rules here. Included with every message.</p>
    <textarea id="notes" bind:value={notes} disabled={generating || !canSave} placeholder="Who is this story about?" class="notes"></textarea>
    <p class="hint">Your conversation, notes, and opening draft are saved together on this device.</p>
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
      <textarea aria-label="Scene text" class="scene" bind:value={scene} disabled={generating || !canSave} placeholder="Every story starts somewhere…"></textarea>
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
        <textarea id="prompt" bind:value={prompt} disabled={generating || !canSave} placeholder="Continue, but have her recognize the stranger. Keep it understated." rows="3" onkeydown={(event) => {
          if (event.key === 'Enter' && (event.ctrlKey || event.metaKey) && !event.isComposing) { event.preventDefault(); void ask(); }
        }}></textarea>
        <div class="composer-actions"><span class="hint">Remembers earlier turns · Ctrl+Enter to send</span><button type="submit" class="primary" disabled={generating || connecting || !model || !prompt.trim() || !canSave}>{generating ? 'Writing…' : 'Send'}</button></div>
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
