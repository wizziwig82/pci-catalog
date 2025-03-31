# MongoDB Schema for Music Library Manager

This document outlines the MongoDB database schema for the Music Library Manager application. The MongoDB database is used to store metadata for albums and tracks, while Cloudflare R2 is used to store the actual audio files and album artwork.

## Database: `music_library`

### Collection: `albums`

Albums collection stores information about music albums.

```json
{
  "_id": "string",           // Unique album identifier (can be generated or based on album name)
  "name": "string",          // Album name
  "art_path": "string",      // Path to album artwork in R2 (optional)
  "track_ids": ["string"]    // Array of track IDs associated with this album
}
```

#### Indexes:
- Text index on `name` for efficient search functionality

### Collection: `tracks`

Tracks collection stores detailed metadata about individual music tracks.

```json
{
  "_id": "string",                 // Unique track identifier
  "title": "string",               // Track title
  "album_id": "string",            // Reference to parent album
  "filename": "string",            // Original filename
  "duration": "number",            // Track duration in seconds
  "comments": "string",            // Additional track comments (optional)
  
  "writers": [                     // Array of writers with percentage ownership
    {
      "name": "string",            // Writer's name
      "percentage": "number"       // Percentage ownership (should sum to 100 for all writers)
    }
  ],
  
  "publishers": [                  // Array of publishers with percentage ownership
    {
      "name": "string",            // Publisher's name
      "percentage": "number"       // Percentage ownership (should sum to 100 for all publishers)
    }
  ],
  
  "genre": "string",               // Music genre (optional)
  "instruments": ["string"],       // Array of instruments used in the track
  "mood": "string",                // Track mood descriptor (optional)
  
  "path": {                        // Paths to different quality versions in R2
    "original": "string",          // Path to original quality file
    "medium": "string",            // Path to medium quality file
    "low": "string"                // Path to low quality file
  }
}
```

#### Indexes:
- Text index on `title` and `genre` for efficient search functionality
- Regular index on `album_id` for efficient album-track relationship queries

## Data Integrity Rules

1. Writer percentages must sum to 100% within each track
2. Publisher percentages must sum to 100% within each track
3. Track IDs in an album's `track_ids` array must correspond to existing tracks in the `tracks` collection
4. Album IDs referenced in a track's `album_id` field must correspond to existing albums in the `albums` collection

## Usage Patterns

### Common Queries

1. Get all albums (for browsing)
2. Get all tracks for a specific album
3. Search for tracks by title or genre
4. Search for albums by name
5. Get a single track by ID (for playback or editing)
6. Get a single album by ID (for displaying album details)

### Common Updates

1. Create a new album when uploading tracks
2. Update track metadata (writers, publishers, genre, etc.)
3. Update album metadata (name, artwork)
4. Add tracks to an existing album 