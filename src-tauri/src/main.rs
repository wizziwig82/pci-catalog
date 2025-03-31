// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use tauri::{command, State, Manager};
use tokio::sync::Mutex;
use log::{info, error};
// Use keyring import
use keyring::Entry; 
use tauri_plugin_dialog::DialogExt;
use std::fs;

// Import audio module
mod audio;
use audio::{extract_metadata, AudioMetadata};

/// Keychain service names (using keyring convention)
const KEYCHAIN_SERVICE_MONGO: &str = "com.musiclibrarymanager.mongo";
const KEYCHAIN_SERVICE_R2: &str = "com.musiclibrarymanager.r2";
const KEYCHAIN_ACCOUNT_MONGO: &str = "mongodb_connection_string";
const KEYCHAIN_ACCOUNT_R2: &str = "r2_credentials";

/// MongoDB client state
pub struct MongoState {
    client: Mutex<Option<mongodb::Client>>,
}

/// R2 client state
pub struct R2State {
    client: Mutex<Option<aws_sdk_s3::Client>>,
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
    pub access_key_id: String,
    pub secret_access_key: String,
}

/// Stores MongoDB connection string in Keychain using keyring
#[command]
async fn store_mongo_credentials(
    connection_string: String,
) -> Result<(), String> {
    let entry = Entry::new(KEYCHAIN_SERVICE_MONGO, KEYCHAIN_ACCOUNT_MONGO)
        .map_err(|e| format!("Failed to create Keychain entry for Mongo: {}", e))?;
    entry.set_password(&connection_string)
        .map_err(|e| format!("Failed to set password in Keychain for Mongo: {}", e))?;
    Ok(())
}

/// Stores R2 credentials in Keychain using keyring
#[command]
async fn store_r2_credentials(
    account_id: String,
    access_key_id: String,
    secret_access_key: String,
) -> Result<(), String> {
    let credentials = R2Credentials {
        account_id,
        access_key_id,
        secret_access_key,
    };
    let json_credentials = serde_json::to_string(&credentials)
        .map_err(|e| format!("Failed to serialize R2 credentials: {}", e))?;

    let entry = Entry::new(KEYCHAIN_SERVICE_R2, KEYCHAIN_ACCOUNT_R2)
        .map_err(|e| format!("Failed to create Keychain entry for R2: {}", e))?;
    entry.set_password(&json_credentials)
        .map_err(|e| format!("Failed to set password in Keychain for R2: {}", e))?;

    Ok(())
}

/// Retrieves MongoDB connection string from Keychain using keyring
#[command]
async fn get_mongo_credentials() -> Result<String, String> {
     let entry = Entry::new(KEYCHAIN_SERVICE_MONGO, KEYCHAIN_ACCOUNT_MONGO)
        .map_err(|e| format!("Failed to create Keychain entry for Mongo retrieval: {}", e))?;
    match entry.get_password() {
        Ok(password) => Ok(password),
        Err(keyring::Error::NoEntry) => Err("MongoDB credentials not found in Keychain.".to_string()),
        Err(e) => Err(format!("Failed to get password from Keychain for Mongo: {}", e)),
    }
}

/// Retrieves R2 credentials from Keychain using keyring
#[command]
async fn get_r2_credentials() -> Result<R2Credentials, String> {
    let entry = Entry::new(KEYCHAIN_SERVICE_R2, KEYCHAIN_ACCOUNT_R2)
        .map_err(|e| format!("Failed to create Keychain entry for R2 retrieval: {}", e))?;
    match entry.get_password() {
        Ok(json_credentials) => {
            serde_json::from_str(&json_credentials)
                .map_err(|e| format!("Failed to deserialize R2 credentials from Keychain: {}", e))
        }
        Err(keyring::Error::NoEntry) => Err("R2 credentials not found in Keychain.".to_string()),
        Err(e) => Err(format!("Failed to get password from Keychain for R2: {}", e)),
    }
}

/// Checks if credentials exist in Keychain using keyring
#[command]
async fn has_credentials(credential_type: String) -> Result<bool, String> {
    let (service, account) = match credential_type.as_str() {
         "mongo" => (KEYCHAIN_SERVICE_MONGO, KEYCHAIN_ACCOUNT_MONGO),
         "r2" => (KEYCHAIN_SERVICE_R2, KEYCHAIN_ACCOUNT_R2),
         _ => return Err("Invalid credential type".to_string()),
     };

     let entry = Entry::new(service, account)
         .map_err(|e| format!("Failed to create Keychain entry for checking {}: {}", credential_type, e))?;

     match entry.get_password() {
         Ok(_) => Ok(true), // Found entry
         Err(keyring::Error::NoEntry) => Ok(false), // No entry found
         Err(e) => Err(format!("Failed to check Keychain for {}: {}", credential_type, e)),
     }
}

/// Deletes credentials from Keychain using keyring
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
         Ok(_) => Ok(()), // Successfully deleted
         Err(keyring::Error::NoEntry) => Ok(()), // Already deleted, consider it success
         Err(e) => Err(format!("Failed to delete password from Keychain for {}: {}", credential_type, e)),
     }
}

/// Initializes MongoDB client using stored credentials
#[command]
async fn init_mongo_client(
    mongo_state: State<'_, MongoState>,
) -> Result<bool, String> {
    let credentials_result = get_mongo_credentials().await;
    if let Err(e) = credentials_result {
        return Err(format!("Failed to get MongoDB credentials: {}", e));
    }
    
    let connection_string = credentials_result.unwrap();
    info!("Initializing MongoDB client with connection string: {}", connection_string);
    
    match mongodb::Client::with_uri_str(&connection_string).await {
        Ok(client) => {
            // Test connection
            match client.database("admin").run_command(mongodb::bson::doc! {"ping": 1}, None).await {
                Ok(_) => {
                    let mut lock = mongo_state.client.lock().await;
                    *lock = Some(client);
                    Ok(true)
                },
                Err(e) => Err(format!("MongoDB connection test failed: {}", e)),
            }
        },
        Err(e) => Err(format!("Failed to initialize MongoDB client: {}", e)),
    }
}

/// Initializes R2 client using stored credentials
#[command]
async fn init_r2_client(
    r2_state: State<'_, R2State>,
) -> Result<bool, String> {
    let credentials_result = get_r2_credentials().await;
    if let Err(e) = credentials_result {
        return Err(format!("Failed to get R2 credentials: {}", e));
    }
    
    let credentials = credentials_result.unwrap();
    
    let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .region(aws_sdk_s3::config::Region::new("auto"))
        .endpoint_url(format!(
            "https://{}.r2.cloudflarestorage.com",
            credentials.account_id
        ))
        .credentials_provider(
            aws_sdk_s3::config::Credentials::new(
                credentials.access_key_id,
                credentials.secret_access_key,
                None, None, "r2-credentials"
            )
        )
        .load()
        .await;
    
    let client = aws_sdk_s3::Client::new(&config);
    
    // Test the connection
    let test_result = client.list_buckets().send().await;
    match test_result {
        Ok(_) => {
            let mut lock = r2_state.client.lock().await;
            *lock = Some(client);
            Ok(true)
        },
        Err(e) => Err(format!("R2 connection test failed: {}", e)),
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
            match client.list_buckets().send().await {
                Ok(_) => Ok(true),
                Err(e) => Err(format!("R2 connection test failed: {}", e)),
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
                let path_strings = paths
                    .iter()
                    .map(|p| p.as_path()
                          .map(|path| path.to_string_lossy().to_string())
                          .unwrap_or_default())
                    .collect::<Vec<String>>();
                
                let _ = tx.lock().unwrap().send(Ok(path_strings));
            } else {
                // User cancelled the dialog
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

/// Initialize the Tauri application
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default().build())
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
