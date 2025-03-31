<script lang="ts">
  import FileUploader from '$lib/components/FileUploader.svelte';
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  
  // Store for selected files
  let selectedFiles: File[] = [];
  // Store for extracted metadata
  let extractedMetadata: any[] = [];
  // Store for file paths
  let selectedFilePaths: string[] = [];
  // Loading state
  let isLoading = false;
  let error: string | null = null;
  
  // Handle file selection event from the FileUploader component
  async function handleFilesSelected(event: CustomEvent<{ files: File[] }>) {
    selectedFiles = event.detail.files;
    console.log('Selected files:', selectedFiles);
    extractedMetadata = [];
    error = null;
    
    try {
      // Open the file dialog to select music files
      isLoading = true;
      selectedFilePaths = await invoke<string[]>('select_audio_files');
      
      if (selectedFilePaths.length > 0) {
        await extractMetadata();
      } else {
        isLoading = false;
      }
    } catch (err) {
      console.error('Failed to select files:', err);
      error = err instanceof Error ? err.message : String(err);
      isLoading = false;
    }
  }
  
  // Extract metadata from selected files
  async function extractMetadata() {
    if (selectedFilePaths.length === 0) {
      isLoading = false;
      return;
    }
    
    isLoading = true;
    error = null;
    
    try {
      // Call the Tauri command to extract metadata from the selected file paths
      console.log('Extracting metadata from paths:', selectedFilePaths);
      const metadata = await invoke<any[]>('extract_audio_metadata_batch', {
        filePaths: selectedFilePaths
      });
      
      extractedMetadata = metadata;
      console.log('Extracted metadata:', extractedMetadata);
    } catch (err) {
      console.error('Failed to extract metadata:', err);
      error = err instanceof Error ? err.message : String(err);
    } finally {
      isLoading = false;
    }
  }
  
  // Function to start the upload process
  function startUpload() {
    if (selectedFilePaths.length === 0) {
      alert('Please select at least one file to upload.');
      return;
    }
    
    // This will be expanded in future tasks to handle the actual upload
    console.log('Starting upload of', selectedFilePaths.length, 'files');
    // Future: call to Tauri backend for metadata extraction and upload
  }

  // Open file dialog directly
  async function openFileDialog() {
    try {
      isLoading = true;
      error = null;
      extractedMetadata = [];
      
      // Open the file dialog to select music files
      selectedFilePaths = await invoke<string[]>('select_audio_files');
      console.log('Selected file paths:', selectedFilePaths);
      
      if (selectedFilePaths.length > 0) {
        // Update the UI to show selected files
        selectedFiles = selectedFilePaths.map(path => {
          const filename = path.split('/').pop() || path;
          // Create a mock File object since we don't have the actual File object
          return {
            name: filename,
            size: 0,
            type: 'audio/unknown'
          } as File;
        });
        
        // Try to get file sizes
        try {
          const promises = selectedFilePaths.map(async (path, index) => {
            const stats = await invoke<{size: number}>('get_file_stats', { path });
            if (stats && selectedFiles[index]) {
              selectedFiles[index] = {...selectedFiles[index], size: stats.size};
            }
          });
          await Promise.allSettled(promises);
        } catch (err) {
          console.error('Error getting file stats:', err);
        }
        
        // Extract metadata
        await extractMetadata();
      }
    } catch (err) {
      console.error('Failed to select files:', err);
      error = err instanceof Error ? err.message : String(err);
    } finally {
      isLoading = false;
    }
  }
</script>

<div class="upload-page">
  <div class="header">
    <h1>Upload Music Files</h1>
    <p>Select music files to upload to your library</p>
  </div>
  
  <div class="upload-container">
    <div class="file-selector">
      <button class="select-button" on:click={openFileDialog}>
        Select Audio Files
      </button>
      <p class="or-text">or</p>
      <FileUploader on:filesSelected={handleFilesSelected} />
    </div>
    
    {#if isLoading}
      <div class="loading">
        <p>
          {#if extractedMetadata.length === 0}
            Loading...
          {:else}
            Extracting metadata from {selectedFilePaths.length} {selectedFilePaths.length === 1 ? 'file' : 'files'}...
          {/if}
        </p>
      </div>
    {/if}
    
    {#if error}
      <div class="error">
        <p>Error: {error}</p>
      </div>
    {/if}
    
    {#if extractedMetadata.length > 0}
      <div class="metadata-preview">
        <h3>Metadata Preview</h3>
        <div class="metadata-list">
          {#each extractedMetadata as item, i}
            <div class="metadata-item">
              <h4>{item.track.title}</h4>
              <div class="metadata-details">
                <div class="detail"><strong>Album:</strong> {item.album.name}</div>
                <div class="detail"><strong>Artist:</strong> {item.album.artist}</div>
                {#if item.track.duration}
                  <div class="detail"><strong>Duration:</strong> {Math.floor(item.track.duration / 60)}:{Math.floor(item.track.duration % 60).toString().padStart(2, '0')}</div>
                {/if}
                {#if item.track.genre && item.track.genre.length > 0}
                  <div class="detail"><strong>Genre:</strong> {item.track.genre.join(', ')}</div>
                {/if}
              </div>
            </div>
          {/each}
        </div>
      </div>
    {/if}
    
    {#if selectedFilePaths.length > 0}
      <div class="upload-actions">
        <button class="upload-button" on:click={startUpload} disabled={isLoading}>
          Upload {selectedFilePaths.length} {selectedFilePaths.length === 1 ? 'File' : 'Files'}
        </button>
      </div>
    {/if}
  </div>
  
  <div class="instructions">
    <h3>Supported file formats:</h3>
    <ul>
      <li>MP3 (.mp3)</li>
      <li>WAV (.wav)</li>
      <li>FLAC (.flac)</li>
      <li>AAC (.aac, .m4a)</li>
      <li>OGG (.ogg)</li>
      <li>AIFF (.aiff)</li>
    </ul>
  </div>
</div>

<style lang="postcss">
  .upload-page {
    max-width: 800px;
    margin: 0 auto;
    padding: 20px;
  }
  
  .header {
    margin-bottom: 30px;
  }
  
  .header h1 {
    margin-bottom: 8px;
    color: #2d3748;
  }
  
  .header p {
    color: #718096;
    margin: 0;
  }
  
  .upload-container {
    margin-bottom: 30px;
  }
  
  .file-selector {
    display: flex;
    flex-direction: column;
    align-items: center;
    margin-bottom: 20px;
  }
  
  .select-button {
    background-color: #4299e1;
    color: white;
    border: none;
    border-radius: 4px;
    padding: 10px 20px;
    font-size: 16px;
    cursor: pointer;
    transition: background-color 0.3s;
    width: 200px;
    margin-bottom: 10px;
  }
  
  .select-button:hover {
    background-color: #3182ce;
  }
  
  .or-text {
    margin: 10px 0;
    color: #718096;
  }
  
  .loading {
    margin-top: 20px;
    padding: 15px;
    background-color: #ebf8ff;
    border-radius: 6px;
    color: #2b6cb0;
    text-align: center;
  }
  
  .error {
    margin-top: 20px;
    padding: 15px;
    background-color: #fff5f5;
    border-radius: 6px;
    color: #c53030;
  }
  
  .metadata-preview {
    margin-top: 25px;
    background-color: #f7fafc;
    border-radius: 8px;
    padding: 20px;
  }
  
  .metadata-preview h3 {
    margin-top: 0;
    margin-bottom: 15px;
    color: #2d3748;
  }
  
  .metadata-list {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 15px;
  }
  
  .metadata-item {
    background-color: white;
    border-radius: 6px;
    padding: 15px;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  }
  
  .metadata-item h4 {
    margin-top: 0;
    margin-bottom: 10px;
    color: #4a5568;
    font-size: 16px;
  }
  
  .metadata-details {
    display: flex;
    flex-direction: column;
    gap: 5px;
  }
  
  .detail {
    font-size: 14px;
    color: #718096;
  }
  
  .upload-actions {
    margin-top: 20px;
    display: flex;
    justify-content: flex-end;
  }
  
  .upload-button {
    background-color: #38a169;
    color: white;
    border: none;
    border-radius: 4px;
    padding: 10px 20px;
    font-size: 16px;
    cursor: pointer;
    transition: background-color 0.3s;
  }
  
  .upload-button:hover:not(:disabled) {
    background-color: #2f855a;
  }
  
  .upload-button:disabled {
    background-color: #a0aec0;
    cursor: not-allowed;
  }
  
  .instructions {
    background-color: #f7fafc;
    border-radius: 8px;
    padding: 20px;
    margin-top: 30px;
  }
  
  .instructions h3 {
    margin-top: 0;
    margin-bottom: 12px;
    color: #2d3748;
    font-size: 18px;
  }
  
  .instructions ul {
    margin: 0;
    padding-left: 20px;
    columns: 2;
  }
  
  .instructions li {
    margin-bottom: 6px;
    color: #4a5568;
  }
</style> 