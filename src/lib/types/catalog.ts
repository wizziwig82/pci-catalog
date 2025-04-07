// Define TypeScript interfaces related to the catalog

// Interface for metadata extracted by the backend before upload
// Matches the Rust struct UploadItemMetadata in src-tauri/src/upload.rs (or similar)
export interface UploadItemMetadata {
    title?: string;
    artist?: string;
    album?: string;
    track_number?: number;
    duration_sec?: number;
    genre?: string;
    composer?: string;
    year?: number;
    comments?: string;
    writers?: { name: string; percentage: number }[]; // Combined writers and percentages
    // writer_percentages?: number[]; // Removed, now part of writers object array
    publishers?: { name: string; percentage: number }[]; // Publisher entries using the same pattern as writers
    // Added by frontend after extraction to link back to the original file
    original_path: string;
}

export interface PathInfo {
  original: string;
  medium: string;
}

export interface Track {
  id: string; // Changed from _id based on console log
  title: string;
  album_id: string;
  album_name?: string;
  duration?: number; // Make duration optional to match potential backend data
  genre?: string[]; // Expecting array based on upload page and potential future needs
  filename: string;
  writers: { name: string; percentage: number }[]; // Combined writers and percentages
  // writer_percentages: number[]; // Removed
  publishers: string[];
  publisher_percentages: number[];
  instruments: string[];
  mood: string[];
  comments?: string;
  path: PathInfo;
  // Add other fields as needed, ensure they match the Rust TrackDocument structure
}

// Interface for the data being edited (might be a subset or slightly different structure)
// Note: EditingTrackData is not exported as it seems specific to the component's internal state
// If it needs to be shared, it should be exported as well.
interface EditingTrackData extends Omit<Track, 'id' | 'path' | 'album_id'> {
  // Use Omit to exclude fields not directly editable or handled differently
  // Add any temporary fields needed for editing UI, like string representations of arrays
  genreString?: string; // Example: for comma-separated input
}


export interface TrackListResponse {
  success: boolean;
  message?: string;
  tracks: Track[];
  total_count: number;
}