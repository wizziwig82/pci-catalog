use serde::{Deserialize, Serialize};
use tauri::{command, State};
use log::{info, error};
use std::sync::Arc;

use crate::MongoState;

#[derive(Debug, Serialize, Deserialize)]
pub struct ClearTestDataResponse {
    pub success: bool,
    pub message: String,
}

/// Command to clear test data from the database
/// This is only available in development/test mode, not in production
#[command]
pub async fn clear_test_data(
    mongo_state: State<'_, MongoState>,
) -> Result<ClearTestDataResponse, String> {
    info!("Clearing test data from database");
    
    // Get Mongo client from state
    let mongo_client_lock = mongo_state.client.lock().await;
    let mongo_client = mongo_client_lock.as_ref()
        .ok_or("MongoDB client not initialized. Please configure credentials first.")?;
    
    // First delete tracks with titles containing 'Test Track' or from test albums
    let tracks_collection: mongodb::Collection<mongodb::bson::Document> = mongo_client.database("music_library").collection("tracks");
    
    // Delete tracks with test data
    let test_track_filter = mongodb::bson::doc! {
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
    let albums_collection: mongodb::Collection<mongodb::bson::Document> = mongo_client.database("music_library").collection("albums");
    let test_album_filter = mongodb::bson::doc! {
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