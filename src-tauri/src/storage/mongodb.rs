use mongodb::{
    bson::{doc, Document, to_bson},
    options::{ClientOptions, IndexOptions},
    IndexModel,
    Client, Collection, Database,
};
use futures_util::stream::TryStreamExt;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::sync::Arc;

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