pub mod metadata;
pub mod transcoding;

pub use metadata::{extract_metadata, AudioMetadata, TrackMetadata, AlbumMetadata};
pub use transcoding::{transcode_file, TranscodingOptions, TranscodingResult}; 