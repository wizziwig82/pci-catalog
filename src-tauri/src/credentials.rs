use serde::{Deserialize, Serialize};
use tauri::{command, Runtime};

// MongoDB credentials structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MongoCredentials {
    pub uri: String,
    pub database: String,
}

// R2 credentials structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct R2Credentials {
    pub bucket_name: String,
    pub access_key: String,
    pub secret_key: String,
    pub endpoint: String,
}

// The generic result type for credential operations
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CredentialStoreResult {
    pub success: bool,
    pub message: Option<String>,
}

// Store MongoDB credentials in secure storage
#[command]
pub async fn legacy_store_mongo_credentials<R: Runtime>(app: tauri::AppHandle<R>, credentials: MongoCredentials) -> Result<(), String> {
    // This function is now deprecated, keeping for reference only
    Ok(())
}

// Retrieve MongoDB credentials from secure storage
#[command]
pub async fn legacy_get_mongo_credentials<R: Runtime>(app: tauri::AppHandle<R>) -> Result<MongoCredentials, String> {
    // This function is now deprecated, keeping for reference only
    Err("Deprecated storage method".to_string())
}

// Delete MongoDB credentials from secure storage
#[command]
pub async fn legacy_delete_mongo_credentials<R: Runtime>(app: tauri::AppHandle<R>) -> Result<(), String> {
    // This function is now deprecated, keeping for reference only
    Ok(())
}

// Store R2 credentials in secure storage
#[command]
pub async fn legacy_store_r2_credentials<R: Runtime>(app: tauri::AppHandle<R>, credentials: R2Credentials) -> Result<(), String> {
    // This function is now deprecated, keeping for reference only
    Ok(())
}

// Retrieve R2 credentials from secure storage
#[command]
pub async fn legacy_get_r2_credentials<R: Runtime>(app: tauri::AppHandle<R>) -> Result<R2Credentials, String> {
    // This function is now deprecated, keeping for reference only
    Err("Deprecated storage method".to_string())
}

// Delete R2 credentials from secure storage
#[command]
pub async fn legacy_delete_r2_credentials<R: Runtime>(app: tauri::AppHandle<R>) -> Result<(), String> {
    // This function is now deprecated, keeping for reference only
    Ok(())
}

// Check if credentials exist in secure storage
#[command]
pub async fn legacy_has_credentials<R: Runtime>(app: tauri::AppHandle<R>, credential_type: String) -> Result<bool, String> {
    // This function is now deprecated, keeping for reference only
    Ok(false)
} 