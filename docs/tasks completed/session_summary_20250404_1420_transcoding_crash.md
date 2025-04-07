# Debug Session Summary: Transcoding Crash (2025-04-04 14:20)

## Problem

The application crashes during the "transcode files" step after selecting multiple audio files for upload. The backend Rust process terminates unexpectedly.

## Analysis

1.  **Reviewed `src-tauri/src/audio/transcoding.rs`**: Examined the core FFmpeg execution logic. Found reasonable error handling for FFmpeg process start/exit codes and file I/O, including the use of temporary files.
2.  **Reviewed `src-tauri/src/commands.rs`**: Checked for commands calling the transcoding function; none found directly handling the initial batch transcoding.
3.  **Reviewed `src-tauri/src/upload.rs`**: Found the `upload_transcoded_tracks` command, which handles uploading *already* transcoded files and saving metadata. This confirmed transcoding happens *before* this step.
4.  **Reviewed `src-tauri/src/main.rs`**: Located the `transcode_audio_batch` command. This command receives the list of selected file paths from the frontend.

## Key Finding & Diagnosis

The `transcode_audio_batch` function iterates through the input file paths and uses `tokio::spawn` followed by `tokio::task::spawn_blocking` to call the `audio::transcoding::transcode_file` function for *each* file. **Crucially, there is no limit on the number of concurrently executing transcoding tasks.**

The most likely cause of the application crash is **Resource Exhaustion (CPU/Memory)**. Spawning potentially dozens or hundreds of concurrent, resource-intensive FFmpeg processes overwhelms the system, leading to process termination.

## Proposed Solution

Limit the number of concurrent transcoding operations within `transcode_audio_batch` using a `tokio::sync::Semaphore`. A limit of 2-4 concurrent tasks was suggested as a starting point.

## User Decision

The user agreed that resource exhaustion due to unlimited concurrency is the likely cause. However, the decision was made to **hold off on applying the fix** for now. This summary documents the findings up to this point.