<script lang="ts">
  import { onMount } from 'svelte'; // onDestroy might be removed if no longer needed
  import { safeInvoke } from '$lib/utils/invokeWrapper'; // Import the wrapper
  import { showSuccessToast, showErrorToast } from '$lib/stores/notifications'; // Import success and error toasts
  import FileUploader from '$lib/components/common/FileUploader.svelte';
  import type { UploadItemMetadata } from '$lib/types/catalog'; // Assuming type is defined here or create it
  import UploadMetadataEditor from '$features/upload/components/UploadMetadataEditor.svelte'; // Import the new component

  // Define TypeScript interfaces (UploadItemMetadata will be primary)
  // Using UploadItemMetadata from types/catalog.ts
  // Example inline definition (prefer importing if it exists):
  /*
  interface UploadItemMetadata {
      title?: string;
      artist?: string;
      album?: string;
      track_number?: number;
      duration_sec?: number;
      genre?: string;
      composer?: string;
      year?: number;
      comments?: string;
      // Add a field to store the original file path, crucial for linking back
      original_path: string;
  }
  */
  
  // Store for selected files
  let selectedFiles: File[] = []; // Keep track of original File objects if needed
  // Store for extracted metadata using the new structure
  let uploadItemsMetadata: UploadItemMetadata[] = [];
  // Store for file paths (still useful)
  let selectedFilePaths: string[] = [];
  // Loading state
  let isLoading = false;
  let error: string | null = null;
  let isUploading = false; // Simple flag for upload queue call
  
  
  // Add mongoStatus variable at the top of the script section
  let mongoStatus = '';

  // Metadata editing state (now managed within UploadMetadataEditor)
  let showMetadataEditor = false; // Simple flag to show/hide the editor component


  onMount(async () => {
    try {
      console.log('Attempting to invoke ping command...');
      const result = await safeInvoke('ping'); // Using safeInvoke as imported
      console.log('Ping command successful, result:', result); // Should log "pong"
    } catch (error) {
      // safeInvoke typically handles errors and shows toasts, but we log here too.
      console.error('Error invoking ping command (onMount catch):', error);
    }
  });
  
  
  // Handle file selection from FileUploader component
  async function handleFileSelection(event: CustomEvent<{ files: File[], paths: string[] }>) {
    selectedFiles = event.detail.files;
    selectedFilePaths = event.detail.paths;
    console.log('Files selected:', selectedFilePaths);
    // Reset states when new files are selected
    uploadItemsMetadata = []; // Reset new metadata store
    showMetadataEditor = false; // Hide editor on new selection
    error = null;
    isUploading = false; // Reset upload state
    // Automatically extract metadata after selection
    if (selectedFilePaths.length > 0) {
      await extractMetadataForAllFiles(); // Call the updated function
    }
  }


  // Extract metadata for all selected files using the new backend command
  async function extractMetadataForAllFiles() {
    if (selectedFilePaths.length === 0) {
      return; // Nothing to process
    }
    isLoading = true;
    error = null;
    const results: UploadItemMetadata[] = [];
    let hasErrors = false;

    console.log(`Starting metadata extraction for ${selectedFilePaths.length} files...`);

    for (const filePath of selectedFilePaths) {
      console.log(`Extracting metadata for: ${filePath}`);
      // Switch to the wrapper command
      const result = await safeInvoke<UploadItemMetadata>('extract_metadata_wrapper', {
         filePath: filePath // Match the parameter name expected by Rust
      });

      if (result !== null) {
        // Add the original path to the metadata object for reference
        results.push({ ...result, original_path: filePath });
        console.log(`Successfully extracted metadata for: ${filePath}`, result);
      } else {
        // safeInvoke already showed an error toast
        console.error(`Failed to extract metadata for: ${filePath}`);
        // Optionally add a placeholder or skip the file
        // For now, we just note that there was an error
        hasErrors = true;
      }
    }

    uploadItemsMetadata = results; // Update the state with all results (successes and potentially placeholders for failures)
    console.log('Finished metadata extraction. Results:', uploadItemsMetadata);

    if (hasErrors) {
       // Maybe show a general error if some failed, though individual errors were toasted
       error = "Metadata extraction failed for one or more files."; // Set component error state
    } else {
       showSuccessToast(`Successfully extracted metadata for ${results.length} files.`);
    }

    isLoading = false;
  }
  

  // --- Event Handlers for UploadMetadataEditor ---

  function handleMetadataUpdated(event: CustomEvent<{ updatedMetadata: UploadItemMetadata[] }>) {
    console.log('Metadata updated from editor component:', event.detail.updatedMetadata);
    uploadItemsMetadata = event.detail.updatedMetadata;
    // Optionally add a success toast or indication
  }

  function handleEditorCancel() {
    console.log('Metadata editing cancelled.');
    showMetadataEditor = false;
  }

  function handleEditorFinalize() {
    console.log('Metadata editing finalized.');
    showMetadataEditor = false;
    // Proceed to upload or next step if needed, but upload is now separate button
  }

  // Function to show the editor
  function openMetadataEditor() {
    showMetadataEditor = true;
  }

  // --- End Event Handlers ---






































































































































































    
    // Function to start the upload queue using the new backend command
    async function startUploadQueue() {
      if (uploadItemsMetadata.length === 0) {
        showErrorToast('No files with metadata ready for upload.');
        console.log('Upload queue start aborted - No metadata.');
        return;
      }

      isUploading = true; // Set loading state
      error = null; // Clear previous errors
      // Reset previous upload results if needed

      console.log(`Starting upload queue for ${uploadItemsMetadata.length} items.`);

      // Call the backend command to start the queue
      const result = await safeInvoke<boolean>('start_upload_queue', {
        items: uploadItemsMetadata // Pass the array of metadata objects
      });

      if (result === true) {
        showSuccessToast(`Upload queue started for ${uploadItemsMetadata.length} items. Monitor progress via events.`);
        console.log('Upload queue started successfully.');
        // Listen for upload progress/completion events from Tauri backend
        // For now, just clear the list after starting
        uploadItemsMetadata = [];
        selectedFilePaths = [];
        selectedFiles = [];
      } else {
        // safeInvoke already showed an error toast
        console.error('Failed to start the upload queue.');
        // Keep items in the list for retry? Or clear? Decide based on desired UX.
      }

      isUploading = false; // Reset loading state
    }

    // Open file dialog directly
    async function openFileDialog() {
      try {
        isLoading = true;
        error = null;
        uploadItemsMetadata = []; // Reset new metadata store
        
        // Open the file dialog to select music files
        selectedFilePaths = (await safeInvoke<string[]>('select_audio_files')) ?? [];
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
              const stats = await safeInvoke<{size: number}>('get_file_stats', { path });
              if (stats && selectedFiles[index]) {
                selectedFiles[index] = {...selectedFiles[index], size: stats.size};
              }
            });
            await Promise.allSettled(promises);
          } catch (err) {
            console.error('Error getting file stats:', err);
          }
          
          // Extract metadata
          await extractMetadataForAllFiles(); // Call updated function
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
    </div>
    
    <!-- Add debug button -->
    {#if import.meta.env.DEV}
      <div class="debug-section">
        <button 
          class="debug-button" 
          on:click={async () => {
            // Refactored debug logic using safeInvoke
            mongoStatus = 'Checking...';
            const stateInfo = await safeInvoke<string>('debug_mongo_state');
            mongoStatus = stateInfo ?? "Error checking state.";
            console.log('MongoDB state:', mongoStatus);

            if (mongoStatus.includes("NOT initialized")) {
              mongoStatus = "Attempting initialization...";
              const initSuccess = await safeInvoke<boolean>('init_mongo_client');
              if (initSuccess) {
                mongoStatus = "Initialization successful. Testing connection...";
                const testSuccess = await safeInvoke<boolean>('test_mongo_connection');
                if (testSuccess) {
                  mongoStatus = "Connection test successful!";
                  alert("MongoDB Connection Test Successful!");
                } else {
                  mongoStatus = "Initialization succeeded, but connection test failed (check toast/console).";
                  alert("MongoDB Connection Test Failed after initialization.");
                }
              } else {
                mongoStatus = "Initialization failed (check toast/console). Trying direct credential check...";
                // If init failed, check if credentials exist at least
                const credsExist = await safeInvoke<string>('get_mongo_credentials_wrapper');
                if (credsExist !== null) {
                   mongoStatus += " Credentials seem to exist but client init failed.";
                   alert("Client initialization failed, but credentials seem to exist. Check connection string format or network.");
                } else {
                   mongoStatus += " Could not retrieve credentials either.";
                   alert("Client initialization failed, and could not retrieve credentials.");
                }
              }
            } else if (mongoStatus.includes("initialized")) {
               // Already initialized, just test connection
               mongoStatus = "Client already initialized. Testing connection...";
               const testSuccess = await safeInvoke<boolean>('test_mongo_connection');
               if (testSuccess) {
                 mongoStatus = "Connection test successful!";
                 alert("MongoDB Connection Test Successful!");
               } else {
                 mongoStatus = "Connection test failed (check toast/console).";
                 alert("MongoDB Connection Test Failed.");
               }
            }
          }}
        >
          Test MongoDB Connection
        </button>
        <div class="debug-status">
          MongoDB Status: {mongoStatus || 'Unknown'}
        </div>
      </div>
    {/if}
    
    {#if isLoading}
      <div class="loading">
        <p>
          {#if uploadItemsMetadata.length === 0}
            Loading files...
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
    
    {#if uploadItemsMetadata.length > 0 && !showMetadataEditor}
      <div class="metadata-preview">
        <h3>Files Ready for Upload ({uploadItemsMetadata.length})</h3>
        <div class="metadata-list">
          {#each uploadItemsMetadata as item, i (item.original_path)}
            <div class="metadata-item">
              <!-- Display basic info from UploadItemMetadata -->
              <h4>{item.title ?? item.original_path.split('/').pop() ?? 'Unknown Title'}</h4>
              <div class="metadata-details">
                {#if item.album}<div class="detail"><strong>Album:</strong> {item.album}</div>{/if}
                {#if item.artist}<div class="detail"><strong>Artist:</strong> {item.artist}</div>{/if}
                {#if item.duration_sec}
                  <div class="detail"><strong>Duration:</strong> {Math.floor(item.duration_sec / 60)}:{Math.floor(item.duration_sec % 60).toString().padStart(2, '0')}</div>
                {/if}
                {#if item.genre}
                  <div class="detail"><strong>Genre:</strong> {item.genre}</div>
                {/if}
                 <div class="detail"><strong>Path:</strong> {item.original_path}</div>
              </div>
            </div>
          {/each}
        </div>
         <div class="edit-metadata-actions">
            <button
              class="edit-metadata-button"
              on:click={openMetadataEditor}
              disabled={isLoading || isUploading}
            >
              Edit Metadata Before Upload
            </button>
          </div>
      </div>
    {/if}


    <!-- Use the new UploadMetadataEditor component -->
    {#if showMetadataEditor && uploadItemsMetadata.length > 0}
      <UploadMetadataEditor
        bind:uploadItemsMetadata={uploadItemsMetadata}
        on:metadataUpdated={handleMetadataUpdated}
        on:cancel={handleEditorCancel}
        on:finalize={handleEditorFinalize}
      />
    {/if}






































































































































































    
    {#if isUploading}
      <div class="upload-progress">
        <p>Starting upload queue... Please wait.</p>
      </div>
    {/if}
    
    
    {#if uploadItemsMetadata.length > 0 && !showMetadataEditor}
      <div class="upload-actions">
        
        <button
          class="upload-button"
          on:click={startUploadQueue}
          disabled={isLoading || isUploading}
        >
          Start Upload Queue ({uploadItemsMetadata.length} {uploadItemsMetadata.length === 1 ? 'Item' : 'Items'})
        </button>
      </div>
      
      {#if import.meta.env.DEV}
        <div class="debug-section">
          <div class="debug-info">
            <p><strong>Button Control Variables:</strong></p>
            <ul>
              <li>isLoading: {isLoading} (Metadata Extraction)</li>
              <li>isUploading: {isUploading} (Starting Queue)</li>
              <li>uploadItemsMetadata.length: {uploadItemsMetadata.length}</li>
              <li>Upload Button should be {isLoading || isUploading ? 'DISABLED' : 'ENABLED'}</li>
            </ul>
            <button 
              class="debug-button" 
              on:click={() => {
                isLoading = false; // Reset metadata loading
                isUploading = false; // Reset queue start loading
                console.log('Manually reset loading flags');
              }}
            >
              Reset Loading Flags
            </button>
          </div>
        </div>
      {/if}
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
  
  .loading, .error {
    padding: 10px;
    border-radius: 4px;
    margin: 10px 0;
  }
  
  .loading {
    background-color: #ebf8ff;
    color: #3182ce;
  }
  
  .error {
    background-color: #fff5f5;
    color: #e53e3e;
  }
  
  .metadata-preview, .transcoding-options, .transcoded-files, .uploaded-files {
    margin-top: 20px;
    padding: 16px;
    background-color: #f9f9f9;
    border-radius: 8px;
  }
  
  .metadata-preview h3, .transcoding-options h3, .transcoded-files h3, .uploaded-files h3 {
    margin-top: 0;
    margin-bottom: 16px;
    color: #2d3748;
  }
  
  .uploaded-files h4 {
    margin-top: 16px;
    margin-bottom: 8px;
    color: #4a5568;
    font-size: 16px;
  }
  
  .metadata-list {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 16px;
  }
  
  .metadata-item {
    background-color: white;
    padding: 12px;
    border-radius: 4px;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  }
  
  .metadata-item h4 {
    margin-top: 0;
    margin-bottom: 8px;
    color: #2d3748;
  }
  
  .metadata-details {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  
  .detail {
    font-size: 14px;
    color: #4a5568;
  }
  
  .options-form {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 16px;
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
  
  .form-group input, .form-group select {
    padding: 8px;
    border: 1px solid #e2e8f0;
    border-radius: 4px;
    background-color: white;
  }
  
  .transcoding-progress, .upload-progress {
    margin-top: 20px;
    padding: 16px;
    background-color: #ebf8ff;
    border-radius: 8px;
  }
  
  .progress-bar {
    height: 10px;
    background-color: #e2e8f0;
    border-radius: 5px;
    overflow: hidden;
    margin-top: 10px;
  }
  
  .progress-fill {
    height: 100%;
    background-color: #4299e1;
    transition: width 0.3s ease;
  }
  
  .transcoded-list, .uploaded-list, .failed-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  
  .transcoded-item, .uploaded-item, .failed-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 12px;
    background-color: white;
    border-radius: 4px;
    border-left: 3px solid #48bb78;
  }
  
  .transcoded-item.failed, .failed-item {
    border-left-color: #e53e3e;
  }
  
  .file-path {
    font-size: 14px;
    color: #4a5568;
  }
  
  .file-info {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  
  .file-title {
    font-weight: 600;
    font-size: 14px;
    color: #2d3748;
  }
  
  .file-album {
    font-size: 12px;
    color: #718096;
  }
  
  .success-label {
    font-size: 12px;
    color: #48bb78;
  }
  
  .error-label {
    font-size: 12px;
    color: #e53e3e;
  }
  
  .upload-actions {
    display: flex;
    justify-content: center;
    gap: 16px;
    margin-top: 24px;
  }
  
  .transcode-button, .upload-button {
    padding: 10px 20px;
    border: none;
    border-radius: 4px;
    font-size: 16px;
    cursor: pointer;
    transition: background-color 0.3s;
  }
  
  .transcode-button {
    background-color: #48bb78;
    color: white;
  }
  
  .transcode-button:hover {
    background-color: #38a169;
  }
  
  .upload-button {
    background-color: #4299e1;
    color: white;
  }
  
  .upload-button:hover {
    background-color: #3182ce;
  }
  
  .upload-button:disabled, .transcode-button:disabled {
    background-color: #a0aec0;
    cursor: not-allowed;
  }
  
  .instructions {
    background-color: #f9f9f9;
    padding: 16px;
    border-radius: 8px;
  }
  
  .instructions h3 {
    margin-top: 0;
    margin-bottom: 12px;
    color: #2d3748;
  }
  
  .instructions ul {
    margin: 0;
    padding-left: 20px;
  }
  
  .instructions li {
    margin-bottom: 6px;
    color: #4a5568;
  }
  
  .debug-section {
    margin-top: 16px;
    border-top: 1px dashed #ccc;
    padding-top: 16px;
  }
  
  .debug-button {
    background-color: #6b7280;
    color: white;
    padding: 8px 16px;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 14px;
  }
  
  .debug-button:hover {
    background-color: #4b5563;
  }
  
  .debug-status {
    margin-top: 8px;
    padding: 4px 8px;
    background-color: #f3f4f6;
    border-radius: 4px;
    font-size: 14px;
    color: #4b5563;
  }
  
  .debug-info {
    margin-bottom: 16px;
  }
  

  /* Keep existing styles for preview, buttons etc. */
  .edit-metadata-actions { /* Style for the button container */
    margin-top: 20px;
    display: flex;
    justify-content: center;
  }

  .edit-metadata-button { /* Style for the "Edit Metadata" button */
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

  .edit-metadata-button:disabled {
    background-color: #a0aec0;
    cursor: not-allowed;
  }

</style>
