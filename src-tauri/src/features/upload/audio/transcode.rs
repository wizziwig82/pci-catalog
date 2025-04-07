use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::fs;
use std::io::Read; // Import Read trait

use super::error::TranscodingError; // Use the specific error type

/// Transcodes an audio file to 256kbps AAC format using the ffmpeg CLI.
///
/// # Arguments
///
/// * `input_path` - Path to the input audio file.
/// * `output_path` - Desired path for the output AAC file.
///
/// # Returns
///
/// * `Ok(())` if transcoding is successful.
/// * `Err(TranscodingError)` if any error occurs during the process.
pub fn transcode_to_aac(input_path: &Path, output_path: &Path) -> Result<(), TranscodingError> {
    // --- Input Validation ---
    if !input_path.exists() {
        return Err(TranscodingError::InputFileNotFound(input_path.to_path_buf()));
    }

    // --- Ensure Output Directory Exists ---
    if let Some(parent_dir) = output_path.parent() {
        if !parent_dir.exists() {
            // Use the helper function from error.rs
            fs::create_dir_all(parent_dir)
                .map_err(|e| TranscodingError::output_dir_creation_failed(parent_dir.to_path_buf(), e))?;
        }
    }

    // --- Construct FFmpeg Command ---
    let mut command = Command::new("ffmpeg");
    command
        .arg("-i") // Input file flag
        .arg(input_path)
        .arg("-vn") // Disable video recording
        .arg("-acodec") // Audio codec flag
        .arg("aac") // Specify AAC codec
        .arg("-b:a") // Audio bitrate flag
        .arg("256k") // Specify 256kbps bitrate
        .arg("-y") // Overwrite output file if it exists
        .arg(output_path)
        .stdout(Stdio::null()) // Discard stdout
        .stderr(Stdio::piped()); // Capture stderr for error reporting

    // --- Execute FFmpeg ---
    // Use the helper function from error.rs
    let mut child = command.spawn().map_err(TranscodingError::process_start_failed)?;

    // --- Capture Stderr ---
    let mut stderr_output = String::new();
    if let Some(mut stderr) = child.stderr.take() {
        // Read stderr into the string
        // Use the helper function from error.rs
        stderr.read_to_string(&mut stderr_output)
              .map_err(TranscodingError::stderr_read_failed)?;
    }


    // --- Wait for Completion and Check Status ---
    // The `?` here uses the `From<std::io::Error>` implementation in error.rs
    let status = child.wait()?;

    if !status.success() {
        return Err(TranscodingError::ProcessExecutionFailed {
            status: status.code(),
            stderr: stderr_output, // Include captured stderr in the error
        });
    }

    // --- Check if FFmpeg command itself was found ---
    // This check is a bit indirect. If spawning fails withErrorKind::NotFound,
    // it's likely ffmpeg isn't in PATH. We handle this in ProcessStartFailed.
    // A more direct check could involve `which` or similar before spawning,
    // but this keeps dependencies minimal for now.
    // We could refine the ProcessStartFailed mapping to specifically check for NotFound.

    Ok(())
}

// Basic test (requires ffmpeg in PATH and a dummy input file)
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    // Helper to create a dummy file
    fn create_dummy_file(path: &Path) -> std::io::Result<()> {
        File::create(path)?;
        Ok(())
    }

    #[test]
    fn test_transcode_success_mock() {
        // This test requires ffmpeg to be installed and in the PATH.
        // It also needs a valid audio file. For CI/simplicity, we might
        // only test error paths or use a tiny, known-good audio sample.

        // Let's test an error path first: input file not found
        let temp_dir = tempdir().unwrap();
        let input_path = temp_dir.path().join("non_existent_input.mp3");
        let output_path = temp_dir.path().join("output.aac");

        let result = transcode_to_aac(&input_path, &output_path);
        assert!(matches!(result, Err(TranscodingError::InputFileNotFound(_))));
    }

     #[test]
     fn test_output_dir_creation() {
         let temp_dir = tempdir().unwrap();
         let input_path = temp_dir.path().join("dummy_input.tmp");
         create_dummy_file(&input_path).unwrap(); // Create a dummy file

         let nested_output_dir = temp_dir.path().join("nested").join("deeper");
         let output_path = nested_output_dir.join("output.aac");

         // We expect this to fail because ffmpeg won't find a valid audio stream
         // in the dummy file, but the directory should be created.
         let _ = transcode_to_aac(&input_path, &output_path);

         assert!(nested_output_dir.exists());
         assert!(nested_output_dir.is_dir());
     }

    // Add more tests:
    // - Test actual transcoding with a small, valid sample file (if feasible in test env)
    // - Test ffmpeg not found (might require manipulating PATH or mocking Command)
    // - Test ffmpeg execution failure (e.g., invalid input format)
}