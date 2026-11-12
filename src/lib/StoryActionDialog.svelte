<script lang="ts">
  import { onMount, tick } from 'svelte';
  let { mode, title, busy, error, onconfirm, onclose }: {
    mode: 'new' | 'rename' | 'delete'; title: string; busy: boolean; error: string;
    onconfirm: (name: string) => void; onclose: () => void;
  } = $props();
  let name = $state('');
  let dialog: HTMLDialogElement;
  let nameInput = $state<HTMLInputElement>();
  onMount(() => { name = mode === 'new' ? '' : title; dialog.showModal(); void tick().then(() => nameInput?.focus()); });
</script>

<dialog bind:this={dialog} aria-labelledby="story-dialog-title" oncancel={(event) => { event.preventDefault(); if (!busy) onclose(); }}>
  <form onsubmit={(event) => { event.preventDefault(); onconfirm(name); }}>
    <h2 id="story-dialog-title">{mode === 'new' ? 'New story' : mode === 'rename' ? 'Rename story' : 'Delete story?'}</h2>
    {#if mode === 'delete'}
      <p>Delete <strong>{title}</strong> and its draft, notes, and conversation? This cannot be undone.</p>
    {:else}
      <label for="story-name">Story title</label><input id="story-name" bind:this={nameInput} bind:value={name} maxlength="250" required disabled={busy} placeholder="Give your story a name" />
    {/if}
    {#if error}<p class="error" role="alert">{error}</p>{/if}
    <div class="dialog-actions"><button type="button" onclick={onclose} disabled={busy}>Cancel</button><button type="submit" class:primary={mode !== 'delete'} class:danger={mode === 'delete'} disabled={busy || (mode !== 'delete' && !name.trim())}>{busy ? 'Working…' : mode === 'new' ? 'Create story' : mode === 'rename' ? 'Rename' : 'Delete story'}</button></div>
  </form>
</dialog>
