use crate::features::upload::UploadItemMetadata; // Updated path
use std::path::Path;
use std::fs::File;
// Removed unused Read import
// Removed unused HashMap import
use serde::{Serialize, Deserialize}; // Keep for UploadItemMetadata if it derives Serialize/Deserialize
use log::{info, error, warn};
use id3::{Tag, TagLike};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::formats::{FormatOptions, FormatReader};
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::default::get_probe;
// Removed unused Uuid import
// Removed unused chrono imports

// Removed internal TrackMetadata, AlbumMetadata, and AudioMetadata structs
// as we now return UploadItemMetadata directly.

/// Extract metadata from an audio file
#[tauri::command]
pub fn extract_metadata(filePath: String) -> Result<UploadItemMetadata, String> { // Changed parameter to filePath
    // Original function body restored:
    info!("Extracting metadata from: {}", filePath);

    let path = Path::new(&filePath);
    if !path.exists() {
        error!("File does not exist: {}", filePath); // Changed to error! for clarity
        return Err(format!("File does not exist: {}", filePath));
    }

    // Initialize UploadItemMetadata with None values
    let mut metadata = UploadItemMetadata {
        title: None,
        artist: None,
        album: None,
        track_number: None,
        duration_sec: None,
        genre: None,
        composer: None, // Composer extraction not implemented here yet
        year: None,
        comments: None,
    };

    // --- Extract Duration using Symphonia ---
    match extract_duration_symphonia(&filePath) {
        Ok(duration) => {
            metadata.duration_sec = Some(duration);
            info!("Extracted duration (Symphonia): {}s for {}", duration, filePath);
        },
        Err(e) => {
            warn!("Failed to extract duration using Symphonia for {}: {}", filePath, e);
            // Continue without duration if extraction fails
        }
    }

    // --- Extract Metadata using ID3 ---
    // Attempt to read ID3 tags (common for MP3)
    match Tag::read_from_path(path) {
        Ok(tag) => {
            info!("Successfully read ID3 tags for: {}", filePath);
            metadata.title = tag.title().map(String::from);
            metadata.artist = tag.artist().map(String::from);
            metadata.album = tag.album().map(String::from);
            metadata.track_number = tag.track();
            metadata.year = tag.year();
            metadata.genre = tag.genre().map(String::from);
            // Get the first comment if available
            metadata.comments = tag.comments().next().map(|c| c.text.clone());

            // Log missing fields
            if metadata.title.is_none() { warn!("ID3: Title missing for {}", filePath); }
            if metadata.artist.is_none() { warn!("ID3: Artist missing for {}", filePath); }
            if metadata.album.is_none() { warn!("ID3: Album missing for {}", filePath); }
        }
        Err(e) => {
            warn!("Failed to read ID3 tags for {}: {}", filePath, e);
            // If ID3 fails, try falling back to filename for title if still None
            if metadata.title.is_none() {
                 metadata.title = path.file_stem().and_then(|s| s.to_str()).map(String::from);
                 if let Some(ref title) = metadata.title { // Use if let for cleaner logging
                     info!("Fell back to filename for title: {}", title);
                 } else {
                     warn!("Could not determine title from filename either for {}", filePath);
                 }
            }
        }
    }

    // --- Fallback Title (if still None) ---
    if metadata.title.is_none() {
        metadata.title = Some("Unknown Title".to_string());
        warn!("Setting title to 'Unknown Title' for {}", filePath);
    }

    // Note: Symphonia can also extract metadata, potentially supporting more formats.
    // This could be added as a fallback or alternative if ID3 fails or for non-MP3 files.
    // For now, we rely primarily on ID3 and Symphonia for duration.

    info!("Finished extracting metadata for {}: {:?}", filePath, metadata);
    Ok(metadata)
}

fn extract_duration_symphonia(filePath: &str) -> Result<f64, String> {
    // Open the media file
    let file = match File::open(filePath) {
        Ok(file) => file,
        Err(e) => return Err(format!("Failed to open file: {}", e)),
    };
    
    // Create a MediaSourceStream
    let source = MediaSourceStream::new(Box::new(file), Default::default());
    
    // Create a hint to help the format registry
    let mut hint = Hint::new();
    
    // Add file extension hint if available
    if let Some(extension) = Path::new(filePath).extension() {
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