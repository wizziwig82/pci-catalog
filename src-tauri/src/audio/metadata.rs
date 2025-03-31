use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use log::{info, error, warn};
use id3::{Tag, TagLike};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::formats::{FormatOptions, FormatReader};
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::default::get_probe;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrackMetadata {
    /// Track title
    pub title: String,
    /// Track filename
    pub filename: String,
    /// Track duration in seconds
    pub duration: Option<f64>,
    /// Track number in album
    pub track_number: Option<u32>,
    /// The album this track belongs to
    pub album_id: Option<String>,
    /// Artists who performed this track
    pub artists: Vec<String>,
    /// Original file path
    pub original_path: String,
    /// MIME type of the file
    pub mime_type: String,
    /// File size in bytes
    pub file_size: u64,
    /// Writers and their percentage shares
    pub writers: HashMap<String, f32>,
    /// Publishers and their percentage shares
    pub publishers: HashMap<String, f32>,
    /// Genre of the track
    pub genre: Vec<String>,
    /// Instruments featured in the track
    pub instruments: Vec<String>,
    /// Mood of the track
    pub mood: Vec<String>,
    /// Additional comments
    pub comments: String,
    /// Date added to library
    pub date_added: DateTime<Utc>,
    /// File extension
    pub extension: String,
    /// Unique ID for this track
    pub track_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AlbumMetadata {
    /// Album title
    pub name: String,
    /// Album artist
    pub artist: String,
    /// Year of release
    pub year: Option<i32>,
    /// Path to album artwork
    pub art_path: Option<String>,
    /// List of genres
    pub genres: Vec<String>,
    /// Date added to library
    pub date_added: DateTime<Utc>,
    /// Unique ID for this album
    pub album_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AudioMetadata {
    pub track: TrackMetadata,
    pub album: AlbumMetadata,
}

/// Extract metadata from an audio file
pub fn extract_metadata(file_path: &str) -> Result<AudioMetadata, String> {
    info!("Extracting metadata from: {}", file_path);
    
    let path = Path::new(file_path);
    if !path.exists() {
        return Err(format!("File does not exist: {}", file_path));
    }

    // Get the file extension
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();
    
    // Get the file size
    let file_size = match std::fs::metadata(path) {
        Ok(metadata) => metadata.len(),
        Err(e) => {
            error!("Failed to get file size: {}", e);
            return Err(format!("Failed to get file size: {}", e));
        }
    };
    
    // Get the file MIME type
    let mime_type = mime_guess::from_path(path)
        .first_or_octet_stream()
        .to_string();
    
    // Generate UUIDs for track and album
    let track_id = Uuid::new_v4().to_string();
    let album_id = Uuid::new_v4().to_string();
    
    // Initialize with default values
    let mut track_metadata = TrackMetadata {
        title: path
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("Unknown Title")
            .to_string(),
        filename: path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown.mp3")
            .to_string(),
        duration: None,
        track_number: None,
        album_id: Some(album_id.clone()),
        artists: Vec::new(),
        original_path: file_path.to_string(),
        mime_type,
        file_size,
        writers: HashMap::new(),
        publishers: HashMap::new(),
        genre: Vec::new(),
        instruments: Vec::new(),
        mood: Vec::new(),
        comments: String::new(),
        date_added: Utc::now(),
        extension,
        track_id,
    };
    
    let mut album_metadata = AlbumMetadata {
        name: "Unknown Album".to_string(),
        artist: "Unknown Artist".to_string(),
        year: None,
        art_path: None,
        genres: Vec::new(),
        date_added: Utc::now(),
        album_id,
    };
    
    // Try to get ID3 tags first (MP3 files)
    if let Ok(tag) = Tag::read_from_path(path) {
        // Extract ID3 metadata
        if let Some(title) = tag.title() {
            track_metadata.title = title.to_string();
        }

        if let Some(artist) = tag.artist() {
            album_metadata.artist = artist.to_string();
            track_metadata.artists.push(artist.to_string());
        }

        if let Some(album) = tag.album() {
            album_metadata.name = album.to_string();
        }

        if let Some(track_number) = tag.track() {
            track_metadata.track_number = Some(track_number);
        }

        if let Some(year) = tag.year() {
            album_metadata.year = Some(year);
        }

        if let Some(genre) = tag.genre() {
            track_metadata.genre.push(genre.to_string());
            album_metadata.genres.push(genre.to_string());
        }

        if let Some(comment) = tag.comments().next() {
            track_metadata.comments = comment.text.clone();
        }
    } 
    
    // Try to extract duration using symphonia (for all audio formats)
    match extract_duration_symphonia(file_path) {
        Ok(duration) => {
            track_metadata.duration = Some(duration);
        },
        Err(e) => {
            warn!("Failed to extract duration: {}", e);
        }
    }
    
    // Return the combined metadata
    Ok(AudioMetadata {
        track: track_metadata,
        album: album_metadata,
    })
}

fn extract_duration_symphonia(file_path: &str) -> Result<f64, String> {
    // Open the media file
    let file = match File::open(file_path) {
        Ok(file) => file,
        Err(e) => return Err(format!("Failed to open file: {}", e)),
    };
    
    // Create a MediaSourceStream
    let source = MediaSourceStream::new(Box::new(file), Default::default());
    
    // Create a hint to help the format registry
    let mut hint = Hint::new();
    
    // Add file extension hint if available
    if let Some(extension) = Path::new(file_path).extension() {
        if let Some(ext_str) = extension.to_str() {
            hint.with_extension(ext_str);
        }
    }
    
    // Use the default format registry
    let format_opts = FormatOptions {
        enable_gapless: true,
        ..Default::default()
    };
    
    let metadata_opts = MetadataOptions::default();
    
    // Probe the format
    let probe_result = match get_probe().format(&hint, source, &format_opts, &metadata_opts) {
        Ok(probe_result) => probe_result,
        Err(e) => return Err(format!("Failed to probe format: {}", e)),
    };
    
    // Get the format reader
    let format = probe_result.format;
    
    // Get the default track
    let track = match format.default_track() {
        Some(track) => track,
        None => return Err("No default track found".to_string()),
    };
    
    // Get the track timebase
    let timebase = match track.codec_params.time_base {
        Some(timebase) => timebase,
        None => return Err("No timebase found".to_string()),
    };
    
    // Get the track duration
    let duration = match track.codec_params.n_frames {
        Some(n_frames) => {
            let time = n_frames as f64 * timebase.numer as f64 / timebase.denom as f64;
            time
        },
        None => return Err("No frames count found".to_string()),
    };
    
    Ok(duration)
} 