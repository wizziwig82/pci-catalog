<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  
  // Define TypeScript interfaces
  interface PathInfo {
    original: string;
    medium: string;
  }
  
  interface Track {
    _id: string;
    title: string;
    album_id: string;
    album_name?: string;
    duration: number;
    genre?: string;
    filename: string;
    writers: string[];
    writer_percentages: number[];
    publishers: string[];
    publisher_percentages: number[];
    instruments: string[];
    mood: string[];
    comments?: string;
    path: PathInfo;
  }
  
  interface TrackListResponse {
    success: boolean;
    message?: string;
    tracks: Track[];
    total_count: number;
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
  let selectedTrackIndex = -1;
  let isEditing = false;
  
  // Sorting state
  let sortField = 'title';
  let sortDirection = 'asc'; // 'asc' or 'desc'
  
  // Pagination
  let currentPage = 1;
  let tracksPerPage = 50;
  
  onMount(async () => {
    await loadTracks();
  });
  
  async function loadTracks() {
    isLoading = true;
    error = null;
    
    try {
      console.log('Attempting to initialize MongoDB client...');
      // Initialize MongoDB client
      const mongoInitialized = await invoke<boolean>('init_mongo_client');
      console.log('MongoDB client initialization result:', mongoInitialized);
      
      if (!mongoInitialized) {
        throw new Error('Failed to initialize MongoDB client. Please check your credentials in Settings.');
      }
      
      // Calculate skip value for pagination
      const skip = (currentPage - 1) * tracksPerPage;
      
      console.log('Fetching tracks with params:', {
        sortField,
        sortDirection,
        limit: tracksPerPage,
        skip
      });
      
      // Fetch tracks from MongoDB
      const result = await invoke<TrackListResponse>('fetch_all_tracks', {
        sortField,
        sortDirection,
        limit: tracksPerPage,
        skip
      });
      
      console.log('Track fetch result:', result);
      
      if (result.success) {
        tracks = result.tracks;
        totalTracks = result.total_count;
        console.log('Tracks loaded:', tracks.length, 'Total count:', totalTracks);
      } else {
        throw new Error(result.message || 'Failed to fetch tracks');
      }
    } catch (err) {
      console.error('Failed to load tracks:', err);
      error = err instanceof Error ? err.message : String(err);
    } finally {
      isLoading = false;
    }
  }
  
  async function testMongoDBCollections() {
    isTestingMongo = true;
    mongoTestResult = null;
    
    try {
      console.log('Testing MongoDB collections...');
      const result = await invoke<string>('test_mongodb_collections');
      console.log('MongoDB test result:', result);
      mongoTestResult = result;
    } catch (err) {
      console.error('Failed to test MongoDB collections:', err);
      mongoTestResult = `Error: ${err instanceof Error ? err.message : String(err)}`;
    } finally {
      isTestingMongo = false;
    }
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
  
  function handleTrackSelect(trackId: string, index: number) {
    selectedTrackIndex = index;
    
    // Toggle selection if already selected
    if (selectedTrackIds.includes(trackId)) {
      selectedTrackIds = selectedTrackIds.filter(id => id !== trackId);
    } else {
      selectedTrackIds = [...selectedTrackIds, trackId];
    }
  }
  
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
    <div class="table-actions">
      <button disabled={selectedTrackIds.length === 0}>Edit Selected</button>
      <button disabled={selectedTrackIds.length === 0}>Delete Selected</button>
      <button disabled={selectedTrackIds.length !== 1}>Replace Audio</button>
    </div>
    
    <div class="table-container">
      <table class="tracks-table">
        <thead>
          <tr>
            <th class="select-column">
              <input 
                type="checkbox" 
                checked={selectedTrackIds.length === tracks.length && tracks.length > 0} 
                on:change={() => {
                  if (selectedTrackIds.length === tracks.length) {
                    selectedTrackIds = [];
                  } else {
                    selectedTrackIds = tracks.map(track => track._id);
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
          {#each tracks as track, index}
            <tr class:selected={selectedTrackIds.includes(track._id)} on:click={() => handleTrackSelect(track._id, index)}>
              <td>
                <input 
                  type="checkbox" 
                  checked={selectedTrackIds.includes(track._id)} 
                  on:click={(e) => e.stopPropagation()} 
                  on:change={() => handleTrackSelect(track._id, index)}
                />
              </td>
              <td>{track.title}</td>
              <td>{track.album_name || 'Unknown Album'}</td>
              <td>{formatDuration(track.duration)}</td>
              <td>{track.genre || 'Unknown'}</td>
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
</style> 