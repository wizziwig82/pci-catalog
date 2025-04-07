<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { UploadItemMetadata } from '$lib/types/catalog';
  import WriterEditor from './WriterEditor.svelte';
  import PublisherEditor from './PublisherEditor.svelte';
  import type { PublisherEntry } from './PublisherEditor.svelte';

  // Props
  export let trackData: UploadItemMetadata; // Receive a copy of the track data

  // Local state derived from props for editing
  let localTrackData: UploadItemMetadata;
  $: localTrackData = JSON.parse(JSON.stringify(trackData)); // Deep copy to avoid modifying original
  $: if (localTrackData && !localTrackData.writers) {
      localTrackData.writers = []; // Ensure writers array exists for binding
  }
  $: if (localTrackData && !localTrackData.publishers) {
      localTrackData.publishers = []; // Ensure publishers array exists for binding
  }

  const dispatch = createEventDispatcher<{
    save: UploadItemMetadata;
    cancel: void;
  }>();

  function handleSave() {
    // TODO: Add validation if needed
    dispatch('save', localTrackData);
  }

  function handleCancel() {
    dispatch('cancel');
  }

  // TODO: Add logic for handling complex fields like writers/percentages, potentially using WriterEditor
  // TODO: Add TagSelector integration if needed for instruments/mood

</script>

<div class="individual-edit-panel">
  <h4>Edit Metadata: {localTrackData.title || localTrackData.original_path.split('/').pop()}</h4>
  <div class="individual-edit-form">
    <!-- Form fields will be moved here -->
    <div class="form-row">
      <div class="form-group">
        <label for="track-title-form">Title</label>
        <input id="track-title-form" type="text" bind:value={localTrackData.title} />
      </div>
      <div class="form-group">
        <label for="track-album-form">Album</label>
        <input id="track-album-form" type="text" bind:value={localTrackData.album} />
      </div>
    </div>
     <div class="form-row">
       <div class="form-group">
         <label for="track-artist-form">Artist</label>
         <input id="track-artist-form" type="text" bind:value={localTrackData.artist} />
       </div>
        <div class="form-group">
          <label for="track-genre-form">Genre</label>
          <input id="track-genre-form" type="text" bind:value={localTrackData.genre} placeholder="Single genre" />
        </div>
     </div>

     <!-- Placeholder for WriterEditor or direct writer inputs -->
     <div class="form-row">
        <div class="form-group form-group-tags">
            <label>Writers</label>
            {#if localTrackData.writers}
              <WriterEditor bind:writers={localTrackData.writers} />
            {:else}
              <!-- Optional: Show a loading or placeholder state if writers might be async loaded, though unlikely here -->
              <p>Initializing writer editor...</p>
            {/if}
        </div>
     </div>

     <!-- Add PublisherEditor component -->
     <div class="form-row">
        <div class="form-group form-group-tags">
            <label>Publishers</label>
            {#if localTrackData.publishers}
              <PublisherEditor bind:publishers={localTrackData.publishers} />
            {:else}
              <p>Initializing publisher editor...</p>
            {/if}
        </div>
     </div>

     <!-- Placeholder for other fields like comments, instruments, mood -->
      <div class="form-section">
        <div class="form-group">
          <label for="track-comments-form">Comments</label>
          <textarea id="track-comments-form" bind:value={localTrackData.comments} rows="3"></textarea>
        </div>
      </div>


    <div class="form-actions">
      <button class="save-button" on:click={handleSave}>Save Changes</button>
      <button class="cancel-button" on:click={handleCancel}>Back to List</button>
    </div>
  </div>
</div>

<style>
  /* Styles for the individual form will be moved or defined here */
  .individual-edit-panel {
    padding: 1rem;
    border: 1px solid #ccc;
  }
  .individual-edit-form .form-row {
      display: flex;
      gap: 1rem;
      margin-bottom: 1rem;
  }
  .individual-edit-form .form-group {
      flex: 1;
      display: flex;
      flex-direction: column;
  }
   .individual-edit-form .form-group label {
      margin-bottom: 0.25rem;
      font-weight: bold;
   }
   .individual-edit-form .form-group input[type="text"],
   .individual-edit-form .form-group textarea {
      padding: 0.5rem;
      border: 1px solid #ccc;
      border-radius: 4px;
   }
   .individual-edit-form .form-actions {
      margin-top: 1rem;
      display: flex;
      gap: 0.5rem;
   }
   /* Add more specific styles as needed */
</style>