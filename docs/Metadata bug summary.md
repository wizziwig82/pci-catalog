```markdown
# Troubleshooting Summary: "Command audio::metadata::extract_metadata not found" Error (Session: 2025-04-05)

## Initial Problem Description

After refactoring the backend audio processing workflow and updating the frontend, selecting an audio file (WAV/AAC) on the upload page resulted in a console error: `Error invoking command 'audio::metadata::extract_metadata' - "Command audio::metadata::extract_metadata not found"`. This prevented metadata from being extracted and displayed.

## Troubleshooting Steps (Chronological)

1.  **Hypothesis:** The backend command registration might be incorrect or missing.
    *   **Actions:** Delegated a task to Code mode to examine `src-tauri/src/audio/metadata.rs` for the `#[tauri::command]` attribute on the `extract_metadata` function and `src-tauri/src/main.rs` for the correct entry (`audio::metadata::extract_metadata`) within the `tauri::generate_handler!` macro.
    *   **Outcome:** Verification confirmed the command was correctly defined with the attribute and correctly registered in the handler macro. No discrepancies found.

2.  **Hypothesis:** The frontend might be calling the command with an incorrect name string.
    *   **Actions:** Delegated a task to Code mode to examine `src/routes/upload/+page.svelte` and `src/lib/utils/invokeWrapper.ts` (specifically the `safeInvoke` call) to verify the command name string being used.
    *   **Outcome:** Verification confirmed the frontend was using the exact string `"audio::metadata::extract_metadata"`. No mismatch found.

3.  **Hypothesis:** The running Tauri development server (`npm run tauri dev`) might not have picked up the latest backend changes.
    *   **Actions:** Instructed the user to manually stop (`Ctrl+C`) and restart the `npm run tauri dev` process.
    *   **Outcome:** User reported the "Command not found" error persisted after restarting the server.

4.  **Hypothesis:** The Rust module structure (`audio` module and `metadata` submodule) might be incorrectly declared or not publicly visible.
    *   **Actions:** Delegated a task to Code mode to examine `src-tauri/src/main.rs` for `mod audio;` and `src-tauri/src/audio/mod.rs` for `pub mod metadata;`.
    *   **Outcome:** Verification confirmed the module structure and visibility (`mod`, `pub mod`) were correctly implemented according to standard Rust practices.

5.  **Hypothesis:** Inconsistent Rust build artifacts might be causing the issue.
    *   **Actions:** Delegated a task to Code mode to execute `cargo clean` within the `src-tauri` directory using the `execute_command` tool.
    *   **Outcome:** `cargo clean` executed successfully.

6.  **Hypothesis:** A clean build might resolve the issue after clearing artifacts.
    *   **Actions:** Instructed the user to restart the `npm run tauri dev` server again after `cargo clean`.
    *   **Outcome:** User reported a new error: a 500 Internal Server Error occurred when launching the application. The original "Command not found" error could not be tested.

7.  **Hypothesis:** The 500 Internal Server Error originates from SvelteKit's Server-Side Rendering (SSR).
    *   **Actions:** Analyzed the terminal stack trace provided by the user, which pointed to `src/routes/+layout.svelte:37` and involved `paraglideMiddleware`. Delegated a task to Code mode to debug `+layout.svelte` and related SSR components.
    *   **Outcome:** The Code mode task identified the issue within the Paraglide middleware (`src/hooks.server.ts`) where the `event.request` object was being overwritten. The middleware was modified to avoid this overwrite.

8.  **Hypothesis:** Fixing the SSR error might allow the application to launch and reveal the status of the original command error.
    *   **Actions:** Instructed the user to restart the `npm run tauri dev` server after the SSR fix.
    *   **Outcome:** User reported the application launched successfully, but the "Command audio::metadata::extract_metadata not found" error *still* occurred when selecting a file.

9.  **Hypothesis:** Conditional compilation (`#[cfg(...)]`) might be excluding the command or its modules.
    *   **Actions:** Delegated a task to Code mode to check `main.rs`, `audio/mod.rs`, and `audio/metadata.rs` for relevant `#[cfg(...)]` attributes.
    *   **Outcome:** Verification confirmed no conditional compilation attributes were affecting the command definition, module declarations, or registration.

10. **Hypothesis:** The fundamental Tauri command invocation mechanism might be broken.
    *   **Actions:**
        *   Delegated a task to Code mode to add a simple `#[tauri::command] fn ping() -> String` to `src-tauri/src/main.rs` and register it.
        *   Delegated a task to Code mode to add an `onMount` call in `src/routes/upload/+page.svelte` to invoke the `"ping"` command using `safeInvoke` and log the result.
    *   **Outcome:** User restarted the server and confirmed the console logged "Ping command successful, result: pong". This proved the basic command invocation works.

11. **Hypothesis:** Cargo feature flags might be disabling the audio module or dependencies.
    *   **Actions:** Delegated a task to Code mode to examine `src-tauri/Cargo.toml` for relevant `[features]` or optional dependencies.
    *   **Outcome:** Verification confirmed no active features or optional dependencies were configured that would disable the audio module or its required libraries.

12. **Hypothesis:** A subtle typo or invisible character exists in the command registration string in `main.rs`.
    *   **Actions:** Delegated a task to Code mode to perform a meticulous character-by-character comparison of the `audio::metadata::extract_metadata` entry in `generate_handler!` against the target string.
    *   **Outcome:** Verification confirmed an *exact* character-by-character match. No discrepancies found.

13. **Hypothesis:** The internal logic, signature, or return type (`UploadItemMetadata`) of the *original* `extract_metadata` function prevents successful registration by Tauri, even if the `generate_handler!` macro looks correct.
    *   **Actions:** Delegated a task to Code mode to temporarily comment out the original body of `extract_metadata` in `src-tauri/src/audio/metadata.rs` and replace it with a dummy implementation returning `Ok("dummy_metadata_extracted".to_string())`.
    *   **Outcome:** User restarted the server and tested file selection. The console output showed the "Command not found" error was **gone**, and logs indicated the simplified command was called (though the frontend processed the dummy result as an empty array). This suggested the issue was indeed related to the original function's complexity/signature.

14. **Hypothesis:** The previous steps (cleaning, restarting, simplifying/recompiling) might have resolved the underlying state issue, allowing the original command to register correctly now.
    *   **Actions:** Delegated a task to Code mode to restore the original function body and signature for `extract_metadata` in `src-tauri/src/audio/metadata.rs`. Code mode also ran `cargo clean` again to resolve a macro expansion error encountered during restoration.
    *   **Outcome (Current Status):** User restarted the server and tested file selection. The "Command audio::metadata::extract_metadata not found" error **reappeared**.

## Current Status

Despite extensive troubleshooting, including verifying code, configuration, build state, and isolating the issue to the specific command's complexity/signature, the "Command audio::metadata::extract_metadata not found" error persists after restoring the original function logic. The root cause remains unidentified.
```