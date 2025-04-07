# Metadata Extraction Issue Resolution

**Date:** April 6, 2025  
**Issue:** Command not found error when trying to extract metadata from audio files  
**Resolution:** Created a wrapper command to handle metadata extraction

## Issue Description

When attempting to extract metadata from audio files on the upload page, we encountered a persistent error:

```
Error invoking command 'features::upload::audio::metadata::extract_metadata': – "Command features::upload::audio::metadata::extract_metadata not found"
```

This error prevented the application from reading and displaying audio file metadata, which is a critical feature for the upload workflow.

## Root Cause Analysis

After extensive troubleshooting, we determined that despite registering the command correctly in `main.rs` with:

```rust
// Audio/File Commands
features::upload::audio::metadata::extract_metadata, // Updated path
```

And defining it correctly in `src-tauri/src/features/upload/audio/metadata.rs` with:

```rust
#[tauri::command]
pub fn extract_metadata(filePath: String) -> Result<UploadItemMetadata, String> {
    // Implementation...
}
```

The command was not being properly discovered by Tauri's command system. The exact cause remains unknown, but appears to be related to how Tauri handles nested module paths in command registration.

## Solution

We implemented a workaround by creating a wrapper command directly in `main.rs` that calls the actual metadata extraction function:

```rust
#[tauri::command]
fn extract_metadata_wrapper(filePath: String) -> Result<serde_json::Value, String> {
    // Call the actual metadata extraction function
    info!("Wrapper calling extract_metadata for: {}", filePath);
    match features::upload::audio::metadata::extract_metadata(filePath) {
        Ok(metadata) => {
            // Convert the UploadItemMetadata struct to a JSON value
            match serde_json::to_value(metadata) {
                Ok(json) => Ok(json),
                Err(e) => Err(format!("Error serializing metadata: {}", e))
            }
        },
        Err(e) => Err(e)
    }
}
```

The wrapper was registered in the `generate_handler!` macro alongside other commands:

```rust
.invoke_handler(tauri::generate_handler![
    // Other commands...
    test_extract_metadata,
    extract_metadata_wrapper,
])
```

Then we updated the frontend code in `src/routes/upload/+page.svelte` to use this wrapper command:

```javascript
// Switch to the wrapper command
const result = await safeInvoke<UploadItemMetadata>('extract_metadata_wrapper', {
   filePath: filePath // Match the parameter name expected by Rust
});
```

## Key Learnings

1. **Parameter Naming Consistency**: Ensure parameter names match between frontend and backend (we used camelCase `filePath` in both places)

2. **Command Path Simplification**: When dealing with deeply nested module paths in Tauri commands, consider using wrapper functions in `main.rs` to simplify the command path

3. **Testing with Simplified Commands**: Creating a test command with hardcoded values (as we did with `test_extract_metadata`) is an effective way to isolate and troubleshoot command discovery issues

## Future Considerations

For future development:

1. If upgrading Tauri versions, revisit this issue to see if direct command registration works with the nested path

2. Consider moving more audio-related functionality to wrapper commands in `main.rs` to maintain consistency

3. If a pattern of command discovery issues continues, consider restructuring the module hierarchy to use flatter command paths 