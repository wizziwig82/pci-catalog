use std::path::PathBuf;
use serde::Serialize; // Re-add Serialize import
use thiserror::Error;

/// Errors that can occur during the audio transcoding process.
#[derive(Debug, Error, Serialize)] // Re-add Serialize derive
pub enum TranscodingError {
    #[error("FFmpeg command not found. Please ensure FFmpeg is installed and in the system's PATH.")]
    FFmpegNotFound,

    #[error("Input file not found: {0}")]
    InputFileNotFound(PathBuf),

    // Store IO error message as String for serialization
    #[error("Failed to create output directory for {path}: {source_message}")]
    OutputDirectoryCreationFailed {
        path: PathBuf,
        source_message: String,
    },

    // Store IO error message as String for serialization
    #[error("FFmpeg process failed to start: {source_message}")]
    ProcessStartFailed { source_message: String },

    // Use {:?} to debug print the Option<i32> status code
    #[error("FFmpeg execution failed. Exit code: {status:?}. Stderr: {stderr}")]
    ProcessExecutionFailed {
        status: Option<i32>,
        stderr: String,
    },

    // Store IO error message as String for serialization
    #[error("Failed to read FFmpeg stderr: {source_message}")]
    StderrReadFailed { source_message: String },

    // Store IO error message as String for serialization
    #[error("An unexpected I/O error occurred: {source_message}")]
    IoError { source_message: String },
}

// --- Conversion from std::io::Error ---
// Implement From manually to convert io::Error into the String variants

impl From<std::io::Error> for TranscodingError {
    fn from(error: std::io::Error) -> Self {
        // Basic conversion, context might be lost.
        // Consider where the error originated if more specific variants are needed.
        TranscodingError::IoError { source_message: error.to_string() }
    }
}

// --- Helper to create specific variants from IO errors with context ---

impl TranscodingError {
    pub fn output_dir_creation_failed(path: PathBuf, error: std::io::Error) -> Self {
        TranscodingError::OutputDirectoryCreationFailed {
            path,
            source_message: error.to_string(),
        }
    }

    pub fn process_start_failed(error: std::io::Error) -> Self {
        TranscodingError::ProcessStartFailed {
            source_message: error.to_string(),
        }
    }

     pub fn stderr_read_failed(error: std::io::Error) -> Self {
        TranscodingError::StderrReadFailed {
            source_message: error.to_string(),
        }
    }
}