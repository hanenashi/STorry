<script lang="ts">
  import type { StorySummary } from './library';
  let { stories, busy, available, error, onopen, onnew, onrename, ondelete, onimport, onrefresh }: {
    stories: StorySummary[]; busy: boolean; available: boolean; error: string;
    onopen: (id: string) => void; onnew: () => void;
    onrename: (story: StorySummary) => void; ondelete: (story: StorySummary) => void;
    onimport: (file: File) => void; onrefresh: () => void;
  } = $props();
  let search = $state('');
  let importInput: HTMLInputElement;
  const filtered = $derived(stories.filter(story => story.title.toLocaleLowerCase().includes(search.trim().toLocaleLowerCase())));
  function date(value: number) { return new Date(value).toLocaleString(undefined, { dateStyle: 'medium', timeStyle: 'short' }); }
</script>

<section class="library-screen" aria-label="Story library" aria-busy={busy}>
  <div class="library-heading"><div><h1>Your stories</h1><p class="hint">Pick up a story or start another.</p></div><div class="library-tools"><button onclick={() => importInput.click()} disabled={busy || !available}>Import story</button><button class="primary" onclick={onnew} disabled={busy || !available}>New story</button></div></div>
  <input class="file-input" type="file" accept=".json,.storry.json" aria-label="Import story file" bind:this={importInput} onchange={() => { const file = importInput.files?.[0]; if (file) onimport(file); importInput.value = ''; }} />
  <div class="library-filter"><input aria-label="Search stories" placeholder="Search stories…" bind:value={search} /><span>{stories.length} {stories.length === 1 ? 'story' : 'stories'}</span><button onclick={onrefresh} disabled={busy}>Refresh</button></div>
  {#if error}<p class="error" role="alert">{error}</p>{/if}
  {#if !stories.length}
    <div class="library-empty"><h2>A fresh page awaits.</h2><p>Create your first story, or import a saved STorry file.</p></div>
  {:else if !filtered.length}
    <p class="hint">No stories match your search.</p>
  {:else}
    <ul class="story-list">
      {#each filtered as story (story.id)}
        <li class="story-row">
          <div class="story-info"><h2>{story.title}</h2><p class="story-excerpt">{story.excerpt || 'No writing yet.'}</p><p class="hint">{story.words} words · {story.turns} {story.turns === 1 ? 'reply' : 'replies'} · Saved {date(story.updatedAt)}</p></div>
          <div class="story-actions"><button class="primary" onclick={() => onopen(story.id)} disabled={busy} aria-label={`Open ${story.title}`}>Open</button><button onclick={() => onrename(story)} disabled={busy} aria-label={`Rename ${story.title}`}>Rename</button><button class="danger-text" onclick={() => ondelete(story)} disabled={busy} aria-label={`Delete ${story.title}`}>Delete</button></div>
        </li>
      {/each}
    </ul>
  {/if}
  <p class="hint library-footnote">Stories save automatically on this device. Use “Save a copy” inside a story to keep a portable backup.</p>
</section>
