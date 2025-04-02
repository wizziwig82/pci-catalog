<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import FileUploader from '$lib/components/FileUploader.svelte';
  import TagSelector from '$lib/components/TagSelector.svelte';
  import { instrumentTags, moodTags, tagsToString } from '$lib/stores/tagData';
  
  // Define TypeScript interfaces
  interface TranscodingResult {
    success: boolean;
    input_path: string;
    medium_quality_path?: string;
    error?: string;
    output_format: string;
    bitrate: number;
  }

  interface Track {
    title: string;
    album_id: string;
    genre: string[];
    duration: number;
    original_path: string;
    writers: string[];
    writer_percentages: number[];
    publishers: string[];
    publisher_percentages: number[];
    instruments: string[];
    mood: string[];
    comments: string;
    // Add other track fields as needed
  }

  interface Album {
    name: string;
    artist: string;
    // Add other album fields as needed
  }

  interface ExtractedMetadata {
    track: Track;
    album: Album;
    fileInfo: {
      path: string;
      size: number;
      // Add other file info fields as needed
    };
  }
  
  // Store for selected files
  let selectedFiles: File[] = [];
  // Store for extracted metadata
  let extractedMetadata: ExtractedMetadata[] = [];
  // Store for file paths
  let selectedFilePaths: string[] = [];
  // Loading state
  let isLoading = false;
  let error: string | null = null;
  // Transcoding state
  let isTranscoding = false;
  let transcodingProgress = 0;
  let transcodedFiles: any[] = [];
  // Upload state
  let isUploading = false;
  let uploadProgress = 0;
  let uploadedFiles: any[] = [];
  let failedUploads: any[] = [];
  
  // Transcoding options
  let mediumBitrate = 128;
  let outputFormat = 'mp3';
  let outputDir = 'transcoded';
  
  // Add mongoStatus variable at the top of the script section
  let mongoStatus = '';

  // Metadata editing state
  let isEditingMetadata = false;
  let selectedTrackIndex = -1;
  let bulkEditMode = false;
  let selectedTrackIndices: number[] = [];
  
  // Fields for bulk editing
  let bulkEditFields = {
    album: "",
    artist: "",
    genre: "",
    writers: "",
    writer_percentages: [] as number[],
    publishers: "",
    publisher_percentages: [] as number[],
    instruments: "",
    mood: ""
  };

  // Writer and publisher percentages validation
  let writerPercentagesValid = true;
  let publisherPercentagesValid = true;
  
  // Add selected track state variables for instruments and moods
  let selectedTrackInstruments: string[] = [];
  let selectedTrackMood: string[] = [];
  
  // Handle tag selector changes
  function handleInstrumentTagsChanged(event: CustomEvent<{ tags: string[] }>) {
    if (selectedTrackIndex >= 0) {
      selectedTrackInstruments = event.detail.tags;
      extractedMetadata[selectedTrackIndex].track.instruments = event.detail.tags;
    }
  }
  
  function handleMoodTagsChanged(event: CustomEvent<{ tags: string[] }>) {
    if (selectedTrackIndex >= 0) {
      selectedTrackMood = event.detail.tags;
      extractedMetadata[selectedTrackIndex].track.mood = event.detail.tags;
    }
  }
  
  // Handle bulk edit tag changes
  function handleBulkInstrumentTagsChanged(event: CustomEvent<{ tags: string[] }>) {
    bulkEditFields.instruments = tagsToString(event.detail.tags);
  }
  
  function handleBulkMoodTagsChanged(event: CustomEvent<{ tags: string[] }>) {
    bulkEditFields.mood = tagsToString(event.detail.tags);
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
  
  // Function to transcode files in a batch
  async function transcodeFiles() {
    if (selectedFilePaths.length === 0) {
      error = 'No files selected';
      return;
    }
    
    isTranscoding = true;
    transcodingProgress = 0;
    error = null;
    transcodedFiles = []; // Clear previous transcoded files
    
    try {
      console.log('Starting batch transcoding of', selectedFilePaths.length, 'files');
      
      // Call the batch transcoding function
      const results = await invoke<TranscodingResult[]>('transcode_audio_batch', {
        filePaths: selectedFilePaths,
        mediumBitrate, 
        outputFormat,
        outputDir 
      });
      
      console.log('Transcoding results:', results);
      
      // Store the transcoding results
      transcodedFiles = results || [];
      
      // Filter successful transcodes
      const successfulTranscodes = transcodedFiles.filter(file => file.success);
      console.log(`Successfully transcoded ${successfulTranscodes.length} of ${selectedFilePaths.length} files`);
      
      if (successfulTranscodes.length === 0) {
        error = 'No files were successfully transcoded';
      }
    } catch (err) {
      console.error('Failed to transcode files:', err);
      error = err instanceof Error ? err.message : String(err);
    } finally {
      isTranscoding = false;
      transcodingProgress = 100;
      console.log('Transcoding completed, isTranscoding set to:', isTranscoding);
      console.log('Number of transcoded files:', transcodedFiles?.length || 0);
    }
  }
  
  // Function to upload files to R2 and store metadata in MongoDB
  async function uploadFiles() {
    console.log('uploadFiles() called - Starting upload process');
    console.log('Current state - isLoading:', isLoading, 'isTranscoding:', isTranscoding, 'isUploading:', isUploading, 'transcodedFiles.length:', transcodedFiles.length);
    
    if (transcodedFiles.length === 0 || extractedMetadata.length === 0) {
      alert('Please transcode files before uploading.');
      console.log('Upload aborted - No transcoded files or metadata');
      return;
    }
    
    // Check if any transcoding was successful
    const successfulTranscodes = transcodedFiles.filter(file => file.success);
    console.log('Successful transcodes:', successfulTranscodes.length, 'of', transcodedFiles.length);
    
    if (successfulTranscodes.length === 0) {
      alert('No files were successfully transcoded. Cannot proceed with upload.');
      console.log('Upload aborted - No successful transcodes');
      return;
    }
    
    isUploading = true;
    uploadProgress = 0;
    uploadedFiles = [];
    failedUploads = [];
    error = null;
    
    try {
      console.log('Uploading transcoded files to R2 and storing metadata in MongoDB');
      
      // Initialize R2 client first
      console.log('Attempting to initialize R2 client...');
      const r2Initialized = await invoke<boolean>('init_r2_client');
      console.log('R2 client initialization result:', r2Initialized);
      if (!r2Initialized) {
        throw new Error('Failed to initialize R2 client. Please check your credentials in Settings.');
      }
      
      // Initialize MongoDB client
      console.log('Attempting to initialize MongoDB client...');
      const mongoInitialized = await invoke<boolean>('init_mongo_client');
      console.log('MongoDB client initialization result:', mongoInitialized);
      if (!mongoInitialized) {
        throw new Error('Failed to initialize MongoDB client. Please check your credentials in Settings.');
      }
      
      // Filter out metadata for files that were successfully transcoded
      const successfulFilePaths = successfulTranscodes.map(file => file.input_path);
      console.log('Successful file paths:', successfulFilePaths);
      console.log('Metadata:', extractedMetadata);
      
      const filteredMetadata = extractedMetadata.filter(metadata => {
        const metadataPath = metadata.track.original_path || metadata.fileInfo.path;
        return successfulFilePaths.some(path => path === metadataPath);
      });
      
      console.log('Filtered metadata:', filteredMetadata);
      console.log('Transcoding results to upload:', successfulTranscodes);
      
      if (filteredMetadata.length !== successfulTranscodes.length) {
        console.warn(`Mismatch between metadata (${filteredMetadata.length}) and transcoded files (${successfulTranscodes.length})`);
      }
      
      // Process metadata into the format expected by the backend
      const processedMetadata = filteredMetadata.map(metadata => {
        const processed = JSON.parse(JSON.stringify(metadata));
        
        // Ensure all required fields exist with valid data
        if (!processed.track.writers) {
          processed.track.writers = {};
        } else if (Array.isArray(processed.track.writers)) {
          // Convert array to object with equal shares
          const writerObj: Record<string, number> = {};
          processed.track.writers.forEach((writer: string) => {
            writerObj[writer] = Math.floor(100 / processed.track.writers.length);
          });
          processed.track.writers = writerObj;
        }
        
        if (!processed.track.publishers) {
          processed.track.publishers = {};
        } else if (Array.isArray(processed.track.publishers)) {
          // Convert array to object with equal shares
          const publisherObj: Record<string, number> = {};
          processed.track.publishers.forEach((publisher: string) => {
            publisherObj[publisher] = Math.floor(100 / processed.track.publishers.length);
          });
          processed.track.publishers = publisherObj;
        }
        
        // Ensure genre is always an array
        if (processed.track.genre && !Array.isArray(processed.track.genre)) {
          processed.track.genre = [processed.track.genre];
        } else if (!processed.track.genre || processed.track.genre.length === 0) {
          processed.track.genre = ["Unclassified"];
        }
        
        // Make sure instruments is available at the top level since MongoDB expects it
        if (!processed.track.instruments) {
          processed.track.instruments = [];
        }
        
        // Make sure mood is available at the top level
        if (!processed.track.mood) {
          processed.track.mood = [];
        }
        
        // Add comments to custom_metadata if it exists
        if (processed.track.comments) {
          processed.track.custom_metadata = processed.track.custom_metadata || {};
          processed.track.custom_metadata.comments = processed.track.comments;
          delete processed.track.comments;
        }
        
        return processed;
      });
      
      // Upload files one by one with delay
      console.log('Starting sequential upload process...');
      
      const uploadedTracks: any[] = [];
      const failedTracks: any[] = [];
      let currentIndex = 0;
      
      const uploadNextTrack = async () => {
        if (currentIndex >= successfulTranscodes.length) {
          // All tracks processed, show results
          console.log('All tracks processed. Successes:', uploadedTracks.length, 'Failures:', failedTracks.length);
          handleUploadResults({
            success: failedTracks.length === 0,
            message: failedTracks.length === 0 
              ? `Successfully uploaded ${uploadedTracks.length} tracks` 
              : `Uploaded ${uploadedTracks.length} tracks with ${failedTracks.length} failures`,
            uploaded_tracks: uploadedTracks,
            failed_tracks: failedTracks
          });
          return;
        }
        
        // Update progress
        uploadProgress = Math.floor((currentIndex / successfulTranscodes.length) * 100);
        
        // Get current track and metadata
        const currentTranscode = successfulTranscodes[currentIndex];
        const currentMetadata = processedMetadata[currentIndex];
        
        try {
          console.log(`Uploading track ${currentIndex + 1}/${successfulTranscodes.length}: ${currentTranscode.input_path}`);
          
          // Create a single-item array for this track's metadata
          const singleItemArray = [currentMetadata];
          
          // Try to upload this single track
          const result = await invoke<{
            success: boolean;
            message: string;
            uploaded_tracks: any[];
            failed_tracks: any[];
          }>('upload_transcoded_tracks', {
            transcodingResults: [currentTranscode],
            audioMetadataList: singleItemArray,
            pathConfig: {
              original_prefix: 'tracks/original',
              medium_prefix: 'tracks/medium',
              album_art_prefix: 'albums/artwork'
            }
          });
          
          console.log(`Upload result for track ${currentIndex + 1}:`, result);
          
          // Add results to our tracking arrays
          if (result.uploaded_tracks && result.uploaded_tracks.length > 0) {
            uploadedTracks.push(...result.uploaded_tracks);
          }
          
          if (result.failed_tracks && result.failed_tracks.length > 0) {
            failedTracks.push(...result.failed_tracks);
          }
        } catch (err) {
          console.error(`Failed to upload track ${currentIndex + 1}:`, err);
          
          // Add to failed tracks
          failedTracks.push({
            original_path: currentTranscode.input_path,
            error: err instanceof Error ? err.message : String(err)
          });
        }
        
        // Move to next track after a small delay to avoid overwhelming MongoDB
        currentIndex++;
        setTimeout(uploadNextTrack, 500); // 500ms delay between uploads
      };
      
      // Start the sequential upload process
      uploadNextTrack();
    } catch (err) {
      console.error('Failed to upload files:', err);
      error = err instanceof Error ? err.message : String(err);
      isUploading = false;
      uploadProgress = 100;
    }
  }
  
  // Helper function to handle upload results
  function handleUploadResults(uploadResult: { 
    success: boolean; 
    message: string; 
    uploaded_tracks: any[]; 
    failed_tracks: any[];
  }) {
    console.log('Upload complete, final results:', uploadResult);
    
    if (uploadResult.uploaded_tracks) {
      uploadedFiles = uploadResult.uploaded_tracks;
    }
    
    if (uploadResult.failed_tracks) {
      failedUploads = uploadResult.failed_tracks;
    }
    
    if (uploadResult.message) {
      if (!uploadResult.success) {
        error = uploadResult.message;
      } else {
        alert(uploadResult.message);
      }
    }
    
    // Mark upload as complete
    isUploading = false;
    uploadProgress = 100;
  }

  // Function to enable metadata editing mode after transcoding
  function editMetadata() {
    isEditingMetadata = true;
  }

  // Function to edit a specific track
  function editTrack(index: number) {
    selectedTrackIndex = index;
    bulkEditMode = false;
  }

  // Function to toggle track selection for bulk editing
  function toggleTrackSelection(index: number) {
    const idx = selectedTrackIndices.indexOf(index);
    if (idx === -1) {
      selectedTrackIndices = [...selectedTrackIndices, index];
    } else {
      selectedTrackIndices = selectedTrackIndices.filter(i => i !== index);
    }
  }

  // Function to select/deselect all tracks
  function selectAllTracks(select: boolean) {
    if (select) {
      // Select all tracks
      selectedTrackIndices = [...Array(extractedMetadata.length).keys()];
    } else {
      // Deselect all tracks
      selectedTrackIndices = [];
    }
  }

  // Function to enable bulk edit mode
  function startBulkEdit() {
    bulkEditMode = true;
    selectedTrackIndex = -1;
  }

  // Function to apply bulk edits to selected tracks
  function applyBulkEdits() {
    if (selectedTrackIndices.length === 0) {
      alert('Please select at least one track to edit');
      return;
    }

    for (const index of selectedTrackIndices) {
      if (bulkEditFields.album) {
        extractedMetadata[index].album.name = bulkEditFields.album;
      }
      if (bulkEditFields.artist) {
        extractedMetadata[index].album.artist = bulkEditFields.artist;
      }
      if (bulkEditFields.genre) {
        extractedMetadata[index].track.genre = bulkEditFields.genre.split(',').map(g => g.trim());
      }
      if (bulkEditFields.writers) {
        extractedMetadata[index].track.writers = bulkEditFields.writers.split(',').map(w => w.trim());
        // Reset percentages when writers change
        extractedMetadata[index].track.writer_percentages = 
          extractedMetadata[index].track.writers.map(() => 
            Math.floor(100 / extractedMetadata[index].track.writers.length));
      }
      if (bulkEditFields.publishers) {
        extractedMetadata[index].track.publishers = bulkEditFields.publishers.split(',').map(p => p.trim());
        // Reset percentages when publishers change
        extractedMetadata[index].track.publisher_percentages = 
          extractedMetadata[index].track.publishers.map(() => 
            Math.floor(100 / extractedMetadata[index].track.publishers.length));
      }
      if (bulkEditFields.instruments) {
        extractedMetadata[index].track.instruments = bulkEditFields.instruments.split(',').map(i => i.trim());
      }
      if (bulkEditFields.mood) {
        extractedMetadata[index].track.mood = bulkEditFields.mood.split(',').map(m => m.trim());
      }
    }

    // Reset bulk edit fields
    bulkEditFields = {
      album: "",
      artist: "",
      genre: "",
      writers: "",
      writer_percentages: [],
      publishers: "",
      publisher_percentages: [],
      instruments: "",
      mood: ""
    };
    
    bulkEditMode = false;
    selectedTrackIndices = [];
  }

  // Function to validate percentages
  function validatePercentages() {
    if (selectedTrackIndex >= 0) {
      const track = extractedMetadata[selectedTrackIndex].track;
      
      if (track.writer_percentages && track.writer_percentages.length > 0) {
        const writerSum = track.writer_percentages.reduce((sum, percent) => sum + percent, 0);
        writerPercentagesValid = Math.abs(writerSum - 100) < 0.01;
      }
      
      if (track.publisher_percentages && track.publisher_percentages.length > 0) {
        const publisherSum = track.publisher_percentages.reduce((sum, percent) => sum + percent, 0);
        publisherPercentagesValid = Math.abs(publisherSum - 100) < 0.01;
      }
    }
  }

  // Function to save individual track edits
  function saveTrackEdits() {
    validatePercentages();
    if (!writerPercentagesValid) {
      alert('Writer percentages must sum to 100%');
      return;
    }
    if (!publisherPercentagesValid) {
      alert('Publisher percentages must sum to 100%');
      return;
    }
    
    selectedTrackIndex = -1;
  }

  // Function to cancel editing and go back
  function cancelEditing() {
    isEditingMetadata = false;
    selectedTrackIndex = -1;
    bulkEditMode = false;
    selectedTrackIndices = [];
  }

  // Function to finalize metadata and proceed to upload
  function finalizeMetadata() {
    isEditingMetadata = false;
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
      transcodedFiles = [];
      uploadedFiles = [];
      failedUploads = [];
      
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
    </div>
    
    <!-- Add debug button -->
    {#if import.meta.env.DEV}
      <div class="debug-section">
        <button 
          class="debug-button" 
          on:click={async () => {
            try {
              console.log('Getting MongoDB client state...');
              mongoStatus = 'Checking...';
              const stateInfo = await invoke<string>('debug_mongo_state');
              console.log('MongoDB state:', stateInfo);
              mongoStatus = stateInfo;
              
              // Try initializing the client
              console.log('Testing MongoDB client initialization...');
              const mongoInitialized = await invoke<boolean>('init_mongo_client');
              console.log('MongoDB client initialization result:', mongoInitialized);
              
              if (mongoInitialized) {
                // Now test the connection
                try {
                  console.log('Testing MongoDB connection...');
                  const connectionResult = await invoke<boolean>('test_mongo_connection');
                  console.log('MongoDB connection test result:', connectionResult);
                  
                  if (connectionResult) {
                    alert('MongoDB connection successful! Client is properly initialized and connected.');
                    mongoStatus = 'Connection successful! Client is properly initialized and connected.';
                  } else {
                    alert('MongoDB client initialized but connection test failed.');
                    mongoStatus = 'Client initialized but connection test failed.';
                  }
                } catch (connError) {
                  console.error('MongoDB connection test error:', connError);
                  alert(`MongoDB client initialized but connection test failed: ${connError}`);
                  mongoStatus = `Connection test failed: ${connError}`;
                }
              } else {
                // If initialization failed, try to get credentials and create client directly
                try {
                  console.log('Attempting to create MongoDB client directly...');
                  // Create a direct connection with the string we have in the dev file
                  const mongoConn = await invoke<string>('get_mongo_credentials');
                  console.log('Got MongoDB connection string (masked):', 
                    mongoConn.length > 20 ? 
                      `${mongoConn.substring(0, 10)}...${mongoConn.substring(mongoConn.length - 10)}` : 
                      '(too short to mask)');
                  
                  // Try to create the client directly
                  const directClientResult = await invoke<boolean>('create_mongodb_client', { connectionString: mongoConn });
                  console.log('Direct client creation result:', directClientResult);
                  
                  if (directClientResult) {
                    alert('Successfully created MongoDB client directly! The issue was with the client initialization process, not the credentials.');
                    mongoStatus = 'Client created directly with valid credentials.';
                  } else {
                    alert('Failed to create MongoDB client directly. There may be an issue with the connection string format.');
                    mongoStatus = 'Failed to create client directly.';
                  }
                } catch (directErr) {
                  console.error('Error creating MongoDB client directly:', directErr);
                  alert(`Failed to create MongoDB client directly: ${directErr}`);
                  mongoStatus = `Failed to create client directly: ${directErr}`;
                }
              }
            } catch (err) {
              console.error('Error during MongoDB debugging:', err);
              alert(`Error during MongoDB debugging: ${err}`);
              mongoStatus = `Error: ${err}`;
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
      
      <div class="transcoding-options">
        <h3>Transcoding Options</h3>
        <div class="options-form">
          <div class="form-group">
            <label for="medium-bitrate">Medium Quality Bitrate (kbps)</label>
            <input 
              id="medium-bitrate" 
              type="number" 
              bind:value={mediumBitrate} 
              min="64" 
              max="320" 
              step="32"
            />
          </div>
          
          <div class="form-group">
            <label for="output-format">Output Format</label>
            <select id="output-format" bind:value={outputFormat}>
              <option value="mp3">MP3</option>
              <option value="aac">AAC</option>
              <option value="ogg">OGG</option>
              <option value="flac">FLAC</option>
            </select>
          </div>
          
          <div class="form-group">
            <label for="output-dir">Output Directory</label>
            <input 
              id="output-dir" 
              type="text" 
              bind:value={outputDir} 
            />
          </div>
        </div>
      </div>
    {/if}
    
    {#if isTranscoding}
      <div class="transcoding-progress">
        <p>Transcoding files... Please wait.</p>
        <div class="progress-bar">
          <div class="progress-fill" style="width: {transcodingProgress}%"></div>
        </div>
      </div>
    {/if}
    
    {#if transcodedFiles.length > 0}
      <div class="transcoded-files">
        <h3>Transcoded Files</h3>
        <div class="transcoded-list">
          {#each transcodedFiles as file}
            <div class="transcoded-item" class:failed={!file.success}>
              <div class="file-path">{file?.input_path ? file.input_path.split('/').pop() : 'Unknown file'}</div>
              {#if file.success}
                <div class="success-label">✓ Transcoded</div>
              {:else}
                <div class="error-label">✗ Failed: {file.error || 'Unknown error'}</div>
              {/if}
            </div>
          {/each}
        </div>
        
        {#if !isEditingMetadata && extractedMetadata.length > 0}
          <div class="edit-metadata-actions">
            <button 
              class="edit-metadata-button" 
              on:click={editMetadata} 
              disabled={isLoading || isTranscoding || isUploading}
            >
              Edit Metadata Before Upload
            </button>
          </div>
        {/if}
      </div>
    {/if}
    
    {#if isEditingMetadata && extractedMetadata.length > 0}
      <div class="metadata-editor">
        <div class="editor-header">
          <h3>Edit Metadata</h3>
          <div class="editor-actions">
            {#if selectedTrackIndices.length > 0}
              <button class="bulk-edit-button" on:click={startBulkEdit}>
                Bulk Edit ({selectedTrackIndices.length} selected)
              </button>
            {/if}
            <button class="save-button" on:click={finalizeMetadata}>Finalize Metadata</button>
            <button class="cancel-button" on:click={cancelEditing}>Cancel</button>
          </div>
        </div>
        
        {#if bulkEditMode}
          <div class="bulk-edit-panel">
            <h4>Bulk Edit Selected Tracks ({selectedTrackIndices.length} selected)</h4>
            <div class="bulk-edit-form">
              <div class="form-row">
                <div class="form-group">
                  <label for="bulk-album">Album</label>
                  <input id="bulk-album" type="text" bind:value={bulkEditFields.album} placeholder="Album Name" />
                </div>
                
                <div class="form-group">
                  <label for="bulk-artist">Artist</label>
                  <input id="bulk-artist" type="text" bind:value={bulkEditFields.artist} placeholder="Artist Name" />
                </div>
              </div>
              
              <div class="form-row">
                <div class="form-group">
                  <label for="bulk-genre">Genre (comma separated)</label>
                  <input id="bulk-genre" type="text" bind:value={bulkEditFields.genre} placeholder="Rock, Pop, Jazz..." />
                </div>
              </div>
              
              <div class="form-section">
                <h5>Writers</h5>
                <div class="tags-input">
                  <input 
                    type="text" 
                    placeholder="Add writers (comma separated)" 
                    bind:value={bulkEditFields.writers}
                  />
                  <div class="bulk-hint">Writers will be split by commas and percentages will be distributed equally</div>
                </div>
              </div>
              
              <div class="form-section">
                <h5>Publishers</h5>
                <div class="tags-input">
                  <input 
                    type="text" 
                    placeholder="Add publishers (comma separated)" 
                    bind:value={bulkEditFields.publishers}
                  />
                  <div class="bulk-hint">Publishers will be split by commas and percentages will be distributed equally</div>
                </div>
              </div>
              
              <div class="form-section">
                <div class="form-row">
                  <div class="form-group">
                    <label for="bulk-instruments">Instruments</label>
                    <TagSelector 
                      tagOptions={$instrumentTags} 
                      selectedTagsString={bulkEditFields.instruments}
                      placeholder="Add instrument (press Enter)"
                      on:tagsChanged={handleBulkInstrumentTagsChanged}
                    />
                  </div>
                  
                  <div class="form-group">
                    <label for="bulk-mood">Mood</label>
                    <TagSelector 
                      tagOptions={$moodTags} 
                      selectedTagsString={bulkEditFields.mood}
                      placeholder="Add mood (press Enter)"
                      on:tagsChanged={handleBulkMoodTagsChanged}
                    />
                  </div>
                </div>
              </div>
              
              <div class="form-actions">
                <button class="apply-button" on:click={applyBulkEdits}>Apply to Selected</button>
                <button class="cancel-button" on:click={() => bulkEditMode = false}>Cancel</button>
              </div>
            </div>
          </div>
        {:else if selectedTrackIndex >= 0}
          <div class="individual-edit-panel">
            <h4>Edit Track: {extractedMetadata[selectedTrackIndex].track.title}</h4>
            <div class="individual-edit-form">
              <div class="form-row">
                <div class="form-group">
                  <label for="track-title">Title</label>
                  <input 
                    id="track-title" 
                    type="text" 
                    bind:value={extractedMetadata[selectedTrackIndex].track.title} 
                  />
                </div>
                
                <div class="form-group">
                  <label for="track-album">Album</label>
                  <input 
                    id="track-album" 
                    type="text" 
                    bind:value={extractedMetadata[selectedTrackIndex].album.name} 
                  />
                </div>
              </div>
              
              <div class="form-row">
                <div class="form-group">
                  <label for="track-artist">Artist</label>
                  <input 
                    id="track-artist" 
                    type="text" 
                    bind:value={extractedMetadata[selectedTrackIndex].album.artist} 
                  />
                </div>
                
                <div class="form-group">
                  <label for="track-genre">Genre (comma separated)</label>
                  <input 
                    id="track-genre" 
                    type="text" 
                    value={extractedMetadata[selectedTrackIndex].track.genre.join(', ')} 
                    on:input={(e) => {
                      const target = e.target as HTMLInputElement;
                      extractedMetadata[selectedTrackIndex].track.genre = target.value.split(',').map((g: string) => g.trim());
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
                      if (e.key === 'Enter' && target.value) {
                        extractedMetadata[selectedTrackIndex].track.writers = [
                          ...extractedMetadata[selectedTrackIndex].track.writers || [], 
                          target.value
                        ];
                        extractedMetadata[selectedTrackIndex].track.writer_percentages = 
                          extractedMetadata[selectedTrackIndex].track.writers.map(() => 
                            Math.floor(100 / extractedMetadata[selectedTrackIndex].track.writers.length));
                        target.value = '';
                        validatePercentages();
                      }
                    }}
                  />
                </div>
                
                <div class="tags-list">
                  {#if extractedMetadata[selectedTrackIndex].track.writers}
                    {#each extractedMetadata[selectedTrackIndex].track.writers as writer, i}
                      <div class="tag-item">
                        <span>{writer}</span>
                        <div class="percentage-input">
                          <input 
                            type="number" 
                            min="0" 
                            max="100" 
                            bind:value={extractedMetadata[selectedTrackIndex].track.writer_percentages[i]}
                            on:input={validatePercentages}
                          />
                          <span>%</span>
                        </div>
                        <button class="remove-tag" on:click={() => {
                          extractedMetadata[selectedTrackIndex].track.writers = 
                            extractedMetadata[selectedTrackIndex].track.writers.filter((_, idx) => idx !== i);
                          extractedMetadata[selectedTrackIndex].track.writer_percentages = 
                            extractedMetadata[selectedTrackIndex].track.writer_percentages.filter((_, idx) => idx !== i);
                          validatePercentages();
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
                <div class="tags-input">
                  <input 
                    type="text" 
                    placeholder="Add publisher (press Enter)" 
                    on:keydown={(e) => {
                      const target = e.target as HTMLInputElement;
                      if (e.key === 'Enter' && target.value) {
                        extractedMetadata[selectedTrackIndex].track.publishers = [
                          ...extractedMetadata[selectedTrackIndex].track.publishers || [], 
                          target.value
                        ];
                        extractedMetadata[selectedTrackIndex].track.publisher_percentages = 
                          extractedMetadata[selectedTrackIndex].track.publishers.map(() => 
                            Math.floor(100 / extractedMetadata[selectedTrackIndex].track.publishers.length));
                        target.value = '';
                        validatePercentages();
                      }
                    }}
                  />
                </div>
                
                <div class="tags-list">
                  {#if extractedMetadata[selectedTrackIndex].track.publishers}
                    {#each extractedMetadata[selectedTrackIndex].track.publishers as publisher, i}
                      <div class="tag-item">
                        <span>{publisher}</span>
                        <div class="percentage-input">
                          <input 
                            type="number" 
                            min="0" 
                            max="100" 
                            bind:value={extractedMetadata[selectedTrackIndex].track.publisher_percentages[i]}
                            on:input={validatePercentages}
                          />
                          <span>%</span>
                        </div>
                        <button class="remove-tag" on:click={() => {
                          extractedMetadata[selectedTrackIndex].track.publishers = 
                            extractedMetadata[selectedTrackIndex].track.publishers.filter((_, idx) => idx !== i);
                          extractedMetadata[selectedTrackIndex].track.publisher_percentages = 
                            extractedMetadata[selectedTrackIndex].track.publisher_percentages.filter((_, idx) => idx !== i);
                          validatePercentages();
                        }}>×</button>
                      </div>
                    {/each}
                    
                    {#if !publisherPercentagesValid}
                      <div class="validation-error">
                        Publisher percentages must sum to 100%
                      </div>
                    {/if}
                  {/if}
                </div>
              </div>
              
              <div class="form-section">
                <div class="form-row">
                  <div class="form-group">
                    <label for="track-instruments">Instruments</label>
                    <TagSelector 
                      tagOptions={$instrumentTags} 
                      selectedTagsString={extractedMetadata[selectedTrackIndex].track.instruments?.join(', ') || ''}
                      placeholder="Add instrument (press Enter)"
                      on:tagsChanged={handleInstrumentTagsChanged}
                    />
                  </div>
                  
                  <div class="form-group">
                    <label for="track-mood">Mood</label>
                    <TagSelector 
                      tagOptions={$moodTags} 
                      selectedTagsString={extractedMetadata[selectedTrackIndex].track.mood?.join(', ') || ''}
                      placeholder="Add mood (press Enter)"
                      on:tagsChanged={handleMoodTagsChanged}
                    />
                  </div>
                </div>
              </div>
              
              <div class="form-section">
                <div class="form-group">
                  <label for="track-comments">Comments</label>
                  <textarea 
                    id="track-comments" 
                    value={extractedMetadata[selectedTrackIndex].track.comments || ''} 
                    on:input={(e) => {
                      const target = e.target as HTMLTextAreaElement;
                      extractedMetadata[selectedTrackIndex].track.comments = target.value;
                    }}
                    rows="3"
                  ></textarea>
                </div>
              </div>
              
              <div class="form-actions">
                <button class="save-button" on:click={saveTrackEdits}>Save</button>
                <button class="cancel-button" on:click={() => selectedTrackIndex = -1}>Cancel</button>
              </div>
            </div>
          </div>
        {:else}
          <div class="tracks-list">
            <div class="tracks-header">
              <div class="select-all-container">
                <input 
                  type="checkbox" 
                  id="select-all-tracks"
                  checked={selectedTrackIndices.length === extractedMetadata.length && extractedMetadata.length > 0}
                  on:change={(e) => {
                    const target = e.target as HTMLInputElement;
                    selectAllTracks(target.checked);
                  }} 
                />
                <label for="select-all-tracks">Select All</label>
              </div>
              {#if selectedTrackIndices.length > 0}
                <button class="bulk-edit-button-small" on:click={startBulkEdit}>
                  Bulk Edit ({selectedTrackIndices.length})
                </button>
              {/if}
            </div>
            
            {#each extractedMetadata as metadata, i}
              <div class="track-item" class:selected={selectedTrackIndices.includes(i)}>
                <div class="track-select">
                  <input 
                    type="checkbox" 
                    checked={selectedTrackIndices.includes(i)}
                    on:change={() => toggleTrackSelection(i)} 
                  />
                </div>
                
                <div class="track-info" on:click={() => editTrack(i)}>
                  <div class="track-title">{metadata.track.title}</div>
                  <div class="track-details">
                    <span>{metadata.album.artist}</span>
                    <span class="separator">•</span>
                    <span>{metadata.album.name}</span>
                    {#if metadata.track.genre && metadata.track.genre.length > 0}
                      <span class="separator">•</span>
                      <span>{metadata.track.genre.join(', ')}</span>
                    {/if}
                  </div>
                  <div class="track-path">{metadata.track.original_path?.split('/').pop() || metadata.fileInfo.path.split('/').pop()}</div>
                </div>
                
                <div class="track-actions">
                  <button class="edit-button" on:click={() => editTrack(i)}>Edit</button>
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    {/if}
    
    {#if isUploading}
      <div class="upload-progress">
        <p>Uploading files to R2 and storing metadata in MongoDB... Please wait.</p>
        <div class="progress-bar">
          <div class="progress-fill" style="width: {uploadProgress}%"></div>
        </div>
      </div>
    {/if}
    
    {#if uploadedFiles.length > 0 || failedUploads.length > 0}
      <div class="uploaded-files">
        <h3>Upload Results</h3>
        
        {#if uploadedFiles.length > 0}
          <h4>Successfully Uploaded ({uploadedFiles.length})</h4>
          <div class="uploaded-list">
            {#each uploadedFiles as file}
              <div class="uploaded-item">
                <div class="file-info">
                  <div class="file-title">{file.title}</div>
                  <div class="file-album">Album: {file.album_name}</div>
                </div>
                <div class="success-label">✓ Uploaded</div>
              </div>
            {/each}
          </div>
        {/if}
        
        {#if failedUploads.length > 0}
          <h4>Failed Uploads ({failedUploads.length})</h4>
          <div class="failed-list">
            {#each failedUploads as file}
              <div class="failed-item">
                <div class="file-path">{file.original_path.split('/').pop()}</div>
                <div class="error-label">✗ Failed: {file.error}</div>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    {/if}
    
    {#if selectedFilePaths.length > 0}
      <div class="upload-actions">
        <button 
          class="transcode-button" 
          on:click={transcodeFiles} 
          disabled={isLoading || isTranscoding || isUploading}
        >
          Transcode {selectedFilePaths.length} {selectedFilePaths.length === 1 ? 'File' : 'Files'}
        </button>
        
        <button 
          class="upload-button" 
          on:click={uploadFiles} 
          disabled={isLoading || isTranscoding || isUploading || transcodedFiles.length === 0 || isEditingMetadata}
        >
          Upload to Library
        </button>
      </div>
      
      {#if import.meta.env.DEV}
        <div class="debug-section">
          <div class="debug-info">
            <p><strong>Button Control Variables:</strong></p>
            <ul>
              <li>isLoading: {isLoading}</li>
              <li>isTranscoding: {isTranscoding}</li>
              <li>isUploading: {isUploading}</li>
              <li>transcodedFiles.length: {transcodedFiles.length}</li>
              <li>Button should be {isLoading || isTranscoding || isUploading || transcodedFiles.length === 0 ? 'DISABLED' : 'ENABLED'}</li>
            </ul>
            <button 
              class="debug-button" 
              on:click={() => {
                isLoading = false;
                isTranscoding = false;
                isUploading = false;
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
  
  /* Metadata Editing Styles */
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
    max-height: 500px;
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