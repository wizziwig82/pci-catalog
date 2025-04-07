# Audio Workflow Fix Summary (2025-04-05)

This document summarizes the steps taken to debug and fix the audio file processing workflow bug where metadata extraction and UI updates failed after file selection.

**Problem:**
After selecting a WAV or AAC file on macOS, the application failed to extract metadata, display it in the UI, or present the metadata editor. The workflow broke silently after the file picker closed.

**Debugging & Resolution:**

1.  **Backend Analysis & Refactoring:**
    *   Analyzed Rust code (`src-tauri/src/audio/metadata.rs`, `transcode.rs`, `transcoding.rs`).
    *   Identified that the backend workflow had been previously refactored, separating metadata extraction from the upload process.
    *   Refactored the backend further:
        *   Moved metadata extraction logic (`lofty`) into a dedicated Tauri command `audio::metadata::extract_metadata` in `src-tauri/src/audio/metadata.rs`.
        *   Ensured the main upload command `start_upload_queue` (in `src-tauri/src/upload.rs`) now expects finalized metadata from the frontend.
        *   Removed obsolete code (`src-tauri/src/audio/transcoding.rs`).
        *   Updated `src-tauri/src/main.rs` to register the new command correctly.

2.  **Frontend Update:**
    *   Updated the Svelte frontend code, primarily `src/routes/upload/+page.svelte` and types in `src/lib/types/catalog.ts`.
    *   Modified the file selection handler to call the new `audio::metadata::extract_metadata` command via `invokeWrapper`.
    *   Implemented state management to store the extracted metadata.
    *   Updated the UI (`MetadataEditor.svelte` or similar) to display the extracted metadata and allow editing.
    *   Added logic (e.g., an "Upload" button) to call the `start_upload_queue` command with the finalized metadata.
    *   Implemented error handling using `safeInvoke` and notifications.

**Outcome:**
The frontend and backend workflows are now aligned. Upon file selection, metadata is extracted and displayed. The user can review/edit, and then initiate the upload process separately. This resolves the original bug. The workflow should function correctly for WAV and AAC files on macOS.