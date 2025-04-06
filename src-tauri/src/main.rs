// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use log::{error, info, warn};
use tauri_plugin_dialog::{DialogExt};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
// keyring::Entry moved to credentials.rs
use tauri::{
    command, AppHandle, State, Manager, Emitter,
};
use tokio::sync::mpsc;

// Import modules
// mod audio; // Moved to features::upload
// mod storage; // Moved to features::catalog
// mod error; // Moved to lib.rs
// mod upload; // Moved to features::upload
// mod commands; // Moved to core::commands_old
// mod credentials; // Moved to features::credentials
mod features; // NEW: Declare features module
// mod core;     // Moved to lib.rs

// Add a simple test command for metadata extraction
#[tauri::command]
fn test_extract_metadata(filePath: String) -> Result<serde_json::Value, String> {
    info!("Test extract metadata for: {}", filePath);
    // Return a dummy metadata object
    Ok(serde_json::json!({
        "title": "Test Title",
        "artist": "Test Artist",
        "album": "Test Album",
        "duration_sec": 180.0
    }))
}

// Use statements
// Make re-exports explicit
pub use app_lib::error::CommandError;
pub use app_lib::core;
use app_lib::{MongoState, R2State}; // Use items from the library crate
use app_lib::features::upload::audio::transcode; // Import transcode module
use app_lib::features::upload::{ // Corrected path to use app_lib
    start_upload_queue, cancel_upload_queue, UploadState, UploadQueueItem,
};
use app_lib::features::credentials::{ // Corrected path to use app_lib
    store_r2_credentials,
    get_r2_credentials,
    store_mongo_credentials,
    get_mongo_credentials,
    has_credentials,
    delete_credentials,
    R2Credentials, // Re-export struct if needed by other modules called from main
};
// --- Credential constants, structs, and helpers moved to credentials.rs ---

// --- State Structs (MongoState, R2State) moved to lib.rs ---

// UploadState is defined in upload.rs.

// --- Credential Structs (MongoCredentials, R2Credentials) are now in credentials.rs ---
// --- DevCredentials struct and helpers are now in credentials.rs ---

/// Result type for audio transcoding operations
#[derive(Debug, Serialize, Deserialize)]
struct TranscodingResult {
    output_path: String,
}

// --- Client Initialization ---

/// Initializes the R2 client and stores it in state if successful.
#[command]
async fn init_r2_client(r2_state: State<'_, R2State>) -> Result<bool, CommandError> {
    {
        let lock = r2_state.client.lock().await;
        if lock.is_some() {
            info!("R2 client already initialized, reusing existing client");
            return Ok(true);
        }
    }

    let credentials = get_r2_credentials_proxy().await.map_err(|e| {
        if matches!(e, CommandError::Configuration(_)) {
            CommandError::Configuration("R2 credentials not set. Please configure credentials in Settings.".to_string())
        } else {
            e
        }
    })?;

    info!("Creating new R2 client with account ID: {} and access key: {}",
        credentials.account_id, credentials.access_key_id);

    let endpoint = if !credentials.endpoint.is_empty() {
        credentials.endpoint.clone()
    } else {
        format!("https://{}.r2.cloudflarestorage.com", credentials.account_id)
    };

    let aws_creds = aws_sdk_s3::config::Credentials::new(
        &credentials.access_key_id, &credentials.secret_access_key, None, None, "r2-credentials"
    );

    let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .region(aws_sdk_s3::config::Region::new("auto"))
        .endpoint_url(&endpoint)
        .credentials_provider(aws_creds)
        .load().await;

     let s3_config = aws_sdk_s3::config::Builder::from(&config).force_path_style(true).build();
    let client = aws_sdk_s3::Client::from_conf(s3_config);

    info!("Testing R2 connection (list_buckets)");
    client.list_buckets().send().await.map_err(|e| {
        error!("R2 connection test failed (list_buckets): {}", e);
        CommandError::Storage(format!("R2 connection test failed: {}", e))
    })?;

    info!("Testing R2 bucket access: {}", credentials.bucket_name);
     client.list_objects_v2().bucket(&credentials.bucket_name).max_keys(1).send().await
         .map_err(|e| {
             error!("R2 bucket access test failed (list_objects_v2): {}", e);
             CommandError::Storage(format!(
                 "R2 credentials seem valid but couldn't access bucket '{}': {}",
                 credentials.bucket_name, e
             ))
         })?;

    info!("R2 connection and bucket access successful.");
    let mut client_lock = r2_state.client.lock().await;
    *client_lock = Some(client);
    let mut bucket_lock = r2_state.bucket_name.lock().await;
    *bucket_lock = Some(credentials.bucket_name);
    info!("Stored R2 client and bucket name in state.");
    Ok(true)
}

/// Initializes the MongoDB client and stores it in state if successful.
#[command]
async fn init_mongo_client(mongo_state: State<'_, MongoState>) -> Result<bool, CommandError> {
    {
        let lock = mongo_state.client.lock().await;
        if lock.is_some() {
            info!("MongoDB client already initialized, reusing existing client");
            return Ok(true);
        }
    }

    let connection_string = get_mongo_credentials_proxy().await.map_err(|e| {
        if matches!(e, CommandError::Configuration(_)) {
            CommandError::Configuration("MongoDB credentials not set. Please configure credentials in Settings.".to_string())
        } else {
            e
        }
    })?;

    let client_instance = create_mongodb_client(connection_string).await?;

    info!("MongoDB client created and connection tested successfully.");
    let mut lock = mongo_state.client.lock().await;
    *lock = Some(client_instance);
    info!("Stored MongoDB client in state.");
    Ok(true)
}

/// Helper to create and test MongoDB client
async fn create_mongodb_client(connection_string: String) -> Result<mongodb::Client, CommandError> {
    info!("Attempting to connect to MongoDB...");
    let client_options = mongodb::options::ClientOptions::parse(&connection_string)
        .await
        .map_err(|e| CommandError::Configuration(format!("Failed to parse MongoDB connection string: {}", e)))?;

    let client = mongodb::Client::with_options(client_options)
        .map_err(|e| CommandError::Configuration(format!("Failed to create MongoDB client: {}", e)))?;

    // Test connection by listing database names
    client.list_database_names(None, None).await
        .map_err(|e| CommandError::Database(format!("Failed to connect to MongoDB: {}", e)))?;

    Ok(client)
}

// --- Connection Testing Commands ---

/// Test MongoDB connection using stored credentials
#[command]
async fn test_mongo_connection(_mongo_state: State<'_, MongoState>) -> Result<bool, CommandError> {
    info!("Testing MongoDB connection...");
    let connection_string = get_mongo_credentials_proxy().await.map_err(|e| {
        if matches!(e, CommandError::Configuration(_)) {
            CommandError::Configuration("MongoDB credentials not set. Please configure credentials in Settings.".to_string())
        } else {
            e
        }
    })?;

    let client = create_mongodb_client(connection_string).await?;

    client.list_database_names(None, None).await
        .map_err(|e| CommandError::Database(format!("MongoDB connection test failed: {}", e)))?;

    info!("MongoDB connection test successful.");
    Ok(true)
}

/// Test R2 connection using stored credentials
#[command]
async fn test_r2_connection(r2_state: State<'_, R2State>) -> Result<bool, CommandError> {
    info!("Testing R2 connection...");
    init_r2_client(r2_state).await
}

// --- Audio Processing Commands ---

/// Extract metadata from an audio file (Not Implemented)
// Removed unimplemented extract_audio_metadata function previously defined here.
// The actual command is now in audio::metadata::extract_metadata.

/// Extract metadata from multiple audio files
#[command]
async fn extract_audio_metadata_batch(file_paths: Vec<String>) -> Result<Vec<serde_json::Value>, CommandError> {
    info!("Extracting metadata from {} files", file_paths.len());
    
    let mut results = Vec::with_capacity(file_paths.len());
    
    for path in file_paths {
        // Basic implementation that returns file name and path
        info!("Extracting metadata from {}", path);
        
        let file_path = PathBuf::from(&path);
        let file_name = file_path.file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| "Unknown".to_string());
        
        // Return a basic metadata object
        results.push(serde_json::json!({
            "path": path,
            "fileName": file_name,
            "title": file_name.rsplit('.').nth(1).unwrap_or(&file_name), // Simple attempt to get name without extension
            "duration": 0, // We don't have actual duration yet
            "created": fs::metadata(&path).ok()
                        .and_then(|m| m.created().ok())
                        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                        .map(|d| d.as_secs()),
            "size": fs::metadata(&path).map(|m| m.len()).unwrap_or(0),
        }));
    }
    
    Ok(results)
}

/// Open file dialog and return selected file paths
#[command]
async fn select_audio_files(app_handle: tauri::AppHandle) -> Result<Vec<String>, CommandError> {
    use std::sync::{mpsc, Arc as StdArc, Mutex as StdMutex};
    use tauri_plugin_dialog::FilePath;

    let (tx, rx) = mpsc::channel();
    let tx = StdArc::new(StdMutex::new(tx));
    let tx_clone = StdArc::clone(&tx);

    app_handle.dialog().file().pick_files(move |paths_option: Option<Vec<FilePath>>| {
        let sender = tx_clone.lock().unwrap();
        match paths_option {
            Some(paths) => {
                let path_strings: Vec<String> = paths.into_iter()
                    .filter_map(|fp| fp.as_path().map(|p| p.to_string_lossy().into_owned()))
                    .collect();
                let _ = sender.send(Ok(path_strings));
            }
            None => { // User cancelled
                let _ = sender.send(Ok(Vec::new()));
            }
        }
    });

    rx.recv()
        .map_err(|e| CommandError::Unexpected(format!("Failed to receive file paths from dialog channel: {}", e)))?
}

/// Get file stats (size, modified date)
#[command]
async fn get_file_stats(path: String) -> Result<serde_json::Value, CommandError> {
    fs::metadata(&path)
        .map(|metadata| {
            let size = metadata.len();
            let modified_time = metadata.modified().ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs());
            serde_json::json!({ "size": size, "modified": modified_time })
        })
        .map_err(|e| CommandError::FileSystem(format!("Failed to get metadata for {}: {}", path, e)))
}

/// Transcode a single audio file to AAC
#[command(rename_all = "camelCase")]
async fn transcode_audio_file(
    input_path_str: String,
    output_dir_str: String,
) -> Result<TranscodingResult, CommandError> {
    info!("Transcoding {} to AAC in directory {}", input_path_str, output_dir_str);

    let input_path = PathBuf::from(&input_path_str);
    let output_dir = PathBuf::from(&output_dir_str);

    let file_name = input_path.file_name()
        .ok_or_else(|| CommandError::Validation(format!("Invalid input file path: {}", input_path_str)))?
        .to_string_lossy();
    let stem = Path::new(&*file_name).file_stem().map(|s| s.to_string_lossy()).unwrap_or_default();
    let output_file_name = format!("{}.aac", stem);
    let output_path = output_dir.join(output_file_name);

     if !output_dir.exists() {
         fs::create_dir_all(&output_dir).map_err(|e| {
             CommandError::FileSystem(format!("Failed to create output directory {}: {}", output_dir.display(), e))
         })?;
     }

    let output_path_clone = output_path.clone();
    let join_handle = tokio::task::spawn_blocking(move || {
        transcode::transcode_to_aac(&input_path, &output_path_clone) // Use imported module
    });

    // Await the join handle to get the Result<(), TranscodingError>
    match join_handle.await {
        Ok(transcoding_result) => {
            match transcoding_result {
                Ok(()) => { // transcode_to_aac succeeded
                    Ok(TranscodingResult { output_path: output_path.to_string_lossy().into_owned() })
                },
                Err(transcoding_err) => { // transcode_to_aac failed
                    Err(CommandError::from(transcoding_err))
                }
            }
        },
        Err(join_err) => { // spawn_blocking failed
            Err(CommandError::Unexpected(format!("Task join error during transcoding: {}", join_err)))
        }
    }
}


/// Transcode multiple audio files to AAC
#[command]
async fn transcode_audio_batch(
    file_paths: Vec<String>,
    outputDirStr: String,  // Renamed directly
) -> Result<Vec<TranscodingResult>, CommandError> {
    info!("Starting batch transcoding for {} files to {}", file_paths.len(), &outputDirStr);

    let output_dir = PathBuf::from(&outputDirStr);
    if let Err(e) = fs::create_dir_all(&output_dir) {
        let err = CommandError::FileSystem(format!("Failed to create output directory {}: {}", output_dir.display(), e));
        error!("{}", err); return Err(err);
    }

    let mut tasks = Vec::new();
    for input_path_str in file_paths {
        let current_output_dir = output_dir.clone();
        let input_path_str_clone = input_path_str.clone();

        tasks.push(tokio::spawn(async move {
            let input_path = PathBuf::from(&input_path_str_clone);
            let file_name = match input_path.file_name() {
                Some(name) => name.to_string_lossy(),
                None => return Err(CommandError::Validation(format!("Invalid input file path: {}", input_path_str_clone))),
            };
            let stem = Path::new(&*file_name).file_stem().map(|s| s.to_string_lossy()).unwrap_or_default();
            let output_file_name = format!("{}.aac", stem);
            let output_path = current_output_dir.join(output_file_name);
            let output_path_clone = output_path.clone();

            let join_handle = tokio::task::spawn_blocking(move || {
                transcode::transcode_to_aac(&input_path, &output_path_clone) // Use imported module
            });

            // Await the join handle to get the Result<(), TranscodingError>
            match join_handle.await {
                Ok(transcoding_result) => {
                    match transcoding_result {
                        Ok(()) => { // transcode_to_aac succeeded
                            Ok(TranscodingResult { output_path: output_path.to_string_lossy().into_owned() })
                        },
                        Err(transcoding_err) => { // transcode_to_aac failed
                            Err(CommandError::from(transcoding_err))
                        }
                    }
                },
                Err(join_err) => { // spawn_blocking failed
                    Err(CommandError::Unexpected(format!("Task join error for {}: {}", input_path_str_clone, join_err)))
                }
            }
        }));
    }

    let results = futures::future::join_all(tasks).await;

    let mut successful_results: Vec<TranscodingResult> = Vec::new();
    let mut errors: Vec<CommandError> = Vec::new();

    for result in results {
        match result {
            Ok(Ok(transcoding_result)) => successful_results.push(transcoding_result),
            Ok(Err(cmd_err)) => {
                error!("Batch transcoding error: {}", cmd_err);
                errors.push(cmd_err);
            }
            Err(join_err) => { // This is a Tokio JoinError
                let cmd_err = CommandError::Unexpected(format!("Batch task join error: {}", join_err));
                 error!("{}", cmd_err);
                errors.push(cmd_err);
            }
        }
    }

    if let Some(first_error) = errors.into_iter().next() {
        Err(first_error)
    } else {
        info!("Batch transcoding completed successfully for {} files.", successful_results.len());
        Ok(successful_results)
    }
}


// --- Debug Commands ---
#[command]
async fn debug_mongo_state(mongo_state: State<'_, MongoState>) -> Result<String, CommandError> {
    let client_lock = mongo_state.client.lock().await;
    let status = if client_lock.is_some() { "Initialized" } else { "Not Initialized" };
    Ok(format!("MongoDB State: {}", status))
}

#[tauri::command]
fn ping() -> String {
  "pong".to_string()
}

// Add proxies for credential commands to adapt the error types

// R2 credentials proxy
#[command]
async fn store_r2_credentials_proxy(
    account_id: String,
    bucket_name: String,
    access_key_id: String,
    secret_access_key: String,
    endpoint: String,
) -> Result<bool, CommandError> {
    features::credentials::store_r2_credentials(
        account_id, bucket_name, access_key_id, secret_access_key, endpoint
    ).await.map_err(|e| CommandError::Configuration(format!("Failed to store R2 credentials: {}", e)))
}

// MongoDB credentials proxy
#[command]
async fn store_mongo_credentials_proxy(connection_string: String) -> Result<bool, CommandError> {
    features::credentials::store_mongo_credentials(connection_string)
        .await.map_err(|e| CommandError::Configuration(format!("Failed to store MongoDB credentials: {}", e)))
}

#[command]
async fn get_r2_credentials_proxy() -> Result<features::credentials::R2Credentials, CommandError> {
    features::credentials::get_r2_credentials()
        .await.map_err(|e| CommandError::Configuration(format!("Failed to get R2 credentials: {}", e)))
}

#[command]
async fn get_mongo_credentials_proxy() -> Result<String, CommandError> {
    features::credentials::get_mongo_credentials()
        .await.map_err(|e| CommandError::Configuration(format!("Failed to get MongoDB credentials: {}", e)))
}

#[command]
async fn has_credentials_proxy(credential_type: String) -> Result<bool, CommandError> {
    features::credentials::has_credentials(credential_type)
        .await.map_err(|e| CommandError::Configuration(format!("Failed to check credentials: {}", e)))
}

#[command]
async fn delete_credentials_proxy(credential_type: String) -> Result<(), CommandError> {
    features::credentials::delete_credentials(credential_type)
        .await.map_err(|e| CommandError::Configuration(format!("Failed to delete credentials: {}", e)))
}

// --- Main Application Setup ---
fn main() {
    // Setup logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    info!("Starting Music Library Manager application");

    // Create channel for upload queue
    let (upload_tx, upload_rx) = mpsc::channel::<UploadQueueItem>(100);

    // Initialize Tauri application
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(MongoState { client: Mutex::new(None) })
        .manage(R2State { client: Mutex::new(None), bucket_name: Mutex::new(None) })
        .manage(Arc::new(UploadState::new(upload_tx, upload_rx))) // Wrap state in Arc
        .invoke_handler(tauri::generate_handler![
            // Credential Commands (now from credentials module)
            // Credential Commands (now from features::credentials module)
            // features::credentials::store_r2_credentials,
            // features::credentials::get_r2_credentials,
            // features::credentials::store_mongo_credentials,
            // features::credentials::get_mongo_credentials,
            // features::credentials::has_credentials,
            // features::credentials::delete_credentials,
            // Client Init & Test Commands
            init_r2_client,
            init_mongo_client,
            test_mongo_connection,
            test_r2_connection,
            // Audio/File Commands
            features::upload::audio::metadata::extract_metadata, // Updated path
            extract_audio_metadata_batch,
            select_audio_files,
            get_file_stats,
            transcode_audio_file,
            transcode_audio_batch,
            // MongoDB Commands
            features::catalog::storage::mongodb::fetch_all_tracks,
            features::catalog::storage::mongodb::update_track_metadata, // <-- Added update_track_metadata
            // Upload Queue Commands
            // Upload Queue Commands (from features::upload)
            features::upload::start_upload_queue,
            features::upload::cancel_upload_queue,
            // Debug Commands
            debug_mongo_state,
            ping, // Add the new ping command here
            // New proxies
            store_r2_credentials_proxy,
            store_mongo_credentials_proxy,
            get_r2_credentials_proxy,
            get_mongo_credentials_proxy,
            has_credentials_proxy,
            delete_credentials_proxy,
            // New test command
            test_extract_metadata,
            extract_metadata_wrapper,
            // New credential wrappers
            get_mongo_credentials_wrapper,
            get_r2_credentials_wrapper,
            store_mongo_credentials_wrapper,
            store_r2_credentials_wrapper,
        ])
        .setup(|app| {
            info!("Application setup started");
            let app_handle = app.handle().clone();
            
            // Use tauri's async_runtime instead of tokio::spawn directly
            tauri::async_runtime::spawn(async move {
                let mongo_state: State<MongoState> = app_handle.state();
                let r2_state: State<R2State> = app_handle.state();

                info!("Attempting background initialization of MongoDB client...");
                if let Err(e) = init_mongo_client(mongo_state).await {
                    warn!("Background MongoDB initialization failed: {}", e);
                    let _ = app_handle.emit("mongo-init-failed", e.to_string());
                } else {
                     info!("Background MongoDB initialization successful.");
                     let _ = app_handle.emit("mongo-init-success", ());
                }

                info!("Attempting background initialization of R2 client...");
                 if let Err(e) = init_r2_client(r2_state).await {
                     warn!("Background R2 initialization failed: {}", e);
                     let _ = app_handle.emit("r2-init-failed", e.to_string());
                 } else {
                     info!("Background R2 initialization successful.");
                     let _ = app_handle.emit("r2-init-success", ());
                 }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    info!("Application finished");
}

#[tauri::command]
fn extract_metadata_wrapper(filePath: String) -> Result<serde_json::Value, String> {
    // Call the actual metadata extraction function
    info!("Wrapper calling extract_metadata for: {}", filePath);
    match features::upload::audio::metadata::extract_metadata(filePath) {
        Ok(metadata) => {
            // Convert the UploadItemMetadata struct to a JSON value
            match serde_json::to_value(metadata) {
                Ok(json) => Ok(json),
                Err(e) => Err(format!("Error serializing metadata: {}", e))
            }
        },
        Err(e) => Err(e)
    }
}

#[tauri::command]
async fn get_mongo_credentials_wrapper() -> Result<String, String> {
    // Call the actual credentials function 
    info!("Wrapper calling get_mongo_credentials");
    match features::credentials::get_mongo_credentials().await {
        Ok(creds) => Ok(creds),
        Err(e) => Err(format!("Error retrieving MongoDB credentials: {}", e))
    }
}

#[tauri::command]
async fn get_r2_credentials_wrapper() -> Result<serde_json::Value, String> {
    // Call the actual credentials function
    info!("Wrapper calling get_r2_credentials");
    match features::credentials::get_r2_credentials().await {
        Ok(creds) => {
            // Convert the R2Credentials struct to a JSON value
            match serde_json::to_value(creds) {
                Ok(json) => Ok(json),
                Err(e) => Err(format!("Error serializing R2 credentials: {}", e))
            }
        },
        Err(e) => Err(format!("Error retrieving R2 credentials: {}", e))
    }
}

#[tauri::command]
async fn store_mongo_credentials_wrapper(connectionString: String) -> Result<bool, String> {
    // Call the actual store credentials function
    info!("Wrapper calling store_mongo_credentials for connection string");
    match features::credentials::store_mongo_credentials(connectionString).await {
        Ok(result) => Ok(result),
        Err(e) => Err(format!("Error storing MongoDB credentials: {}", e))
    }
}

#[tauri::command]
async fn store_r2_credentials_wrapper(
    accountId: String,
    bucketName: String,
    accessKeyId: String,
    secretAccessKey: String,
    endpoint: String,
) -> Result<bool, String> {
    // Call the actual store credentials function
    info!("Wrapper calling store_r2_credentials");
    match features::credentials::store_r2_credentials(
        accountId, bucketName, accessKeyId, secretAccessKey, endpoint
    ).await {
        Ok(result) => Ok(result),
        Err(e) => Err(format!("Error storing R2 credentials: {}", e))
    }
}
