use mongodb::{
    bson::{self, doc, Document, to_bson}, // Add bson module import
    options::{ClientOptions, IndexOptions, FindOptions},
    IndexModel,
    Client, Collection, Database,
};
use futures_util::stream::TryStreamExt;
use serde::{Deserialize, Serialize}; // Serialize is needed here for derive macros
use std::error::Error;
use std::sync::Arc;
use log::{info, warn, error}; // Ensure error is imported
use std::collections::HashMap;
use tauri::State; // Import State for command arguments
use crate::MongoState; // Import MongoState from lib.rs

use super::UpdateTrackPayload; // Import from parent module (storage/mod.rs)

use self::error::CommandError;

// We need to create a local CommandError type that wraps what we need
// This should ideally be unified with the main CommandError in lib.rs later
mod error {
    // Ensure no 'use serde::Serialize;' is present here
    use serde::Serialize; // THIS LINE SHOULD BE REMOVED

    #[derive(Debug, Serialize)] // This will use the top-level `use serde::Serialize`
    pub enum CommandError {
        Validation(String),
        Database(String),
        NotFound(String),
        Configuration(String), // Added for consistency
    }

    // Implement Display for CommandError
    impl std::fmt::Display for CommandError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                CommandError::Validation(msg) => write!(f, "Validation Error: {}", msg),
                CommandError::Database(msg) => write!(f, "Database Error: {}", msg),
                CommandError::NotFound(msg) => write!(f, "Not Found Error: {}", msg),
                CommandError::Configuration(msg) => write!(f, "Configuration Error: {}", msg),
            }
        }
    }

    // Implement Error for CommandError
    impl std::error::Error for CommandError {}

}


// Credentials structure for MongoDB
#[derive(Debug, Serialize, Deserialize)]
pub struct MongoCredentials {
    pub uri: String,
}

// Album structure based on our MongoDB schema
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Album {
    pub name: String,
    pub track_ids: Vec<String>,
    pub art_path: Option<String>,
    pub release_date: Option<String>,
    pub publisher: Option<String>,
}

// Path information structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PathInfo {
    pub original: String,
    pub medium: String,
}

// Track structure based on our MongoDB schema
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Track {
    pub title: String,
    pub album_id: String,
    pub track_number: Option<i32>,
    pub filename: String,
    pub duration: i32,
    pub writers: Vec<String>,
    pub publishers: Vec<String>,
    pub composers: Option<Vec<String>>,
    pub genre: Option<Vec<String>>, // Changed to Vec<String>
    pub path: String,
    pub waveform_data: Option<Vec<i32>>,
}

// Track list response structure for returning track data with album details
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrackWithAlbum {
    pub id: String, // Use 'id' consistent with frontend expectations
    pub title: String,
    pub album_id: String,
    pub album_name: String,
    pub track_number: Option<i32>,
    pub filename: String,
    pub duration: Option<i32>, // Made Option to handle potential missing data
    pub writers: Vec<String>,
    pub writer_percentages: Option<HashMap<String, f32>>, // Keep as Option<HashMap>
    pub publishers: Vec<String>,
    pub publisher_percentages: Option<HashMap<String, f32>>, // Keep as Option<HashMap>
    pub composers: Option<Vec<String>>,
    pub genre: Option<Vec<String>>, // Changed to Vec<String>
    pub path: String, // Keep path as string (R2 key)
    pub waveform_data: Option<Vec<f32>>,
    pub comments: Option<String>, // Added comments field
}


// Track list response
#[derive(Debug, Serialize)]
pub struct TrackListResponse {
    pub success: bool,
    pub message: Option<String>,
    pub tracks: Vec<TrackWithAlbum>,
    pub total_count: usize,
}

// Track document structure matching exactly what's in MongoDB
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrackDocument {
    pub _id: String, // Use _id for MongoDB interaction
    pub title: String,
    pub album_id: String,
    pub track_number: Option<i32>,
    pub filename: String,
    pub duration: i32,
    pub writers: Vec<String>,
    pub writer_percentages: Option<HashMap<String, f32>>, // Match TrackWithAlbum
    pub publishers: Vec<String>,
    pub publisher_percentages: Option<HashMap<String, f32>>, // Match TrackWithAlbum
    pub composers: Option<Vec<String>>,
    pub genre: Option<Vec<String>>, // Changed to Vec<String>
    pub path: String, // Path to medium quality file in R2
    pub waveform_data: Option<Vec<f32>>,
    pub comments: Option<String>, // Added comments field
}

// MongoDB Client wrapper (No longer needed directly in commands)
// pub struct MongoClient {
//     client: Client,
//     db: Database,
// }

// Response type for database operations (May not be needed if commands return Result)
#[derive(Debug, Serialize)]
pub struct DbResponse<T> {
    pub success: bool,
    pub message: Option<String>,
    pub id: Option<String>,
    pub data: Option<T>,
}

// impl MongoClient { ... } // Methods moved or adapted into commands

// Initialize the MongoDB client (This logic is likely handled in main.rs now)
// pub async fn initialize_mongo_client(...) -> Result<Arc<MongoClient>, Box<dyn Error + Send + Sync>> { ... }

// Create necessary indexes for efficient searching
async fn create_indexes(db: &Database) -> Result<(), Box<dyn Error + Send + Sync>> {
    // Create text index on track title
    let tracks_collection: Collection<Document> = db.collection("tracks");
    let track_index_options = IndexOptions::builder()
        .build();

    let track_index_model = IndexModel::builder()
        .keys(doc! {
            "title": "text",
            "genre": "text",
            "album_name": "text", // Add album_name if needed for search
            "writers": "text",
            "publishers": "text",
            "instruments": "text",
            "mood": "text"
        })
        .options(track_index_options)
        .build();

    tracks_collection.create_index(track_index_model, None).await?;

    // Create text index on album name
    let albums_collection: Collection<Document> = db.collection("albums");
    let album_index_options = IndexOptions::builder()
        .build();

    let album_index_model = IndexModel::builder()
        .keys(doc! {
            "name": "text",
            "artist": "text" // Add artist if needed
        })
        .options(album_index_options)
        .build();

    albums_collection.create_index(album_index_model, None).await?;

    // Create index for album_id in tracks
    let album_track_relation_index = IndexModel::builder()
        .keys(doc! { "album_id": 1 })
        .build();

    tracks_collection.create_index(album_track_relation_index, None).await?;

    Ok(())
}

// Album CRUD operations (These are not commands, keep as helper functions if needed elsewhere)
pub async fn create_album(
    db: &Database, // Accept &Database directly
    album_id: &str,
    album_data: Album,
) -> DbResponse<()> {
    let collection = db.collection::<Document>("albums");
    let mut doc = to_bson(&album_data).unwrap().as_document().unwrap().clone();
    doc.insert("_id", album_id);

    match collection.insert_one(doc, None).await {
        Ok(_) => DbResponse {
            success: true,
            message: Some("Album created successfully".to_string()),
            id: Some(album_id.to_string()),
            data: None,
        },
        Err(e) => DbResponse {
            success: false,
            message: Some(format!("Failed to create album: {}", e)),
            id: None,
            data: None,
        },
    }
}

pub async fn get_album(db: &Database, album_id: &str) -> DbResponse<Album> {
    let collection = db.collection::<Document>("albums");
    match collection.find_one(doc! { "_id": album_id }, None).await {
        Ok(Some(album_doc)) => {
            match mongodb::bson::from_document::<Album>(album_doc) {
                Ok(album) => DbResponse {
                    success: true,
                    message: Some("Album retrieved successfully".to_string()),
                    id: Some(album_id.to_string()),
                    data: Some(album),
                },
                Err(e) => DbResponse {
                    success: false,
                    message: Some(format!("Failed to parse album data: {}", e)),
                    id: None,
                    data: None,
                },
            }
        }
        Ok(None) => DbResponse {
            success: false,
            message: Some(format!("Album with ID {} not found", album_id)),
            id: None,
            data: None,
        },
        Err(e) => DbResponse {
            success: false,
            message: Some(format!("Failed to retrieve album: {}", e)),
            id: None,
            data: None,
        },
    }
}


pub async fn update_album(
    db: &Database,
    album_id: &str,
    album_data: Album,
) -> DbResponse<()> {
    let collection = db.collection::<Document>("albums");
    let update_doc = to_bson(&album_data).unwrap();

    match collection
        .update_one(
            doc! { "_id": album_id },
            doc! {
                "$set": update_doc
            },
            None,
        )
        .await
    {
        Ok(result) => {
            if result.matched_count > 0 {
                DbResponse {
                    success: true,
                    message: Some("Album updated successfully".to_string()),
                    id: Some(album_id.to_string()),
                    data: None,
                }
            } else {
                DbResponse {
                    success: false,
                    message: Some(format!("Album with ID {} not found", album_id)),
                    id: None,
                    data: None,
                }
            }
        }
        Err(e) => DbResponse {
            success: false,
            message: Some(format!("Failed to update album: {}", e)),
            id: None,
            data: None,
        },
    }
}

pub async fn delete_album(db: &Database, album_id: &str) -> DbResponse<()> {
    let collection = db.collection::<Document>("albums");
    match collection.delete_one(doc! { "_id": album_id }, None).await {
        Ok(result) => {
            if result.deleted_count > 0 {
                DbResponse {
                    success: true,
                    message: Some("Album deleted successfully".to_string()),
                    id: Some(album_id.to_string()),
                    data: None,
                }
            } else {
                DbResponse {
                    success: false,
                    message: Some(format!("Album with ID {} not found", album_id)),
                    id: None,
                    data: None,
                }
            }
        }
        Err(e) => DbResponse {
            success: false,
            message: Some(format!("Failed to delete album: {}", e)),
            id: None,
            data: None,
        },
    }
}

// Track CRUD operations (Keep as helper functions if needed)
pub async fn create_track(
    db: &Database,
    track_id: &str,
    track_data: Track,
) -> DbResponse<()> {
    let collection = db.collection::<Document>("tracks");
    let mut doc = to_bson(&track_data).unwrap().as_document().unwrap().clone();
    doc.insert("_id", track_id);

    match collection.insert_one(doc, None).await {
        Ok(_) => DbResponse {
            success: true,
            message: Some("Track created successfully".to_string()),
            id: Some(track_id.to_string()),
            data: None,
        },
        Err(e) => DbResponse {
            success: false,
            message: Some(format!("Failed to create track: {}", e)),
            id: None,
            data: None,
        },
    }
}

pub async fn get_track(db: &Database, track_id: &str) -> DbResponse<Track> {
    let collection = db.collection::<Document>("tracks");
    match collection.find_one(doc! { "_id": track_id }, None).await {
        Ok(Some(track_doc)) => {
             match mongodb::bson::from_document::<Track>(track_doc) {
                Ok(track) => DbResponse {
                    success: true,
                    message: Some("Track retrieved successfully".to_string()),
                    id: Some(track_id.to_string()),
                    data: Some(track),
                },
                Err(e) => DbResponse {
                    success: false,
                    message: Some(format!("Failed to parse track data: {}", e)),
                    id: None,
                    data: None,
                },
            }
        }
        Ok(None) => DbResponse {
            success: false,
            message: Some(format!("Track with ID {} not found", track_id)),
            id: None,
            data: None,
        },
        Err(e) => DbResponse {
            success: false,
            message: Some(format!("Failed to retrieve track: {}", e)),
            id: None,
            data: None,
        },
    }
}

pub async fn update_track(
    db: &Database,
    track_id: &str,
    track_data: Track,
) -> DbResponse<()> {
    let collection = db.collection::<Document>("tracks");
    let update_doc = to_bson(&track_data).unwrap();

    match collection
        .update_one(
            doc! { "_id": track_id },
            doc! {
                "$set": update_doc
            },
            None,
        )
        .await
    {
        Ok(result) => {
            if result.matched_count > 0 {
                DbResponse {
                    success: true,
                    message: Some("Track updated successfully".to_string()),
                    id: Some(track_id.to_string()),
                    data: None,
                }
            } else {
                DbResponse {
                    success: false,
                    message: Some(format!("Track with ID {} not found", track_id)),
                    id: None,
                    data: None,
                }
            }
        }
        Err(e) => DbResponse {
            success: false,
            message: Some(format!("Failed to update track: {}", e)),
            id: None,
            data: None,
        },
    }
}

pub async fn delete_track(db: &Database, track_id: &str) -> DbResponse<()> {
    let collection = db.collection::<Document>("tracks");
    match collection.delete_one(doc! { "_id": track_id }, None).await {
        Ok(result) => {
            if result.deleted_count > 0 {
                DbResponse {
                    success: true,
                    message: Some("Track deleted successfully".to_string()),
                    id: Some(track_id.to_string()),
                    data: None,
                }
            } else {
                DbResponse {
                    success: false,
                    message: Some(format!("Track with ID {} not found", track_id)),
                    id: None,
                    data: None,
                }
            }
        }
        Err(e) => DbResponse {
            success: false,
            message: Some(format!("Failed to delete track: {}", e)),
            id: None,
            data: None,
        },
    }
}

// --- Functions `delete_tracks_by_ids` and `replace_track_audio` moved to `catalog_storage_actions.rs` ---


// Search tracks based on a query string (Not a command, keep as helper)
pub async fn search_tracks(
    db: &Database,
    query: &str,
    limit: Option<i64>,
    skip: Option<i64>,
) -> TrackListResponse {
    info!("Searching tracks with query: {}", query);
    let tracks_collection: Collection<Document> = db.collection("tracks");
    let albums_collection: Collection<Document> = db.collection("albums");

    // Basic text search filter
    let filter = doc! { "$text": { "$search": query } };

    let find_options = FindOptions::builder()
        .limit(limit)
        .skip(skip.map(|s| s as u64))
        .build();

    // Get total count matching the search query
    let total_count = match tracks_collection.count_documents(filter.clone(), None).await {
        Ok(count) => count as usize,
        Err(e) => {
            error!("Failed to count search results: {}", e);
            return TrackListResponse { success: false, message: Some(format!("Failed to count search results: {}", e)), tracks: vec![], total_count: 0 };
        }
    };

    let mut cursor = match tracks_collection.find(filter, find_options).await {
        Ok(cursor) => cursor,
        Err(e) => {
            error!("Failed to execute search query: {}", e);
            return TrackListResponse { success: false, message: Some(format!("Failed to execute search query: {}", e)), tracks: vec![], total_count: 0 };
        }
    };

    let mut tracks_with_album: Vec<TrackWithAlbum> = Vec::new();
    while let Ok(Some(track_doc)) = cursor.try_next().await {
        let track_data = match mongodb::bson::from_document::<TrackDocument>(track_doc.clone()) {
             Ok(data) => data,
             Err(e) => {
                 warn!("Failed to deserialize track document during search: {}. Doc: {:?}", e, track_doc);
                 continue;
             }
         };

        let album_name = if !track_data.album_id.is_empty() {
            let album_filter = doc! { "_id": &track_data.album_id };
            match albums_collection.find_one(album_filter, None).await {
                Ok(Some(album_doc)) => album_doc.get_str("name").unwrap_or("Unknown Album").to_string(),
                _ => "Unknown Album".to_string(),
            }
        } else {
            "No Album ID".to_string()
        };

        tracks_with_album.push(TrackWithAlbum {
            id: track_data._id,
            title: track_data.title,
            album_id: track_data.album_id,
            album_name,
            track_number: track_data.track_number,
            filename: track_data.filename,
            duration: Some(track_data.duration),
            writers: track_data.writers,
            writer_percentages: track_data.writer_percentages,
            publishers: track_data.publishers,
            publisher_percentages: track_data.publisher_percentages,
            composers: track_data.composers,
            genre: track_data.genre,
            path: track_data.path,
            waveform_data: track_data.waveform_data,
            comments: track_data.comments,
        });
    }

    TrackListResponse { success: true, message: None, tracks: tracks_with_album, total_count }
}

// Search albums based on a query string (Not a command, keep as helper)
pub async fn search_albums(
    db: &Database,
    query: &str,
    limit: Option<i64>,
    skip: Option<i64>,
) -> DbResponse<Vec<Album>> {
    info!("Searching albums with query: {}", query);
    let albums_collection: Collection<Document> = db.collection("albums");

    let filter = doc! { "$text": { "$search": query } };

    let find_options = FindOptions::builder()
        .limit(limit)
        .skip(skip.map(|s| s as u64))
        .build();

    match albums_collection.find(filter, find_options).await {
        Ok(mut cursor) => {
            let mut albums: Vec<Album> = Vec::new();
            while let Ok(Some(album_doc)) = cursor.try_next().await {
                match mongodb::bson::from_document::<Album>(album_doc) {
                    Ok(album) => albums.push(album),
                    Err(e) => warn!("Failed to deserialize album document during search: {}", e),
                }
            }
            DbResponse {
                success: true,
                message: Some(format!("Found {} albums", albums.len())),
                id: None,
                data: Some(albums),
            }
        }
        Err(e) => {
            error!("Failed to execute album search query: {}", e);
            DbResponse {
                success: false,
                message: Some(format!("Failed to execute album search query: {}", e)),
                id: None,
                data: None,
            }
        }
    }
}

// Get all tracks associated with a specific album ID (Not a command, keep as helper)
pub async fn get_tracks_by_album(
    db: &Database,
    album_id: &str,
) -> TrackListResponse {
    info!("Fetching tracks for album_id: {}", album_id);
    let tracks_collection: Collection<Document> = db.collection("tracks");
    let albums_collection: Collection<Document> = db.collection("albums");

    // Fetch album name first
    let album_name = match albums_collection.find_one(doc! { "_id": album_id }, None).await {
        Ok(Some(album_doc)) => album_doc.get_str("name").unwrap_or("Unknown Album").to_string(),
        _ => {
            warn!("Album {} not found when fetching tracks by album", album_id);
            "Unknown Album".to_string()
        }
    };

    let filter = doc! { "album_id": album_id };
    let find_options = FindOptions::builder().sort(doc! { "track_number": 1 }).build(); // Sort by track number

    // Get total count for this album
    let total_count = match tracks_collection.count_documents(filter.clone(), None).await {
        Ok(count) => count as usize,
        Err(e) => {
            error!("Failed to count tracks for album {}: {}", album_id, e);
            return TrackListResponse { success: false, message: Some(format!("Failed to count tracks for album {}: {}", album_id, e)), tracks: vec![], total_count: 0 };
        }
    };

    let mut cursor = match tracks_collection.find(filter, find_options).await {
        Ok(cursor) => cursor,
        Err(e) => {
            error!("Failed to fetch tracks for album {}: {}", album_id, e);
            return TrackListResponse { success: false, message: Some(format!("Failed to fetch tracks for album {}: {}", album_id, e)), tracks: vec![], total_count: 0 };
        }
    };

    let mut tracks_with_album: Vec<TrackWithAlbum> = Vec::new();
    while let Ok(Some(track_doc)) = cursor.try_next().await {
        let track_data = match mongodb::bson::from_document::<TrackDocument>(track_doc.clone()) {
             Ok(data) => data,
             Err(e) => {
                 warn!("Failed to deserialize track document for album {}: {}. Doc: {:?}", album_id, e, track_doc);
                 continue;
             }
         };

        tracks_with_album.push(TrackWithAlbum {
            id: track_data._id,
            title: track_data.title,
            album_id: track_data.album_id,
            album_name: album_name.clone(), // Use fetched album name
            track_number: track_data.track_number,
            filename: track_data.filename,
            duration: Some(track_data.duration),
            writers: track_data.writers,
            writer_percentages: track_data.writer_percentages,
            publishers: track_data.publishers,
            publisher_percentages: track_data.publisher_percentages,
            composers: track_data.composers,
            genre: track_data.genre,
            path: track_data.path,
            waveform_data: track_data.waveform_data,
            comments: track_data.comments,
        });
    }

    TrackListResponse { success: true, message: None, tracks: tracks_with_album, total_count }
}

// Get all albums (Not a command, keep as helper)
pub async fn get_all_albums(db: &Database) -> DbResponse<Vec<Album>> {
    let collection = db.collection::<Document>("albums");
    match collection.find(None, None).await {
        Ok(mut cursor) => {
            let mut albums: Vec<Album> = Vec::new();
            while let Ok(Some(album_doc)) = cursor.try_next().await {
                match mongodb::bson::from_document::<Album>(album_doc) {
                    Ok(album) => albums.push(album),
                    Err(e) => warn!("Failed to deserialize album document: {}", e),
                }
            }
            DbResponse {
                success: true,
                message: Some(format!("Found {} albums", albums.len())),
                id: None,
                data: Some(albums),
            }
        }
        Err(e) => {
            error!("Failed to retrieve albums: {}", e);
            DbResponse {
                success: false,
                message: Some(format!("Failed to retrieve albums: {}", e)),
                id: None,
                data: None,
            }
        }
    }
}

// Fetch all tracks with pagination and sorting - TAURI COMMAND
#[tauri::command]
pub async fn fetch_all_tracks(
    mongo_state: State<'_, MongoState>, // <-- Use State
    sort_field: String, // Pass simple types directly
    sort_direction: String,
    limit: Option<i64>,
    skip: Option<i64>,
) -> Result<TrackListResponse, CommandError> { // <-- Return local CommandError
    info!("fetch_all_tracks command: Starting with sort_field={}, sort_direction={}", sort_field, sort_direction);

    // Get Mongo client from state
    let client_lock = mongo_state.client.lock().await;
    let client = match client_lock.as_ref() {
        Some(c) => c,
        None => {
            error!("fetch_all_tracks command: MongoDB client not initialized");
            return Err(CommandError::Configuration("MongoDB client not initialized".to_string()));
        }
    };
    let db = client.database("music_library"); // Get Database instance

    let tracks_collection: Collection<Document> = db.collection("tracks");
    let albums_collection: Collection<Document> = db.collection("albums"); // Needed for album names

    // Determine sort order
    let sort_order = if sort_direction == "desc" { -1 } else { 1 };
    let sort_doc = doc! { sort_field: sort_order };
    info!("fetch_all_tracks command: Using sort document: {:?}", sort_doc);

    let find_options = FindOptions::builder()
        .sort(sort_doc)
        .limit(limit)
        .skip(skip.map(|s| s as u64))
        .build();

    // Get total count first for pagination
    let total_count = match tracks_collection.count_documents(None, None).await {
        Ok(count) => {
            info!("fetch_all_tracks command: Total track count: {}", count);
            count as usize
        },
        Err(e) => {
            error!("fetch_all_tracks command: Failed to count documents: {}", e);
            return Err(CommandError::Database(format!("Failed to count documents: {}", e)));
        }
    };

    info!("fetch_all_tracks command: Executing find() with options: {:?}", find_options);
    let cursor_result = tracks_collection.find(None, find_options).await;

    let mut cursor = match cursor_result {
        Ok(cursor) => {
            info!("fetch_all_tracks command: Cursor obtained successfully");
            cursor
        },
        Err(e) => {
            error!("fetch_all_tracks command: Failed to execute find query: {}", e);
            return Err(CommandError::Database(format!("Failed to fetch tracks: {:?}", e)));
        }
    };

    let mut tracks_with_album: Vec<TrackWithAlbum> = Vec::new();
    let mut processed_count = 0;

    while let Ok(Some(track_doc)) = cursor.try_next().await {
        processed_count += 1;
        // info!("fetch_all_tracks command: Processing track {}/{}", processed_count, total_count); // Less verbose logging

        let track_data = match mongodb::bson::from_document::<TrackDocument>(track_doc.clone()) {
             Ok(data) => data,
             Err(e) => {
                 warn!("fetch_all_tracks command: Failed to deserialize track doc: {}. Doc: {:?}", e, track_doc);
                 continue;
             }
         };

        // Fetch album name
        let album_name = if !track_data.album_id.is_empty() {
            let album_filter = doc! { "_id": &track_data.album_id };
            match albums_collection.find_one(album_filter, None).await {
                Ok(Some(album_doc)) => {
                    album_doc.get_str("name").unwrap_or("Unknown Album").to_string()
                },
                Ok(None) => {
                    warn!("fetch_all_tracks command: Album not found for ID: {}", track_data.album_id);
                    "Unknown Album".to_string()
                },
                Err(e) => {
                    error!("fetch_all_tracks command: Error fetching album {}: {}", track_data.album_id, e);
                    "Error Fetching Album".to_string()
                }
            }
        } else {
            warn!("fetch_all_tracks command: Track {} has empty album_id", track_data._id);
            "No Album ID".to_string()
        };

        // Convert TrackDocument to TrackWithAlbum
        let track_with_album = TrackWithAlbum {
            id: track_data._id,
            title: track_data.title,
            album_id: track_data.album_id,
            album_name,
            track_number: track_data.track_number,
            filename: track_data.filename,
            duration: Some(track_data.duration),
            writers: track_data.writers,
            writer_percentages: track_data.writer_percentages,
            publishers: track_data.publishers,
            publisher_percentages: track_data.publisher_percentages,
            composers: track_data.composers,
            genre: track_data.genre,
            path: track_data.path,
            waveform_data: track_data.waveform_data,
            comments: track_data.comments,
        };
        tracks_with_album.push(track_with_album);
    }
     info!("fetch_all_tracks command: Processed {} tracks successfully", tracks_with_album.len());

    Ok(TrackListResponse {
        success: true,
        message: None,
        tracks: tracks_with_album,
        total_count,
    })
}

/// Updates the metadata for a track in the database - TAURI COMMAND
#[tauri::command]
pub async fn update_track_metadata(
    mongo_state: State<'_, MongoState>, // <-- Use State
    track_id: String, // Pass simple types
    payload: UpdateTrackPayload, // Pass payload struct
) -> Result<(), CommandError> { // <-- Return local CommandError
    info!("update_track_metadata command: Updating track_id: {}", track_id);

    // Get Mongo client from state
    let client_lock = mongo_state.client.lock().await;
    let client = match client_lock.as_ref() {
        Some(c) => c,
        None => {
            error!("update_track_metadata command: MongoDB client not initialized");
            return Err(CommandError::Configuration("MongoDB client not initialized".to_string()));
        }
    };
    let db = client.database("music_library"); // Get Database instance

    // Convert string ID to ObjectId
    let object_id = match bson::oid::ObjectId::parse_str(&track_id) {
        Ok(id) => id,
        Err(e) => {
            error!("Invalid ObjectId format for track_id: {}", e);
            return Err(CommandError::Validation(format!("Invalid track ID format: {}", e)));
        }
    };

    let tracks_collection = db.collection::<Document>("tracks");

    // Build update document based on provided fields in payload
    let mut update_doc = Document::new();

    if let Some(title) = &payload.title {
        update_doc.insert("title", title);
    }

    if let Some(writers) = &payload.writers {
        update_doc.insert("writers", to_bson(writers).map_err(|e| {
            error!("Failed to convert writers to BSON: {}", e);
            CommandError::Database(format!("Failed to convert writers to BSON: {}", e))
        })?);
    }

    if let Some(publisher_percentages) = &payload.publisher_percentages {
        update_doc.insert("publisher_percentages", to_bson(publisher_percentages).map_err(|e| {
            error!("Failed to convert publisher_percentages to BSON: {}", e);
            CommandError::Database(format!("Failed to convert publisher_percentages to BSON: {}", e))
        })?);
    }

    if let Some(writer_percentages) = &payload.writer_percentages {
        update_doc.insert("writer_percentages", to_bson(writer_percentages).map_err(|e| {
            error!("Failed to convert writer_percentages to BSON: {}", e);
            CommandError::Database(format!("Failed to convert writer_percentages to BSON: {}", e))
        })?);
    }

    if let Some(publishers) = &payload.publishers {
        update_doc.insert("publishers", to_bson(publishers).map_err(|e| {
            error!("Failed to convert publishers to BSON: {}", e);
            CommandError::Database(format!("Failed to convert publishers to BSON: {}", e))
        })?);
    }

    // REMOVED composers block - Field does not exist on UpdateTrackPayload

    if let Some(genre) = &payload.genre {
        update_doc.insert("genre", to_bson(genre).map_err(|e| {
            error!("Failed to convert genre to BSON: {}", e);
            CommandError::Database(format!("Failed to convert genre to BSON: {}", e))
        })?);
    }

     if let Some(comments) = &payload.comments {
        update_doc.insert("comments", comments);
    }

    // REMOVED track_number block - Field does not exist on UpdateTrackPayload


    // Only update if there are fields to change
    if !update_doc.is_empty() {
        let update = doc! { "$set": update_doc };
        match tracks_collection.update_one(doc! { "_id": object_id }, update, None).await {
            Ok(result) => {
                if result.matched_count == 0 {
                    error!("Track not found for update: {}", track_id);
                    return Err(CommandError::NotFound(format!("Track not found: {}", track_id)));
                }
                info!("Successfully updated metadata for track: {}", track_id);
            }
            Err(e) => {
                error!("Failed to update track metadata in MongoDB: {}", e);
                return Err(CommandError::Database(format!("Failed to update track: {}", e)));
            }
        }
    } else {
        info!("No metadata fields provided to update for track: {}", track_id);
    }

    Ok(())
}
