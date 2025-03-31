use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::fs;
use std::collections::HashMap;
use uuid::Uuid;
use log::{info, error, warn};
use tauri::{command, State};
use tokio::sync::Mutex;
use std::string::ToString;
use std::time::Duration;
use tauri::async_runtime;

use crate::audio::{TrackMetadata, AlbumMetadata, AudioMetadata};
use crate::audio::transcoding::{TranscodingOptions, TranscodingResult};
use crate::storage::r2::{R2Client, R2UploadResult};
use crate::storage::mongodb::{MongoClient as MongoClientImpl, Album, Track, DbResponse};

/// Response for bulk upload operation
#[derive(Debug, Serialize, Deserialize)]
pub struct BulkUploadResponse {
    pub success: bool,
    pub message: Option<String>,
    pub uploaded_tracks: Vec<UploadedTrackInfo>,
    pub failed_tracks: Vec<FailedTrackInfo>,
}

/// Information about a successfully uploaded track
#[derive(Debug, Serialize, Deserialize)]
pub struct UploadedTrackInfo {
    pub track_id: String,
    pub title: String,
    pub album_name: String,
    pub original_path: String,
    pub r2_path: String,
    pub medium_quality_path: Option<String>,
}

/// Information about a failed track upload
#[derive(Debug, Serialize, Deserialize)]
pub struct FailedTrackInfo {
    pub original_path: String,
    pub error: String,
}

/// Upload paths configuration for different quality levels
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UploadPathConfig {
    pub original_prefix: String,
    pub medium_prefix: String,
}

impl Default for UploadPathConfig {
    fn default() -> Self {
        Self {
            original_prefix: "tracks/original".to_string(),
            medium_prefix: "tracks/medium".to_string(),
        }
    }
}

/// Upload a transcoded file to R2
async fn upload_track_to_r2(
    r2_client: &aws_sdk_s3::Client,
    file_path: &str,
    r2_path: &str,
    mime_type: &str,
) -> Result<R2UploadResult, String> {
    info!("Uploading file {} to R2 at path {}", file_path, r2_path);
    
    // Read the file content
    let file_data = match fs::read(file_path) {
        Ok(data) => data,
        Err(e) => return Err(format!("Failed to read file {}: {}", file_path, e)),
    };
    
    // Upload the file to R2
    let bucket_name = "music-library-manager";
    let result = r2_client
        .put_object()
        .bucket(bucket_name)
        .key(r2_path)
        .body(file_data.into())
        .content_type(mime_type)
        .send()
        .await;
    
    match result {
        Ok(_) => Ok(R2UploadResult {
            success: true,
            path: Some(format!("{}/{}", bucket_name, r2_path)),
            error: None,
        }),
        Err(e) => Ok(R2UploadResult {
            success: false,
            path: None,
            error: Some(format!("Failed to upload file: {}", e)),
        }),
    }
}

/// Store album and track metadata in MongoDB
async fn store_metadata_in_mongodb(
    mongo_client: &mongodb::Client,
    audio_metadata: &AudioMetadata,
    r2_paths: HashMap<String, String>,
) -> Result<(String, String), String> {
    // Create or update album
    let album_id = audio_metadata.album.album_id.clone();
    let track_id = audio_metadata.track.track_id.clone();
    
    // Create album document
    let album = Album {
        name: audio_metadata.album.name.clone(),
        track_ids: vec![track_id.clone()],
        art_path: audio_metadata.album.art_path.clone(),
        release_date: None,
        publisher: None,
    };
    
    // Create track document
    let track = Track {
        title: audio_metadata.track.title.clone(),
        album_id: album_id.clone(),
        track_number: audio_metadata.track.track_number.map(|n| n as i32),
        filename: audio_metadata.track.filename.clone(),
        duration: audio_metadata.track.duration.map_or(0, |d| d as i32),
        writers: audio_metadata.track.writers.keys().cloned().collect(),
        publishers: audio_metadata.track.publishers.keys().cloned().collect(),
        composers: None,
        genre: audio_metadata.track.genre.first().cloned(),
        path: r2_paths.get("original").unwrap_or(&String::new()).clone(),
        waveform_data: None,
    };
    
    // MongoDB collections
    let albums_collection = mongo_client.database("music_library").collection("albums");
    let tracks_collection = mongo_client.database("music_library").collection("tracks");
    
    // Check if album exists
    let filter = mongodb::bson::doc! { "_id": &album_id };
    let album_exists = albums_collection.count_documents(filter.clone(), None).await
        .map_err(|e| format!("Failed to check if album exists: {}", e))?;
    
    if album_exists > 0 {
        // Album exists, update it with the new track
        info!("Album {} already exists, updating with new track", album_id);
        
        // Get existing album data
        let existing_album_doc = albums_collection.find_one(filter.clone(), None).await
            .map_err(|e| format!("Failed to find album: {}", e))?
            .ok_or_else(|| format!("Album {} not found", album_id))?;
        
        // Convert BSON document to Album struct
        let mut existing_album: Album = mongodb::bson::from_document(existing_album_doc)
            .map_err(|e| format!("Failed to deserialize album: {}", e))?;
        
        // Add track to album if not already present
        if !existing_album.track_ids.contains(&track_id) {
            existing_album.track_ids.push(track_id.clone());
        }
        
        // Update album
        let update = mongodb::bson::doc! { "$set": mongodb::bson::to_document(&existing_album)
            .map_err(|e| format!("Failed to serialize album: {}", e))? };
        
        albums_collection.update_one(filter, update, None).await
            .map_err(|e| format!("Failed to update album: {}", e))?;
    } else {
        // Album doesn't exist, create it
        info!("Creating new album: {}", album_id);
        
        let album_doc = mongodb::bson::to_document(&album)
            .map_err(|e| format!("Failed to serialize album: {}", e))?;
        
        let mut album_doc_with_id = album_doc;
        album_doc_with_id.insert("_id", album_id.clone());
        
        albums_collection.insert_one(album_doc_with_id, None).await
            .map_err(|e| format!("Failed to create album: {}", e))?;
    }
    
    // Create track
    info!("Creating track: {}", track_id);
    
    let track_doc = mongodb::bson::to_document(&track)
        .map_err(|e| format!("Failed to serialize track: {}", e))?;
    
    let mut track_doc_with_id = track_doc;
    track_doc_with_id.insert("_id", track_id.clone());
    
    tracks_collection.insert_one(track_doc_with_id, None).await
        .map_err(|e| format!("Failed to create track: {}", e))?;
    
    Ok((album_id, track_id))
}

/// Process a single transcoded track for upload
async fn process_track_upload(
    r2_client: &aws_sdk_s3::Client,
    mongo_client: &mongodb::Client,
    transcoding_result: &TranscodingResult,
    audio_metadata: &AudioMetadata,
    path_config: &UploadPathConfig,
) -> Result<UploadedTrackInfo, String> {
    let file_name = Path::new(&transcoding_result.input_path)
        .file_name()
        .ok_or("Invalid file path")?
        .to_string_lossy()
        .to_string();
    
    let album_name = audio_metadata.album.name.clone();
    
    // Sanitize file name for R2 storage
    let sanitized_file_name = file_name.replace(" ", "_");
    
    // Create R2 paths
    let original_r2_path = format!("{}/{}", path_config.original_prefix, sanitized_file_name);
    
    // Initialize hashmap to store paths
    let mut r2_paths = HashMap::new();
    
    // Upload original file
    let original_mime_type = audio_metadata.track.mime_type.clone();
    let original_upload_result = upload_track_to_r2(
        r2_client,
        &transcoding_result.input_path,
        &original_r2_path,
        &original_mime_type,
    ).await?;
    
    if !original_upload_result.success {
        return Err(format!("Failed to upload original file: {}", 
            original_upload_result.error.unwrap_or_else(|| "Unknown error".to_string())));
    }
    
    // Store original path
    r2_paths.insert("original".to_string(), original_r2_path.clone());
    
    // Upload medium quality file if available
    let medium_r2_path = if let Some(medium_path) = &transcoding_result.medium_quality_path {
        let medium_file_name = Path::new(medium_path)
            .file_name()
            .ok_or("Invalid medium quality file path")?
            .to_string_lossy()
            .to_string();
        
        let sanitized_medium_file_name = medium_file_name.replace(" ", "_");
        let medium_r2_path = format!("{}/{}", path_config.medium_prefix, sanitized_medium_file_name);
        
        // Determine mime type based on file extension
        let medium_mime_type = mime_guess::from_path(medium_path)
            .first_or_octet_stream()
            .to_string();
        
        let medium_upload_result = upload_track_to_r2(
            r2_client,
            medium_path,
            &medium_r2_path,
            &medium_mime_type,
        ).await?;
        
        if !medium_upload_result.success {
            warn!("Failed to upload medium quality file: {}", 
                medium_upload_result.error.unwrap_or_else(|| "Unknown error".to_string()));
            // Continue even if medium quality upload fails
            None
        } else {
            // Store medium path
            r2_paths.insert("medium".to_string(), medium_r2_path.clone());
            Some(medium_r2_path)
        }
    } else {
        None
    };
    
    // Store metadata in MongoDB
    let (album_id, track_id) = store_metadata_in_mongodb(
        mongo_client,
        audio_metadata,
        r2_paths,
    ).await?;
    
    Ok(UploadedTrackInfo {
        track_id,
        title: audio_metadata.track.title.clone(),
        album_name,
        original_path: transcoding_result.input_path.clone(),
        r2_path: original_r2_path,
        medium_quality_path: medium_r2_path,
    })
}

/// Tauri command to upload transcoded tracks to R2 and store metadata in MongoDB
#[command]
pub async fn upload_transcoded_tracks(
    r2_state: State<'_, crate::R2State>,
    mongo_state: State<'_, crate::MongoState>,
    transcoding_results: Vec<TranscodingResult>,
    audio_metadata_list: Vec<AudioMetadata>,
    path_config: Option<UploadPathConfig>,
) -> Result<BulkUploadResponse, String> {
    // Check that we have matching transcoding results and metadata
    if transcoding_results.len() != audio_metadata_list.len() {
        return Err("Mismatch between transcoding results and metadata".to_string());
    }
    
    // Get R2 client
    let r2_client_lock = r2_state.client.lock().await;
    let r2_client = r2_client_lock.as_ref()
        .ok_or("R2 client not initialized. Please configure R2 credentials first.")?;
    
    // Get MongoDB client
    let mongo_client_lock = mongo_state.client.lock().await;
    let mongo_client = mongo_client_lock.as_ref()
        .ok_or("MongoDB client not initialized. Please configure MongoDB credentials first.")?;
    
    // Use default path config if not provided
    let path_config = path_config.unwrap_or_default();
    
    // Process each track
    let mut uploaded_tracks = Vec::new();
    let mut failed_tracks = Vec::new();
    
    for (i, transcoding_result) in transcoding_results.iter().enumerate() {
        let audio_metadata = &audio_metadata_list[i];
        
        match process_track_upload(
            r2_client,
            mongo_client,
            transcoding_result,
            audio_metadata,
            &path_config,
        ).await {
            Ok(track_info) => {
                uploaded_tracks.push(track_info);
            },
            Err(error) => {
                failed_tracks.push(FailedTrackInfo {
                    original_path: transcoding_result.input_path.clone(),
                    error,
                });
            },
        }
    }
    
    // Return response
    Ok(BulkUploadResponse {
        success: failed_tracks.is_empty(),
        message: if failed_tracks.is_empty() {
            Some(format!("Successfully uploaded {} tracks", uploaded_tracks.len()))
        } else {
            Some(format!("Uploaded {} tracks with {} failures", 
                uploaded_tracks.len(), failed_tracks.len()))
        },
        uploaded_tracks,
        failed_tracks,
    })
} 