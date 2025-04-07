//! Handles storage and retrieval of credentials using the system keychain
//! with a fallback to a local file for development environments.

use serde::{Deserialize, Serialize};
use log::{info, error, warn};
use std::fs;
use std::path::{Path, PathBuf};
use std::env;
use keyring::Entry;
use tauri::command;
use anyhow::{self, Result};
use std::fmt;
use thiserror::Error;
use std::error::Error as StdError;

// Define a custom error type for credentials operations
#[derive(Debug)]
pub enum CredentialsError {
    Validation(String),
    FileSystem(String),
    Database(String),
    Storage(String),
    Configuration(String),
    NotFound(String),
    Unexpected(String),
    Keychain(String),
}

impl fmt::Display for CredentialsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CredentialsError::Validation(s) => write!(f, "Validation error: {}", s),
            CredentialsError::FileSystem(s) => write!(f, "File system error: {}", s),
            CredentialsError::Database(s) => write!(f, "Database error: {}", s),
            CredentialsError::Storage(s) => write!(f, "Storage error: {}", s),
            CredentialsError::Configuration(s) => write!(f, "Configuration error: {}", s),
            CredentialsError::NotFound(s) => write!(f, "Not found: {}", s),
            CredentialsError::Unexpected(s) => write!(f, "Unexpected error: {}", s),
            CredentialsError::Keychain(s) => write!(f, "Keychain error: {}", s),
        }
    }
}

impl StdError for CredentialsError {}

// Convert keyring errors to our CredentialsError type
impl From<keyring::Error> for CredentialsError {
    fn from(err: keyring::Error) -> Self {
        CredentialsError::Unexpected(format!("Keychain error: {}", err))
    }
}

// Note: We're not implementing From<CredentialsError> for CommandError
// because we're using manual conversion in the proxy functions in main.rs

// Use relative path to import from the lib.rs root

// --- Constants ---

const KEYCHAIN_SERVICE_MONGO: &str = "com.musiclibrarymanager.mongo";
const KEYCHAIN_ACCOUNT_MONGO: &str = "mongo_credentials";
const KEYCHAIN_SERVICE_R2: &str = "com.musiclibrarymanager.r2";
const KEYCHAIN_ACCOUNT_R2: &str = "r2_credentials";

// Dev-mode fallback config file path for credentials (only used if keychain fails)
#[cfg(debug_assertions)]
const DEV_CREDENTIALS_FILE: &str = "dev_credentials.json";

// --- Data Structures ---

/// MongoDB credentials structure (placeholder, only connection string is used for storage)
/// Kept here as it's related to credential *storage concept*, even if only string is stored.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MongoCredentials {
    pub username: String,
    pub password: String,
    pub hostname: String,
    pub port: u16,
    // Note: The actual stored value is the connection string.
    // This struct might be used elsewhere for parsing/validation if needed.
}

/// R2 credentials structure
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct R2Credentials {
    pub account_id: String,
    pub bucket_name: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub endpoint: String,
}

/// Development credentials storage structure (used only in debug builds as fallback)
#[cfg(debug_assertions)]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct DevCredentials {
    mongo_connection_string: Option<String>,
    r2_credentials: Option<R2Credentials>,
}

// --- Development Fallback Helpers (Debug Only) ---

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
async fn save_dev_credentials(creds: &DevCredentials) -> Result<(), CredentialsError> {
    let creds_json = serde_json::to_string_pretty(creds)
        .map_err(|e| CredentialsError::Unexpected(format!("Failed to serialize dev credentials: {}", e)))?;
    
    std::fs::write(DEV_CREDENTIALS_FILE, creds_json)
        .map_err(|e| CredentialsError::FileSystem(format!("Failed to write dev credentials file: {}", e)))
}

// --- Tauri Commands ---

/// Stores R2 credentials in Keychain using keyring
#[command]
pub async fn store_r2_credentials(
    account_id: String,
    bucket_name: String,
    access_key_id: String,
    secret_access_key: String,
    endpoint: String,
) -> Result<bool, CredentialsError> {
    info!("Storing R2 credentials in keychain");
    let creds = R2Credentials { account_id, bucket_name, access_key_id, secret_access_key, endpoint };
    let entry_result = Entry::new(KEYCHAIN_SERVICE_R2, KEYCHAIN_ACCOUNT_R2);

    let entry = match entry_result {
        Ok(entry) => entry,
        Err(keyring_error) => {
            error!("Failed to create keyring entry for R2 credentials: {}", keyring_error);
            #[cfg(debug_assertions)] {
                info!("Using development fallback for storing R2 credentials");
                let mut dev_creds = load_dev_credentials().await;
                dev_creds.r2_credentials = Some(creds);
                return save_dev_credentials(&dev_creds).await.map(|_| true);
            }
            #[cfg(not(debug_assertions))] return Err(keyring_error.into());
        }
    };

    let json_str = serde_json::to_string(&creds)
        .map_err(|e| CredentialsError::Unexpected(format!("Failed to serialize R2 credentials: {}", e)))?;
    let _ = entry.delete_credential(); // Attempt to delete existing before setting

    match entry.set_password(&json_str) {
        Ok(_) => {
            info!("Successfully stored R2 credentials");
            #[cfg(debug_assertions)] {
                let mut dev_creds = load_dev_credentials().await;
                dev_creds.r2_credentials = Some(creds);
                let _ = save_dev_credentials(&dev_creds).await; // Save to dev file as well
            }
            Ok(true)
        },
        Err(keyring_error) => {
            error!("Failed to store R2 credentials in keychain: {}", keyring_error);
            #[cfg(debug_assertions)] {
                info!("Using development fallback for storing R2 credentials after keychain failure");
                let mut dev_creds = load_dev_credentials().await;
                dev_creds.r2_credentials = Some(creds);
                return save_dev_credentials(&dev_creds).await.map(|_| true);
            }
            #[cfg(not(debug_assertions))] Err(keyring_error.into())
        }
    }
}

/// Retrieves R2 credentials from Keychain using keyring
#[command]
pub async fn get_r2_credentials() -> Result<R2Credentials, CredentialsError> {
    info!("Retrieving R2 credentials from keychain");
    let entry_result = Entry::new(KEYCHAIN_SERVICE_R2, KEYCHAIN_ACCOUNT_R2);

    let entry = match entry_result {
        Ok(entry) => entry,
        Err(keyring_error) => {
            error!("Failed to create keyring entry for R2 credentials: {}", keyring_error);
            #[cfg(debug_assertions)] {
                info!("Using development fallback for retrieving R2 credentials");
                let dev_creds = load_dev_credentials().await;
                if let Some(creds) = dev_creds.r2_credentials { return Ok(creds); }
            }
            return Err(keyring_error.into());
        }
    };

    match entry.get_password() {
        Ok(json_str) => {
            if json_str.is_empty() {
                // Keychain entry exists but is empty, treat as not found
                 info!("R2 credentials entry found but empty in keychain");
                 #[cfg(debug_assertions)] {
                     info!("Using development fallback for retrieving R2 credentials");
                     let dev_creds = load_dev_credentials().await;
                     if let Some(creds) = dev_creds.r2_credentials { return Ok(creds); }
                 }
                return Err(CredentialsError::NotFound("R2 credentials not set".to_string()));
            }
            serde_json::from_str::<R2Credentials>(&json_str)
                .map_err(|e| CredentialsError::Unexpected(format!("Failed to parse R2 credentials: {}", e)))
        },
        Err(keyring_error) => {
            if matches!(keyring_error, keyring::Error::NoEntry) {
                info!("R2 credentials not found in keychain");
                #[cfg(debug_assertions)] {
                    info!("Using development fallback for retrieving R2 credentials");
                    let dev_creds = load_dev_credentials().await;
                    if let Some(creds) = dev_creds.r2_credentials { return Ok(creds); }
                }
                Err(CredentialsError::NotFound("R2 credentials not found".to_string()))
            } else {
                error!("Failed to get R2 credentials from keychain: {}", keyring_error);
                #[cfg(debug_assertions)] {
                    info!("Using development fallback for retrieving R2 credentials after keychain error");
                    let dev_creds = load_dev_credentials().await;
                    if let Some(creds) = dev_creds.r2_credentials { return Ok(creds); }
                }
                Err(keyring_error.into())
            }
        }
    }
}

/// Stores MongoDB connection string in Keychain using keyring
#[command]
pub async fn store_mongo_credentials(connection_string: String) -> Result<bool, CredentialsError> {
    info!("Storing MongoDB credentials (connection string) in keychain");
    let entry_result = Entry::new(KEYCHAIN_SERVICE_MONGO, KEYCHAIN_ACCOUNT_MONGO);

    let entry = match entry_result {
        Ok(entry) => entry,
        Err(keyring_error) => {
            error!("Failed to create keyring entry for MongoDB credentials: {}", keyring_error);
            #[cfg(debug_assertions)] {
                info!("Using development fallback for storing MongoDB credentials");
                let mut dev_creds = load_dev_credentials().await;
                dev_creds.mongo_connection_string = Some(connection_string.clone());
                 return save_dev_credentials(&dev_creds).await.map(|_| true);
            }
             #[cfg(not(debug_assertions))] return Err(keyring_error.into());
        }
    };

    let _ = entry.delete_credential(); // Attempt to delete existing before setting

    match entry.set_password(&connection_string) {
        Ok(_) => {
            info!("Successfully stored MongoDB credentials");
            #[cfg(debug_assertions)] {
                let mut dev_creds = load_dev_credentials().await;
                dev_creds.mongo_connection_string = Some(connection_string);
                let _ = save_dev_credentials(&dev_creds).await; // Save to dev file as well
            }
            Ok(true)
        },
        Err(keyring_error) => {
            error!("Failed to store MongoDB credentials in keychain: {}", keyring_error);
            #[cfg(debug_assertions)] {
                info!("Using development fallback for storing MongoDB credentials after keychain failure");
                let mut dev_creds = load_dev_credentials().await;
                dev_creds.mongo_connection_string = Some(connection_string);
                 return save_dev_credentials(&dev_creds).await.map(|_| true);
            }
             #[cfg(not(debug_assertions))] Err(keyring_error.into())
        }
    }
}

/// Retrieves MongoDB connection string from Keychain using keyring
#[command]
pub async fn get_mongo_credentials() -> Result<String, CredentialsError> {
    info!("Retrieving MongoDB credentials (connection string) from keychain");
    let entry_result = Entry::new(KEYCHAIN_SERVICE_MONGO, KEYCHAIN_ACCOUNT_MONGO);

    let entry = match entry_result {
         Ok(entry) => entry,
         Err(keyring_error) => {
             error!("Failed to create keyring entry for MongoDB credentials: {}", keyring_error);
             #[cfg(debug_assertions)] {
                 info!("Using development fallback for retrieving MongoDB credentials");
                 let dev_creds = load_dev_credentials().await;
                 if let Some(conn_string) = dev_creds.mongo_connection_string { return Ok(conn_string); }
             }
             return Err(keyring_error.into());
         }
     };

    match entry.get_password() {
        Ok(connection_string) => {
            if connection_string.is_empty() {
                 info!("MongoDB credentials entry found but empty in keychain");
                 #[cfg(debug_assertions)] {
                     info!("Using development fallback for retrieving MongoDB credentials");
                     let dev_creds = load_dev_credentials().await;
                     if let Some(conn_string) = dev_creds.mongo_connection_string { return Ok(conn_string); }
                 }
                Err(CredentialsError::NotFound("MongoDB connection string not set".to_string()))
            } else {
                info!("Successfully retrieved MongoDB credentials");
                Ok(connection_string)
            }
        },
        Err(keyring_error) => {
            if matches!(keyring_error, keyring::Error::NoEntry) {
                info!("MongoDB credentials not found in keychain");
                #[cfg(debug_assertions)] {
                    info!("Using development fallback for retrieving MongoDB credentials");
                    let dev_creds = load_dev_credentials().await;
                    if let Some(conn_string) = dev_creds.mongo_connection_string { return Ok(conn_string); }
                }
                Err(CredentialsError::NotFound("MongoDB credentials not found".to_string()))
            } else {
                error!("Failed to get MongoDB credentials from keychain: {}", keyring_error);
                #[cfg(debug_assertions)] {
                    info!("Using development fallback for retrieving MongoDB credentials after keychain error");
                    let dev_creds = load_dev_credentials().await;
                    if let Some(conn_string) = dev_creds.mongo_connection_string { return Ok(conn_string); }
                }
                Err(keyring_error.into())
            }
        }
    }
}

/// Check if credentials exist in the keychain
#[command]
pub async fn has_credentials(credential_type: String) -> Result<bool, CredentialsError> {
    let (service, account) = match credential_type.as_str() {
         "mongo" => (KEYCHAIN_SERVICE_MONGO, KEYCHAIN_ACCOUNT_MONGO),
         "r2" => (KEYCHAIN_SERVICE_R2, KEYCHAIN_ACCOUNT_R2),
         _ => return Err(CredentialsError::Validation("Invalid credential type provided".to_string())),
     };

     let entry = Entry::new(service, account)
        .map_err(|e| CredentialsError::Keychain(format!("Failed to create Keychain entry for checking {}: {}", credential_type, e)))?;

     match entry.get_password() {
         Ok(pass) => Ok(!pass.is_empty()), // True if entry exists and is not empty
         Err(keyring_error) => {
             if matches!(keyring_error, keyring::Error::NoEntry) {
                  info!("{} credentials not found in keychain", credential_type);
                 Ok(false) // Not found is not an error in this context
             } else {
                 error!("Failed to check if {} credentials exist: {}", credential_type, keyring_error);
                 Err(keyring_error.into()) // Other keychain errors are propagated
             }
         }
     }
}

/// Delete credentials from the keychain
#[command]
pub async fn delete_credentials(credential_type: String) -> Result<(), CredentialsError> {
     let (service, account) = match credential_type.as_str() {
         "mongo" => (KEYCHAIN_SERVICE_MONGO, KEYCHAIN_ACCOUNT_MONGO),
         "r2" => (KEYCHAIN_SERVICE_R2, KEYCHAIN_ACCOUNT_R2),
         _ => return Err(CredentialsError::Validation("Invalid credential type provided".to_string())),
     };

     let entry = Entry::new(service, account)
         .map_err(|e| CredentialsError::Keychain(format!("Failed to create Keychain entry for deleting {}: {}", credential_type, e)))?;

     match entry.delete_credential() {
         Ok(_) => {
             info!("Successfully deleted {} credentials", credential_type);
             #[cfg(debug_assertions)] {
                 // Also clear from dev file if it exists
                 if credential_type == "mongo" {
                     let mut dev_creds = load_dev_credentials().await;
                     if dev_creds.mongo_connection_string.is_some() {
                         dev_creds.mongo_connection_string = None;
                         let _ = save_dev_credentials(&dev_creds).await;
                     }
                 } else if credential_type == "r2" {
                      let mut dev_creds = load_dev_credentials().await;
                     if dev_creds.r2_credentials.is_some() {
                         dev_creds.r2_credentials = None;
                         let _ = save_dev_credentials(&dev_creds).await;
                     }
                 }
             }
             Ok(())
         },
         Err(keyring_error) => {
             if matches!(keyring_error, keyring::Error::NoEntry) {
                 info!("{} credentials not found during deletion attempt, considered successful.", credential_type);
                 Ok(()) // Not finding it is okay for deletion
             } else {
                 error!("Failed to delete {} credentials: {}", credential_type, keyring_error);
                 Err(keyring_error.into())
             }
         }
     }
}
