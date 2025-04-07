// Declare submodules for the 'upload' feature
pub mod audio;

// Final Corrected Imports (Attempt 3)
use crate::features::upload::audio::transcode::transcode_to_aac; // Updated path
use crate::features::upload::audio::error::TranscodingError; // Updated path
// Credentials are not directly used here; bucket name comes from R2State
// Removed unused DbTrack import
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client as S3Client;
// Removed potentially duplicate StreamExt import
// Removed prelude wildcard import to avoid type conflicts
// Reverting to prelude import to resolve trait scope issues
// Removed prelude import again to resolve type conflict
// Explicit imports to avoid prelude conflicts
// Lofty imports removed.
// StdDuration import removed as it was likely only needed for Lofty.
use log::{error, info, warn}; // Removed unused debug import
use mongodb::bson::{self, doc, oid::ObjectId, Document}; // Removed unused BsonDateTime import
use mongodb::Client as MongoDbClient;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
// Removed unused SystemTime import
use tauri::{command, AppHandle, Emitter, Manager, State, Wry}; // Ensure Manager and Emitter traits are imported
use tempfile::Builder as TempFileBuilder; // Removed unused NamedTempFile import
use thiserror::Error;
use tokio::sync::{mpsc, Mutex};
use uuid::Uuid;

// --- Error Enum (Consider moving to a shared error module if applicable) ---

#[derive(Error, Debug, Serialize)]
pub enum UploadError {
    #[error("R2 client not initialized. Configure credentials.")]
    R2ClientNotInitialized,
    #[error("MongoDB client not initialized. Configure credentials.")]
    MongoDbClientNotInitialized,
    #[error("Failed to get R2 credentials: {0}")]
    CredentialsError(String),
    #[error("Filesystem error: {0}")]
    IoError(String),
    #[error("Transcoding failed: {0}")]
    TranscodingError(#[from] crate::features::upload::audio::error::TranscodingError), // Ensure full path if needed
    #[error("R2 upload failed: {0}")]
    R2UploadError(String),
    #[error("MongoDB operation failed: {0}")]
    MongoDbError(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Operation cancelled")]
    Cancelled,
    #[error("Internal error: {0}")]
    InternalError(String),
}

// --- Data Structures ---

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UploadItemMetadata {
    // Core editable fields
    pub title: Option<String>, // Made public
    pub artist: Option<String>, // Made public
    pub album: Option<String>, // Made public
    pub track_number: Option<u32>, // Made public

    // Additional fields expected to be finalized by frontend
    pub duration_sec: Option<f64>,
    pub genre: Option<String>,
    pub composer: Option<String>,
    // Add other relevant fields here if needed (e.g., year, comments)
    pub year: Option<i32>,
    pub comments: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UploadItemInput {
    pub id: String,
    pub path: String,
    pub metadata: UploadItemMetadata,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum UploadStatus {
    Pending,
    Transcoding,
    UploadingOriginal,
    UploadingAAC,
    StoringMetadata,
    Complete,
    Cancelled,
    Error(String),
}

#[derive(Debug, Clone, Serialize)]
pub struct UploadProgress {
    pub item_id: Uuid,
    pub original_path: String,
    pub status: UploadStatus,
    pub error_message: Option<String>,
    pub title: Option<String>,
    pub album: Option<String>,
}

#[derive(Debug)]
pub struct UploadQueueItem { // Make struct public
    id: Uuid,
    input_path: PathBuf,
    metadata: UploadItemMetadata,
    temp_aac_path: Option<PathBuf>,
    r2_original_key: Option<String>,
    r2_aac_key: Option<String>,
    db_track_id: Option<String>,
}

// --- Shared State ---

#[derive(Debug)]
pub struct UploadState {
    pub queue_tx: mpsc::Sender<UploadQueueItem>,
    // Store receiver in Mutex<Option<...>> to allow taking it once
    pub queue_rx: Arc<Mutex<Option<mpsc::Receiver<UploadQueueItem>>>>,
    pub is_processing: Arc<AtomicBool>,
    pub cancel_flag: Arc<AtomicBool>,
    pub progress_map: Arc<Mutex<HashMap<Uuid, UploadProgress>>>,
}

impl UploadState {
    // Modify constructor to accept receiver
    pub fn new(tx: mpsc::Sender<UploadQueueItem>, rx: mpsc::Receiver<UploadQueueItem>) -> Self {
        Self {
            queue_tx: tx,
            queue_rx: Arc::new(Mutex::new(Some(rx))), // Store receiver in Mutex<Option<...>>
            is_processing: Arc::new(AtomicBool::new(false)),
            cancel_flag: Arc::new(AtomicBool::new(false)),
            progress_map: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

// --- Tauri Commands ---

#[command]
pub async fn start_upload_queue(
    items: Vec<UploadItemInput>,
    app_handle: AppHandle<Wry>,
    upload_state: State<'_, Arc<UploadState>>,
    r2_state: State<'_, crate::R2State>,
    mongo_state: State<'_, crate::MongoState>,
) -> Result<(), String> {
    info!("Received request to upload {} items.", items.len());

    if r2_state.client.lock().await.is_none() { return Err(UploadError::R2ClientNotInitialized.to_string()); }
    if mongo_state.client.lock().await.is_none() { return Err(UploadError::MongoDbClientNotInitialized.to_string()); }
    if items.is_empty() { return Err(UploadError::InvalidInput("No items provided for upload.".to_string()).to_string()); }

    upload_state.cancel_flag.store(false, Ordering::SeqCst);
    let mut progress_map = upload_state.progress_map.lock().await;

    for item_input in items {
        let item_id = Uuid::new_v4();
        let input_path = PathBuf::from(&item_input.path);

        if !input_path.exists() {
            warn!("Input file does not exist, skipping: {}", item_input.path);
            let progress = UploadProgress {
                item_id, original_path: item_input.path.clone(),
                status: UploadStatus::Error("File not found".to_string()),
                error_message: Some("Input file does not exist.".to_string()),
                title: item_input.metadata.title.clone(), album: item_input.metadata.album.clone(),
            };
            if let Some(window) = app_handle.get_webview_window("main") {
                 // Clone progress before emitting
                 window.emit("upload://status-update", progress.clone()).map_err(|e| e.to_string())?;
            } else { error!("Could not find main window to emit status update."); }
            progress_map.insert(item_id, progress);
            continue;
        }

        let queue_item = UploadQueueItem {
            id: item_id, input_path: input_path.clone(), metadata: item_input.metadata.clone(),
            temp_aac_path: None, r2_original_key: None, r2_aac_key: None, db_track_id: None,
        };

        if let Err(e) = upload_state.queue_tx.send(queue_item).await {
            error!("Failed to add item {} to upload queue: {}", item_input.path, e);
             let progress = UploadProgress {
                item_id, original_path: item_input.path.clone(),
                status: UploadStatus::Error("Failed to queue".to_string()),
                error_message: Some(format!("Failed to add item to queue: {}", e)),
                title: item_input.metadata.title.clone(), album: item_input.metadata.album.clone(),
            };
            if let Some(window) = app_handle.get_webview_window("main") {
                 // Clone progress before emitting
                 window.emit("upload://status-update", progress.clone()).map_err(|e| e.to_string())?;
            } else { error!("Could not find main window to emit status update."); }
            progress_map.insert(item_id, progress);
        } else {
            let progress = UploadProgress {
                item_id, original_path: item_input.path, status: UploadStatus::Pending,
                error_message: None, title: item_input.metadata.title, album: item_input.metadata.album,
            };
             if let Some(window) = app_handle.get_webview_window("main") {
                  // Clone progress before emitting
                  window.emit("upload://status-update", progress.clone()).map_err(|e| e.to_string())?;
             } else { error!("Could not find main window to emit status update."); }
            progress_map.insert(item_id, progress);
        }
    }
    drop(progress_map);

    if !upload_state.is_processing.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
        info!("Spawning upload processing task.");
        let state_clone = Arc::clone(&upload_state);
        let app_handle_clone = app_handle.clone();

        tauri::async_runtime::spawn(async move {
            let rx_option = state_clone.queue_rx.lock().await.take();

            if let Some(rx) = rx_option {
                info!("Passing receiver to process_upload_queue task.");
                process_upload_queue(app_handle_clone.clone(), state_clone.clone(), rx).await;
            } else {
                error!("Upload queue receiver has already been taken!");
                state_clone.is_processing.store(false, Ordering::SeqCst);
            }
            state_clone.is_processing.store(false, Ordering::SeqCst);
            info!("Upload processing task finished.");
            if let Some(window) = app_handle_clone.get_webview_window("main") {
                 window.emit("upload://queue-finished", ()).unwrap_or_else(|e| {
                     error!("Failed to emit queue-finished event: {}", e);
                 });
            } else { error!("Could not find main window to emit queue-finished event."); }
        });
    } else {
        info!("Upload processing task already running.");
    }
    Ok(())
}

#[command]
pub async fn cancel_upload_queue(upload_state: State<'_, Arc<UploadState>>) -> Result<(), String> {
    info!("Received request to cancel upload queue.");
    upload_state.cancel_flag.store(true, Ordering::SeqCst);
    Ok(())
}

// --- Core Processing Logic ---

async fn process_upload_queue(
    app_handle: AppHandle<Wry>,
    state: Arc<UploadState>,
    mut rx: mpsc::Receiver<UploadQueueItem>,
) {
    let progress_map = Arc::clone(&state.progress_map);
    let cancel_flag = Arc::clone(&state.cancel_flag);

    // --- Get Clients from App State ---
    let r2_state = match app_handle.try_state::<crate::R2State>() {
         Some(state) => state, None => { error!("R2State not found."); return; }
    };
    let mongo_state = match app_handle.try_state::<crate::MongoState>() {
         Some(state) => state, None => { error!("MongoState not found."); return; }
    };
    let r2_client_opt = r2_state.client.lock().await;
    let mongo_client_opt = mongo_state.client.lock().await;
    let r2_client = match r2_client_opt.as_ref() {
        Some(client) => client, None => { error!("R2 client not initialized."); return; }
    };
    let mongo_client = match mongo_client_opt.as_ref() {
        Some(client) => client, None => { error!("MongoDB client not initialized."); return; }
    };
    let bucket_name_opt = r2_state.bucket_name.lock().await;
    let bucket_name = match bucket_name_opt.as_deref() {
        Some(name) => name.to_string(), None => { error!("R2 bucket name not found in state."); return; }
    };
    drop(bucket_name_opt); // Drop lock

    // --- Processing Loop ---
    while let Some(mut item) = rx.recv().await {
        let item_id = item.id;
        let original_path_str = item.input_path.to_string_lossy().to_string();
        info!("Processing item: {} ({})", original_path_str, item_id);
        let mut current_status = UploadStatus::Pending;

        // Check for cancellation before starting work
        if cancel_flag.load(Ordering::SeqCst) {
            info!("Cancellation detected before processing item {}", item_id);
            current_status = UploadStatus::Cancelled;
            update_progress(&app_handle, &progress_map, item_id, current_status.clone(), None, &item.metadata, &original_path_str).await;
            continue; // Skip to next item
        }

        // --- Transcoding ---
        current_status = UploadStatus::Transcoding;
        update_progress(&app_handle, &progress_map, item_id, current_status.clone(), None, &item.metadata, &original_path_str).await;

        let transcoding_result = run_transcoding(&item.input_path).await;

        if cancel_flag.load(Ordering::SeqCst) {
            info!("Cancellation detected after transcoding attempt for item {}", item_id);
            current_status = UploadStatus::Cancelled;
            update_progress(&app_handle, &progress_map, item_id, current_status.clone(), None, &item.metadata, &original_path_str).await;
            if let Ok(ref temp_path) = transcoding_result { cleanup_temp_file(temp_path); }
            break; // Stop queue processing on cancel
        }

        match transcoding_result {
            Ok(temp_aac_path) => {
                item.temp_aac_path = Some(temp_aac_path);
            }
            Err(e) => {
                error!("Transcoding failed for {}: {}", original_path_str, e);
                current_status = UploadStatus::Error(format!("Transcoding failed: {}", e));
                update_progress(&app_handle, &progress_map, item_id, current_status.clone(), Some(e.to_string()), &item.metadata, &original_path_str).await;
                continue; // Skip to next item
            }
        };
        let aac_path_ref = item.temp_aac_path.as_ref(); // Borrow for later use

        // --- Upload Original ---
        current_status = UploadStatus::UploadingOriginal;
        update_progress(&app_handle, &progress_map, item_id, current_status.clone(), None, &item.metadata, &original_path_str).await;
        let original_mime = mime_guess::from_path(&item.input_path).first_or_octet_stream();
        let original_key = format!("tracks/original/{}", item.input_path.file_name().unwrap_or_default().to_string_lossy());
        let upload_orig_res = upload_file_to_r2(r2_client, &item.input_path, &bucket_name, &original_key, original_mime.as_ref(), true).await;
        item.r2_original_key = Some(original_key.clone()); // Store key

        if cancel_flag.load(Ordering::SeqCst) {
            info!("Cancellation detected after original upload for item {}", item_id);
            current_status = UploadStatus::Cancelled;
            update_progress(&app_handle, &progress_map, item_id, current_status.clone(), None, &item.metadata, &original_path_str).await;
            perform_cleanup(r2_client, &bucket_name, mongo_client, &item).await;
            break;
        }

        if let Err(e) = upload_orig_res {
             error!("Original upload failed for {}: {}", original_path_str, e);
             current_status = UploadStatus::Error(format!("Original upload failed: {}", e));
             update_progress(&app_handle, &progress_map, item_id, current_status.clone(), Some(e.to_string()), &item.metadata, &original_path_str).await;
             perform_cleanup(r2_client, &bucket_name, mongo_client, &item).await; // Cleanup original R2 + temp AAC
             continue;
        }
        info!("Original upload successful for {}: {}", original_path_str, original_key);

        // --- Upload AAC ---
        if let Some(aac_path) = aac_path_ref {
            current_status = UploadStatus::UploadingAAC;
            update_progress(&app_handle, &progress_map, item_id, current_status.clone(), None, &item.metadata, &original_path_str).await;
            let aac_mime = mime_guess::from_path::<&Path>(aac_path).first_or_octet_stream();
            let aac_key = format!("tracks/aac/{}", aac_path.file_name().unwrap_or_default().to_string_lossy());
            let upload_aac_res = upload_file_to_r2(r2_client, aac_path, &bucket_name, &aac_key, aac_mime.as_ref(), true).await;
            item.r2_aac_key = Some(aac_key.clone()); // Store key

            if cancel_flag.load(Ordering::SeqCst) {
                info!("Cancellation detected after AAC upload for item {}", item_id);
                current_status = UploadStatus::Cancelled;
                update_progress(&app_handle, &progress_map, item_id, current_status.clone(), None, &item.metadata, &original_path_str).await;
                perform_cleanup(r2_client, &bucket_name, mongo_client, &item).await;
                break;
            }

            if let Err(e) = upload_aac_res {
                error!("AAC upload failed for {}: {}", original_path_str, e);
                current_status = UploadStatus::Error(format!("AAC upload failed: {}", e));
                update_progress(&app_handle, &progress_map, item_id, current_status.clone(), Some(e.to_string()), &item.metadata, &original_path_str).await;
                perform_cleanup(r2_client, &bucket_name, mongo_client, &item).await; // Cleanup R2 + temp AAC
                continue;
            }
            info!("AAC upload successful for {}: {}", original_path_str, aac_key);
        } else {
            info!("No AAC file to upload for {}", original_path_str);
            item.r2_aac_key = None;
        }

        // --- Store Metadata ---
        current_status = UploadStatus::StoringMetadata;
        update_progress(&app_handle, &progress_map, item_id, current_status.clone(), None, &item.metadata, &original_path_str).await;
        let db_result = store_track_metadata(mongo_client, &item, item.r2_original_key.as_deref(), item.r2_aac_key.as_deref()).await;

        if cancel_flag.load(Ordering::SeqCst) {
            info!("Cancellation detected after DB write attempt for item {}", item_id);
            current_status = UploadStatus::Cancelled;
            update_progress(&app_handle, &progress_map, item_id, current_status.clone(), None, &item.metadata, &original_path_str).await;
            if let Ok(ref track_id) = db_result { item.db_track_id = Some(track_id.clone()); } // Store ID if write succeeded
            perform_cleanup(r2_client, &bucket_name, mongo_client, &item).await;
            break;
        }

        match db_result {
            Ok(track_id) => {
                item.db_track_id = Some(track_id.clone()); // Store track ID
                info!("Metadata stored successfully for {}: Track ID {}", original_path_str, track_id);
                current_status = UploadStatus::Complete;
                update_progress(&app_handle, &progress_map, item_id, current_status.clone(), None, &item.metadata, &original_path_str).await;
            }
            Err(e) => {
                 error!("Metadata storage failed for {}: {}", original_path_str, e);
                 current_status = UploadStatus::Error(format!("Metadata storage failed: {}", e));
                 update_progress(&app_handle, &progress_map, item_id, current_status.clone(), Some(e.to_string()), &item.metadata, &original_path_str).await;
                 perform_cleanup(r2_client, &bucket_name, mongo_client, &item).await; // Cleanup R2 + temp AAC
                 continue;
            }
        }

        // --- Cleanup Temp AAC ---
        if current_status == UploadStatus::Complete {
            if let Some(path) = item.temp_aac_path.take() { cleanup_temp_file(&path); }
        }
    } // End while
} // End process_upload_queue

// --- Helper Functions ---

async fn run_transcoding(input_path: &Path) -> Result<PathBuf, TranscodingError> {
    let temp_aac_file = TempFileBuilder::new().prefix("transcoded_").suffix(".m4a").tempfile().map_err(|e| TranscodingError::IoError { source_message: e.to_string() })?;
    let output_path = temp_aac_file.path().to_path_buf();
    info!("Transcoding {:?} to temporary file {:?}", input_path, output_path);
    
    // Using spawn_blocking to run the CPU-intensive transcoding in a separate thread pool
    let input_path_clone = input_path.to_path_buf();
    let output_path_clone = output_path.clone();
    tokio::task::spawn_blocking(move || {
        transcode_to_aac(&input_path_clone, &output_path_clone)
    }).await.map_err(|e| TranscodingError::IoError { 
        source_message: format!("Task join error: {}", e) 
    })??;

    match temp_aac_file.keep() {
        Ok((_file, path)) => { info!("Persisted temporary transcoded file: {:?}", path); Ok(path) }
        // Corrected IoError construction
        Err(e) => { error!("Failed to persist temporary file {:?}: {}", output_path, e.error); let _ = std::fs::remove_file(&output_path); Err(TranscodingError::IoError { source_message: e.error.to_string() }) }
    }
}

async fn upload_file_to_r2(r2_client: &S3Client, file_path: &Path, bucket_name: &str, r2_key: &str, mime_type: &str, _make_public: bool) -> Result<(), UploadError> {
    info!("Uploading file {:?} to R2 bucket '{}' key '{}'", file_path, bucket_name, r2_key);
    let body = ByteStream::from_path(file_path).await.map_err(|e| UploadError::IoError(format!("Failed to read file {:?}: {}", file_path, e)))?;
    r2_client.put_object().bucket(bucket_name).key(r2_key).content_type(mime_type).body(body).send().await.map_err(|e| UploadError::R2UploadError(format!("S3 PutObject failed: {}", e)))?;
    Ok(())
}

async fn store_track_metadata(
    mongo_client: &MongoDbClient,
    item: &UploadQueueItem,
    original_r2_key: Option<&str>,
    aac_r2_key: Option<&str>,
) -> Result<String, UploadError> {
    let db = mongo_client.database("music_library");
    let tracks_collection = db.collection::<Document>("tracks");
    let albums_collection = db.collection::<Document>("albums");

    info!("Storing metadata for: {}", item.input_path.display());

    // --- Use Finalized Metadata from item.metadata ---
    // These fields are now expected to be provided and finalized by the frontend
    let title = item.metadata.title.clone().unwrap_or_else(|| {
        item.input_path.file_stem().unwrap_or_default().to_string_lossy().into_owned() // Fallback to filename stem
    });
    let artist = item.metadata.artist.clone().unwrap_or_else(|| "Unknown Artist".to_string());
    let album_title = item.metadata.album.clone().unwrap_or_else(|| "Unknown Album".to_string());
    let track_number = item.metadata.track_number;
    let duration_sec = item.metadata.duration_sec; // Use directly from finalized metadata
    let genre = item.metadata.genre.clone(); // Use directly from finalized metadata
    let composer = item.metadata.composer.clone(); // Use directly from finalized metadata
    let year = item.metadata.year; // Use directly from finalized metadata
    let comments = item.metadata.comments.clone(); // Use directly from finalized metadata

    // --- Get Basic File Info ---
    let file_size = match std::fs::metadata(&item.input_path) {
         Ok(m) => m.len(),
         Err(e) => {
             warn!("Failed to get file size for {}: {}. Using 0.", item.input_path.display(), e);
             0 // Default to 0 if metadata fails
         }
    };
    let mime_type = mime_guess::from_path(&item.input_path)
        .first_or_octet_stream()
        .to_string();
    let file_extension = item.input_path.extension().unwrap_or_default().to_string_lossy().to_string();

    // --- Find or Create Album ---
    // Use finalized metadata for album lookup/creation
    let album_doc = albums_collection
        .find_one(doc! { "name": &album_title, "artist": &artist }, None)
        .await
        .map_err(|e| UploadError::MongoDbError(format!("Album lookup failed: {}", e)))?;

    let album_id = match album_doc {
        Some(doc) => doc.get_object_id("_id").map_err(|_| UploadError::MongoDbError("Invalid album ID format".to_string()))?,
        None => {
            // Create new album using finalized metadata
            let new_album_id = ObjectId::new();
            let new_album_doc = doc! {
                "_id": new_album_id,
                "name": &album_title,
                "artist": &artist,
                "year": year, // Use finalized year
                "genres": if let Some(g) = &genre { vec![g.clone()] } else { Vec::<String>::new() }, // Use finalized genre
                "art_path": null, // Placeholder for album art
                "date_added": bson::DateTime::now(),
            };
            albums_collection.insert_one(new_album_doc, None).await.map_err(|e| UploadError::MongoDbError(format!("Album insert failed: {}", e)))?;
            info!("Created new album '{}' with ID: {}", album_title, new_album_id);
            new_album_id
        }
    };

    // --- Create Track Document ---
    let track_id = ObjectId::new();
    let track_doc = doc! {
        "_id": track_id,
        "title": title,
        "filename": item.input_path.file_name().unwrap_or_default().to_string_lossy().to_string(),
        "duration": duration_sec, // Use finalized duration
        "track_number": track_number, // Use finalized track number
        "album_id": album_id,
        "artists": vec![artist.clone()], // Assuming single artist for now from finalized metadata
        "original_path": item.input_path.to_string_lossy().to_string(),
        "mime_type": mime_type,
        "file_size": file_size as i64, // Store as i64 for BSON compatibility
        "writers": bson::Document::new(), // Placeholder - Should this be part of finalized metadata?
        "publishers": bson::Document::new(), // Placeholder - Should this be part of finalized metadata?
        "genre": if let Some(g) = genre { vec![g] } else { Vec::<String>::new() }, // Use finalized genre
        "composer": composer, // Use finalized composer
        "instruments": Vec::<String>::new(), // Placeholder - Should this be part of finalized metadata?
        "mood": Vec::<String>::new(), // Placeholder - Should this be part of finalized metadata?
        "comments": comments, // Use finalized comments
        "date_added": bson::DateTime::now(),
        "extension": file_extension,
        "r2_original_key": original_r2_key,
        "r2_aac_key": aac_r2_key,
        // Add other fields as needed based on finalized metadata
    };

    // --- Insert Track ---
    tracks_collection.insert_one(track_doc, None).await.map_err(|e| UploadError::MongoDbError(format!("Track insert failed: {}", e)))?;
    info!("Stored track metadata for '{}' with ID: {}", item.input_path.display(), track_id);

    Ok(track_id.to_hex())
}


async fn update_progress(app_handle: &AppHandle<Wry>, progress_map: &Arc<Mutex<HashMap<Uuid, UploadProgress>>>, item_id: Uuid, status: UploadStatus, error_message: Option<String>, metadata: &UploadItemMetadata, original_path: &str) {
    let mut map = progress_map.lock().await;
    let progress = map.entry(item_id).or_insert_with(|| UploadProgress {
        item_id,
        original_path: original_path.to_string(),
        status: UploadStatus::Pending, // Default status
        error_message: None,
        title: metadata.title.clone(),
        album: metadata.album.clone(),
    });

    progress.status = status;
    progress.error_message = error_message;

    // Emit update event - Clone progress before emitting
    if let Some(window) = app_handle.get_webview_window("main") {
         // Clone the progress struct here
         window.emit("upload://status-update", progress.clone()).unwrap_or_else(|e| {
             error!("Failed to emit status update for {}: {}", item_id, e);
         });
    } else { error!("Could not find main window to emit status update for {}.", item_id); }
}

fn cleanup_temp_file(path: &Path) {
    if let Err(e) = std::fs::remove_file(path) {
        warn!("Failed to clean up temporary file {:?}: {}", path, e);
    } else {
        info!("Cleaned up temporary file: {:?}", path);
    }
}

// --- Cleanup Logic ---

async fn delete_r2_object(r2_client: &S3Client, bucket_name: &str, key: &str) {
    info!("Attempting to delete R2 object: {}/{}", bucket_name, key);
    if let Err(e) = r2_client.delete_object().bucket(bucket_name).key(key).send().await {
        error!("Failed to delete R2 object {}/{}: {}", bucket_name, key, e);
    } else {
        info!("Successfully deleted R2 object: {}/{}", bucket_name, key);
    }
}

async fn delete_mongodb_track(mongo_client: &MongoDbClient, track_id_hex: &str) {
    info!("Attempting to delete MongoDB track: {}", track_id_hex);
    match ObjectId::parse_str(track_id_hex) {
        Ok(oid) => {
            let db = mongo_client.database("music_library");
            let tracks_collection = db.collection::<Document>("tracks");
            if let Err(e) = tracks_collection.delete_one(doc! { "_id": oid }, None).await {
                error!("Failed to delete MongoDB track {}: {}", track_id_hex, e);
            } else {
                info!("Successfully deleted MongoDB track: {}", track_id_hex);
            }
        }
        Err(e) => {
            error!("Invalid ObjectId format for track deletion {}: {}", track_id_hex, e);
        }
    }
}

async fn perform_cleanup(r2_client: &S3Client, bucket_name: &str, mongo_client: &MongoDbClient, item: &UploadQueueItem) {
    warn!("Performing cleanup for failed/cancelled item: {}", item.id);
    if let Some(path) = &item.temp_aac_path { cleanup_temp_file(path); }
    if let Some(key) = &item.r2_original_key { delete_r2_object(r2_client, bucket_name, key).await; }
    if let Some(key) = &item.r2_aac_key { delete_r2_object(r2_client, bucket_name, key).await; }
    if let Some(id) = &item.db_track_id { delete_mongodb_track(mongo_client, id).await; }
}