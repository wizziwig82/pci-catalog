use serde::{Deserialize, Serialize};
use tauri::{command, State};
use log::{info, error, warn};

use crate::MongoState;
use crate::storage::mongodb::{get_all_tracks, TrackListResponse};

#[derive(Debug, Serialize, Deserialize)]
pub struct ClearTestDataResponse {
    pub success: bool,
    pub message: String,
}

/// Command to fetch all tracks from MongoDB with sorting and pagination
#[command]
pub async fn fetch_all_tracks(
    mongo_state: State<'_, MongoState>,
    sort_field: String,
    sort_direction: String,
    limit: Option<i64>,
    skip: Option<i64>,
) -> Result<TrackListResponse, String> {
    info!("Fetching tracks from MongoDB with sort: {}, direction: {}, limit: {:?}, skip: {:?}", 
          sort_field, sort_direction, limit, skip);
    
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
    
    // Call the function directly with the database
    info!("Calling get_all_tracks function");
    let response = get_all_tracks(&db, &sort_field, &sort_direction, limit, skip).await;
    
    info!("get_all_tracks returned: success={}, track count={}, total_count={}", 
          response.success, response.tracks.len(), response.total_count);
    
    Ok(response)
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

/// Command to test MongoDB connectivity and check collection stats
#[command]
pub async fn test_mongodb_collections(
    mongo_state: State<'_, MongoState>,
) -> Result<String, String> {
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
        let tracks_collection: mongodb::Collection<mongodb::bson::Document> = db.collection("tracks");
        match tracks_collection.count_documents(None, None).await {
            Ok(count) => {
                info!("Tracks collection has {} documents", count);
                result.push_str(&format!("Tracks collection: {} documents\n", count));
                
                // Get a sample track if any exist
                if count > 0 {
                    match tracks_collection.find_one(None, None).await {
                        Ok(Some(doc)) => {
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
        let albums_collection: mongodb::Collection<mongodb::bson::Document> = db.collection("albums");
        match albums_collection.count_documents(None, None).await {
            Ok(count) => {
                info!("Albums collection has {} documents", count);
                result.push_str(&format!("Albums collection: {} documents\n", count));
                
                // Get a sample album if any exist
                if count > 0 {
                    match albums_collection.find_one(None, None).await {
                        Ok(Some(doc)) => {
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