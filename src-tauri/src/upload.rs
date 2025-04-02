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
use mime_guess;

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
    pub album_art_prefix: String,
}

impl Default for UploadPathConfig {
    fn default() -> Self {
        Self {
            original_prefix: "tracks/original".to_string(),
            medium_prefix: "tracks/medium".to_string(),
            album_art_prefix: "albums/artwork".to_string(),
        }
    }
}

/// Upload a transcoded file to R2
async fn upload_track_to_r2(
    r2_client: &aws_sdk_s3::Client,
    file_path: &str,
    r2_path: &str,
    mime_type: &str,
    bucket_name: &str,
    make_public: bool,
) -> Result<R2UploadResult, String> {
    info!("Uploading file {} to R2 at path {}", file_path, r2_path);
    
    // Read the file content
    let file_data = match fs::read(file_path) {
        Ok(data) => data,
        Err(e) => return Err(format!("Failed to read file {}: {}", file_path, e)),
    };
    
    // Create the put_object request
    let mut put_request = r2_client
        .put_object()
        .bucket(bucket_name)
        .key(r2_path)
        .body(file_data.into())
        .content_type(mime_type);
    
    // Set ACL if needed (Note: R2 might not fully support all S3 ACL options)
    // Uncomment this if your R2 setup supports ACL
    /*
    if make_public {
        put_request = put_request.acl("public-read");
    }
    */
    
    // Upload the file to R2
    let result = put_request.send().await;
    
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
    // Original album ID from metadata
    let original_album_id = audio_metadata.album.album_id.clone();
    let track_id = audio_metadata.track.track_id.clone();
    
    // MongoDB collections
    let albums_collection: mongodb::Collection<mongodb::bson::Document> = mongo_client.database("music_library").collection("albums");
    let tracks_collection: mongodb::Collection<mongodb::bson::Document> = mongo_client.database("music_library").collection("tracks");
    
    // First, check if a similar album already exists based on name and artist
    let album_name = audio_metadata.album.name.clone();
    
    // Skip album existence check if album name is "Unknown Album" as this is likely
    // a default value when metadata extraction failed
    let mut album_id = original_album_id.clone();
    if album_name != "Unknown Album" {
        let album_query = mongodb::bson::doc! { 
            "name": &album_name 
        };
        
        // Try to find existing album with the same name
        let existing_album_doc = albums_collection.find_one(album_query, None).await
            .map_err(|e| format!("Failed to search for existing album: {}", e))?;
        
        if let Some(doc) = existing_album_doc {
            // Album exists, use its ID instead of the generated one
            album_id = doc.get_str("_id")
                .map_err(|e| format!("Failed to get album ID from document: {}", e))?
                .to_string();
            
            info!("Found existing album '{}' with ID {}, using instead of generated ID", album_name, album_id);
        }
    }
    
    // Create album document
    let album = Album {
        name: album_name,
        track_ids: vec![track_id.clone()],
        art_path: audio_metadata.album.art_path.clone(),
        release_date: None,
        publisher: None,
    };
    
    // Create track document, ensuring it points to the potentially updated album_id
    let track = Track {
        title: audio_metadata.track.title.clone(),
        album_id: album_id.clone(), // Use potentially updated album_id here
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
    
    // Check if album exists by direct ID now
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
    bucket_name: &str,
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
        bucket_name,
        true, // Make original files public so they can be streamed
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
            bucket_name,
            false,
        ).await?;
        
        if !medium_upload_result.success {
            warn!("Failed to upload medium quality file: {}", 
                medium_upload_result.error.unwrap_or_else(|| "Unknown error".to_string()));
            // Continue even if medium quality upload fails but log the warning
            None
        } else {
            // Store medium path
            r2_paths.insert("medium".to_string(), medium_r2_path.clone());
            info!("Successfully uploaded medium quality file to {}", medium_r2_path);
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
    
    // Return information about the uploaded track
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
    
    // Get R2 credentials to find the bucket name
    let credentials = match crate::get_r2_credentials().await {
        Ok(creds) => creds,
        Err(e) => return Err(format!("Failed to get R2 credentials: {}", e)),
    };
    
    // Get bucket name from credentials
    let bucket_name = credentials.bucket_name;
    info!("Using R2 bucket: {}", bucket_name);
    
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
            &bucket_name,
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

/// Upload album artwork to R2
#[command]
pub async fn upload_album_artwork(
    r2_state: State<'_, crate::R2State>,
    album_id: String,
    image_path: String,
    path_config: Option<UploadPathConfig>,
) -> Result<R2UploadResult, String> {
    // Get R2 client
    let r2_client_lock = r2_state.client.lock().await;
    let r2_client = r2_client_lock.as_ref()
        .ok_or("R2 client not initialized. Please configure R2 credentials first.")?;
    
    // Get R2 credentials to find the bucket name
    let credentials = match crate::get_r2_credentials().await {
        Ok(creds) => creds,
        Err(e) => return Err(format!("Failed to get R2 credentials: {}", e)),
    };
    
    // Use default path config if not provided
    let path_config = path_config.unwrap_or_default();
    
    // Determine MIME type based on file extension
    let image_mime_type = mime_guess::from_path(&image_path)
        .first_or_octet_stream()
        .to_string();
    
    // Create artwork path in R2
    // Format is: albums/artwork/{album_id}.{ext}
    let file_extension = Path::new(&image_path)
        .extension()
        .map(|ext| ext.to_string_lossy().to_string())
        .unwrap_or_else(|| "jpg".to_string());
    
    let r2_path = format!("{}/{}.{}", 
        path_config.album_art_prefix, 
        album_id, 
        file_extension);
    
    info!("Uploading album artwork for {} to {}", album_id, r2_path);
    
    // Upload the artwork to R2
    let upload_result = upload_track_to_r2(
        r2_client,
        &image_path,
        &r2_path,
        &image_mime_type,
        &credentials.bucket_name,
        true, // Make artwork public for web access
    ).await?;
    
    // Return the upload result
    Ok(upload_result)
} 