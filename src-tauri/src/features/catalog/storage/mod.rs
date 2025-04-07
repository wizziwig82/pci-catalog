// pub mod r2; // R2 logic likely belongs elsewhere (e.g., core or upload feature)
pub mod mongodb;
pub mod catalog_storage_actions; // Declare the new module
// Removed re-exports, will use full paths in commands.rs
// pub use r2::R2Client; // Remove R2 re-export

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Payload for updating track metadata selectively
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateTrackPayload {
    pub title: Option<String>,
    pub genre: Option<Vec<String>>,
    pub writers: Option<Vec<String>>,
    pub writer_percentages: Option<HashMap<String, f32>>, // Match TrackDocument/TrackWithAlbum
    pub publishers: Option<Vec<String>>,
    pub publisher_percentages: Option<HashMap<String, f32>>, // Match TrackDocument/TrackWithAlbum
    pub instruments: Option<Vec<String>>, // Assuming Vec<String> based on usage pattern
    pub mood: Option<Vec<String>>, // Assuming Vec<String> based on usage pattern
    pub comments: Option<String>,
    // Add other optional fields if needed for updates
}