// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{command, State};
use tokio::sync::Mutex;
use anyhow::Result;
use log::info;

/// MongoDB client state
pub struct MongoState {
    client: Mutex<Option<mongodb::Client>>,
}

/// R2 client state
pub struct R2State {
    client: Mutex<Option<aws_sdk_s3::Client>>,
}

/// MongoDB credentials structure
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MongoCredentials {
    pub username: String,
    pub password: String,
    pub hostname: String,
    pub port: u16,
}

/// R2 credentials structure
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct R2Credentials {
    pub account_id: String,
    pub access_key_id: String,
    pub secret_access_key: String,
}

/// Combined credentials structure
#[derive(Serialize, Deserialize, Debug, Default)]
struct CombinedCredentials {
    #[serde(skip_serializing_if = "Option::is_none")]
    mongodb_connection_string: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    r2: Option<R2Credentials>,
}

/// Gets the base directory for storing credentials
fn get_credentials_dir() -> Result<PathBuf> {
    let mut dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
    dir.push("music-library-manager");
    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }
    Ok(dir)
}

/// Gets the path for MongoDB credentials - MODIFIED FOR TEMP TESTING
fn get_mongo_credentials_path() -> Result<PathBuf> {
    // Point to the R2 file temporarily
    get_r2_credentials_path() 
}

/// Gets the path for R2 credentials
fn get_r2_credentials_path() -> Result<PathBuf> {
    let mut path = get_credentials_dir()?;
    path.push("r2_credentials.json");
    Ok(path)
}

/// Stores MongoDB credentials - MODIFIED FOR TEMP TESTING (Connection String)
#[command]
async fn store_mongo_credentials(
    connection_string: String,
) -> Result<(), String> {
    let path = get_mongo_credentials_path().map_err(|e| e.to_string())?;
    
    // Read existing credentials or create default
    let mut combined_creds: CombinedCredentials = if path.exists() {
        let json = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        // Handle empty file case
        if json.trim().is_empty() {
            CombinedCredentials::default()
        } else {
            serde_json::from_str(&json).map_err(|e| format!("Failed to parse existing credentials: {}", e))?
        }
    } else {
        CombinedCredentials::default()
    };

    // Update the mongo connection string part
    combined_creds.mongodb_connection_string = Some(connection_string);
    
    // Write back the combined structure
    let json = serde_json::to_string_pretty(&combined_creds).map_err(|e| e.to_string())?;
    // Ensure directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(path, json).map_err(|e| e.to_string())?;
    
    Ok(())
}

/// Stores R2 credentials - MODIFIED FOR TEMP TESTING
#[command]
async fn store_r2_credentials(
    account_id: String,
    access_key_id: String,
    secret_access_key: String,
) -> Result<(), String> {
    let new_credentials = R2Credentials {
        account_id,
        access_key_id,
        secret_access_key,
    };
    
    let path = get_r2_credentials_path().map_err(|e| e.to_string())?; // Use R2 path getter directly
    
     // Read existing credentials or create default
     let mut combined_creds: CombinedCredentials = if path.exists() {
        let json = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        // Handle empty file case
        if json.trim().is_empty() {
             CombinedCredentials::default()
        } else {
            serde_json::from_str(&json).map_err(|e| format!("Failed to parse existing credentials: {}", e))?
        }
    } else {
        CombinedCredentials::default()
    };

    // Update the R2 part
    combined_creds.r2 = Some(new_credentials);
    
    // Write back the combined structure
    let json = serde_json::to_string_pretty(&combined_creds).map_err(|e| e.to_string())?;
     // Ensure directory exists
     if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(path, json).map_err(|e| e.to_string())?;
    
    Ok(())
}

/// Retrieves MongoDB credentials - MODIFIED FOR TEMP TESTING (Connection String)
#[command]
async fn get_mongo_credentials() -> Result<String, String> {
    let path = get_mongo_credentials_path().map_err(|e| e.to_string())?;
    if !path.exists() {
        return Err("Credentials file not found".to_string());
    }
    
    let json = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    // Handle empty file
    if json.trim().is_empty() {
        return Err("Credentials file is empty".to_string());
    }

    let combined_creds: CombinedCredentials = serde_json::from_str(&json)
        .map_err(|e| format!("Failed to parse credentials file: {}", e))?;
    
    combined_creds.mongodb_connection_string
        .ok_or_else(|| "MongoDB connection string not found in credentials file".to_string())
}

/// Retrieves R2 credentials
#[command]
async fn get_r2_credentials() -> Result<R2Credentials, String> {
    let path = get_r2_credentials_path().map_err(|e| e.to_string())?;
    if !path.exists() {
        return Err("R2 credentials not found".to_string());
    }
    
    let json = fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&json).map_err(|e| e.to_string())
}

/// Checks if credentials exist
#[command]
async fn has_credentials(credential_type: String) -> Result<bool, String> {
    let path = match credential_type.as_str() {
        "mongo" => get_mongo_credentials_path().map_err(|e| e.to_string())?,
        "r2" => get_r2_credentials_path().map_err(|e| e.to_string())?,
        _ => return Err("Invalid credential type".to_string()),
    };
    
    Ok(path.exists())
}

/// Deletes credentials
#[command]
async fn delete_credentials(credential_type: String) -> Result<(), String> {
    let path = match credential_type.as_str() {
        "mongo" => get_mongo_credentials_path().map_err(|e| e.to_string())?,
        "r2" => get_r2_credentials_path().map_err(|e| e.to_string())?,
        _ => return Err("Invalid credential type".to_string()),
    };
    
    if path.exists() {
        fs::remove_file(path).map_err(|e| e.to_string())?;
    }
    
    Ok(())
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

/// Initialize the Tauri application
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default().build())
        .plugin(tauri_plugin_fs::init())
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
