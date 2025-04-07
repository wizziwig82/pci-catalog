//! This module orchestrates storage actions involving multiple systems,
//! primarily MongoDB and R2 cloud storage.

use mongodb::{bson::doc, Collection, Database};
use futures_util::stream::TryStreamExt;
use log::{info, warn, error};
use std::collections::HashMap;
use anyhow::{Result, anyhow}; // Use anyhow for error handling

// Import AWS S3 SDK directly
use aws_sdk_s3;

// Define local R2Client struct to avoid dependency issues
#[derive(Clone)]
pub struct MyR2Client {
    pub client: aws_sdk_s3::Client,
    pub bucket_name: String,
}

// Add local r2 module with required functions
mod r2_operations {
    use super::*;
    
    pub struct R2UploadResult {
        pub success: bool,
        pub error: Option<String>,
        pub key: Option<String>,
    }
    
    // Placeholder for R2 delete files function
    pub async fn delete_files(r2_client: &MyR2Client, file_paths: &[String]) -> Result<()> {
        // Implementation would go here
        info!("Placeholder: Would delete {} files from R2", file_paths.len());
        Ok(())
    }
    
    // Placeholder for R2 upload function
    pub async fn upload_file_from_path(
        r2_client: MyR2Client,
        local_path: String,
        r2_key: String,
        content_type: String,
    ) -> R2UploadResult {
        // Implementation would go here
        info!("Placeholder: Would upload {} to R2 key {}", local_path, r2_key);
        R2UploadResult {
            success: true,
            error: None,
            key: Some(r2_key),
        }
    }
}

/// Deletes multiple tracks from the database and corresponding files from R2.
pub async fn delete_tracks_by_ids(db: &Database, r2_client: &MyR2Client, track_ids: &[String]) -> Result<()> {
    info!("Attempting to delete tracks with IDs: {:?}", track_ids);
    let collection: Collection<mongodb::bson::Document> = db.collection("tracks");

    // Ensure IDs are not empty before proceeding
    if track_ids.is_empty() {
        warn!("delete_tracks_by_ids called with empty track_ids list.");
        return Ok(()); // Nothing to delete
    }

    // Create the filter to find the tracks
    let filter = doc! { "_id": { "$in": track_ids } };
    // 1. Find the documents first to get file paths
    let tracks_to_delete = match collection.find(filter.clone(), None).await {
        Ok(cursor) => {
            match cursor.try_collect::<Vec<mongodb::bson::Document>>().await {
                Ok(docs) => {
                    info!("Found {} track documents to delete.", docs.len());
                    docs
                },
                Err(e) => {
                    error!("Error collecting track documents for deletion: {}", e);
                    return Err(anyhow!("MongoDB find error: {}", e));
                }
            }
        },
        Err(e) => {
            error!("Error finding tracks to delete: {}", e);
            return Err(anyhow!("MongoDB find error: {}", e));
        }
    };

    // Extract file paths and album IDs
    let mut album_updates: HashMap<String, Vec<String>> = HashMap::new(); // album_id -> [track_id_to_remove]
    let file_paths_to_delete: Vec<String> = tracks_to_delete.iter()
        .filter_map(|doc| {
            let path = doc.get_str("path").ok().map(String::from);
            // Use track_id (which is _id in the doc)
            if let (Ok(track_id), Ok(album_id)) = (doc.get_str("_id"), doc.get_str("album_id")) {
                 if !album_id.is_empty() { // Only update if album_id is present
                    album_updates.entry(album_id.to_string()).or_default().push(track_id.to_string());
                 }
            }
            path
        })
        .collect();

    info!("File paths identified for R2 deletion: {:?}", file_paths_to_delete);
    info!("Album updates needed: {:?}", album_updates);

    // 2. Now, delete the documents from MongoDB
    match collection.delete_many(filter, None).await {
        Ok(delete_result) => {
            info!("Successfully deleted {} tracks from MongoDB.", delete_result.deleted_count);
            if delete_result.deleted_count != tracks_to_delete.len() as u64 {
                warn!("Mismatch between found documents ({}) and deleted count ({}).", tracks_to_delete.len(), delete_result.deleted_count);
            }

            // Delete corresponding files from R2
            if !file_paths_to_delete.is_empty() {
                info!("Attempting to delete {} files from R2.", file_paths_to_delete.len());
                // Assuming a function `delete_files` exists in the r2 module
                match r2_operations::delete_files(r2_client, &file_paths_to_delete).await { // Use the imported r2 module
                    Ok(_) => info!("Successfully requested deletion of files from R2."),
                    Err(e) => {
                        // Log the error but don't necessarily fail the whole operation,
                        // as the DB deletion might have succeeded.
                        error!("Failed to delete files from R2: {:?}", e);
                        // Optionally, return an error or partial success indicator here
                    }
                }
            }

            // 3. Update affected albums
            let albums_collection: Collection<mongodb::bson::Document> = db.collection("albums");
            for (album_id, track_ids_to_remove) in album_updates {
                info!("Updating album {} to remove tracks {:?}", album_id, track_ids_to_remove);
                let update_result = albums_collection.update_one(
                    doc! { "_id": &album_id },
                    doc! { "$pull": { "track_ids": { "$in": track_ids_to_remove } } },
                    None
                ).await;

                match update_result {
                    Ok(res) => {
                        if res.modified_count == 0 {
                            warn!("Album {} not found or no tracks removed during update.", album_id);
                        } else {
                            info!("Successfully updated album {}.", album_id);
                        }
                    },
                    Err(e) => {
                        // Log error but don't fail the whole operation
                        error!("Failed to update album {}: {}", album_id, e);
                    }
                }
            }

            Ok(())
        }
        Err(e) => {
            error!("Failed to delete tracks from MongoDB after finding them: {}", e);
            Err(anyhow!("MongoDB deletion error: {}", e))
        }
    }
}

/// Replaces the audio file for a track, uploading the new file to R2 and updating MongoDB.
pub async fn replace_track_audio(
    db: &Database,
    r2_client: &MyR2Client,
    track_id: &str,
    new_medium_quality_local_path: &str, // Path of the newly transcoded file on local disk
) -> Result<()> {
    info!("Starting audio replacement for track_id: {}", track_id);
    let tracks_collection: Collection<mongodb::bson::Document> = db.collection("tracks");

    // 1. Fetch the existing track document
    let filter = doc! { "_id": track_id };
    let track_doc = match tracks_collection.find_one(filter.clone(), None).await {
        Ok(Some(doc)) => doc,
        Ok(None) => {
            error!("Track {} not found for audio replacement.", track_id);
            return Err(anyhow!("Track {} not found", track_id));
        }
        Err(e) => {
            error!("Failed to fetch track {}: {}", track_id, e);
            return Err(anyhow!("MongoDB find error: {}", e));
        }
    };

    // 2. Determine old and new R2 paths
    // Assuming the path stored in DB is the R2 key for the medium quality file
    let old_r2_medium_path = track_doc.get_str("path").ok() // Adjust field name if needed
        .map(String::from);
    // TODO: Determine if original file also needs deletion/replacement logic

    // Construct the new R2 path/key (e.g., using track ID and a standard extension)
    // This logic might need refinement based on desired R2 structure
    let new_r2_medium_key = format!("tracks/{}/medium.mp3", track_id); // Example structure
    info!("Old R2 path: {:?}, New R2 key: {}", old_r2_medium_path, new_r2_medium_key);

    // 3. Upload the new file to R2
    info!("Uploading new file from {} to R2 key {}", new_medium_quality_local_path, new_r2_medium_key);
    // Assuming upload_file_from_path exists and takes R2Client, local path, R2 key, content type
    let upload_result = r2_operations::upload_file_from_path( // Use the imported r2 module
        r2_client.clone(), // Clone the client if needed by the function
        new_medium_quality_local_path.to_string(),
        new_r2_medium_key.clone(),
        "audio/mpeg".to_string(), // Assuming MP3, adjust if format varies
    ).await;

    // Handle the R2UploadResult directly
    if !upload_result.success {
        error!("Failed to upload replacement file to R2: {:?}", upload_result.error);
        return Err(anyhow!("R2 upload failed: {:?}", upload_result.error));
    }
    info!("Successfully uploaded replacement file to R2.");

    // 4. Update the track document in MongoDB
    info!("Updating MongoDB document for track {} with new path {}", track_id, new_r2_medium_key);
    let update_doc = doc! { "$set": { "path": &new_r2_medium_key } }; // Adjust field name if needed
    match tracks_collection.update_one(filter, update_doc, None).await {
        Ok(update_result) => {
            if update_result.matched_count == 0 {
                // This shouldn't happen if find_one succeeded, but handle defensively
                error!("Track {} not found during update phase.", track_id);
                // Consider rolling back the R2 upload? For now, return error.
                return Err(anyhow!("Track {} disappeared during update", track_id));
            }
            if update_result.modified_count == 0 {
                warn!("Track {} document was matched but not modified (perhaps path was already correct?).", track_id);
            }
            info!("Successfully updated track document in MongoDB.");
        }
        Err(e) => {
            error!("Failed to update track document {}: {}", track_id, e);
            // Consider rolling back the R2 upload? For now, return error.
            return Err(anyhow!("MongoDB update error: {}", e));
        }
    }

    // 5. Delete the old file(s) from R2
    if let Some(old_path) = old_r2_medium_path {
        if !old_path.is_empty() && old_path != new_r2_medium_key {
            info!("Deleting old R2 file: {}", old_path);
            // Assuming delete_files exists and takes R2Client and a slice of keys
            match r2_operations::delete_files(r2_client, &[old_path.clone()]).await { // Use the imported r2 module
                Ok(_) => info!("Successfully deleted old file {} from R2.", old_path),
                Err(e) => {
                    // Log error but don't fail the overall operation, as the main goal (replacement) succeeded.
                    error!("Failed to delete old R2 file {}: {:?}", old_path, e);
                }
            }
        } else {
             info!("Old path was empty or same as new path, skipping deletion.");
        }
    } else {
        info!("No old path found in document, skipping deletion.");
    }

    Ok(())
}