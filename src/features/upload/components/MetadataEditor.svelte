<script lang="ts">
  import { createEventDispatcher } from 'svelte'; // Removed unused onMount
  import TagSelector from '$lib/components/common/TagSelector.svelte';
  import { instrumentTags, moodTags, tagsToString } from '$lib/stores/tagData';
  import PublisherEditor from './PublisherEditor.svelte'; // Added import
  import type { PublisherEntry } from './PublisherEditor.svelte'; // Import type

  // --- Type Definitions (Simplified for component props/state) ---
  // These might need adjustment based on how data is passed from the catalog page
  interface TrackEditData {
    _id?: string; // Assuming MongoDB ObjectId might be present
    title: string;
    album_name: string; // Flattened for simplicity
    artist_name: string; // Flattened for simplicity
    genre: string[];
    writers: string[];
    writer_percentages: number[];
    publishers: PublisherEntry[]; // Use PublisherEntry type
    // publisher_percentages removed, included in PublisherEntry
    instruments: string[];
    mood: string[];
    comments: string;
    // Add other relevant fields from the catalog view if needed
  }

  interface BulkEditData {
    album_name: string;
    artist_name: string;
    genre: string; // Comma-separated string for bulk edit UI
    writers: string[];
    writer_percentages: number[];
    publishers: PublisherEntry[]; // Use PublisherEntry type
    // publisher_percentages removed, included in PublisherEntry
    instruments: string; // Comma-separated string for bulk edit UI
    mood: string; // Comma-separated string for bulk edit UI
  }

  // --- Props ---
  /** Array of track data objects to be edited */
  export let tracks: TrackEditData[] = [];
  /** Indices of the tracks selected for editing */
  export let selectedIndices: number[] = [];
  /** Current editing mode: 'individual' or 'bulk' */
  export let mode: 'individual' | 'bulk' = 'individual';

  // --- State ---
  let editData: TrackEditData | BulkEditData | null = null; // Holds the data being actively edited
  let writerPercentagesValid = true;
  // let publisherPercentagesValid = true; // Removed, handled by PublisherEditor
  let internalMode = mode; // Internal copy to manage transitions

  // --- Event Dispatcher ---
  const dispatch = createEventDispatcher<{
    save: { data: TrackEditData | BulkEditData | null; mode: 'individual' | 'bulk' };
    cancel: void;
  }>();

  // --- Reactive Logic ---
  // Initialize or update editData when props change
  $: {
    internalMode = mode; // Update internal mode when prop changes
    if (internalMode === 'individual' && selectedIndices.length === 1 && tracks[selectedIndices[0]]) {
      // Deep copy the track data for individual editing to avoid modifying the original object directly
      editData = JSON.parse(JSON.stringify(tracks[selectedIndices[0]])) as TrackEditData;
      validatePercentages(); // Validate initial percentages
    } else if (internalMode === 'bulk' && selectedIndices.length > 0) {
      // Initialize bulk edit fields (logic adapted from startBulkEdit in upload page)
      initializeBulkEditData();
      validatePercentages(); // Validate initial percentages
    } else {
      editData = null; // Reset if no valid selection or mode
    }
  }

  // --- Functions ---

  function initializeBulkEditData() {
      // Initialize with empty or common values
      const firstTrack = tracks[selectedIndices[0]];
      let commonAlbum = firstTrack.album_name;
      let commonArtist = firstTrack.artist_name;
      let commonGenre = firstTrack.genre.join(', ');
      let commonInstruments = firstTrack.instruments.join(', ');
      let commonMood = firstTrack.mood.join(', ');

      // Check if all selected tracks share the same values for simple fields
      for (let i = 1; i < selectedIndices.length; i++) {
          const track = tracks[selectedIndices[i]];
          if (track.album_name !== commonAlbum) commonAlbum = '';
          if (track.artist_name !== commonArtist) commonArtist = '';
          if (track.genre.join(', ') !== commonGenre) commonGenre = '';
          if (track.instruments.join(', ') !== commonInstruments) commonInstruments = '';
          if (track.mood.join(', ') !== commonMood) commonMood = '';
      }

      // Collect unique writers/publishers and set initial equal percentages
      const writersSet = new Set<string>();
      const publishersSet = new Set<string>(); // Collect unique publisher names first
      selectedIndices.forEach(index => {
          const track = tracks[index];
          track.writers?.forEach(w => writersSet.add(w));
          // Assuming original track.publishers is PublisherEntry[] or needs conversion
          // For now, let's assume it might still be string[] from the source `tracks` prop
          // and we need to create PublisherEntry objects here.
          // If `tracks` prop already contains PublisherEntry[], this needs adjustment.
          track.publishers?.forEach(p => {
              if (typeof p === 'string') {
                  publishersSet.add(p); // If it's a string array
              } else if (typeof p === 'object' && p !== null && 'name' in p) {
                  publishersSet.add(p.name); // If it's already PublisherEntry[]
              }
          });
      });
      const writers = Array.from(writersSet);
      const uniquePublisherNames = Array.from(publishersSet);
      const writerPercentage = writers.length > 0 ? Math.floor(100 / writers.length) : 0;

      // Create PublisherEntry array with distributed percentages
      const bulkPublishers: PublisherEntry[] = distributePercentagesForNames(uniquePublisherNames);

      editData = {
          album_name: commonAlbum,
          artist_name: commonArtist,
          genre: commonGenre,
          writers: writers,
          writer_percentages: writers.map(() => writerPercentage),
          publishers: bulkPublishers, // Assign the created PublisherEntry array
          // publisher_percentages removed
          instruments: commonInstruments,
          mood: commonMood,
      } as BulkEditData;
  }


  // Adapted from upload/+page.svelte
  function validatePercentages() {
    writerPercentagesValid = true; // Assume valid initially
    // publisherPercentagesValid = true; // Removed

    if (!editData) return;

    if (internalMode === 'individual') {
      const track = editData as TrackEditData;
      if (track.writer_percentages && track.writer_percentages.length > 0) {
        const writerSum = track.writer_percentages.reduce((sum, percent) => sum + (Number(percent) || 0), 0);
        writerPercentagesValid = Math.abs(writerSum - 100) < 0.01;
      }
      // Publisher validation removed - handled by PublisherEditor component
      // if (track.publishers && track.publishers.length > 0) {
      //   const publisherSum = track.publishers.reduce((sum, entry) => sum + (Number(entry.percentage) || 0), 0);
      //   // publisherPercentagesValid = Math.abs(publisherSum - 100) < 0.01; // Validation done in child
      // }
    } else if (internalMode === 'bulk') {
      const bulkData = editData as BulkEditData;
       if (bulkData.writer_percentages && bulkData.writer_percentages.length > 0) {
        const writerSum = bulkData.writer_percentages.reduce((sum, percent) => sum + (Number(percent) || 0), 0);
        writerPercentagesValid = Math.abs(writerSum - 100) < 0.01;
      }
      // Publisher validation removed - handled by PublisherEditor component
      // if (bulkData.publishers && bulkData.publishers.length > 0) {
      //  const publisherSum = bulkData.publishers.reduce((sum, entry) => sum + (Number(entry.percentage) || 0), 0);
      //  // publisherPercentagesValid = Math.abs(publisherSum - 100) < 0.01; // Validation done in child
      //}
    }
  }

  // --- Event Handlers ---
  function handleSave() {
    validatePercentages();
    if (!writerPercentagesValid /* || !publisherPercentagesValid */) { // Removed publisher check
      alert('Writer and/or Publisher percentages must sum to 100%');
      return;
    }
    // Dispatch the edited data
    dispatch('save', { data: editData, mode: internalMode });
  }

  function handleCancel() {
    dispatch('cancel');
  }

  // TODO: Implement tag handlers (handleInstrumentTagsChanged, handleMoodTagsChanged)
  // TODO: Implement functions to add/remove writers/publishers
  // TODO: Implement applyBulkEdits logic (or handle save directly)

  // Helper function to distribute percentages for bulk edit initialization
  function distributePercentagesForNames(names: string[]): PublisherEntry[] {
      const count = names.length;
      if (count === 0) return [];

      let assignedPercentage = 0;
      const basePercentage = Math.floor(100 / count);
      const remainder = 100 % count;

      const entries = names.map((name, index) => {
          let percentage = basePercentage;
          if (index < remainder) {
              percentage += 1; // Distribute remainder
          }
          assignedPercentage += percentage;
          return { name, percentage };
      });

      // Final check to ensure sum is exactly 100
      let finalSum = entries.reduce((sum, p) => sum + p.percentage, 0);
      if(finalSum !== 100 && entries.length > 0) {
          // Adjust the last element slightly if needed
          entries[entries.length - 1].percentage += (100 - finalSum);
      }

      return entries;
  }

  // TODO: Implement applyBulkEdits logic (or handle save directly)

</script>

{#if editData}
  <div class="metadata-editor">
    <div class="editor-header">
      <h3>Edit Metadata</h3>
      <div class="editor-actions">
        {#if internalMode === 'individual' && selectedIndices.length > 1} <!-- Show bulk edit button if multiple items selected in individual mode -->
          <button class="bulk-edit-button" on:click={() => internalMode = 'bulk'}>
            Bulk Edit ({selectedIndices.length} selected)
          </button>
        {/if}
        <button class="save-button" on:click={handleSave}>Save Changes</button> <!-- Changed button text -->
        <button class="cancel-button" on:click={handleCancel}>Cancel</button>
      </div>
    </div>
    
    {#if internalMode === 'bulk' && editData}
      <div class="bulk-edit-panel">
        <h4>Bulk Edit Selected Tracks ({selectedIndices.length} selected)</h4>
        <div class="bulk-edit-form">
          <!-- Bulk Edit Form Content (Copied from upload/+page.svelte lines 973-1133) -->
          <div class="form-row">
            <div class="form-group">
              <label for="bulk-album">Album</label>
              <input id="bulk-album" type="text" bind:value={(editData as BulkEditData).album_name} placeholder="Album Name" />
            </div>
            
            <div class="form-group">
              <label for="bulk-artist">Artist</label>
              <input id="bulk-artist" type="text" bind:value={(editData as BulkEditData).artist_name} placeholder="Artist Name" />
            </div>
          </div>
          
          <div class="form-row">
            <div class="form-group">
              <label for="bulk-genre">Genre (comma separated)</label>
              <input id="bulk-genre" type="text" bind:value={(editData as BulkEditData).genre} placeholder="Rock, Pop, Jazz..." />
            </div>
          </div>
          
          <div class="form-section">
            <h5>Writers</h5>
            <div class="tags-input">
              <input 
                type="text" 
                placeholder="Add writer (press Enter)" 
                on:keydown={(e) => {
                  const target = e.target as HTMLInputElement;
                  if (e.key === 'Enter' && target.value && editData && 'writers' in editData) {
                    (editData as BulkEditData).writers = [...(editData as BulkEditData).writers, target.value];
                    // Recalculate percentages when adding
                    const count = (editData as BulkEditData).writers.length;
                    (editData as BulkEditData).writer_percentages = (editData as BulkEditData).writers.map(() => count > 0 ? Math.floor(100 / count) : 0);
                    target.value = '';
                    validatePercentages();
                  }
                }}
              />
            </div>
            
            <div class="tags-list">
              {#if editData && 'writers' in editData && (editData as BulkEditData).writers.length > 0}
                {#each (editData as BulkEditData).writers as writer, i}
                  <div class="tag-item">
                    <span>{writer}</span>
                    <div class="percentage-input">
                      <input
                        type="number"
                        min="0"
                        max="100"
                        bind:value={(editData as BulkEditData).writer_percentages[i]}
                        on:input={validatePercentages}
                      />
                      <span>%</span>
                    </div>
                    <button class="remove-tag" on:click={() => {
                      if (editData && 'writers' in editData) {
                        (editData as BulkEditData).writers = (editData as BulkEditData).writers.filter((_, idx) => idx !== i);
                        (editData as BulkEditData).writer_percentages = (editData as BulkEditData).writer_percentages.filter((_, idx) => idx !== i);
                        validatePercentages();
                      }
                    }}>×</button>
                  </div>
                {/each}
                
                {#if !writerPercentagesValid}
                  <div class="validation-error">
                    Writer percentages must sum to 100%
                  </div>
                {/if}
              {/if}
            </div>
          </div>
          
          <div class="form-section">
            <h5>Publishers</h5>
            {#if editData && 'publishers' in editData}
              <PublisherEditor
                bind:publishers={(editData as BulkEditData).publishers}
                on:change={() => {
                  // Trigger reactivity/validation if needed, though publisher validation is internal
                  validatePercentages(); // Keep for writer validation or other effects
                  editData = editData; // Force Svelte reactivity if direct mutation doesn't trigger update
                }}
              />
              <!-- {#if !publisherPercentagesValid} Removed - handled internally -->
                <!-- <div class="validation-error"> -->
                  <!-- Publisher percentages must sum to 100% -->
                <!-- </div> -->
              <!-- {/if} -->
            {/if}
          </div>
          
          <div class="form-section">
            <div class="form-row">
              <div class="form-group">
                <label for="bulk-instruments">Instruments</label>
                <TagSelector
                  tagOptions={$instrumentTags}
                  selectedTagsString={(editData as BulkEditData).instruments}
                  placeholder="Add instrument..."
                  on:tagsChanged={(e) => {
                    if (editData && 'instruments' in editData) {
                      (editData as BulkEditData).instruments = tagsToString(e.detail.tags);
                    }
                  }}
                />
              </div>
              
              <div class="form-group">
                <label for="bulk-mood">Mood</label>
                <TagSelector
                  tagOptions={$moodTags}
                  selectedTagsString={(editData as BulkEditData).mood}
                  placeholder="Add mood..."
                  on:tagsChanged={(e) => {
                    if (editData && 'mood' in editData) {
                      (editData as BulkEditData).mood = tagsToString(e.detail.tags);
                    }
                  }}
                />
              </div>
            </div>
          </div>
          
          <div class="form-actions">
            <button class="apply-button" on:click={handleSave}>Apply Bulk Edits</button>
            <button class="cancel-button" on:click={handleCancel}>Cancel</button>
          </div>
        </div>
      </div>
    {:else if internalMode === 'individual' && editData}
      <div class="individual-edit-panel">
        <h4>Edit Track: {(editData as TrackEditData).title}</h4>
        <div class="individual-edit-form">
          <!-- Individual Edit Form Content (Copied from upload/+page.svelte lines 1140-1339) -->
           <div class="form-row">
            <div class="form-group">
              <label for="track-title">Title</label>
              <input
                id="track-title"
                type="text"
                bind:value={(editData as TrackEditData).title}
              />
            </div>
            
            <div class="form-group">
              <label for="track-album">Album</label>
              <input
                id="track-album"
                type="text"
                bind:value={(editData as TrackEditData).album_name}
              />
            </div>
          </div>
          
          <div class="form-row">
            <div class="form-group">
              <label for="track-artist">Artist</label>
              <input
                id="track-artist"
                type="text"
                bind:value={(editData as TrackEditData).artist_name}
              />
            </div>
            
            <div class="form-group">
              <label for="track-genre">Genre (comma separated)</label>
              <input
                id="track-genre"
                type="text"
                value={(editData as TrackEditData).genre?.join(', ') || ''}
                on:input={(e) => {
                  const target = e.target as HTMLInputElement;
                  if (editData && 'genre' in editData) {
                    (editData as TrackEditData).genre = target.value.split(',').map(g => g.trim()).filter(g => g);
                  }
                }}
              />
            </div>
          </div>
          
          <div class="form-section">
            <h5>Writers</h5>
            <div class="tags-input">
              <input 
                type="text" 
                placeholder="Add writer (press Enter)" 
                on:keydown={(e) => {
                  const target = e.target as HTMLInputElement;
                  if (e.key === 'Enter' && target.value && editData && 'writers' in editData) {
                     (editData as TrackEditData).writers = [...((editData as TrackEditData).writers || []), target.value];
                     // Recalculate percentages
                     const count = (editData as TrackEditData).writers.length;
                     (editData as TrackEditData).writer_percentages = (editData as TrackEditData).writers.map(() => count > 0 ? Math.floor(100 / count) : 0);
                     target.value = '';
                     validatePercentages();
                  }
                }}
              />
            </div>
            
            <div class="tags-list">
              {#if editData && 'writers' in editData && (editData as TrackEditData).writers?.length > 0}
                {#each (editData as TrackEditData).writers as writer, i}
                  <div class="tag-item">
                    <span>{writer}</span>
                    <div class="percentage-input">
                      <input
                        type="number"
                        min="0"
                        max="100"
                        bind:value={(editData as TrackEditData).writer_percentages[i]}
                        on:input={validatePercentages}
                      />
                      <span>%</span>
                    </div>
                    <button class="remove-tag" on:click={() => {
                      if (editData && 'writers' in editData) {
                        (editData as TrackEditData).writers = (editData as TrackEditData).writers.filter((_, idx) => idx !== i);
                        (editData as TrackEditData).writer_percentages = (editData as TrackEditData).writer_percentages.filter((_, idx) => idx !== i);
                        validatePercentages();
                      }
                    }}>×</button>
                  </div>
                {/each}
                
                {#if !writerPercentagesValid}
                  <div class="validation-error">
                    Writer percentages must sum to 100%
                  </div>
                {/if}
              {/if}
            </div>
          </div>
          
          <div class="form-section">
            <h5>Publishers</h5>
             {#if editData && 'publishers' in editData}
              <PublisherEditor
                bind:publishers={(editData as TrackEditData).publishers}
                on:change={() => {
                  // Trigger reactivity/validation if needed
                  validatePercentages(); // Keep for writer validation or other effects
                  editData = editData; // Force Svelte reactivity
                }}
              />
              <!-- {#if !publisherPercentagesValid} Removed - handled internally -->
                <!-- <div class="validation-error"> -->
                  <!-- Publisher percentages must sum to 100% -->
                <!-- </div> -->
              <!-- {/if} -->
            {/if}
          </div>
          
          <div class="form-section">
            <div class="form-row">
              <div class="form-group">
                <label for="track-instruments">Instruments</label>
                <TagSelector
                  tagOptions={$instrumentTags}
                  selectedTagsString={(editData as TrackEditData).instruments?.join(', ') || ''}
                  placeholder="Add instrument (press Enter)"
                  on:tagsChanged={(e) => {
                    if (editData && 'instruments' in editData) {
                      (editData as TrackEditData).instruments = e.detail.tags;
                      // No need to trigger reactivity here, Svelte handles array changes
                    }
                  }}
                />
              </div>
              
              <div class="form-group">
                <label for="track-mood">Mood</label>
                 <TagSelector
                   tagOptions={$moodTags}
                   selectedTagsString={(editData as TrackEditData).mood?.join(', ') || ''}
                   placeholder="Add mood (press Enter)"
                   on:tagsChanged={(e) => {
                     if (editData && 'mood' in editData) {
                       (editData as TrackEditData).mood = e.detail.tags;
                       // No need to trigger reactivity here, Svelte handles array changes
                     }
                   }}
                 />
              </div>
            </div>
          </div>
          
          <div class="form-section">
            <div class="form-group">
              <label for="track-comments">Comments</label>
              <textarea
                id="track-comments"
                bind:value={(editData as TrackEditData).comments}
                rows="3"
              ></textarea>
            </div>
          </div>
          
          <div class="form-actions">
            <button class="save-button" on:click={handleSave}>Save</button>
            <button class="cancel-button" on:click={handleCancel}>Cancel</button>
          </div>
        </div>
      </div>
    {/if} <!-- End of individual/bulk edit panels -->
  </div> <!-- End of metadata-editor -->
{/if} <!-- End of {#if editData} -->

<style lang="postcss">
  /* Remove styles related to .tracks-list as it's handled by parent */
  /* Metadata Editing Styles (Copied from upload/+page.svelte lines 1806-2131) */
  .edit-metadata-actions {
    margin-top: 20px;
    display: flex;
    justify-content: center;
  }
  
  .edit-metadata-button {
    background-color: #805ad5;
    color: white;
    padding: 10px 20px;
    border: none;
    border-radius: 4px;
    font-size: 16px;
    cursor: pointer;
    transition: background-color 0.3s;
  }
  
  .edit-metadata-button:hover {
    background-color: #6b46c1;
  }
  
  .metadata-editor {
    margin-top: 30px;
    background-color: #f8fafc;
    border-radius: 8px;
    padding: 20px;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.05);
  }
  
  .editor-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
    padding-bottom: 10px;
    border-bottom: 1px solid #e2e8f0;
  }
  
  .editor-header h3 {
    margin: 0;
    color: #2d3748;
  }
  
  .editor-actions {
    display: flex;
    gap: 10px;
  }
  
  .save-button, .apply-button {
    background-color: #4299e1;
    color: white;
    padding: 8px 16px;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    transition: background-color 0.3s;
  }
  
  .save-button:hover, .apply-button:hover {
    background-color: #3182ce;
  }
  
  .cancel-button {
    background-color: #a0aec0;
    color: white;
    padding: 8px 16px;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    transition: background-color 0.3s;
  }
  
  .cancel-button:hover {
    background-color: #718096;
  }
  
  .bulk-edit-button {
    background-color: #805ad5;
    color: white;
    padding: 8px 16px;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    transition: background-color 0.3s;
  }
  
  .bulk-edit-button:hover {
    background-color: #6b46c1;
  }
  
  .edit-button {
    background-color: #4299e1;
    color: white;
    padding: 4px 8px;
    border: none;
    border-radius: 4px;
    font-size: 14px;
    cursor: pointer;
    transition: background-color 0.3s;
  }
  
  .edit-button:hover {
    background-color: #3182ce;
  }
  
  .bulk-edit-panel, .individual-edit-panel {
    background-color: white;
    border-radius: 6px;
    padding: 16px;
    margin-bottom: 20px;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  }
  
  .bulk-edit-panel h4, .individual-edit-panel h4 {
    margin-top: 0;
    margin-bottom: 16px;
    color: #2d3748;
  }
  
  .bulk-edit-form, .individual-edit-form {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  
  .form-row {
    display: flex;
    gap: 16px;
  }
  
  .form-row .form-group {
    flex: 1;
  }
  
  .form-section {
    margin-top: 16px;
    margin-bottom: 16px;
  }
  
  .form-section h5 {
    margin-top: 0;
    margin-bottom: 8px;
    color: #4a5568;
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  
  .form-group label {
    font-size: 14px;
    color: #4a5568;
  }
  
  .form-group input, .form-group select, .form-group textarea {
    padding: 8px;
    border: 1px solid #e2e8f0;
    border-radius: 4px;
    background-color: white;
  }
  
  .tags-input {
    margin-bottom: 8px;
  }
  
  .tags-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-bottom: 16px;
  }
  
  .tag-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 8px;
    background-color: #edf2f7;
    border-radius: 4px;
  }
  
  .tag-item span {
    flex: 1;
  }
  
  .percentage-input {
    display: flex;
    align-items: center;
    gap: 4px;
  }
  
  .percentage-input input {
    width: 60px;
    padding: 4px;
    border: 1px solid #e2e8f0;
    border-radius: 2px;
  }
  
  .remove-tag {
    background: none;
    border: none;
    color: #a0aec0;
    cursor: pointer;
    font-size: 16px;
    padding: 0 4px;
  }
  
  .remove-tag:hover {
    color: #e53e3e;
  }
  
  .validation-error {
    color: #e53e3e;
    font-size: 14px;
  }
  
  .tracks-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
    max-height: 500px; /* Adjust as needed */
    overflow-y: auto;
  }
  
  .tracks-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 10px;
    background-color: #f7fafc;
    border-radius: 4px;
    margin-bottom: 8px;
  }
  
  .select-all-container {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  
  .select-all-container label {
    font-size: 14px;
    color: #4a5568;
    cursor: pointer;
  }
  
  .bulk-edit-button-small {
    background-color: #805ad5;
    color: white;
    padding: 4px 12px;
    border: none;
    border-radius: 4px;
    font-size: 14px;
    cursor: pointer;
    transition: background-color 0.3s;
  }
  
  .bulk-edit-button-small:hover {
    background-color: #6b46c1;
  }
  
  .track-item {
    display: flex;
    align-items: center;
    padding: 10px;
    background-color: white;
    border-radius: 4px;
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);
    transition: background-color 0.2s;
  }
  
  .track-item:hover {
    background-color: #f7fafc;
  }
  
  .track-item.selected {
    background-color: #ebf8ff;
  }
  
  .track-select {
    padding-right: 10px;
  }
  
  .track-info {
    flex: 1;
    cursor: pointer;
    padding: 0 10px;
  }
  
  .track-title {
    font-weight: 600;
    color: #2d3748;
    margin-bottom: 4px;
  }
  
  .track-details {
    display: flex;
    align-items: center;
    font-size: 14px;
    color: #4a5568;
    margin-bottom: 4px;
  }
  
  .separator {
    margin: 0 6px;
    color: #cbd5e0;
  }
  
  .track-path {
    font-size: 12px;
    color: #718096;
  }
  
  .track-actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  
  textarea {
    width: 100%;
    padding: 8px;
    border: 1px solid #e2e8f0;
    border-radius: 4px;
    resize: vertical;
    font-family: inherit;
  }
  
  .form-actions {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    margin-top: 16px;
  }
  
  .bulk-hint {
    font-size: 12px;
    color: #718096;
    margin-top: 4px;
    font-style: italic;
  }
</style>