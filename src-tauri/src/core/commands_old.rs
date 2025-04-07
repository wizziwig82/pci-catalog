use ::mongodb::bson::{self, doc}; // Import bson module and doc macro
// Remove direct Collection import
use serde::{Deserialize, Serialize};
use tauri::{command, State};
use log::{info, error, warn};
use futures_util::StreamExt; // Add StreamExt for cursor.next()

use crate::{MongoState, R2State}; // State structs are now in lib.rs root
// Removed unused imports related to removed functions
use crate::core::r2::R2Client; // R2Client is in core::r2
use crate::error::CommandError; // Correct path (from lib.rs) - This is the main error enum

#[derive(Debug, Serialize, Deserialize)]
pub struct ClearTestDataResponse {
    pub success: bool,
    pub message: String,
}

// REMOVED fetch_all_tracks function

/// Command to clear test data from the database
/// This is only available in development/test mode, not in production
#[command]
pub async fn clear_test_data(
    mongo_state: State<'_, MongoState>,
) -> Result<ClearTestDataResponse, String> { // Keep String error type for this specific command for now
    info!("Clearing test data from database");

    // Get Mongo client from state
    let mongo_client_lock = mongo_state.client.lock().await;
    let mongo_client = mongo_client_lock.as_ref()
        .ok_or("MongoDB client not initialized. Please configure credentials first.")?;

    // First delete tracks with titles containing 'Test Track' or from test albums
    let tracks_collection: ::mongodb::Collection<bson::Document> = mongo_client.database("music_library").collection("tracks"); // Use fully qualified type

    // Delete tracks with test data
    let test_track_filter = doc! { // Keep imported doc! macro
        "$or": [
            { "title": { "$regex": "Test Track" } },
            { "title": { "$regex": "Integration Test Track" } }
        ]
    };

    let delete_tracks_result = tracks_collection.delete_many(test_track_filter, None).await;
    let tracks_deleted = match delete_tracks_result {
        Ok(result) => result.deleted_count,
        Err(e) => {
            error!("Failed to delete test tracks: {}", e);
            return Ok(ClearTestDataResponse {
                success: false,
                message: format!("Failed to delete test tracks: {}", e),
            });
        }
    };

    // Delete test albums
    let albums_collection: ::mongodb::Collection<bson::Document> = mongo_client.database("music_library").collection("albums"); // Use fully qualified type
    let test_album_filter = doc! { // Keep imported doc! macro
        "$or": [
            { "name": "Test Album" },
            { "name": "Integration Test Album" }
        ]
    };

    let delete_albums_result = albums_collection.delete_many(test_album_filter, None).await;
    let albums_deleted = match delete_albums_result {
        Ok(result) => result.deleted_count,
        Err(e) => {
            error!("Failed to delete test albums: {}", e);
            return Ok(ClearTestDataResponse {
                success: false,
                message: format!("Failed to delete test albums: {}", e),
            });
        }
    };

    info!("Deleted {} test tracks and {} test albums", tracks_deleted, albums_deleted);

    Ok(ClearTestDataResponse {
        success: true,
        message: format!("Successfully cleared test data: {} tracks and {} albums deleted",
            tracks_deleted, albums_deleted),
    })
}

/// Command to test MongoDB connectivity and check collection stats
#[command]
pub async fn test_mongodb_collections(
    mongo_state: State<'_, MongoState>,
) -> Result<String, String> { // Keep String error type for this specific command for now
    info!("Testing MongoDB collections");

    // Get Mongo client from state
    let mongo_client_lock = mongo_state.client.lock().await;
    let mongo_client = match mongo_client_lock.as_ref() {
        Some(client) => {
            info!("MongoDB client found in state");
            client
        },
        None => {
            error!("MongoDB client not initialized");
            return Err("MongoDB client not initialized. Please configure credentials first.".to_string());
        }
    };

    // Create a database reference
    info!("Creating database reference to 'music_library'");
    let db = mongo_client.database("music_library");

    // Get collection names
    let collection_names = match db.list_collection_names(None).await {
        Ok(names) => {
            info!("Found collections: {:?}", names);
            names
        },
        Err(e) => {
            error!("Failed to list collections: {}", e);
            return Err(format!("Failed to list collections: {}", e));
        }
    };

    let mut result = format!("Found {} collections: {:?}\n", collection_names.len(), collection_names);

    // Check tracks collection
    if collection_names.contains(&"tracks".to_string()) {
        let tracks_collection: ::mongodb::Collection<bson::Document> = db.collection("tracks"); // Use fully qualified type
        match tracks_collection.count_documents(None, None).await {
            Ok(count) => {
                info!("Tracks collection has {} documents", count);
                result.push_str(&format!("Tracks collection: {} documents\n", count));

                // Get a sample track if any exist
                if count > 0 {
                    match tracks_collection.find_one(None, None).await {
                        Ok(Some(doc)) => { // doc here is a variable, not the macro
                            info!("Sample track document: {:?}", doc);
                            result.push_str(&format!("Sample track fields: {}\n",
                                doc.keys().map(|k| k.to_string()).collect::<Vec<String>>().join(", ")));
                        },
                        Ok(None) => {
                            warn!("No track found despite count > 0");
                            result.push_str("Could not retrieve sample track\n");
                        },
                        Err(e) => {
                            error!("Error fetching sample track: {}", e);
                            result.push_str(&format!("Error fetching sample track: {}\n", e));
                        }
                    }
                }
            },
            Err(e) => {
                error!("Error counting tracks: {}", e);
                result.push_str(&format!("Error counting tracks: {}\n", e));
            }
        }
    }

    // Check albums collection
    if collection_names.contains(&"albums".to_string()) {
        let albums_collection: ::mongodb::Collection<bson::Document> = db.collection("albums"); // Use fully qualified type
        match albums_collection.count_documents(None, None).await {
            Ok(count) => {
                info!("Albums collection has {} documents", count);
                result.push_str(&format!("Albums collection: {} documents\n", count));

                // Get a sample album if any exist
                if count > 0 {
                    match albums_collection.find_one(None, None).await {
                        Ok(Some(doc)) => { // doc here is a variable, not the macro
                            info!("Sample album document: {:?}", doc);
                            result.push_str(&format!("Sample album fields: {}\n",
                                doc.keys().map(|k| k.to_string()).collect::<Vec<String>>().join(", ")));
                        },
                        Ok(None) => {
                            warn!("No album found despite count > 0");
                            result.push_str("Could not retrieve sample album\n");
                        },
                        Err(e) => {
                            error!("Error fetching sample album: {}", e);
                            result.push_str(&format!("Error fetching sample album: {}\n", e));
                        }
                    }
                }
            },
            Err(e) => {
                error!("Error counting albums: {}", e);
                result.push_str(&format!("Error counting albums: {}\n", e));
            }
        }
    }

    Ok(result)
}

 // REMOVED update_track_metadata command function

// --- Track Deletion Command ---

/// Command to delete tracks from MongoDB and their audio files from R2
#[command]
pub async fn delete_tracks(
   track_ids: Vec<String>, // Expecting a list of track IDs from the frontend
   mongo_state: State<'_, MongoState>,
   r2_state: State<'_, R2State>, // Add R2State
) -> Result<(), CommandError> {
    info!("Deleting {} tracks: {:?}", track_ids.len(), track_ids);

    // Get Mongo client from state
    let mongo_client_lock = mongo_state.client.lock().await;
    let mongo_client = mongo_client_lock.as_ref().ok_or_else(|| {
        error!("MongoDB client not initialized during delete_tracks");
        CommandError::Configuration("Database client not initialized".to_string())
    })?;

    // Create a database reference
    let db = mongo_client.database("music_library");
    let tracks_collection = db.collection::<bson::Document>("tracks");

    // For each track ID, get the track first to obtain file paths
    let filter = doc! {
        "_id": {
            "$in": track_ids.iter().filter_map(|id| bson::oid::ObjectId::parse_str(id).ok()).collect::<Vec<_>>()
        }
    };

    // Fetch the tracks to get their audio file paths
    let mut cursor = tracks_collection.find(filter.clone(), None).await?;

    // Collect R2 paths to delete
    let mut r2_paths = Vec::new();

    while let Some(result) = cursor.next().await {
        match result {
            Ok(doc) => {
                // Extract audio file paths
                if let Some(medium_quality) = doc.get_str("medium_quality_url").ok() {
                    r2_paths.push(medium_quality.to_string());
                }
                if let Some(high_quality) = doc.get_str("high_quality_url").ok() {
                    r2_paths.push(high_quality.to_string());
                }
                if let Some(original_quality) = doc.get_str("original_quality_url").ok() {
                    r2_paths.push(original_quality.to_string());
                }
            },
            Err(e) => {
                error!("Error fetching track while preparing for deletion: {}", e);
                // Continue with the rest of the tracks
            }
        }
    }

    // Get R2 client from state
    let r2_client_lock = r2_state.client.lock().await;
    let bucket_name_lock = r2_state.bucket_name.lock().await;

    // Delete the files from R2 if there are any paths
    if !r2_paths.is_empty() && r2_client_lock.is_some() && bucket_name_lock.is_some() {
        let r2_client = r2_client_lock.as_ref().unwrap();
        let bucket_name = bucket_name_lock.as_ref().unwrap();

        // Create R2Client wrapper
        let r2_client = R2Client::new(r2_client.clone(), bucket_name.clone());

        // Delete files from R2 using the new method
        match r2_client.delete_objects(&r2_paths).await {
            Ok(_) => {
                info!("Successfully deleted {} files from R2", r2_paths.len());
            },
            Err(e) => {
                error!("Failed to delete files from R2: {}", e);
                // Continue with MongoDB deletion even if R2 deletion failed
            }
        }
    }

    // Delete tracks from MongoDB
    let delete_result = tracks_collection.delete_many(filter, None).await?;

    info!("Deleted {} tracks from MongoDB", delete_result.deleted_count);

    Ok(())
}

/// Command to replace a track's audio file with a new one
#[command]
pub async fn replace_track_audio(
   track_id: String,
   new_medium_quality_path: String, // Path to the *already transcoded* new file
   mongo_state: State<'_, MongoState>,
   r2_state: State<'_, R2State>,
) -> Result<(), CommandError> {
    info!("Replacing audio for track {}", track_id);

    // Get Mongo client from state
    let mongo_client_lock = mongo_state.client.lock().await;
    let mongo_client = mongo_client_lock.as_ref().ok_or_else(|| {
        error!("MongoDB client not initialized during replace_track_audio");
        CommandError::Configuration("Database client not initialized".to_string())
    })?;

    // Create a database reference
    let db = mongo_client.database("music_library");
    let tracks_collection = db.collection::<bson::Document>("tracks");

    // Get the track to obtain current file paths
    let object_id = bson::oid::ObjectId::parse_str(&track_id)
        .map_err(|e| CommandError::Validation(format!("Invalid track ID format: {}", e)))?;

    let filter = doc! { "_id": object_id };

    let track_doc = tracks_collection.find_one(filter.clone(), None).await?
        .ok_or_else(|| CommandError::NotFound(format!("Track with ID {} not found", track_id)))?;

    // Extract current audio file path
    let current_medium_quality = track_doc.get_str("medium_quality_url")
        .map_err(|_| CommandError::Metadata("medium_quality_url field missing or invalid".to_string()))?;

    // Get R2 client from state
    let r2_client_lock = r2_state.client.lock().await;
    let bucket_name_lock = r2_state.bucket_name.lock().await;

    let r2_client = r2_client_lock.as_ref().ok_or_else(|| {
        error!("R2 client not initialized during replace_track_audio");
        CommandError::Configuration("R2 client not initialized".to_string())
    })?;

    let bucket_name = bucket_name_lock.as_ref().ok_or_else(|| {
        error!("R2 bucket name not set during replace_track_audio");
        CommandError::Configuration("R2 bucket name not set".to_string())
    })?;

    // Create R2Client wrapper
    let r2_client = R2Client::new(r2_client.clone(), bucket_name.clone());

    // Read the new file
    let new_file_data = std::fs::read(&new_medium_quality_path)
        .map_err(|e| CommandError::FileSystem(format!("Failed to read new audio file: {}", e)))?;

    // Upload the new file with the same key as the original
    r2_client.upload_object(current_medium_quality, new_file_data, "audio/mpeg").await
        .map_err(|e| CommandError::Storage(format!("Failed to upload new audio file: {}", e)))?;

    info!("Successfully replaced audio for track {}", track_id);

    Ok(())
}