<script lang="ts">
  import { onMount } from 'svelte';
  import { browser } from '$app/environment'; // Import browser check
  // import { invoke } from '@tauri-apps/api/core'; // No longer needed directly
  import { safeInvoke } from '$lib/utils/invokeWrapper'; // Import the wrapper
  import MetadataEditor from '$features/upload/components/MetadataEditor.svelte';
  import type { Track, TrackListResponse } from '$lib/types/catalog'; // Import types (Removed PathInfo as it seems unused here)
  import { replaceTrackAudioWorkflow, deleteTracksWorkflow } from '$features/catalog/utils'; // Import new action functions
  // TagSelector and tagData imports might be removable if only used in the old inline editor
  // import TagSelector from '$lib/components/common/TagSelector.svelte';
  // import { instrumentTags, moodTags } from '$lib/stores/tagData';
  // Types are now imported from $lib/types/catalog
  
  // Interface for the data being edited (remains component-specific for now)
  interface EditingTrackData extends Omit<Track, 'id' | 'path' | 'album_id'> {
      // Use Omit to exclude fields not directly editable or handled differently
      // Add any temporary fields needed for editing UI, like string representations of arrays
      genreString?: string; // Example: for comma-separated input
  }
  
  // State variables for the catalog
  let tracks: Track[] = [];
  let isLoading = true;
  let error: string | null = null;
  let totalTracks = 0;
  let mongoTestResult: string | null = null;
  let isTestingMongo = false;
  
  // Selected track state
  let selectedTrackIds: string[] = [];
  
  // Editing state
  let isEditing = false; // Flag for showing the editor component
  let editMode: 'individual' | 'bulk' = 'individual'; // Mode for the editor component
  // editingTrackData, writerPercentagesValid, publisherPercentagesValid are removed (handled by component)
  // --- Removed state related to audio replacement workflow (now handled in catalogActions.ts) ---
  // let replacementFilePath: string | null = null;
  // let isReplacingTranscoding = false;
  // let replacementTranscodeResult: any | null = null;
  // let isReplacingUploading = false;
  let isDeleting = false; // State for deletion status (keep for UI feedback)
  let isReplacing = false; // Combined state for replacement workflow UI feedback
  
  // Sorting state
  let sortField = 'title';
  let sortDirection = 'asc'; // 'asc' or 'desc'
  
  // Pagination
  let currentPage = 1;
  let tracksPerPage = 50;
  
  // Declare the variable for reactive calculation (moved earlier)
  let calculatedSelectedIndices: number[] = [];
  let allSelected = false; // State for the header checkbox
  // Load tracks only on the client-side
  if (browser) {
    onMount(async () => {
      await loadTracks();
    });
  }
  
  async function loadTracks() {
    isLoading = true;
    error = null;
    
    // Use safeInvoke for initialization
    const mongoInitialized = await safeInvoke<boolean>('init_mongo_client');
    if (mongoInitialized === null || !mongoInitialized) {
      // Error handled by safeInvoke, update UI state
      error = 'Failed to initialize database connection. Check Settings.';
      isLoading = false;
      return;
    }
  
    console.log('MongoDB client initialized successfully for catalog.');

    // Calculate skip value for pagination
    const skip = (currentPage - 1) * tracksPerPage;

    console.log('Fetching tracks with params:', { sortField, sortDirection, limit: tracksPerPage, skip });

    // Use safeInvoke to fetch tracks
    const result = await safeInvoke<TrackListResponse>('fetch_all_tracks', {
      sortField,
      sortDirection,
      limit: tracksPerPage,
      skip
    });

    if (result !== null && result.success) {
      tracks = result.tracks;
      totalTracks = result.total_count;
      console.log('Tracks loaded:', tracks.length, 'Total count:', totalTracks);
      error = null; // Clear previous errors
    } else {
      // Error handled by safeInvoke or command reported failure
      tracks = []; // Clear potentially stale data
      totalTracks = 0;
      // error = result?.message || 'Failed to fetch tracks.'; // Error shown by toast
    }
    isLoading = false;
  }
  
  async function testMongoDBCollections() {
    isTestingMongo = true;
    mongoTestResult = null;
    
    console.log('Testing MongoDB collections...');
    // Use safeInvoke
    const result = await safeInvoke<string>('test_mongodb_collections');
    if (result !== null) {
      mongoTestResult = result;
      console.log('MongoDB test result:', result);
    } else {
      // Error handled by safeInvoke
      mongoTestResult = 'Failed to test collections (see toast/console).';
    }
    isTestingMongo = false;
  }
  
  function handleSort(field: string) {
    if (sortField === field) {
      // Toggle sort direction if clicking the same field
      sortDirection = sortDirection === 'asc' ? 'desc' : 'asc';
    } else {
      // Set new sort field and reset to ascending
      sortField = field;
      sortDirection = 'asc';
    }
    
    loadTracks();
  }
  
  // Renamed original function slightly to clarify its role
  function updateTrackSelection(trackId: string | null) { // Removed unused index parameter
    console.log(`updateTrackSelection called with trackId: ${trackId}, type: ${typeof trackId}`); // Log initial value
    if (!trackId) {
      console.error("updateTrackSelection called with null trackId");
      return; // Don't process null IDs
    }
    // selectedTrackIndex = index; // Removed unused assignment

    // Toggle selection
    if (selectedTrackIds.includes(trackId)) {
      selectedTrackIds = selectedTrackIds.filter(id => id !== trackId);
    } else {
      selectedTrackIds = [...selectedTrackIds, trackId];
    }
    console.log('updateTrackSelection updated selectedTrackIds:', JSON.stringify(selectedTrackIds));
    // Update allSelected state after selection changes
    updateAllSelectedState();
  }


  // --- Editing Functions ---
  
  function startEdit() {
    if (selectedTrackIds.length === 0) {
      alert('Please select at least one track to edit.');
      return;
    }
    // Determine mode based on selection count
    editMode = selectedTrackIds.length === 1 ? 'individual' : 'bulk';
    isEditing = true; // Show the editor component
    // The component itself will fetch the correct data based on selectedIndices prop
  }
  
  // Removed old editing functions: validateEditPercentages, saveEdit, cancelEdit
  // Logic is now handled by MetadataEditor component and new event handlers below

  // --- New Event Handlers for MetadataEditor Component ---

  async function handleSaveEdit(event: CustomEvent<{ data: any; mode: 'individual' | 'bulk' }>) {
    const { data, mode: savedMode } = event.detail;
    console.log(`Save event received from editor. Mode: ${savedMode}`, data);

    if (!data) {
      console.error("Save event received with no data.");
      return;
    }

    let success: boolean | null = false; // Allow null from safeInvoke

    if (savedMode === 'individual') {
      if (selectedTrackIds.length !== 1) {
         console.error("Individual save attempted without exactly one track selected.");
         alert("Error: No track selected for saving.");
         return;
      }
      const trackIdToUpdate = selectedTrackIds[0];
      // Prepare payload matching UpdateTrackPayload
      const payload = {
          title: data.title,
          album_name: data.album_name,
          genre: data.genre,
          writers: data.writers,
          writer_percentages: data.writer_percentages?.map((p: string | number) => Number(p || 0)),
          publishers: data.publishers,
          publisher_percentages: data.publisher_percentages?.map((p: string | number) => Number(p || 0)),
          instruments: data.instruments,
          mood: data.mood,
          comments: data.comments,
      };
      console.log('Attempting to save individual edit for track:', trackIdToUpdate, 'Payload:', payload);
      success = await safeInvoke<boolean>('update_track_metadata', {
        trackId: trackIdToUpdate,
        payload: payload
      });

      if (success) { // Check if success is true (handles null/false as falsy)
         // Update local track data
         const index = tracks.findIndex(t => t.id === trackIdToUpdate);
         if (index !== -1) {
           tracks[index] = { ...tracks[index], ...payload }; // Merge updates
           tracks = [...tracks]; // Trigger reactivity
           console.log('Local track data updated for', trackIdToUpdate);
         } else {
            console.warn('Track not found locally after update.');
            await loadTracks(); // Reload if local update fails
         }
      }

    } else if (savedMode === 'bulk') {
       // Prepare bulk payload (assuming backend expects similar structure but applied to multiple IDs)
       // Define a type for the bulk payload for safer access
       interface BulkUpdatePayload {
         album_name?: string;
         artist_name?: string; // Assuming artist_name is needed for bulk too
         genre?: string[];
         writers?: string[];
         writer_percentages?: number[];
         publishers?: string[];
         publisher_percentages?: number[];
         instruments?: string[];
         mood?: string[];
         comments?: string; // Add if needed for bulk
         [key: string]: any; // Index signature to allow string indexing
       }

       // NOTE: Backend command 'update_tracks_bulk' needs to be implemented
       const bulkPayload: BulkUpdatePayload = {
          album_name: data.album_name || undefined,
          artist_name: data.artist_name || undefined, // Assuming artist_name is part of bulk data
          genre: data.genre ? data.genre.split(',').map((g: string) => g.trim()).filter((g: string) => g) : undefined,
          writers: data.writers?.length > 0 ? data.writers : undefined,
          writer_percentages: data.writers?.length > 0 ? data.writer_percentages?.map((p: string | number) => Number(p || 0)) : undefined,
          publishers: data.publishers?.length > 0 ? data.publishers : undefined,
          publisher_percentages: data.publishers?.length > 0 ? data.publisher_percentages?.map((p: string | number) => Number(p || 0)) : undefined,
          instruments: data.instruments ? data.instruments.split(',').map((t: string) => t.trim()).filter((t: string) => t) : undefined,
          mood: data.mood ? data.mood.split(',').map((t: string) => t.trim()).filter((t: string) => t) : undefined,
       };
       // Remove undefined fields before sending using the defined type
       Object.keys(bulkPayload).forEach((key) => {
          if (bulkPayload[key] === undefined) {
             delete bulkPayload[key];
          }
       });

       console.log('Attempting to save bulk edit for tracks:', selectedTrackIds, 'Payload:', bulkPayload);
       // success = await safeInvoke<boolean>('update_tracks_bulk', { // Replace with actual command
       //   trackIds: selectedTrackIds,
       //   payload: bulkPayload
       // });
       alert("Bulk update backend command not yet implemented."); // Placeholder
       success = false; // Simulate failure until backend is ready

       if (success) { // Check if success is true (handles null/false as falsy)
          console.log('Bulk update successful, reloading tracks.');
          await loadTracks(); // Reload tracks after bulk update
       }
    }

    if (success === true) {
      isEditing = false;
      selectedTrackIds = [];
      // selectedTrackIndex = -1; // Removed unused assignment
    } else {
      console.error(`Failed to save edits (Mode: ${savedMode}).`);
      // Optionally keep editor open on failure
    }
  }

  function handleCancelEdit() {
    console.log("Cancel event received from editor.");
    isEditing = false;
    selectedTrackIds = [];
    // selectedTrackIndex = -1; // Removed unused assignment
  }

  // --- End New Event Handlers ---

  // --- Audio Replacement Function (using workflow) ---
  async function handleReplaceAudio() {
    if (selectedTrackIds.length !== 1) {
      alert("Please select exactly one track to replace the audio for.");
      return;
    }
    const trackIdToReplace = selectedTrackIds[0];

    isReplacing = true; // Set loading state for UI
    const success = await replaceTrackAudioWorkflow(safeInvoke, trackIdToReplace);
    isReplacing = false; // Reset loading state

    if (success) {
      // Workflow handles success/error messages
      // Reset selection and reload catalog
      selectedTrackIds = [];
      await loadTracks();
    }
    // On failure, workflow handles error messages, keep selection for potential retry
  }
  // --- End Audio Replacement Function ---
































































































  // Reactive calculation for selected indices based on IDs (moved to top level)
  $: calculatedSelectedIndices = selectedTrackIds
      .map(id => tracks.findIndex(track => track.id === id))
      .filter(index => index !== -1);

  // Update allSelected state whenever tracks or selectedTrackIds change
  $: if (tracks && tracks.length > 0) {
      updateAllSelectedState();
  }

  function updateAllSelectedState() {
    if (tracks.length === 0) {
      allSelected = false;
      return;
    }
    const displayedTrackIds = tracks.map(t => t.id);
    allSelected = displayedTrackIds.every(id => selectedTrackIds.includes(id)) && selectedTrackIds.length >= displayedTrackIds.length;
     console.log(`updateAllSelectedState: allSelected=${allSelected}, selected=${selectedTrackIds.length}, displayed=${displayedTrackIds.length}`);
  }

  function toggleSelectAll(event: Event) {
    const target = event.target as HTMLInputElement;
    const isChecked = target.checked;
    console.log(`toggleSelectAll called: isChecked=${isChecked}`);
    if (isChecked) {
      // Select all currently displayed tracks
      selectedTrackIds = [...new Set([...selectedTrackIds, ...tracks.map(t => t.id)])];
    } else {
      // Deselect all currently displayed tracks
      const displayedTrackIds = tracks.map(t => t.id);
      selectedTrackIds = selectedTrackIds.filter(id => !displayedTrackIds.includes(id));
    }
     console.log('toggleSelectAll updated selectedTrackIds:', JSON.stringify(selectedTrackIds));
     // No need to call updateAllSelectedState here, reactive statement handles it
  }

  // --- Deletion Function (using workflow) ---
  async function handleDeleteSelected() {
    if (selectedTrackIds.length === 0) {
      alert("Please select tracks to delete.");
      return;
    }

    const confirmation = confirm(`Are you sure you want to delete ${selectedTrackIds.length} track(s)? This action cannot be undone.`);
    if (!confirmation) {
      return;
    }

    isDeleting = true; // Set loading state for UI
    const success = await deleteTracksWorkflow(safeInvoke, selectedTrackIds);
    isDeleting = false; // Reset loading state

    if (success) {
      // Workflow handles success/error messages
      // Reset selection and reload catalog
      selectedTrackIds = [];
      await loadTracks();
    }
    // On failure, workflow handles error messages, keep selection for potential retry
  }
  // --- End Deletion Function ---












  
  function formatDuration(seconds: number): string {
    const minutes = Math.floor(seconds / 60);
    const remainingSeconds = seconds % 60;
    return `${minutes}:${remainingSeconds.toString().padStart(2, '0')}`;
  }
  
  function changePage(page: number) {
    if (page < 1 || page > Math.ceil(totalTracks / tracksPerPage)) {
      return;
    }
    
    currentPage = page;
    loadTracks();
  }
</script>

<svelte:head>
  <title>Music Library - Catalog</title>
</svelte:head>

<div class="catalog-container">
  <h1>Music Catalog</h1>
  
  <div class="debug-actions">
    <button on:click={testMongoDBCollections} disabled={isTestingMongo}>
      {isTestingMongo ? 'Testing MongoDB...' : 'Test MongoDB Connection'}
    </button>
    
    {#if mongoTestResult}
      <div class="mongo-test-result">
        <h3>MongoDB Test Results:</h3>
        <pre>{mongoTestResult}</pre>
      </div>
    {/if}
  </div>
  
  {#if isLoading}
    <div class="loading">
      <p>Loading catalog data...</p>
    </div>
  {:else if error}
    <div class="error">
      <p>Error: {error}</p>
      <button on:click={loadTracks}>Retry</button>
    </div>
  {:else if tracks.length === 0}
    <div class="empty-state">
      <p>No tracks found in the catalog.</p>
      <p>Go to the <a href="/upload">Upload page</a> to add music to your library.</p>
    </div>
  {:else}
    <div class="table-actions" style:visibility={isEditing ? 'hidden' : 'visible'}>
      <!-- Updated Edit button to handle multi-select for bulk edit -->
      <button on:click={startEdit} disabled={selectedTrackIds.length === 0}>
        {selectedTrackIds.length === 1 ? 'Edit Selected' : selectedTrackIds.length > 1 ? `Bulk Edit (${selectedTrackIds.length})` : 'Edit Selected'}
      </button>
      <button on:click={handleDeleteSelected} disabled={selectedTrackIds.length === 0 || isDeleting}>
        {isDeleting ? 'Deleting...' : `Delete Selected (${selectedTrackIds.length})`}
      </button>
      <button on:click={handleReplaceAudio} disabled={selectedTrackIds.length !== 1 || isReplacing}>
        {isReplacing ? 'Replacing...' : 'Replace Audio'}
      </button>
    </div>
    
    <div class="table-container">
      <table class="tracks-table">
        <thead>
          <tr>
            <th><input type="checkbox" bind:checked={allSelected} on:change={toggleSelectAll} title="Select/Deselect All Visible"/></th>
          <tr>
            <th class="select-column">
              <input 
                type="checkbox" 
                checked={selectedTrackIds.length === tracks.length && tracks.length > 0} 
                on:change={() => {
                  if (selectedTrackIds.length === tracks.length) {
                    selectedTrackIds = [];
                  } else {
                    selectedTrackIds = tracks.map(track => track.id);
                  }
                }}
              />
            </th>
            <th on:click={() => handleSort('title')} class:sorted={sortField === 'title'} class:asc={sortDirection === 'asc' && sortField === 'title'} class:desc={sortDirection === 'desc' && sortField === 'title'}>
              Title
            </th>
            <th on:click={() => handleSort('album.name')} class:sorted={sortField === 'album.name'} class:asc={sortDirection === 'asc' && sortField === 'album.name'} class:desc={sortDirection === 'desc' && sortField === 'album.name'}>
              Album
            </th>
            <th on:click={() => handleSort('duration')} class:sorted={sortField === 'duration'} class:asc={sortDirection === 'asc' && sortField === 'duration'} class:desc={sortDirection === 'desc' && sortField === 'duration'}>
              Duration
            </th>
            <th on:click={() => handleSort('genre')} class:sorted={sortField === 'genre'} class:asc={sortDirection === 'asc' && sortField === 'genre'} class:desc={sortDirection === 'desc' && sortField === 'genre'}>
              Genre
            </th>
          </tr>
        </thead>
        <tbody>
          {#each tracks as track, index (console.log('Rendering track:', track), track)}
            <tr
              class:selected={selectedTrackIds.includes(track.id)}
              data-track-id={track.id}
              on:click={() => { console.log(`Row click - track object in handler: ${JSON.stringify(track)}`); updateTrackSelection(track.id); }}
            >
              <td>
                <input 
                  type="checkbox" 
                  checked={selectedTrackIds.includes(track.id)}
                  on:click={(e) => e.stopPropagation()} 
                  on:change={() => { console.log(`Checkbox change - track object in handler: ${JSON.stringify(track)}`); updateTrackSelection(track.id); }}
                />
              </td>
              <td>{track.title}</td>
              <td>{track.album_name || 'Unknown Album'}</td>
              <td>{formatDuration(track.duration ?? 0)}</td> <!-- Add nullish coalescing for potentially missing duration -->
              <td>{track.genre?.join(', ') || 'Unknown'}</td> <!-- Display genre array -->
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
    
    {#if totalTracks > tracksPerPage}
      <div class="pagination">
        <button on:click={() => changePage(1)} disabled={currentPage === 1}>First</button>
        <button on:click={() => changePage(currentPage - 1)} disabled={currentPage === 1}>Previous</button>
        <span>Page {currentPage} of {Math.ceil(totalTracks / tracksPerPage)}</span>
        <button on:click={() => changePage(currentPage + 1)} disabled={currentPage === Math.ceil(totalTracks / tracksPerPage)}>Next</button>
        <button on:click={() => changePage(Math.ceil(totalTracks / tracksPerPage))} disabled={currentPage === Math.ceil(totalTracks / tracksPerPage)}>Last</button>
      </div>
    {/if}
  {/if}
  
  {#if isEditing}
    <MetadataEditor
      tracks={tracks.map(t => ({
          id: t.id, // Use the correct property name 'id'
          title: t.title,
          album_name: t.album_name || '', // Ensure album_name exists
          artist_name: t.album_name || '', // Use album_name as artist_name for now, adjust if needed
          genre: t.genre || [],
          writers: t.writers || [],
          writer_percentages: t.writer_percentages || [],
          publishers: t.publishers || [],
          publisher_percentages: t.publisher_percentages || [],
          instruments: t.instruments || [],
          mood: t.mood || [],
          comments: t.comments || '',
      }))}
      selectedIndices={calculatedSelectedIndices}
      mode={editMode}
      on:save={handleSaveEdit}
      on:cancel={handleCancelEdit}
    />
  {/if}
</div>

<style lang="postcss">
  .catalog-container {
    max-width: 1200px;
    margin: 0 auto;
    padding: 20px;
  }
  
  h1 {
    margin-bottom: 20px;
  }
  
  .debug-actions {
    margin-bottom: 20px;
    padding: 15px;
    background-color: #f8f9fa;
    border-radius: 5px;
    border: 1px solid #dee2e6;
  }
  
  .mongo-test-result {
    margin-top: 15px;
    padding: 10px;
    background-color: #f8f9fa;
    border-radius: 4px;
    border: 1px solid #dee2e6;
  }
  
  .mongo-test-result h3 {
    margin-top: 0;
    margin-bottom: 10px;
    font-size: 16px;
  }
  
  .mongo-test-result pre {
    margin: 0;
    padding: 10px;
    background-color: #fff;
    border-radius: 4px;
    border: 1px solid #dee2e6;
    white-space: pre-wrap;
    font-size: 14px;
  }
  
  .loading, .error, .empty-state {
    padding: 20px;
    text-align: center;
    background: #f8f9fa;
    border-radius: 5px;
    margin: 20px 0;
  }
  
  .error {
    background: #f8d7da;
    color: #721c24;
  }
  
  .table-actions {
    margin-bottom: 10px;
    display: flex;
    gap: 10px;
  }
  
  .table-container {
    overflow-x: auto;
    margin-bottom: 20px;
  }
  
  .tracks-table {
    width: 100%;
    border-collapse: collapse;
  }
  
  .tracks-table th, .tracks-table td {
    padding: 10px;
    text-align: left;
    border-bottom: 1px solid #dee2e6;
  }
  
  .tracks-table th {
    background-color: #f8f9fa;
    cursor: pointer;
    user-select: none;
  }
  
  .tracks-table th.sorted {
    background-color: #e9ecef;
  }
  
  .tracks-table th.sorted.asc::after {
    content: " ↑";
  }
  
  .tracks-table th.sorted.desc::after {
    content: " ↓";
  }
  
  .tracks-table tbody tr:hover {
    background-color: #f8f9fa;
  }
  
  .tracks-table tbody tr.selected {
    background-color: #e2f0ff;
  }
  
  .select-column {
    width: 40px;
  }
  
  .pagination {
    display: flex;
    justify-content: center;
    align-items: center;
    gap: 10px;
    margin-top: 20px;
  }
  
  .pagination button {
    padding: 5px 10px;
    border: 1px solid #dee2e6;
    background: #f8f9fa;
    border-radius: 4px;
    cursor: pointer;
  }
  
  .pagination button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  /* Removed unused CSS rules for the old inline editor */
</style>