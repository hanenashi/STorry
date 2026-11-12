<script lang="ts">
  import { onMount } from 'svelte';
  import { generate, listModels, type Model } from '$lib/ollama';
  import '../app.css';
  let title = $state('Untitled story');
  let scene = $state('');
  let notes = $state('');
  let baseUrl = $state('http://127.0.0.1:11434');
  let model = $state('');
  let prompt = $state('');
  let suggestion = $state('');
  let previousScene = $state<string | null>(null);
  let models = $state<Model[]>([]);
  let status = $state('Connect to Ollama to load your models.');
  let error = $state('');
  let connecting = $state(false);
  let generating = $state(false);
  let ready = $state(false);
  let saveStatus = $state('Draft stored on this device');
  const words = $derived(scene.trim() ? scene.trim().split(/\s+/).length : 0);

  onMount(() => {
    try {
      const stored = localStorage.getItem('storry.draft.v1');
      if (stored) {
        const draft = JSON.parse(stored);
        if (typeof draft.title === 'string') title = draft.title;
        if (typeof draft.scene === 'string') scene = draft.scene;
        if (typeof draft.notes === 'string') notes = draft.notes;
        if (typeof draft.baseUrl === 'string') baseUrl = draft.baseUrl;
        if (typeof draft.model === 'string') model = draft.model;
      }
    } catch { saveStatus = 'Saved draft could not be loaded'; }
    ready = true;
  });
  $effect(() => {
    const draft = JSON.stringify({ title, scene, notes, baseUrl, model });
    if (ready) {
      const timer = setTimeout(() => {
        try { localStorage.setItem('storry.draft.v1', draft); saveStatus = 'Draft stored on this device'; }
        catch { saveStatus = 'Draft could not be saved — export a copy'; }
      }, 250);
      return () => clearTimeout(timer);
    }
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
  async function ask() {
    generating = true; error = '';
    try { suggestion = await generate({ baseUrl, model, prompt, scene, notes }); }
    catch (cause) { error = String(cause); }
    finally { generating = false; }
  }
  function insert() {
    previousScene = scene;
    scene = [scene.trimEnd(), suggestion].filter(Boolean).join('\n\n');
    suggestion = '';
  }
  function exportStory() {
    const blob = new Blob([`# ${title}\n\n${scene}\n`], { type: 'text/markdown;charset=utf-8' });
    const url = URL.createObjectURL(blob);
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
    <label for="title">Title</label><input id="title" bind:value={title} />
    <label for="notes">Story notes</label>
    <p class="hint">Characters, setting, tone, and things to remember.</p>
    <textarea id="notes" bind:value={notes} placeholder="Who is this story about?" class="notes"></textarea>
    <p class="hint">This first skeleton keeps one story and one scene on this device.</p>
  </aside>
  <section class="manuscript" aria-label="Manuscript">
    <div class="section-header"><h1>Scene one</h1><span>{words} words</span></div>
    <textarea aria-label="Scene text" class="scene" bind:value={scene} placeholder="Every story starts somewhere…"></textarea>
    <footer><span role="status">{saveStatus}</span>
      {#if previousScene !== null}<button onclick={() => { scene = previousScene ?? ''; previousScene = null; }}>Undo last insertion</button>{/if}
    </footer>
  </section>
  <aside class="assistant-panel">
    <h2>Writing assistant</h2>
    <label for="server">Ollama server</label>
    <div class="connection"><input id="server" bind:value={baseUrl} disabled={connecting || generating} /><button onclick={connect} disabled={connecting || generating}>{connecting ? 'Connecting…' : 'Connect'}</button></div>
    <p class="hint" role="status">{status}</p>
    <label for="model">Model</label>
    <select id="model" bind:value={model} disabled={!models.length || generating}>
      {#if !models.length}<option value="">Connect to load models</option>{/if}
      {#each models as entry}<option value={entry.name}>{entry.name}</option>{/each}
    </select>
    <label for="prompt">What would you like to write?</label>
    <textarea id="prompt" bind:value={prompt} placeholder="Continue this scene with a quiet sense of unease…" rows="4"></textarea>
    <p class="hint">Includes your current scene and story notes.</p>
    <button class="primary" onclick={ask} disabled={generating || connecting || !model || !prompt.trim()}>{generating ? 'Writing…' : 'Generate suggestion'}</button>
    {#if error}<p class="error" role="alert">{error}</p>{/if}
    <section class="suggestion" aria-label="Suggestion" aria-busy={generating}>
      <h3>Suggestion</h3>
      {#if suggestion}
        <div class="prose">{suggestion}</div>
        <div class="actions"><button class="primary" onclick={insert} disabled={generating}>Insert at end</button><button onclick={() => suggestion = ''} disabled={generating}>Discard</button></div>
      {:else}<p class="hint">Your next possibility will appear here. Insert it when it feels right.</p>{/if}
    </section>
  </aside>
</main>
