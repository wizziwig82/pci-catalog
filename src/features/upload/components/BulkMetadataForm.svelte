<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import type { UploadItemMetadata } from '$lib/types/catalog';
  import WriterEditor from './WriterEditor.svelte';
  import PublisherEditor from './PublisherEditor.svelte';
  import type { PublisherEntry } from './PublisherEditor.svelte';

  // Define the structure for bulk edit fields
  interface BulkEditFields {
    album?: string | null; // Use null to indicate "no change" or "mixed" initially
    artist?: string | null;
    genre?: string | null;
    writers?: { name: string; percentage: number }[] | null; // Use the new structure
    publishers?: PublisherEntry[] | null; // Use the PublisherEntry type
    // Add other bulk-editable fields (instruments, mood, etc.) as needed
    // Example: instruments?: string | null;
    // Example: mood?: string | null;
  }

  // Props
  export let selectedItems: UploadItemMetadata[] = []; // Receive the actual selected items

  // State
  let bulkEditFields: BulkEditFields = {};
  let modifiedFields: Set<keyof BulkEditFields> = new Set(); // Track which fields user touched

  const dispatch = createEventDispatcher<{
    apply: { bulkData: BulkEditFields; modifiedFields: Set<keyof BulkEditFields> };
    cancel: void;
  }>();

  onMount(() => {
    initializeBulkFields();
  });

  // Reactive update if selectedItems change (might be needed depending on parent logic)
  $: if (selectedItems.length > 0) {
      initializeBulkFields();
  }

  function initializeBulkFields() {
    modifiedFields.clear(); // Reset modified tracking
    if (selectedItems.length === 0) {
      bulkEditFields = {};
      return;
    }

    const firstItem = selectedItems[0];
    let commonAlbum: string | null = firstItem.album ?? null;
    let commonArtist: string | null = firstItem.artist ?? null;
    let commonGenre: string | null = firstItem.genre ?? null;
    // Initialize writers as null (mixed) by default for bulk edit
    let commonWriters: { name: string; percentage: number }[] | null = null;
    // Initialize publishers as null (mixed) by default for bulk edit
    let commonPublishers: PublisherEntry[] | null = null;

    for (let i = 1; i < selectedItems.length; i++) {
      const item = selectedItems[i];
      if (commonAlbum !== (item.album ?? null)) commonAlbum = null;
      if (commonArtist !== (item.artist ?? null)) commonArtist = null;
      if (commonGenre !== (item.genre ?? null)) commonGenre = null;
      
      // Simple check for writers: if any item has different writers (or none vs some), mark as mixed
      if (JSON.stringify(firstItem.writers ?? []) !== JSON.stringify(item.writers ?? [])) {
          commonWriters = null;
      } else if (i === 1 && firstItem.writers) {
          // If the first two match and have writers, initialize with the first item's writers
          commonWriters = JSON.parse(JSON.stringify(firstItem.writers));
      }
      
      // Simple check for publishers: if any item has different publishers (or none vs some), mark as mixed
      if (JSON.stringify(firstItem.publishers ?? []) !== JSON.stringify(item.publishers ?? [])) {
          commonPublishers = null;
      } else if (i === 1 && firstItem.publishers) {
          // If the first two match and have publishers, initialize with the first item's publishers
          commonPublishers = JSON.parse(JSON.stringify(firstItem.publishers));
      }
    }

     // If writers remained consistent across all items, use the common list. Otherwise, it stays null.
     if (commonWriters === null && selectedItems.length > 0 && firstItem.writers && selectedItems.every(item => JSON.stringify(item.writers ?? []) === JSON.stringify(firstItem.writers ?? []))) {
         commonWriters = JSON.parse(JSON.stringify(firstItem.writers));
     }

     // If publishers remained consistent across all items, use the common list. Otherwise, it stays null.
     if (commonPublishers === null && selectedItems.length > 0 && firstItem.publishers && selectedItems.every(item => JSON.stringify(item.publishers ?? []) === JSON.stringify(firstItem.publishers ?? []))) {
         commonPublishers = JSON.parse(JSON.stringify(firstItem.publishers));
     }

    bulkEditFields = {
      album: commonAlbum,
      artist: commonArtist,
      genre: commonGenre,
      writers: commonWriters, // Initialize based on commonality check
      publishers: commonPublishers, // Initialize based on commonality check
      // Initialize other fields...
    };
  }

  function handleInput(fieldName: keyof BulkEditFields) {
    modifiedFields.add(fieldName);
  }

  function handleApply() {
    // TODO: Add validation if needed (e.g., writer percentages sum to 100)
    dispatch('apply', { bulkData: bulkEditFields, modifiedFields });
  }

  function handleCancel() {
    dispatch('cancel');
  }

  // TODO: Add logic for handling complex fields like writers/percentages, potentially using WriterEditor
  // TODO: Add TagSelector integration if needed

</script>

<div class="bulk-edit-panel">
  <h4>Bulk Edit Selected Tracks ({selectedItems.length} selected)</h4>
  <div class="bulk-edit-form">
    <!-- Form fields will be moved here -->
    <div class="form-row">
      <div class="form-group">
        <label for="bulk-album-form">Album</label>
        <input
          id="bulk-album-form"
          type="text"
          bind:value={bulkEditFields.album}
          placeholder={bulkEditFields.album === null ? '(Mixed Values)' : 'Enter Album'}
          on:input={() => handleInput('album')}
        />
         <span class="bulk-hint">(Leave blank to keep original values if mixed)</span>
      </div>
      <div class="form-group">
        <label for="bulk-artist-form">Artist</label>
        <input
          id="bulk-artist-form"
          type="text"
          bind:value={bulkEditFields.artist}
          placeholder={bulkEditFields.artist === null ? '(Mixed Values)' : 'Enter Artist'}
          on:input={() => handleInput('artist')}
        />
         <span class="bulk-hint">(Leave blank to keep original values if mixed)</span>
      </div>
    </div>
     <div class="form-row">
        <div class="form-group">
          <label for="bulk-genre-form">Genre</label>
          <input
            id="bulk-genre-form"
            type="text"
            bind:value={bulkEditFields.genre}
            placeholder={bulkEditFields.genre === null ? '(Mixed Values)' : 'Enter Genre'}
            on:input={() => handleInput('genre')}
          />
           <span class="bulk-hint">(Leave blank to keep original values if mixed)</span>
        </div>
     </div>

     <!-- Placeholder for WriterEditor or direct writer inputs -->
     <div class="form-row">
        <div class="form-group form-group-tags">
            <label>Writers</label>
            {#if bulkEditFields.writers === null}
                <div class="mixed-value-notice">
                    <p>(Mixed Values)</p>
                    <button class="edit-overwrite-button" type="button" on:click={() => { bulkEditFields.writers = []; handleInput('writers'); }}>
                        Edit Writers (Will Overwrite All Selected)
                    </button>
                </div>
            {:else if Array.isArray(bulkEditFields.writers)}
                 <WriterEditor bind:writers={bulkEditFields.writers} on:input={() => handleInput('writers')} />
                 <span class="bulk-hint">(Changes will apply to all selected tracks)</span>
            {:else}
                 <!-- Should not happen if initialized correctly, but provides a fallback -->
                 <p>Error initializing writer editor.</p>
            {/if}
        </div>
     </div>

     <!-- Add PublisherEditor component -->
     <div class="form-row">
        <div class="form-group form-group-tags">
            <label>Publishers</label>
            {#if bulkEditFields.publishers === null}
                <div class="mixed-value-notice">
                    <p>(Mixed Values)</p>
                    <button class="edit-overwrite-button" type="button" on:click={() => { bulkEditFields.publishers = []; handleInput('publishers'); }}>
                        Edit Publishers (Will Overwrite All Selected)
                    </button>
                </div>
            {:else if Array.isArray(bulkEditFields.publishers)}
                 <PublisherEditor bind:publishers={bulkEditFields.publishers} on:input={() => handleInput('publishers')} />
                 <span class="bulk-hint">(Changes will apply to all selected tracks)</span>
            {:else}
                 <!-- Should not happen if initialized correctly, but provides a fallback -->
                 <p>Error initializing publisher editor.</p>
            {/if}
        </div>
     </div>

     <!-- Placeholder for other bulk fields -->


    <div class="form-actions">
      <button class="apply-button" on:click={handleApply}>Apply to Selected</button>
      <button class="cancel-button" on:click={handleCancel}>Cancel Bulk Edit</button>
    </div>
  </div>
</div>

<style>
  /* Styles for the bulk form will be moved or defined here */
  .bulk-edit-panel {
    padding: 1rem;
    border: 1px solid #ccc;
  }
   .bulk-edit-form .form-row {
      display: flex;
      gap: 1rem;
      margin-bottom: 1rem;
  }
  .bulk-edit-form .form-group {
      flex: 1;
      display: flex;
      flex-direction: column;
  }
   .bulk-edit-form .form-group label {
      margin-bottom: 0.25rem;
      font-weight: bold;
   }
   .bulk-edit-form .form-group input[type="text"] {
      padding: 0.5rem;
      border: 1px solid #ccc;
      border-radius: 4px;
   }
    .bulk-edit-form .form-group .bulk-hint {
        font-size: 0.8em;
        color: #666;
        margin-top: 0.25rem;
    }
   .bulk-edit-form .form-actions {
      margin-top: 1rem;
      display: flex;
      gap: 0.5rem;
   }
   /* Add more specific styles as needed */
</style>