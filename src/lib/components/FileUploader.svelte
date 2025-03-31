<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  
  // Event dispatcher to communicate with parent components
  const dispatch = createEventDispatcher<{
    filesSelected: { files: File[] };
  }>();
  
  let fileInput: HTMLInputElement;
  let selectedFiles: File[] = [];
  let dragActive = false;
  
  // Accepted audio file types
  const acceptedFileTypes = [
    'audio/mpeg', 'audio/mp3', 'audio/wav', 'audio/x-wav',
    'audio/flac', 'audio/x-flac', 'audio/aac', 'audio/m4a',
    'audio/ogg', 'audio/x-aiff', 'audio/aiff'
  ];
  
  // File type string for input element
  const fileTypeString = '.mp3,.wav,.flac,.aac,.m4a,.ogg,.aiff';
  
  function handleFileSelect(event: Event) {
    const input = event.target as HTMLInputElement;
    if (input.files) {
      processFiles(input.files);
    }
  }
  
  function handleDragOver(event: DragEvent) {
    event.preventDefault();
    dragActive = true;
  }
  
  function handleDragLeave() {
    dragActive = false;
  }
  
  function handleDrop(event: DragEvent) {
    event.preventDefault();
    dragActive = false;
    
    if (event.dataTransfer?.files) {
      processFiles(event.dataTransfer.files);
    }
  }
  
  function processFiles(fileList: FileList) {
    const files = Array.from(fileList);
    
    // Filter for audio files
    const audioFiles = files.filter(file => {
      return acceptedFileTypes.includes(file.type) || 
        fileTypeString.includes(file.name.split('.').pop()?.toLowerCase() || '');
    });
    
    if (audioFiles.length > 0) {
      selectedFiles = [...selectedFiles, ...audioFiles];
      dispatch('filesSelected', { files: selectedFiles });
    }
  }
  
  function clearSelectedFiles() {
    selectedFiles = [];
    if (fileInput) fileInput.value = '';
    dispatch('filesSelected', { files: selectedFiles });
  }
</script>

<div class="file-uploader">
  <div 
    class="drop-area" 
    class:active={dragActive}
    on:dragover={handleDragOver}
    on:dragleave={handleDragLeave}
    on:drop={handleDrop}
  >
    <div class="drop-content">
      <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="feather feather-music"><path d="M9 18V5l12-2v13"></path><circle cx="6" cy="18" r="3"></circle><circle cx="18" cy="16" r="3"></circle></svg>
      <p>Drag and drop audio files here<br>or</p>
      <button 
        class="select-button"
        on:click={() => fileInput.click()}
      >
        Select Files
      </button>
      <input 
        bind:this={fileInput}
        type="file" 
        accept={fileTypeString}
        multiple
        on:change={handleFileSelect}
        style="display: none;"
      />
    </div>
  </div>
  
  {#if selectedFiles.length > 0}
    <div class="selected-files">
      <div class="files-header">
        <h3>Selected Files ({selectedFiles.length})</h3>
        <button class="clear-button" on:click={clearSelectedFiles}>Clear All</button>
      </div>
      <ul class="file-list">
        {#each selectedFiles as file, i}
          <li class="file-item">
            <span class="file-name">{file.name}</span>
            <span class="file-size">({(file.size / (1024 * 1024)).toFixed(2)} MB)</span>
          </li>
        {/each}
      </ul>
    </div>
  {/if}
</div>

<style lang="postcss">
  .file-uploader {
    width: 100%;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Open Sans', 'Helvetica Neue', sans-serif;
  }
  
  .drop-area {
    border: 2px dashed #ccc;
    border-radius: 8px;
    padding: 40px 20px;
    text-align: center;
    cursor: pointer;
    transition: all 0.3s;
    background-color: #f9f9f9;
  }
  
  .drop-area:hover, .drop-area.active {
    border-color: #4299e1;
    background-color: #ebf8ff;
  }
  
  .drop-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
  }
  
  .drop-content svg {
    color: #718096;
    margin-bottom: 12px;
  }
  
  .drop-content p {
    margin: 0;
    color: #4a5568;
    line-height: 1.5;
  }
  
  .select-button {
    background-color: #4299e1;
    color: white;
    border: none;
    border-radius: 4px;
    padding: 8px 16px;
    font-size: 14px;
    cursor: pointer;
    transition: background-color 0.3s;
  }
  
  .select-button:hover {
    background-color: #3182ce;
  }
  
  .selected-files {
    margin-top: 20px;
    background-color: #f9f9f9;
    border-radius: 8px;
    padding: 16px;
  }
  
  .files-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
  }
  
  .files-header h3 {
    margin: 0;
    font-size: 16px;
    color: #2d3748;
  }
  
  .clear-button {
    background-color: transparent;
    color: #e53e3e;
    border: 1px solid #e53e3e;
    border-radius: 4px;
    padding: 4px 8px;
    font-size: 12px;
    cursor: pointer;
    transition: all 0.3s;
  }
  
  .clear-button:hover {
    background-color: #e53e3e;
    color: white;
  }
  
  .file-list {
    list-style: none;
    padding: 0;
    margin: 0;
    max-height: 300px;
    overflow-y: auto;
  }
  
  .file-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 12px;
    border-bottom: 1px solid #edf2f7;
  }
  
  .file-item:last-child {
    border-bottom: none;
  }
  
  .file-name {
    font-size: 14px;
    color: #4a5568;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 70%;
  }
  
  .file-size {
    font-size: 12px;
    color: #718096;
  }
</style> 