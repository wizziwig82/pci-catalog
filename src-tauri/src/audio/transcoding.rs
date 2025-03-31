use std::path::{Path, PathBuf};
use std::fs;
use std::process::Command;
use serde::{Serialize, Deserialize};
use log::{info, error, warn};
use anyhow::{Result, anyhow, Context};
use thiserror::Error;
use tempfile::TempDir;
use regex::Regex;

#[derive(Debug, Error)]
pub enum TranscodingError {
    #[error("FFmpeg error: {0}")]
    FFmpegError(String),
    
    #[error("Input file error: {0}")]
    InputFileError(String),
    
    #[error("Output file error: {0}")]
    OutputFileError(String),
    
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Other error: {0}")]
    Other(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TranscodingOptions {
    /// Target audio bitrate for medium quality in kbps
    pub medium_bitrate: u32,
    /// Output format (e.g., "mp3", "aac")
    pub format: String,
    /// Output directory for transcoded files
    pub output_dir: String,
}

impl Default for TranscodingOptions {
    fn default() -> Self {
        Self {
            medium_bitrate: 128,
            format: "mp3".to_string(),
            output_dir: "transcoded".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TranscodingResult {
    pub success: bool,
    pub input_path: String,
    pub medium_quality_path: Option<String>,
    pub error_message: Option<String>,
}

/// Check if ffmpeg is installed and available
fn check_ffmpeg() -> Result<(), TranscodingError> {
    let output = Command::new("ffmpeg")
        .arg("-version")
        .output()
        .map_err(|e| TranscodingError::FFmpegError(format!("Failed to run ffmpeg: {}", e)))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(TranscodingError::FFmpegError(format!("ffmpeg check failed: {}", stderr)));
    }
    
    Ok(())
}

/// Check if the format is supported by ffmpeg
fn check_format(format: &str) -> Result<(), TranscodingError> {
    // Common audio formats that are typically supported by ffmpeg
    let supported_formats = ["mp3", "aac", "m4a", "ogg", "flac", "wav"];
    
    if !supported_formats.contains(&format.to_lowercase().as_str()) {
        return Err(TranscodingError::InvalidFormat(format!("Unsupported format: {}", format)));
    }
    
    Ok(())
}

/// Transcode an audio file to medium quality and return paths to both original and transcoded files
pub fn transcode_file(
    input_path: &str,
    options: TranscodingOptions,
) -> Result<TranscodingResult, TranscodingError> {
    // Check if ffmpeg is available
    info!("Starting transcoding process for file: {}", input_path);
    info!("DEBUG: Using options: bitrate={}, format={}, output_dir={}", 
          options.medium_bitrate, options.format, options.output_dir);
    
    match check_ffmpeg() {
        Ok(_) => info!("FFmpeg check passed"),
        Err(e) => {
            error!("FFmpeg check failed: {}", e);
            return Err(e);
        }
    }
    
    // Check if the format is supported
    info!("Checking format: {}", options.format);
    match check_format(&options.format) {
        Ok(_) => info!("Format check passed for: {}", options.format),
        Err(e) => {
            error!("Format check failed: {}", e);
            return Err(e);
        }
    }
    
    let input_path = Path::new(input_path);
    
    // Verify the input file exists
    if !input_path.exists() {
        let err = TranscodingError::InputFileError(format!("Input file does not exist: {}", input_path.display()));
        error!("{}", err);
        return Err(err);
    }
    
    info!("Input file exists: {}", input_path.display());
    
    // Verify we can read the input file
    match fs::metadata(input_path) {
        Ok(metadata) => {
            if metadata.len() == 0 {
                let err = TranscodingError::InputFileError(format!("Input file is empty: {}", input_path.display()));
                error!("{}", err);
                return Err(err);
            }
            info!("Input file size: {} bytes", metadata.len());
        },
        Err(e) => {
            let err = TranscodingError::InputFileError(format!("Failed to read input file metadata: {}", e));
            error!("{}", err);
            return Err(err);
        }
    }
    
    info!("DEBUG: Input file validation passed");
    
    // Ensure output directory is absolute
    let output_dir = if Path::new(&options.output_dir).is_absolute() {
        PathBuf::from(&options.output_dir)
    } else {
        // Create output directory with absolute path
        match std::env::current_dir() {
            Ok(cwd) => {
                let absolute_dir = cwd.join(&options.output_dir);
                info!("Using absolute output directory: {}", absolute_dir.display());
                absolute_dir
            },
            Err(e) => {
                error!("Failed to get current directory: {}, falling back to relative path", e);
                // Fall back to relative path
                PathBuf::from(&options.output_dir)
            }
        }
    };
    
    info!("DEBUG: Resolved output directory: {}", output_dir.display());
    
    // Ensure output directory exists
    info!("Creating output directory: {}", output_dir.display());
    
    match fs::create_dir_all(&output_dir) {
        Ok(_) => info!("Output directory created or already exists: {}", output_dir.display()),
        Err(e) => {
            let err = TranscodingError::OutputFileError(format!("Failed to create output directory: {}", e));
            error!("{}", err);
            return Err(err);
        }
    }
    
    // Test if directory is writable
    let test_path = output_dir.join("write_test.tmp");
    match fs::File::create(&test_path) {
        Ok(_) => {
            info!("Output directory is writable");
            // Clean up the test file
            match fs::remove_file(&test_path) {
                Ok(_) => info!("DEBUG: Test file removed successfully"),
                Err(e) => warn!("DEBUG: Failed to remove test file: {}", e),
            }
        },
        Err(e) => {
            let err = TranscodingError::OutputFileError(format!("Output directory is not writable: {}", e));
            error!("{}", err);
            return Err(err);
        }
    }
    
    info!("DEBUG: Output directory validation passed");
    
    // Generate output file name for medium quality
    let file_stem = match input_path.file_stem() {
        Some(stem) => {
            let stem_str = stem.to_string_lossy();
            info!("Got file stem: {}", stem_str);
            stem_str.to_string()
        },
        None => {
            let err = TranscodingError::InputFileError("Invalid input file name".to_string());
            error!("{}", err);
            return Err(err);
        }
    };

    // Sanitize the file stem for safe file naming
    let sanitized_stem = file_stem.replace(" ", "_").replace(".", "_");
    info!("DEBUG: Sanitized file stem: {}", sanitized_stem);

    // Ensure we have a valid, complete output path including filename
    let medium_quality_path = if output_dir.is_absolute() {
        // If output_dir is already absolute, just join it with the filename
        let path = output_dir.join(format!("{}_medium.{}", sanitized_stem, options.format));
        info!("Medium quality output path (from absolute dir): {}", path.display());
        path
    } else {
        // Create with current directory as base
        match std::env::current_dir() {
            Ok(cwd) => {
                let abs_dir = cwd.join(&output_dir);
                let path = abs_dir.join(format!("{}_medium.{}", sanitized_stem, options.format));
                info!("Medium quality output path (from relative dir): {}", path.display());
                path
            },
            Err(e) => {
                // If we can't get current directory, still try with the relative path
                error!("Failed to get current directory: {}, using relative path", e);
                let path = output_dir.join(format!("{}_medium.{}", sanitized_stem, options.format));
                info!("Medium quality output path (fallback): {}", path.display());
                path
            }
        }
    };

    info!("Final medium quality output path: {}", medium_quality_path.display());
    
    // Check if the parent directory of the output file exists
    if let Some(parent) = medium_quality_path.parent() {
        info!("DEBUG: Ensuring parent directory exists: {}", parent.display());
        match fs::create_dir_all(parent) {
            Ok(_) => info!("DEBUG: Parent directory created or already exists"),
            Err(e) => {
                let err = TranscodingError::OutputFileError(format!("Failed to create parent directory for output file: {}", e));
                error!("{}", err);
                return Err(err);
            }
        }
    }
    
    // Log what we're doing
    info!("Transcoding file: {} to medium quality at {} kbps", input_path.display(), options.medium_bitrate);
    
    // Just directly call the synchronous version to avoid Tokio runtime issues
    let transcode_result = match std::panic::catch_unwind(|| {
        transcode_to_quality_internal(
            input_path,
            &medium_quality_path,
            options.medium_bitrate,
            &options.format,
        )
    }) {
        Ok(result) => result,
        Err(panic_err) => {
            let panic_msg = match panic_err.downcast_ref::<&str>() {
                Some(s) => format!("Panic during internal transcoding: {}", s),
                None => "Unknown panic during internal transcoding".to_string(),
            };
            error!("DEBUG: PANIC in transcode_to_quality_internal: {}", panic_msg);
            return Err(TranscodingError::FFmpegError(panic_msg));
        }
    };
    
    match transcode_result {
        Ok(()) => {
            info!("Successfully transcoded file to: {}", medium_quality_path.display());
            
            // Verify the output file exists and has content
            match fs::metadata(&medium_quality_path) {
                Ok(metadata) => {
                    if metadata.len() == 0 {
                        let err = TranscodingError::OutputFileError(format!("Output file is empty: {}", medium_quality_path.display()));
                        error!("{}", err);
                        return Err(err);
                    }
                    info!("DEBUG: Output file verified: {} bytes", metadata.len());
                },
                Err(e) => {
                    let err = TranscodingError::OutputFileError(format!("Failed to verify output file: {}", e));
                    error!("{}", err);
                    return Err(err);
                }
            }
            
            // Return success with file paths
            let result = TranscodingResult {
                success: true,
                input_path: input_path.to_string_lossy().to_string(),
                medium_quality_path: Some(medium_quality_path.to_string_lossy().to_string()),
                error_message: None,
            };
            
            info!("DEBUG: Returning successful TranscodingResult");
            Ok(result)
        },
        Err(e) => {
            error!("Failed to transcode file: {}", e);
            
            // Check if the output file was partially created but failed
            if medium_quality_path.exists() {
                warn!("DEBUG: Removing partial output file after failed transcoding");
                match fs::remove_file(&medium_quality_path) {
                    Ok(_) => info!("DEBUG: Removed partial output file"),
                    Err(rm_err) => warn!("DEBUG: Failed to remove partial output file: {}", rm_err),
                }
            }
            
            Err(e)
        },
    }
}

// Synchronous implementation using std::process::Command
fn transcode_to_quality_internal(
    input_path: &Path,
    output_path: &Path,
    bitrate: u32,
    format: &str,
) -> Result<(), TranscodingError> {
    info!("DEBUG: [transcode_internal] Starting transcoding process");
    
    // Get absolute paths for input
    let input_absolute = match input_path.canonicalize() {
        Ok(p) => p,
        Err(e) => {
            let err = TranscodingError::InputFileError(format!("Failed to get absolute path for input: {}", e));
            error!("{}", err);
            return Err(err);
        }
    };
    
    info!("Input absolute path: {}", input_absolute.display());
    
    // Make sure output directory exists
    if let Some(parent) = output_path.parent() {
        if !parent.exists() {
            match std::fs::create_dir_all(parent) {
                Ok(_) => info!("Created output directory: {}", parent.display()),
                Err(e) => {
                    let err = TranscodingError::OutputFileError(format!("Failed to create output directory: {}", e));
                    error!("{}", err);
                    return Err(err);
                }
            }
        }
        
        // Verify output directory exists and is writable
        let test_file = parent.join("write_test_output.tmp");
        match std::fs::File::create(&test_file) {
            Ok(_) => {
                info!("Output directory is writable: {}", parent.display());
                match std::fs::remove_file(&test_file) {
                    Ok(_) => info!("DEBUG: [transcode_internal] Test file removed successfully"),
                    Err(e) => warn!("DEBUG: [transcode_internal] Failed to remove test file: {}", e),
                }
            },
            Err(e) => {
                let err = TranscodingError::OutputFileError(format!("Output directory is not writable: {}", e));
                error!("{}", err);
                return Err(err);
            }
        };
    } else {
        let err = TranscodingError::OutputFileError("Invalid output path: no parent directory".to_string());
        error!("{}", err);
        return Err(err);
    }
    
    info!("DEBUG: [transcode_internal] Output directory validated");
    
    // Use which to find ffmpeg path
    let mut ffmpeg_path = match std::process::Command::new("which")
        .arg("ffmpeg")
        .output() {
            Ok(output) => {
                if output.status.success() {
                    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    info!("Found ffmpeg at: {}", path);
                    path
                } else {
                    info!("Could not find ffmpeg path with 'which', using 'ffmpeg'");
                    "ffmpeg".to_string()
                }
            },
            Err(e) => {
                info!("Error finding ffmpeg path: {}, using 'ffmpeg'", e);
                "ffmpeg".to_string()
            }
        };
    
    // Check common ffmpeg locations on macOS
    if !std::path::Path::new(&ffmpeg_path).exists() || ffmpeg_path == "ffmpeg" {
        let potential_paths = [
            "/opt/homebrew/bin/ffmpeg",
            "/usr/local/bin/ffmpeg",
            "/usr/bin/ffmpeg"
        ];
        
        for path in potential_paths.iter() {
            if std::path::Path::new(path).exists() {
                info!("Found ffmpeg at alternative path: {}", path);
                ffmpeg_path = path.to_string();
                break;
            }
        }
    }
    
    info!("Using ffmpeg at path: {}", ffmpeg_path);
    
    // Verify ffmpeg exists at the specified path
    if !std::path::Path::new(&ffmpeg_path).exists() {
        let err = TranscodingError::FFmpegError(format!("FFmpeg not found at path: {}", ffmpeg_path));
        error!("{}", err);
        return Err(err);
    }
    
    info!("DEBUG: [transcode_internal] FFmpeg validated at: {}", ffmpeg_path);
    
    // Create temp directory for output to prevent incomplete files
    let temp_dir = match tempfile::TempDir::new() {
        Ok(dir) => {
            info!("Created temporary directory for output: {}", dir.path().display());
            dir
        },
        Err(e) => {
            let err = TranscodingError::OutputFileError(format!("Failed to create temporary directory: {}", e));
            error!("{}", err);
            return Err(err);
        }
    };
    
    let temp_output = temp_dir.path().join(format!("temp_output.{}", format));
    info!("Using temporary output file: {}", temp_output.display());
    
    // Create the ffmpeg command with appropriate options
    let mut cmd = std::process::Command::new(&ffmpeg_path);
    
    // Add input options with error checking
    cmd.arg("-i").arg(&input_absolute);
    
    // Audio codec and bitrate
    match format.to_lowercase().as_str() {
        "mp3" => {
            info!("Using mp3 codec with bitrate {}k", bitrate);
            cmd.arg("-codec:a").arg("libmp3lame")
               .arg("-b:a").arg(format!("{}k", bitrate));
        },
        "aac" | "m4a" => {
            info!("Using aac codec with bitrate {}k", bitrate);
            cmd.arg("-codec:a").arg("aac")
               .arg("-b:a").arg(format!("{}k", bitrate));
        },
        "ogg" => {
            info!("Using vorbis codec with bitrate {}k", bitrate);
            cmd.arg("-codec:a").arg("libvorbis")
               .arg("-b:a").arg(format!("{}k", bitrate));
        },
        "flac" => {
            info!("Using flac codec with compression level 5");
            cmd.arg("-codec:a").arg("flac")
               .arg("-compression_level").arg("5"); // Medium compression
        },
        "wav" => {
            info!("Using pcm_s16le codec for WAV");
            cmd.arg("-codec:a").arg("pcm_s16le"); // Standard WAV format
        },
        _ => {
            let err = TranscodingError::InvalidFormat(format!("Unsupported format: {}", format));
            error!("{}", err);
            return Err(err);
        }
    }
    
    // Output options
    cmd.arg("-y") // Overwrite output files without asking
       .arg("-map_metadata").arg("0") // Copy metadata
       .arg("-nostdin") // Don't expect any input from stdin
       .arg("-threads").arg("4") // Limit thread usage
       .arg("-loglevel").arg("info") // Increased logging
       .arg(&temp_output);
    
    // Run the command
    let command_str = format!("{:?}", cmd);
    info!("Running ffmpeg command: {}", command_str);
    
    let output = match cmd.output() {
        Ok(output) => output,
        Err(e) => {
            let err = TranscodingError::FFmpegError(format!("Failed to run ffmpeg: {}", e));
            error!("{}", err);
            // Clean up temp directory
            let _ = temp_dir.close();
            return Err(err);
        }
    };
    
    // Check if the command was successful
    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        info!("FFmpeg stdout: {}", stdout);
        error!("FFmpeg stderr: {}", stderr);
        
        // Clean up temp directory
        let _ = temp_dir.close();
        
        return Err(TranscodingError::FFmpegError(format!("ffmpeg failed: {}", stderr)));
    }
    
    info!("FFmpeg command executed successfully");
    
    // Check if the temp output file was created
    if !temp_output.exists() {
        let err = TranscodingError::OutputFileError("Output file was not created".to_string());
        error!("{}", err);
        // Clean up temp directory
        let _ = temp_dir.close();
        return Err(err);
    }
    
    // Check temp file size
    match fs::metadata(&temp_output) {
        Ok(metadata) => {
            if metadata.len() == 0 {
                let err = TranscodingError::OutputFileError("Output file is empty".to_string());
                error!("{}", err);
                // Clean up temp directory
                let _ = temp_dir.close();
                return Err(err);
            }
            info!("Temporary output file size: {} bytes", metadata.len());
        },
        Err(e) => {
            let err = TranscodingError::OutputFileError(format!("Failed to check temp output file: {}", e));
            error!("{}", err);
            // Clean up temp directory
            let _ = temp_dir.close();
            return Err(err);
        }
    }
    
    info!("DEBUG: [transcode_internal] FFmpeg transcoding completed successfully, copying from temp to final location");
    
    // Copy the temp file to the output location
    match fs::copy(&temp_output, output_path) {
        Ok(bytes_copied) => {
            info!("Copied {} bytes from temp to output location", bytes_copied);
        },
        Err(e) => {
            let err = TranscodingError::OutputFileError(format!("Failed to copy temp file to output location: {}", e));
            error!("{}", err);
            // Clean up temp directory
            let _ = temp_dir.close();
            return Err(err);
        }
    }
    
    // Clean up temp directory
    match temp_dir.close() {
        Ok(_) => info!("Temporary directory cleaned up successfully"),
        Err(e) => warn!("Failed to clean up temporary directory: {}", e),
    }
    
    // Check the final output file
    match fs::metadata(output_path) {
        Ok(metadata) => {
            if metadata.len() == 0 {
                let err = TranscodingError::OutputFileError("Final output file is empty".to_string());
                error!("{}", err);
                return Err(err);
            }
            info!("Final output file size: {} bytes", metadata.len());
        },
        Err(e) => {
            let err = TranscodingError::OutputFileError(format!("Failed to check final output file: {}", e));
            error!("{}", err);
            return Err(err);
        }
    }
    
    info!("Output file exists and is valid: {}", output_path.display());
    info!("DEBUG: [transcode_internal] Transcoding process completed successfully");
    
    Ok(())
}

// Async wrapper that accepts a string path and options struct to match the main.rs call
pub async fn transcode_to_quality(
    input_path_str: &str, 
    options: &TranscodingOptions
) -> Result<String, TranscodingError> {
    let input_path = Path::new(input_path_str);
    
    // Verify the input file exists
    if !input_path.exists() {
        let err = TranscodingError::InputFileError(format!("Input file does not exist: {}", input_path.display()));
        error!("{}", err);
        return Err(err);
    }
    
    info!("Input file exists: {}", input_path.display());
    
    // Generate sanitized output file name
    let file_stem = match input_path.file_stem() {
        Some(stem) => {
            let stem_str = stem.to_string_lossy();
            info!("Got file stem: {}", stem_str);
            stem_str.to_string()
        },
        None => {
            let err = TranscodingError::InputFileError("Invalid input file name".to_string());
            error!("{}", err);
            return Err(err);
        }
    };

    // Sanitize the file stem for safe file naming
    let sanitized_stem = file_stem.replace(" ", "_").replace(".", "_");
    
    // Ensure output directory is absolute
    let output_dir = if Path::new(&options.output_dir).is_absolute() {
        PathBuf::from(&options.output_dir)
    } else {
        // Create output directory with absolute path
        match std::env::current_dir() {
            Ok(cwd) => {
                let absolute_dir = cwd.join(&options.output_dir);
                info!("Using absolute output directory: {}", absolute_dir.display());
                absolute_dir
            },
            Err(e) => {
                error!("Failed to get current directory: {}, falling back to relative path", e);
                // Fall back to relative path
                PathBuf::from(&options.output_dir)
            }
        }
    };
    
    // Create the medium quality output path
    let medium_quality_path = output_dir.join(format!("{}_medium.{}", sanitized_stem, options.format));
    info!("Medium quality output path: {}", medium_quality_path.display());
    
    // Call the synchronous implementation
    match transcode_to_quality_internal(
        input_path,
        &medium_quality_path,
        options.medium_bitrate,
        &options.format,
    ) {
        Ok(()) => {
            info!("Successfully transcoded file to: {}", medium_quality_path.display());
            Ok(medium_quality_path.to_string_lossy().to_string())
        },
        Err(e) => {
            error!("Failed to transcode file: {}", e);
            Err(e)
        }
    }
} 