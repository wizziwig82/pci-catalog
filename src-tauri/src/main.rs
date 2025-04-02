// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use log::{debug, error, info, warn};
use mongodb::{bson::doc, Client};
// Commented out problematic imports
// use rusty_s3::{Bucket, S3Action};
use tauri_plugin_dialog::DialogExt;
use std::fs;
use std::path::{Path, PathBuf};
// use std::collections::HashMap;
// use std::sync::Arc;
use keyring::Entry;
use tauri::{
    async_runtime, command, AppHandle, Manager, State, Window,
};
// use url::Url;

use crate::audio::transcoding::TranscodingError;

// Define keychain constants
const KEYCHAIN_SERVICE_MONGO: &str = "com.musiclibrarymanager.mongo";
const KEYCHAIN_ACCOUNT_MONGO: &str = "mongo_credentials";
const KEYCHAIN_SERVICE_R2: &str = "com.musiclibrarymanager.r2";
const KEYCHAIN_ACCOUNT_R2: &str = "r2_credentials";

// Dev-mode fallback config file path for credentials (only used if keychain fails)
#[cfg(debug_assertions)]
const DEV_CREDENTIALS_FILE: &str = "dev_credentials.json";

// Import audio module
mod audio;
use audio::{extract_metadata, AudioMetadata};
// use audio::{TrackMetadata, AlbumMetadata};
use audio::transcoding::{transcode_file, TranscodingOptions, TranscodingResult, transcode_to_quality};

// Import storage module
mod storage;
// Simplify imports to avoid unused warnings
// use storage::r2::{R2Client, R2UploadResult};
// use storage::mongodb::{MongoClient as MongoClientImpl, Album, Track, DbResponse};

// Import the upload module and its functions
mod upload;
use upload::upload_transcoded_tracks;
use upload::upload_album_artwork;
// use upload::{BulkUploadResponse, UploadedTrackInfo, FailedTrackInfo, UploadPathConfig};

// Import commands module
mod commands;

/// MongoDB client state
pub struct MongoState {
    client: Mutex<Option<mongodb::Client>>,
}

/// R2 client state
pub struct R2State {
    client: Mutex<Option<aws_sdk_s3::Client>>,
}

// Development credentials storage structure
#[cfg(debug_assertions)]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct DevCredentials {
    mongo_connection_string: Option<String>,
    r2_credentials: Option<R2Credentials>,
}

// Development-mode fallback for credential storage/retrieval
#[cfg(debug_assertions)]
async fn load_dev_credentials() -> DevCredentials {
    let path = PathBuf::from(DEV_CREDENTIALS_FILE);
    if path.exists() {
        match fs::read_to_string(&path) {
            Ok(json_str) => {
                match serde_json::from_str::<DevCredentials>(&json_str) {
                    Ok(creds) => {
                        info!("Loaded development credentials from file");
                        return creds;
                    },
                    Err(e) => {
                        warn!("Failed to parse development credentials file: {}", e);
                    }
                }
            },
            Err(e) => {
                warn!("Failed to read development credentials file: {}", e);
            }
        }
    }
    
    DevCredentials::default()
}

#[cfg(debug_assertions)]
async fn save_dev_credentials(creds: &DevCredentials) -> Result<(), String> {
    let json_str = match serde_json::to_string_pretty(creds) {
        Ok(s) => s,
        Err(e) => return Err(format!("Failed to serialize development credentials: {}", e)),
    };
    
    match fs::write(DEV_CREDENTIALS_FILE, json_str) {
        Ok(_) => {
            info!("Saved development credentials to file");
            Ok(())
        },
        Err(e) => {
            warn!("Failed to write development credentials file: {}", e);
            Err(format!("Failed to write development credentials file: {}", e))
        }
    }
}

/// MongoDB credentials structure (keep for type hint, though we only store the connection string now)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MongoCredentials {
    pub username: String,
    pub password: String,
    pub hostname: String,
    pub port: u16,
}

/// R2 credentials structure (still needed for serialization)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct R2Credentials {
    pub account_id: String,
    pub bucket_name: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub endpoint: String,
}

/// Stores R2 credentials in Keychain using keyring
#[command]
async fn store_r2_credentials(
    account_id: String,
    bucket_name: String,
    access_key_id: String,
    secret_access_key: String,
    endpoint: String,
) -> Result<bool, String> {
    // Log that we're storing R2 credentials
    info!("Storing R2 credentials in keychain");
    
    // Create the credentials struct
    let creds = R2Credentials {
        account_id,
        bucket_name,
        access_key_id,
        secret_access_key,
        endpoint,
    };
    
    // Create the keyring entry
    let entry = match Entry::new(KEYCHAIN_SERVICE_R2, KEYCHAIN_ACCOUNT_R2) {
        Ok(entry) => entry,
        Err(e) => {
            error!("Failed to create keyring entry for R2 credentials: {}", e);
            
            // In debug mode, fall back to file storage
            #[cfg(debug_assertions)]
            {
                info!("Using development fallback for storing R2 credentials");
                let mut dev_creds = load_dev_credentials().await;
                dev_creds.r2_credentials = Some(creds);
                match save_dev_credentials(&dev_creds).await {
                    Ok(_) => return Ok(true),
                    Err(e) => return Err(format!("Failed to store R2 credentials in dev fallback: {}", e)),
                }
            }
            
            return Err(format!("Failed to access keychain: {}", e));
        }
    };
    
    // Serialize the credentials to a JSON string
    let json_str = match serde_json::to_string(&creds) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to serialize R2 credentials to JSON: {}", e);
            return Err(format!("Failed to serialize R2 credentials: {}", e));
        }
    };
    
    // First try to delete any existing entry to prevent duplicates
    let _ = entry.delete_credential();
    
    // Store the password in the keyring
    match entry.set_password(&json_str) {
        Ok(_) => {
            info!("Successfully stored R2 credentials");
            
            // In debug mode, also save to file as a backup
            #[cfg(debug_assertions)]
            {
                let mut dev_creds = load_dev_credentials().await;
                dev_creds.r2_credentials = Some(creds);
                let _ = save_dev_credentials(&dev_creds).await;
            }
            
            Ok(true)
        },
        Err(e) => {
            error!("Failed to store R2 credentials in keychain: {}", e);
            
            // In debug mode, fall back to file storage
            #[cfg(debug_assertions)]
            {
                info!("Using development fallback for storing R2 credentials after keychain failure");
                let mut dev_creds = load_dev_credentials().await;
                dev_creds.r2_credentials = Some(creds);
                match save_dev_credentials(&dev_creds).await {
                    Ok(_) => return Ok(true),
                    Err(e) => return Err(format!("Failed to store R2 credentials in dev fallback: {}", e)),
                }
            }
            
            Err(format!("Failed to store R2 credentials: {}", e))
        }
    }
}

/// Retrieves R2 credentials from Keychain using keyring
#[command]
async fn get_r2_credentials() -> Result<R2Credentials, String> {
    // Log that we're getting R2 credentials
    info!("Retrieving R2 credentials from keychain");
    
    let entry = match Entry::new(KEYCHAIN_SERVICE_R2, KEYCHAIN_ACCOUNT_R2) {
        Ok(entry) => entry,
        Err(e) => {
            error!("Failed to create keyring entry for R2 credentials: {}", e);
            
            // In debug mode, try to get from file storage
            #[cfg(debug_assertions)]
            {
                info!("Using development fallback for retrieving R2 credentials");
                let dev_creds = load_dev_credentials().await;
                if let Some(creds) = dev_creds.r2_credentials {
                    return Ok(creds);
                }
            }
            
            return Err(format!("Failed to access keychain: {}", e));
        }
    };
    
    // Try to get the password from the keyring
    match entry.get_password() {
        Ok(json_str) => {
            if json_str.is_empty() {
                info!("R2 credentials are empty");
                return Err("R2 credentials not set".to_string());
            }
            
            // Parse the JSON string into R2Credentials
            match serde_json::from_str::<R2Credentials>(&json_str) {
                Ok(creds) => {
                    info!("Successfully retrieved R2 credentials");
                    Ok(creds)
                },
                Err(e) => {
                    error!("Failed to parse R2 credentials JSON: {}", e);
                    Err(format!("Failed to parse R2 credentials: {}", e))
                }
            }
        },
        Err(e) => {
            // Check if this is a "not found" error
            if e.to_string().to_lowercase().contains("not found") ||
               e.to_string().to_lowercase().contains("no item") {
                info!("R2 credentials not found in keychain");
                
                // In debug mode, try to get from file storage
                #[cfg(debug_assertions)]
                {
                    info!("Using development fallback for retrieving R2 credentials");
                    let dev_creds = load_dev_credentials().await;
                    if let Some(creds) = dev_creds.r2_credentials {
                        return Ok(creds);
                    }
                }
                
                return Err("R2 credentials not found".to_string());
            }
            
            error!("Failed to get R2 credentials from keychain: {}", e);
            
            // In debug mode, try to get from file storage
            #[cfg(debug_assertions)]
            {
                info!("Using development fallback for retrieving R2 credentials after keychain error");
                let dev_creds = load_dev_credentials().await;
                if let Some(creds) = dev_creds.r2_credentials {
                    return Ok(creds);
                }
            }
            
            Err(format!("Failed to get R2 credentials: {}", e))
        }
    }
}

/// Stores MongoDB connection string in Keychain using keyring
#[command]
async fn store_mongo_credentials(
    connection_string: String,
) -> Result<bool, String> {
    // Log that we're storing MongoDB credentials
    info!("Storing MongoDB credentials in keychain");
    
    // Create the keyring entry
    let entry = match Entry::new(KEYCHAIN_SERVICE_MONGO, KEYCHAIN_ACCOUNT_MONGO) {
        Ok(entry) => entry,
        Err(e) => {
            error!("Failed to create keyring entry for MongoDB credentials: {}", e);
            
            // In debug mode, fall back to file storage
            #[cfg(debug_assertions)]
            {
                info!("Using development fallback for storing MongoDB credentials");
                let mut dev_creds = load_dev_credentials().await;
                dev_creds.mongo_connection_string = Some(connection_string.clone());
                match save_dev_credentials(&dev_creds).await {
                    Ok(_) => return Ok(true),
                    Err(e) => return Err(format!("Failed to store MongoDB credentials in dev fallback: {}", e)),
                }
            }
            
            return Err(format!("Failed to access keychain: {}", e));
        }
    };
    
    // First try to delete any existing entry to prevent duplicates
    let _ = entry.delete_credential();
    
    // Store the password in the keyring
    match entry.set_password(&connection_string) {
        Ok(_) => {
            info!("Successfully stored MongoDB credentials");
            
            // In debug mode, also save to file as a backup
            #[cfg(debug_assertions)]
            {
                let mut dev_creds = load_dev_credentials().await;
                dev_creds.mongo_connection_string = Some(connection_string);
                let _ = save_dev_credentials(&dev_creds).await;
            }
            
            Ok(true)
        },
        Err(e) => {
            error!("Failed to store MongoDB credentials in keychain: {}", e);
            
            // In debug mode, fall back to file storage
            #[cfg(debug_assertions)]
            {
                info!("Using development fallback for storing MongoDB credentials after keychain failure");
                let mut dev_creds = load_dev_credentials().await;
                dev_creds.mongo_connection_string = Some(connection_string);
                match save_dev_credentials(&dev_creds).await {
                    Ok(_) => return Ok(true),
                    Err(e) => return Err(format!("Failed to store MongoDB credentials in dev fallback: {}", e)),
                }
            }
            
            Err(format!("Failed to store MongoDB credentials: {}", e))
        }
    }
}

/// Retrieves MongoDB connection string from Keychain using keyring
#[command]
async fn get_mongo_credentials() -> Result<String, String> {
    // Log that we're getting MongoDB credentials
    info!("Retrieving MongoDB credentials from keychain");
    
    let entry = match Entry::new(KEYCHAIN_SERVICE_MONGO, KEYCHAIN_ACCOUNT_MONGO) {
        Ok(entry) => entry,
        Err(e) => {
            error!("Failed to create keyring entry for MongoDB credentials: {}", e);
            
            // In debug mode, try to get from file storage
            #[cfg(debug_assertions)]
            {
                info!("Using development fallback for retrieving MongoDB credentials");
                let dev_creds = load_dev_credentials().await;
                if let Some(conn_string) = dev_creds.mongo_connection_string {
                    return Ok(conn_string);
                }
            }
            
            return Err(format!("Failed to access keychain: {}", e));
        }
    };
    
    // Try to get the password from the keyring
    match entry.get_password() {
        Ok(connection_string) => {
            if connection_string.is_empty() {
                info!("MongoDB connection string is empty");
                return Err("MongoDB connection string not set".to_string());
            }
            
            info!("Successfully retrieved MongoDB credentials");
            Ok(connection_string)
        },
        Err(e) => {
            // Check if this is a "not found" error
            if e.to_string().to_lowercase().contains("not found") ||
               e.to_string().to_lowercase().contains("no item") {
                info!("MongoDB credentials not found in keychain");
                
                // In debug mode, try to get from file storage
                #[cfg(debug_assertions)]
                {
                    info!("Using development fallback for retrieving MongoDB credentials");
                    let dev_creds = load_dev_credentials().await;
                    if let Some(conn_string) = dev_creds.mongo_connection_string {
                        return Ok(conn_string);
                    }
                }
                
                return Err("MongoDB credentials not found".to_string());
            }
            
            error!("Failed to get MongoDB credentials from keychain: {}", e);
            
            // In debug mode, try to get from file storage
            #[cfg(debug_assertions)]
            {
                info!("Using development fallback for retrieving MongoDB credentials after keychain error");
                let dev_creds = load_dev_credentials().await;
                if let Some(conn_string) = dev_creds.mongo_connection_string {
                    return Ok(conn_string);
                }
            }
            
            Err(format!("Failed to get MongoDB credentials: {}", e))
        }
    }
}

/// Check if credentials exist in the keychain
#[command]
async fn has_credentials(credential_type: String) -> Result<bool, String> {
    let (service, account) = match credential_type.as_str() {
         "mongo" => (KEYCHAIN_SERVICE_MONGO, KEYCHAIN_ACCOUNT_MONGO),
         "r2" => (KEYCHAIN_SERVICE_R2, KEYCHAIN_ACCOUNT_R2),
         _ => return Err("Invalid credential type".to_string()),
     };
     
     // Create the entry
     let entry = match Entry::new(service, account) {
         Ok(entry) => entry,
         Err(e) => {
             error!("Failed to create keyring entry for checking {}: {}", credential_type, e);
             return Err(format!("Failed to access keychain: {}", e));
         }
     };
     
     // Try to get the password - if successful, credentials exist
     match entry.get_password() {
         Ok(_) => {
             info!("Found {} credentials in keychain", credential_type);
             Ok(true)
         },
         Err(e) => {
             // Check if this is a "not found" error
             if e.to_string().to_lowercase().contains("not found") ||
                e.to_string().to_lowercase().contains("no item") {
                 info!("{} credentials not found in keychain", credential_type);
                 return Ok(false);
             }
             
             error!("Failed to check if {} credentials exist: {}", credential_type, e);
             Err(format!("Failed to check credentials: {}", e))
         }
     }
}

/// Delete credentials from the keychain
#[command]
async fn delete_credentials(credential_type: String) -> Result<(), String> {
     let (service, account) = match credential_type.as_str() {
         "mongo" => (KEYCHAIN_SERVICE_MONGO, KEYCHAIN_ACCOUNT_MONGO),
         "r2" => (KEYCHAIN_SERVICE_R2, KEYCHAIN_ACCOUNT_R2),
         _ => return Err("Invalid credential type".to_string()),
     };
 
     let entry = Entry::new(service, account)
         .map_err(|e| format!("Failed to create Keychain entry for deleting {}: {}", credential_type, e))?;
 
     match entry.delete_credential() {
         Ok(_) => {
             info!("Successfully deleted {} credentials", credential_type);
             Ok(())
         },
         Err(e) => {
             // If it's a "not found" error, consider it a success
             if e.to_string().to_lowercase().contains("not found") ||
                e.to_string().to_lowercase().contains("no item") {
                 info!("{} credentials already deleted or not found", credential_type);
                 return Ok(());
             }
             
             error!("Failed to delete {} credentials: {}", credential_type, e);
             Err(format!("Failed to delete credentials: {}", e))
         }
     }
}

/// Initializes R2 client using stored credentials
#[command]
async fn init_r2_client(
    r2_state: State<'_, R2State>,
) -> Result<bool, String> {
    // Check if client is already initialized
    {
        let lock = r2_state.client.lock().await;
        if lock.is_some() {
            info!("R2 client already initialized, reusing existing client");
            return Ok(true);
        }
    }
    
    let credentials_result = get_r2_credentials().await;
    if let Err(e) = credentials_result {
        // If credentials are not found, return a specific error
        if e.contains("not found") {
            return Err("R2 credentials not set. Please configure credentials in Settings.".to_string());
        }
        return Err(format!("Failed to get R2 credentials: {}", e));
    }
    
    let credentials = credentials_result.unwrap();
    
    info!("Creating new R2 client with account ID: {} and access key: {}", 
        credentials.account_id, credentials.access_key_id);
    
    // Determine the endpoint - use the provided endpoint if available,
    // otherwise construct it from account_id
    let endpoint = if !credentials.endpoint.is_empty() {
        credentials.endpoint.clone()
    } else {
        format!("https://{}.r2.cloudflarestorage.com", credentials.account_id)
    };
    
    let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .region(aws_sdk_s3::config::Region::new("auto"))
        .endpoint_url(&endpoint)
        .credentials_provider(
            aws_sdk_s3::config::Credentials::new(
                &credentials.access_key_id,
                &credentials.secret_access_key,
                None, None, "r2-credentials"
            )
        )
        .load()
        .await;
    
    let client = aws_sdk_s3::Client::new(&config);
    
    // Test the connection
    info!("Testing R2 connection");
    let test_result = client.list_buckets().send().await;
    match test_result {
        Ok(_) => {
            info!("R2 connection test successful");
            let mut lock = r2_state.client.lock().await;
            *lock = Some(client);
            Ok(true)
        },
        Err(e) => {
            error!("R2 connection test failed: {}", e);
            Err(format!("R2 connection test failed: {}", e))
        }
    }
}

/// Initialize MongoDB client using stored credentials
#[command]
async fn init_mongo_client(mongo_state: State<'_, MongoState>) -> Result<bool, String> {
    info!("Initializing MongoDB client");
    
    // Get existing client first
    {
        let client = mongo_state.client.lock().await;
        if client.is_some() {
            info!("MongoDB client is already initialized");
            return Ok(true);
        }
    }
    
    // Get connection string from keychain
    let connection_string = match get_mongo_credentials().await {
        Ok(connection_string) => {
            // Log a masked version of the connection string for debugging
            let masked_string = if connection_string.len() > 20 {
                format!("{}...{}", &connection_string[0..10], &connection_string[connection_string.len() - 10..])
            } else {
                "too short to mask".to_string()
            };
            info!("Retrieved MongoDB connection string (masked): {}", masked_string);
            
            // Validate connection string format
            if !connection_string.starts_with("mongodb://") && !connection_string.starts_with("mongodb+srv://") {
                let error_msg = "Invalid MongoDB connection string format. Must start with 'mongodb://' or 'mongodb+srv://'";
                error!("{}", error_msg);
                return Err(error_msg.to_string());
            }
            
            // Check for any line breaks or invalid characters that could cause connection issues
            if connection_string.contains('\n') || connection_string.contains('\r') {
                let error_msg = "MongoDB connection string contains line breaks which may cause connection issues";
                error!("{}", error_msg);
                return Err(error_msg.to_string());
            }
            
            connection_string
        },
        Err(e) => {
            error!("Failed to get MongoDB connection string: {}", e);
            return Err(format!("MongoDB credentials not set: {}", e));
        }
    };
    
    // Create MongoDB client options with proper timeout settings
    info!("Setting up MongoDB client options");
    let mut client_options = match mongodb::options::ClientOptions::parse(&connection_string).await {
        Ok(options) => {
            info!("Successfully parsed MongoDB connection string");
            options
        },
        Err(e) => {
            error!("Failed to parse MongoDB connection string: {}", e);
            return Err(format!("Failed to parse MongoDB connection string: {}", e));
        }
    };
    
    // Set timeout options
    client_options.connect_timeout = Some(std::time::Duration::from_secs(10));
    client_options.server_selection_timeout = Some(std::time::Duration::from_secs(10));
    
    // Create MongoDB client with options
    info!("Creating MongoDB client with timeout options");
    match mongodb::Client::with_options(client_options) {
        Ok(client) => {
            // Try a quick ping to verify the connection works
            info!("MongoDB client created, testing with ping");
            match client.database("admin").run_command(mongodb::bson::doc! {"ping": 1}, None).await {
                Ok(_) => {
                    info!("MongoDB ping successful");
                    // Store the client in state
                    let mut state_client = mongo_state.client.lock().await;
                    *state_client = Some(client);
                    info!("MongoDB client initialized successfully");
                    Ok(true)
                },
                Err(e) => {
                    error!("MongoDB ping failed: {}", e);
                    Err(format!("MongoDB connection succeeded but ping failed: {}", e))
                }
            }
        }
        Err(e) => {
            error!("Failed to create MongoDB client with specific error: {}", e);
            Err(format!("Failed to create MongoDB client: {}", e))
        }
    }
}

/// Tests MongoDB connection
#[command]
async fn test_mongo_connection(
    mongo_state: State<'_, MongoState>,
) -> Result<bool, String> {
    let lock = mongo_state.client.lock().await;
    match &*lock {
        Some(client) => {
            match client.database("admin").run_command(mongodb::bson::doc! {"ping": 1}, None).await {
                Ok(_) => Ok(true),
                Err(e) => Err(format!("MongoDB connection test failed: {}", e)),
            }
        }
        None => Err("MongoDB client not initialized".to_string()),
    }
}

/// Tests R2 connection
#[command]
async fn test_r2_connection(
    r2_state: State<'_, R2State>,
) -> Result<bool, String> {
    let lock = r2_state.client.lock().await;
    match &*lock {
        Some(client) => {
            // First test general credentials by listing buckets
            match client.list_buckets().send().await {
                Ok(_) => {
                    // Next, test specifically accessing the configured bucket by listing objects
                    // Get the credentials to find the bucket name
                    match get_r2_credentials().await {
                        Ok(creds) => {
                            match client.list_objects_v2()
                                .bucket(&creds.bucket_name)
                                .max_keys(1)
                                .send()
                                .await {
                                    Ok(_) => {
                                        info!("Successfully connected to R2 and accessed bucket: {}", creds.bucket_name);
                                        Ok(true)
                                    },
                                    Err(e) => {
                                        error!("R2 bucket access test failed: {}", e);
                                        Err(format!("R2 credentials are valid but couldn't access bucket '{}': {}", 
                                            creds.bucket_name, e))
                                    }
                                }
                        },
                        Err(e) => {
                            // If we can't get credentials but can list buckets, something is odd
                            warn!("Could list R2 buckets but couldn't retrieve bucket name: {}", e);
                            Ok(true) // Return true since we could at least connect
                        }
                    }
                },
                Err(e) => {
                    error!("R2 connection test failed: {}", e);
                    Err(format!("R2 connection test failed: {}", e))
                }
            }
        }
        None => Err("R2 client not initialized".to_string()),
    }
}

/// Extract metadata from an audio file
#[command]
async fn extract_audio_metadata(file_path: String) -> Result<AudioMetadata, String> {
    info!("Extracting metadata from file: {}", file_path);
    extract_metadata(&file_path)
}

/// Extract metadata from multiple audio files
#[command]
async fn extract_audio_metadata_batch(file_paths: Vec<String>) -> Result<Vec<AudioMetadata>, String> {
    info!("Extracting metadata from {} files", file_paths.len());
    
    let mut results = Vec::with_capacity(file_paths.len());
    
    for path in file_paths {
        match extract_metadata(&path) {
            Ok(metadata) => results.push(metadata),
            Err(e) => {
                // Log error but continue processing other files
                error!("Failed to extract metadata from {}: {}", path, e);
            }
        }
    }
    
    Ok(results)
}

/// Open file dialog and return selected file paths
#[command]
async fn select_audio_files(app_handle: tauri::AppHandle) -> Result<Vec<String>, String> {
    use std::sync::{Arc, Mutex as StdMutex};
    
    // Create channels to communicate the result
    let (tx, rx) = std::sync::mpsc::channel();
    let tx = Arc::new(StdMutex::new(tx));
    
    // Get a handle to the dialog manager and use the non-blocking pick_files
    app_handle.dialog()
        .file()
        .add_filter("Audio Files", &["mp3", "wav", "flac", "aac", "m4a", "ogg", "aiff"])
        .set_directory("/")
        .set_title("Select Audio Files")
        .pick_files(move |file_paths| {
            if let Some(paths) = file_paths {
                let mut valid_paths = Vec::new();
                
                for path in paths.iter() {
                    if let Some(path_str) = path.as_path() {
                        let path_string = path_str.to_string_lossy().to_string();
                        
                        // Verify the file exists
                        if std::path::Path::new(&path_string).exists() {
                            info!("Selected valid audio file: {}", path_string);
                            valid_paths.push(path_string);
                        } else {
                            warn!("Selected file does not exist: {}", path_string);
                        }
                    }
                }
                
                if valid_paths.is_empty() && !paths.is_empty() {
                    warn!("No valid files were selected");
                }
                
                let _ = tx.lock().unwrap().send(Ok(valid_paths));
            } else {
                // User cancelled the dialog
                info!("File selection canceled by user");
                let _ = tx.lock().unwrap().send(Ok(Vec::new()));
            }
        });
    
    // Wait for the result
    match rx.recv() {
        Ok(result) => result,
        Err(e) => Err(format!("Failed to get file dialog result: {}", e)),
    }
}

/// Get file statistics
#[command]
async fn get_file_stats(path: String) -> Result<serde_json::Value, String> {
    info!("Getting file stats for: {}", path);
    
    match fs::metadata(&path) {
        Ok(metadata) => {
            let size = metadata.len();
            let result = serde_json::json!({
                "size": size,
                "is_file": metadata.is_file(),
                "is_dir": metadata.is_dir(),
            });
            
            Ok(result)
        },
        Err(e) => Err(format!("Failed to get file metadata: {}", e))
    }
}

/// Transcode an audio file to create medium quality version
#[command]
async fn transcode_audio_file(
    file_path: String,
    medium_bitrate: u32,
    output_format: String,
    output_dir: String,
) -> Result<TranscodingResult, String> {
    info!("Transcoding audio file: {}", file_path);
    
    let options = TranscodingOptions {
        medium_bitrate,
        format: output_format,
        output_dir,
    };
    
    match transcode_file(&file_path, options) {
        Ok(result) => Ok(result),
        Err(e) => {
            error!("Failed to transcode file: {}", e);
            Err(format!("Failed to transcode file: {}", e))
        }
    }
}

/// Transcode multiple audio files in batch
#[command]
async fn transcode_audio_batch(
    file_paths: Vec<String>,
    medium_bitrate: u32,
    output_format: String,
    output_dir: String,
) -> Result<Vec<TranscodingResult>, String> {
    info!("Batch transcoding {} audio files", file_paths.len());
    
    // Input validation
    if file_paths.is_empty() {
        return Err("No files provided for transcoding".to_string());
    }
    
    if medium_bitrate < 32 || medium_bitrate > 320 {
        let error_msg = format!("Invalid bitrate: {}. Must be between 32 and 320 kbps", medium_bitrate);
        error!("{}", error_msg);
        return Err(error_msg);
    }
    
    let supported_formats = ["mp3", "aac", "m4a", "ogg", "flac", "wav"];
    if !supported_formats.contains(&output_format.to_lowercase().as_str()) {
        let error_msg = format!("Unsupported format: {}. Supported formats are: {:?}", output_format, supported_formats);
        error!("{}", error_msg);
        return Err(error_msg);
    }
    
    info!("DEBUG: Input validation completed successfully");
    
    // Use system temp directory instead of project directory to avoid auto-rebuilds
    let temp_base = std::env::temp_dir();
    let unique_dir = temp_base.join("music-library-transcoded");
    info!("Using system temp directory for output: {}", unique_dir.display());
    
    // Ensure we have an absolute path
    let absolute_output_dir = unique_dir.to_string_lossy().to_string();
    
    info!("Using absolute output directory: {}", absolute_output_dir);
    info!("DEBUG: Absolute output directory determined: {}", absolute_output_dir);
    
    // Create the output directory
    if !PathBuf::from(&absolute_output_dir).exists() {
        match std::fs::create_dir_all(&absolute_output_dir) {
            Ok(_) => info!("Created output directory: {}", absolute_output_dir),
            Err(e) => {
                let error_msg = format!("Failed to create output directory: {}", e);
                error!("{}", error_msg);
                return Err(error_msg);
            }
        }
    }
    info!("DEBUG: Output directory created successfully");
    
    // Test if the output directory is writable
    let test_file_path = PathBuf::from(&absolute_output_dir).join("write_test.tmp");
    match std::fs::File::create(&test_file_path) {
        Ok(_) => {
            info!("Output directory is writable");
            // Clean up the test file
            if let Err(e) = std::fs::remove_file(&test_file_path) {
                warn!("Failed to remove test file: {}", e);
            } else {
                info!("DEBUG: Test file removed successfully");
            }
        },
        Err(e) => {
            let error_msg = format!("Output directory is not writable: {}", e);
            error!("{}", error_msg);
            return Err(error_msg);
        }
    }
    
    info!("DEBUG: Output directory verified as writable");
    
    // Prepare transcoding options
    let options = TranscodingOptions {
        medium_bitrate,
        format: output_format.clone(),
        output_dir: absolute_output_dir.clone(),
    };
    
    info!("Transcoding with options: bitrate={}k, format={}, output_dir={}", 
          medium_bitrate, output_format, options.output_dir);
    
    let mut results = Vec::new();
    
    // Process each file
    for (index, file_path) in file_paths.iter().enumerate() {
        info!("Processing file {}/{}: {}", index + 1, file_paths.len(), file_path);
        
        // Check if file exists before attempting to transcode
        let path = PathBuf::from(file_path);
        if !path.exists() {
            let error_msg = format!("File does not exist: {}", file_path);
            warn!("{}", error_msg);
            results.push(TranscodingResult {
                success: false,
                input_path: file_path.clone(),
                medium_quality_path: None,
                error_message: Some(error_msg),
            });
            continue;
        }
        
        info!("DEBUG: File exists, proceeding with transcoding: {}", file_path);
        
        // Catch any panics that might occur during transcoding
        info!("DEBUG: Beginning transcode_file call for: {}", file_path);
        let transcode_result = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            transcode_file(file_path, options.clone())
        })) {
            Ok(result) => result,
            Err(panic_err) => {
                let panic_msg = match panic_err.downcast_ref::<&str>() {
                    Some(s) => format!("Panic during transcoding: {}", s),
                    None => match panic_err.downcast_ref::<String>() {
                        Some(s) => format!("Panic during transcoding: {}", s),
                        None => "Unknown panic during transcoding".to_string(),
                    },
                };
                
                error!("{}", panic_msg);
                Err(TranscodingError::FFmpegError(panic_msg))
            }
        };
        
        match transcode_result {
            Ok(result) => {
                info!("Successfully transcoded file {}: {}", index + 1, file_path);
                results.push(result);
            }
            Err(error) => {
                error!("Failed to transcode file {}: {}", index + 1, error);
                results.push(TranscodingResult {
                    success: false,
                    input_path: file_path.clone(),
                    medium_quality_path: None,
                    error_message: Some(error.to_string()),
                });
            }
        }
    }
    
    info!("Completed batch transcoding: {} successful, {} failed", 
          results.iter().filter(|r| r.success).count(),
          results.iter().filter(|r| !r.success).count());
    
    Ok(results)
}

/// Get MongoDB client state for debugging
#[command]
async fn debug_mongo_state(mongo_state: State<'_, MongoState>) -> Result<String, String> {
    let client_lock = mongo_state.client.lock().await;
    
    let has_client = client_lock.is_some();
    info!("debug_mongo_state: MongoDB client is initialized: {}", has_client);
    
    match get_mongo_credentials().await {
        Ok(conn_string) => {
            // Mask credentials for logging
            let masked = if conn_string.len() > 20 {
                format!("{}...{}", &conn_string[0..10], &conn_string[conn_string.len()-10..])
            } else {
                "too short to mask".to_string()
            };
            info!("debug_mongo_state: MongoDB credentials found, masked: {}", masked);
            
            if has_client {
                Ok(format!("MongoDB client is initialized and credentials are available"))
            } else {
                Ok(format!("MongoDB client is NOT initialized but credentials are available"))
            }
        },
        Err(e) => {
            error!("debug_mongo_state: Failed to get MongoDB credentials: {}", e);
            Ok(format!("MongoDB client is {} and failed to get credentials: {}", 
                if has_client { "initialized" } else { "NOT initialized" }, e))
        }
    }
}

/// Create and initialize a new MongoDB client
/// This is a separate function to help with debugging
#[command]
async fn create_mongodb_client(connection_string: String) -> Result<bool, String> {
    info!("Attempting to create MongoDB client with connection string: {}", 
          if connection_string.len() > 20 {
              format!("{}...{}", &connection_string[0..10], &connection_string[connection_string.len()-10..])
          } else {
              "too short to mask".to_string()
          });
    
    // Set up client options with timeouts
    let mut client_options = match mongodb::options::ClientOptions::parse(&connection_string).await {
        Ok(options) => {
            info!("Successfully parsed MongoDB connection string");
            options
        },
        Err(e) => {
            error!("Failed to parse MongoDB connection string: {}", e);
            return Err(format!("Failed to parse MongoDB connection string: {}", e));
        }
    };
    
    // Set timeout options
    client_options.connect_timeout = Some(std::time::Duration::from_secs(10));
    client_options.server_selection_timeout = Some(std::time::Duration::from_secs(10));
    
    // Create MongoDB client with options
    match mongodb::Client::with_options(client_options) {
        Ok(client) => {
            // Try a quick ping to verify the connection works
            info!("MongoDB client created, testing with ping");
            match client.database("admin").run_command(mongodb::bson::doc! {"ping": 1}, None).await {
                Ok(_) => {
                    info!("MongoDB ping successful");
                    Ok(true)
                },
                Err(e) => {
                    error!("MongoDB ping failed: {}", e);
                    Err(format!("MongoDB connection succeeded but ping failed: {}", e))
                }
            }
        },
        Err(e) => {
            error!("Failed to create MongoDB client with specific error: {}", e);
            Err(format!("Failed to create MongoDB client: {}", e))
        }
    }
}

/// Initialize the Tauri application
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(MongoState {
            client: Mutex::new(None),
        })
        .manage(R2State {
            client: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            store_mongo_credentials,
            store_r2_credentials,
            get_mongo_credentials,
            get_r2_credentials,
            has_credentials,
            delete_credentials,
            init_mongo_client,
            init_r2_client,
            test_mongo_connection,
            test_r2_connection,
            extract_audio_metadata,
            extract_audio_metadata_batch,
            select_audio_files,
            get_file_stats,
            transcode_audio_file,
            transcode_audio_batch,
            upload_transcoded_tracks,
            upload_album_artwork,
            debug_mongo_state,
            create_mongodb_client,
            commands::clear_test_data,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
