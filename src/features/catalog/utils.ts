import { showSuccessToast, showErrorToast } from '$lib/stores/notifications';

// Define the type signature for the safeInvoke function locally
type SafeInvokeFn = <T>(cmd: string, args?: Record<string, unknown>) => Promise<T | null>;

/**
 * Handles the multi-step process of replacing a track's audio file.
 * @param safeInvoke The safeInvoke function instance (matching the signature in invokeWrapper.ts).
 * @param trackId The ID of the track to replace.
 * @returns Promise<boolean> True if successful, false otherwise.
 */
export async function replaceTrackAudioWorkflow(safeInvoke: SafeInvokeFn, trackId: string): Promise<boolean> {
    console.log(`Starting audio replacement workflow for track: ${trackId}`);

    // 1. Select Replacement File
    let replacementFilePath: string | null = null;
    try {
        console.log("Attempting to select replacement audio file...");
        // Assuming 'select_audio_files' returns string[] or null
        const selectedPaths = await safeInvoke<string[]>('select_audio_files');
        if (selectedPaths && selectedPaths.length > 0) {
            replacementFilePath = selectedPaths[0];
            console.log('Replacement file selected:', replacementFilePath);
        } else {
            console.log('No replacement file selected.');
            showErrorToast('File selection cancelled or failed.');
            return false;
        }
    } catch (err) {
        console.error('Error selecting replacement file:', err);
        // safeInvoke should have shown a toast
        return false;
    }

    // 2. Transcode Replacement File (Optional - depends on backend requirements)
    // Note: Using 'transcode_audio_batch' might be incorrect if only one file needs processing.
    // A dedicated 'transcode_single_file' or similar might be better.
    // Assuming the backend's 'replace_track_audio' handles transcoding if necessary,
    // or that transcoding happens implicitly during upload. If explicit transcoding
    // is needed *before* calling replace_track_audio, uncomment and adjust this section.

    /*
    console.log('Starting transcoding for replacement file:', replacementFilePath);
    showSuccessToast('Starting transcoding for replacement file...'); // Give user feedback
    const mediumBitrate = 128;
    const outputFormat = 'mp3';
    const outputDir = 'transcoded'; // Needs careful consideration for temporary files

    const transcodeResults = await safeInvoke<any[]>('transcode_audio_batch', {
        filePaths: [replacementFilePath],
        mediumBitrate,
        format: outputFormat,
        outputDir
    });

    let newMediumPath: string | null = null;
    if (transcodeResults && transcodeResults.length > 0 && transcodeResults[0].success && transcodeResults[0].medium_quality_path) {
        newMediumPath = transcodeResults[0].medium_quality_path;
        console.log('Replacement transcoding successful:', newMediumPath);
        showSuccessToast('Replacement file transcoded successfully.');
    } else {
        const errorMsg = transcodeResults?.[0]?.error || 'Transcoding failed or missing output path.';
        console.error('Replacement transcoding failed:', errorMsg);
        showErrorToast(`Transcoding failed: ${errorMsg}`);
        return false;
    }
    */

    // For now, assume replace_track_audio handles necessary processing/upload from the selected path
    const newMediumPath = replacementFilePath; // Pass the original selected path

    // 3. Upload and Replace Track in DB/Storage
    console.log(`Attempting to replace audio for track ${trackId} with file ${newMediumPath}`);
    showSuccessToast('Uploading and replacing audio...'); // User feedback

    // Backend command 'replace_track_audio' needs to handle:
    // - Receiving the new file path.
    // - Uploading the new file to R2 (potentially transcoding first if needed).
    // - Deleting the old file(s) from R2.
    // - Updating the database record for the track with the new path/URL.
    const success = await safeInvoke<boolean>('replace_track_audio', {
        trackId: trackId,
        newMediumQualityPath: newMediumPath // Or pass the original path if backend handles transcoding/upload
    });

    if (success) {
        console.log(`Successfully replaced audio for track ${trackId}`);
        showSuccessToast(`Successfully replaced audio for track ${trackId}.`);
        return true;
    } else {
        console.error(`Failed to replace audio for track ${trackId}`);
        // safeInvoke should show toast
        return false;
    }
}


/**
 * Handles deleting selected tracks.
 * @param safeInvoke The safeInvoke function instance (matching the signature in invokeWrapper.ts).
 * @param trackIds Array of track IDs to delete.
 * @returns Promise<boolean> True if successful, false otherwise.
 */
export async function deleteTracksWorkflow(safeInvoke: SafeInvokeFn, trackIds: string[]): Promise<boolean> {
    if (trackIds.length === 0) {
        showErrorToast("No tracks selected for deletion.");
        return false;
    }

    // No need for confirmation here, assume it's done in the component before calling
    console.log(`Attempting to delete tracks: ${trackIds.join(', ')}`);

    // Backend command 'delete_tracks' should handle:
    // - Deleting records from MongoDB.
    // - Deleting associated files from R2 storage.
    const success = await safeInvoke<boolean>('delete_tracks', {
        trackIds: trackIds
    });

    if (success) {
        console.log(`Successfully deleted ${trackIds.length} tracks.`);
        showSuccessToast(`Successfully deleted ${trackIds.length} tracks.`);
        return true;
    } else {
        console.error(`Failed to delete tracks.`);
        // safeInvoke should show toast
        return false;
    }
}