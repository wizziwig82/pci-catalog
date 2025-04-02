use mongodb::{
    bson::{doc, Document, to_bson},
    options::{ClientOptions, IndexOptions, FindOptions},
    IndexModel,
    Client, Collection, Database,
};
use futures_util::stream::TryStreamExt;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::sync::Arc;
use log;
use std::collections::HashMap;

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
    pub genre: Option<String>,
    pub path: String,
    pub waveform_data: Option<Vec<i32>>,
}

// Track list response structure for returning track data with album details
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrackWithAlbum {
    pub id: String,
    pub title: String,
    pub album_id: String,
    pub album_name: String,
    pub track_number: Option<i32>,
    pub filename: String,
    pub duration: Option<i32>,
    pub writers: Vec<String>,
    pub writer_percentages: Option<HashMap<String, f32>>,
    pub publishers: Vec<String>,
    pub composers: Option<Vec<String>>,
    pub genre: Option<String>,
    pub path: String,
    pub waveform_data: Option<Vec<f32>>,
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
    pub _id: String,
    pub title: String,
    pub album_id: String,
    pub track_number: Option<i32>,
    pub filename: String,
    pub duration: i32,
    pub writers: Vec<String>,
    pub publishers: Vec<String>,
    pub composers: Option<Vec<String>>,
    pub genre: Option<String>,
    pub path: String,
    pub waveform_data: Option<Vec<f32>>,
}

// MongoDB Client wrapper
pub struct MongoClient {
    client: Client,
    db: Database,
}

// Response type for database operations
#[derive(Debug, Serialize)]
pub struct DbResponse<T> {
    pub success: bool,
    pub message: Option<String>,
    pub id: Option<String>,
    pub data: Option<T>,
}

impl MongoClient {
    // Test the MongoDB connection
    pub async fn test_connection(&self) -> DbResponse<()> {
        match self.client.list_database_names(None, None).await {
            Ok(_) => DbResponse {
                success: true,
                message: Some("Connected to MongoDB successfully".to_string()),
                id: None,
                data: None,
            },
            Err(e) => DbResponse {
                success: false,
                message: Some(format!("Failed to connect to MongoDB: {}", e)),
                id: None,
                data: None,
            },
        }
    }

    // Close the MongoDB connection
    pub async fn close(self) {
        // MongoDB client is automatically closed when dropped
    }

    // Get collections
    pub fn albums_collection(&self) -> Collection<Document> {
        self.db.collection("albums")
    }

    pub fn tracks_collection(&self) -> Collection<Document> {
        self.db.collection("tracks")
    }
}

// Initialize the MongoDB client
pub async fn initialize_mongo_client(
    credentials: MongoCredentials,
) -> Result<Arc<MongoClient>, Box<dyn Error + Send + Sync>> {
    let client_options = ClientOptions::parse(&credentials.uri).await?;
    let client = Client::with_options(client_options)?;
    
    // Use music_library as the database name
    let db = client.database("music_library");

    // Create indexes for search functionality
    create_indexes(&db).await?;

    Ok(Arc::new(MongoClient { client, db }))
}

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

// Album CRUD operations
pub async fn create_album(
    client: &Arc<MongoClient>,
    album_id: &str,
    album_data: Album,
) -> DbResponse<()> {
    let collection = client.albums_collection();
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

pub async fn get_album(client: &Arc<MongoClient>, album_id: &str) -> DbResponse<Album> {
    let collection = client.albums_collection();
    match collection.find_one(doc! { "_id": album_id }, None).await {
        Ok(Some(album_doc)) => {
            match serde_json::from_str::<Album>(&serde_json::to_string(&album_doc).unwrap()) {
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
    client: &Arc<MongoClient>,
    album_id: &str,
    album_data: Album,
) -> DbResponse<()> {
    let collection = client.albums_collection();
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

pub async fn delete_album(client: &Arc<MongoClient>, album_id: &str) -> DbResponse<()> {
    let collection = client.albums_collection();
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

// Track CRUD operations
pub async fn create_track(
    client: &Arc<MongoClient>,
    track_id: &str,
    track_data: Track,
) -> DbResponse<()> {
    let collection = client.tracks_collection();
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

pub async fn get_track(client: &Arc<MongoClient>, track_id: &str) -> DbResponse<Track> {
    let collection = client.tracks_collection();
    match collection.find_one(doc! { "_id": track_id }, None).await {
        Ok(Some(track_doc)) => {
            match serde_json::from_str::<Track>(&serde_json::to_string(&track_doc).unwrap()) {
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
    client: &Arc<MongoClient>,
    track_id: &str,
    track_data: Track,
) -> DbResponse<()> {
    let collection = client.tracks_collection();
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

pub async fn delete_track(client: &Arc<MongoClient>, track_id: &str) -> DbResponse<()> {
    let collection = client.tracks_collection();
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

// Search functionality
pub async fn search_tracks(
    client: &Arc<MongoClient>,
    query: &str,
) -> DbResponse<Vec<Track>> {
    let collection = client.tracks_collection();
    let filter = doc! {
        "$text": {
            "$search": query
        }
    };

    match collection.find(filter, None).await {
        Ok(cursor) => {
            match cursor.try_collect::<Vec<Document>>().await {
                Ok(docs) => {
                    let mut tracks: Vec<Track> = Vec::new();
                    for doc in docs {
                        if let Ok(track) = serde_json::from_str::<Track>(&serde_json::to_string(&doc).unwrap()) {
                            tracks.push(track);
                        }
                    }
                    DbResponse {
                        success: true,
                        message: Some(format!("Found {} tracks", tracks.len())),
                        id: None,
                        data: Some(tracks),
                    }
                },
                Err(e) => {
                    DbResponse {
                        success: false,
                        message: Some(format!("Failed to collect tracks: {}", e)),
                        id: None,
                        data: None,
                    }
                }
            }
        }
        Err(e) => DbResponse {
            success: false,
            message: Some(format!("Failed to search tracks: {}", e)),
            id: None,
            data: None,
        },
    }
}

pub async fn search_albums(
    client: &Arc<MongoClient>,
    query: &str,
) -> DbResponse<Vec<Album>> {
    let collection = client.albums_collection();
    let filter = doc! {
        "$text": {
            "$search": query
        }
    };

    match collection.find(filter, None).await {
        Ok(cursor) => {
            match cursor.try_collect::<Vec<Document>>().await {
                Ok(docs) => {
                    let mut albums: Vec<Album> = Vec::new();
                    for doc in docs {
                        if let Ok(album) = serde_json::from_str::<Album>(&serde_json::to_string(&doc).unwrap()) {
                            albums.push(album);
                        }
                    }
                    DbResponse {
                        success: true,
                        message: Some(format!("Found {} albums", albums.len())),
                        id: None,
                        data: Some(albums),
                    }
                },
                Err(e) => {
                    DbResponse {
                        success: false,
                        message: Some(format!("Failed to collect albums: {}", e)),
                        id: None,
                        data: None,
                    }
                }
            }
        }
        Err(e) => DbResponse {
            success: false,
            message: Some(format!("Failed to search albums: {}", e)),
            id: None,
            data: None,
        },
    }
}

// Get all tracks for a specific album
pub async fn get_tracks_by_album(
    client: &Arc<MongoClient>,
    album_id: &str,
) -> DbResponse<Vec<Track>> {
    let collection = client.tracks_collection();
    match collection
        .find(doc! { "album_id": album_id }, None)
        .await
    {
        Ok(cursor) => {
            match cursor.try_collect::<Vec<Document>>().await {
                Ok(docs) => {
                    let mut tracks: Vec<Track> = Vec::new();
                    for doc in docs {
                        if let Ok(track) = serde_json::from_str::<Track>(&serde_json::to_string(&doc).unwrap()) {
                            tracks.push(track);
                        }
                    }
                    DbResponse {
                        success: true,
                        message: Some(format!("Found {} tracks for album", tracks.len())),
                        id: None,
                        data: Some(tracks),
                    }
                },
                Err(e) => {
                    DbResponse {
                        success: false,
                        message: Some(format!("Failed to collect tracks: {}", e)),
                        id: None,
                        data: None,
                    }
                }
            }
        }
        Err(e) => DbResponse {
            success: false,
            message: Some(format!("Failed to get tracks for album: {}", e)),
            id: None,
            data: None,
        },
    }
}

// Get all albums
pub async fn get_all_albums(client: &Arc<MongoClient>) -> DbResponse<Vec<Album>> {
    let collection = client.albums_collection();
    match collection.find(None, None).await {
        Ok(cursor) => {
            match cursor.try_collect::<Vec<Document>>().await {
                Ok(docs) => {
                    let mut albums: Vec<Album> = Vec::new();
                    for doc in docs {
                        if let Ok(album) = serde_json::from_str::<Album>(&serde_json::to_string(&doc).unwrap()) {
                            albums.push(album);
                        }
                    }
                    DbResponse {
                        success: true,
                        message: Some(format!("Found {} albums", albums.len())),
                        id: None,
                        data: Some(albums),
                    }
                },
                Err(e) => {
                    DbResponse {
                        success: false,
                        message: Some(format!("Failed to collect albums: {}", e)),
                        id: None,
                        data: None,
                    }
                }
            }
        }
        Err(e) => DbResponse {
            success: false,
            message: Some(format!("Failed to get all albums: {}", e)),
            id: None,
            data: None,
        },
    }
}

// Function to get all tracks with album names
pub async fn get_all_tracks(
    db: &Database,
    sort_field: &str,
    sort_direction: &str,
    limit: Option<i64>,
    skip: Option<i64>,
) -> TrackListResponse {
    use log::{info, error, warn};
    
    info!("get_all_tracks: Starting to fetch tracks with sort_field={}, sort_direction={}", sort_field, sort_direction);
    
    let tracks_collection: Collection<Document> = db.collection("tracks");
    let albums_collection: Collection<Document> = db.collection("albums");
    
    // Create sort document based on parameters
    let sort_value = if sort_direction == "desc" { -1 } else { 1 };
    let sort_doc = match sort_field {
        "title" => doc! { "title": sort_value },
        "duration" => doc! { "duration": sort_value },
        "genre" => doc! { "genre": sort_value },
        // For album name, we'll handle it after fetching since it requires a join
        "album.name" => doc! { "album_id": sort_value },
        _ => doc! { "title": 1 }  // Default sort by title ascending
    };
    
    info!("get_all_tracks: Using sort document: {:?}", sort_doc);
    
    // Set up find options with sort, limit, and skip
    let mut find_options = FindOptions::builder()
        .sort(sort_doc)
        .build();
    
    // Add limit if provided
    if let Some(limit_val) = limit {
        if limit_val > 0 {
            info!("get_all_tracks: Setting limit to {}", limit_val);
            find_options.limit = Some(limit_val);
        }
    }
    
    // Add skip if provided
    if let Some(skip_val) = skip {
        if skip_val > 0 {
            info!("get_all_tracks: Setting skip to {}", skip_val);
            // Convert to u64, since FindOptions.skip requires u64
            find_options.skip = Some(skip_val as u64);
        }
    }
    
    // Get count of all tracks (for pagination)
    let total_count = match tracks_collection.count_documents(None, None).await {
        Ok(count) => {
            info!("get_all_tracks: Total track count in database: {}", count);
            count as usize
        },
        Err(e) => {
            error!("get_all_tracks: Failed to count documents: {}", e);
            0 // Default to 0 if count fails
        }
    };
    
    if total_count == 0 {
        info!("get_all_tracks: No tracks found in database");
        return TrackListResponse {
            success: true,
            message: Some("No tracks found in database".to_string()),
            tracks: Vec::new(),
            total_count: 0,
        };
    }
    
    // Fetch tracks
    let mut tracks_with_albums: Vec<TrackWithAlbum> = Vec::new();
    
    info!("get_all_tracks: Executing find() with options: {:?}", find_options);
    
    match tracks_collection.find(None, Some(find_options)).await {
        Ok(cursor) => {
            info!("get_all_tracks: Cursor obtained successfully, collecting results");
            let tracks: Vec<Document> = match cursor.try_collect::<Vec<Document>>().await {
                Ok(tracks) => {
                    info!("get_all_tracks: Collected {} tracks from cursor", tracks.len());
                    tracks
                },
                Err(e) => {
                    error!("get_all_tracks: Failed to collect tracks from cursor: {}", e);
                    return TrackListResponse {
                        success: false,
                        message: Some(format!("Failed to collect tracks: {}", e)),
                        tracks: Vec::new(),
                        total_count: 0,
                    };
                }
            };
            
            if tracks.is_empty() {
                info!("get_all_tracks: No tracks returned from query");
                return TrackListResponse {
                    success: true,
                    message: Some("No tracks found for the given query".to_string()),
                    tracks: Vec::new(),
                    total_count: total_count,
                };
            }
            
            // Process each track
            for (index, track_doc) in tracks.iter().enumerate() {
                info!("get_all_tracks: Processing track {}/{}", index + 1, tracks.len());
                
                // Extract track ID and album_id from the document
                let track_id = match track_doc.get("_id") {
                    Some(id) => {
                        let id_str = id.to_string().replace("\"", "");
                        info!("get_all_tracks: Found track ID: {}", id_str);
                        id_str
                    },
                    None => {
                        warn!("get_all_tracks: Track without ID found, skipping");
                        continue; // Skip tracks without ID
                    }
                };
                
                let album_id = match track_doc.get("album_id") {
                    Some(id) => {
                        let id_str = id.to_string().replace("\"", "");
                        info!("get_all_tracks: Track has album_id: {}", id_str);
                        id_str
                    },
                    None => {
                        info!("get_all_tracks: Track has no album_id");
                        "".to_string() // Set empty string for tracks without album_id
                    }
                };
                
                // If we have an album_id, fetch the album name
                let mut album_name = None;
                if !album_id.is_empty() {
                    info!("get_all_tracks: Fetching album name for album_id: {}", album_id);
                    match albums_collection
                        .find_one(doc! { "_id": &album_id }, None)
                        .await
                    {
                        Ok(Some(album_doc)) => {
                            if let Some(name) = album_doc.get("name") {
                                let name_str = name.to_string().replace("\"", "");
                                info!("get_all_tracks: Found album name: {}", name_str);
                                album_name = Some(name_str);
                            } else {
                                warn!("get_all_tracks: Album document has no name field");
                            }
                        }
                        Ok(None) => warn!("get_all_tracks: No album found with ID: {}", album_id),
                        Err(e) => error!("get_all_tracks: Error fetching album: {}", e),
                    }
                }
                
                // Try to convert the track document to TrackWithAlbum
                info!("get_all_tracks: Converting track document to TrackWithAlbum");
                match serde_json::from_str::<TrackDocument>(&serde_json::to_string(&track_doc).unwrap()) {
                    Ok(doc) => {
                        // Map the TrackDocument to TrackWithAlbum
                        let track = TrackWithAlbum {
                            id: doc._id,
                            title: doc.title,
                            album_id: doc.album_id,
                            album_name: album_name.unwrap_or_default(),
                            track_number: doc.track_number,
                            filename: doc.filename,
                            duration: Some(doc.duration),
                            writers: doc.writers,
                            writer_percentages: None, // Not in the MongoDB document
                            publishers: doc.publishers,
                            composers: doc.composers,
                            genre: doc.genre,
                            path: doc.path,
                            waveform_data: doc.waveform_data,
                        };
                        
                        info!("get_all_tracks: Successfully converted track: {}", track.title);
                        tracks_with_albums.push(track);
                    }
                    Err(e) => {
                        error!("get_all_tracks: Failed to parse track document: {}", e);
                        warn!("get_all_tracks: Track document: {:?}", track_doc);
                        continue; // Skip tracks that can't be parsed
                    }
                }
            }
            
            info!("get_all_tracks: Processed {} tracks successfully", tracks_with_albums.len());
            
            // If sorting by album name, do it here
            if sort_field == "album.name" {
                info!("get_all_tracks: Sorting by album name");
                tracks_with_albums.sort_by(|a, b| {
                    if sort_direction == "desc" {
                        b.album_name.cmp(&a.album_name)
                    } else {
                        a.album_name.cmp(&b.album_name)
                    }
                });
            }
            
            TrackListResponse {
                success: true,
                message: Some(format!("Successfully retrieved {} tracks", tracks_with_albums.len())),
                tracks: tracks_with_albums,
                total_count,
            }
        }
        Err(e) => {
            error!("get_all_tracks: Failed to execute find query: {}", e);
            TrackListResponse {
                success: false,
                message: Some(format!("Failed to fetch tracks: {}", e)),
                tracks: Vec::new(),
                total_count: 0,
            }
        },
    }
} 