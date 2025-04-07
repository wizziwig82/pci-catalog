# Music Management App PRD

## Version

- Version: 1.1
- Date: March 30, 2025 (Updated: April 3, 2025)

## Overview
### Product Name

Music Management App

### Purpose

This Mac-only application enables a single user to manage a cloud-based music library, storing files in Cloudflare R2 and metadata in MongoDB. It supports bulk uploading and editing of tracks, transcodes audio files into multiple quality levels for efficient streaming on a future Next.js website, and provides basic search functionality. The app focuses on core functionality without overengineering, tailored for music production metadata management.

### Target Audience

- A single macOS user managing a personal music library, with plans to expose the library via a Next.js website.

### Key Objectives

- Enable bulk upload and editing of music tracks with specific metadata fields.
- Transcode audio files during upload to optimize streaming for website users.
- Store files in Cloudflare R2 and metadata in MongoDB, ensuring compatibility with a Next.js website.
- Provide a simple, efficient interface for a single user.

## Features and Requirements
### 1. Configuration and Security
#### Description

Allow the user to configure Cloudflare R2 and MongoDB credentials securely.

#### Requirements

- Credential Input: UI fields for R2 bucket name, access keys, and MongoDB connection string.
- Storage: Securely store credentials in macOS keychain using Tauri (implemented via the `keyring` crate in `main.rs`). A `dev_credentials.json` file is used as a fallback during debug builds if keychain access fails.
- Validation: Validate credentials during setup by testing connections to R2 and MongoDB.
- Security Option: Initially set R2 files as public, with future option for signed URLs.

#### Success Criteria

- User can successfully configure and save credentials, with validation confirming connectivity.

### 2. Data Model and Storage
#### Description

Structure data in MongoDB and Cloudflare R2 for efficient management and website integration.

#### Requirements

- MongoDB Collections:
  - Albums: name (string), art_path (string, R2 path), track_ids (array of ObjectIDs).
  - Tracks: filename (string, read-only), title (string), album_id (ObjectID), genre (array of strings), duration (number, read-only), writers (array of strings), writer_percentages (array of numbers), publishers (array of strings), publisher_percentages (array of numbers), comments (string), instruments (array of strings), mood (array of strings), file_paths (object with original, low, medium, high strings).
- Cloudflare R2 Storage:
  - Album art: albums/<album_id>/art.jpg
  - Track files: tracks/<album_id>/<track_id>/original.mp3, tracks/<album_id>/<track_id>/low.mp3, tracks/<album_id>/<track_id>/medium.mp3, tracks/<album_id>/<track_id>/high.mp3

#### Success Criteria

- Data is stored consistently, retrievable by both app and website, with unique IDs linking tracks to albums.

### 3. Bulk Uploading Music Files
#### Description

Enable users to upload multiple music files, transcode them, and store metadata.

#### Requirements

- File Selection: UI for selecting multiple local files.
- Metadata Extraction: Use audio-metadata to extract metadata (e.g., title, artist, duration).
- Transcoding: Use ffmpeg-rs to create:
  - Low-quality (128 kbps MP3).
  - Medium-quality (256 kbps MP3).
  - Original file as high-quality.
- Storage: Upload all versions to R2 with distinct paths; store metadata in MongoDB.
- Restrictions: Set filename to original name (read-only), calculate duration (read-only).
- Album Handling: Create new album document if not found, link tracks via album_id.

#### Success Criteria

- Multiple files upload successfully, with transcoded versions stored and metadata populated correctly.

### 4. Bulk Editing Metadata
#### Description

Allow bulk editing of specific metadata fields for multiple tracks.

#### Requirements

- Editable Fields: Writers, Publishers, Album, Genre, Instruments, Mood.
- Validation: Ensure writer_percentages and publisher_percentages sum to 100%.
- UI: Select multiple tracks, apply common values to editable fields.
- Restrictions: filename and duration are read-only; title and comments editable individually.
- Backend: Update MongoDB documents for selected track IDs with validated values.

#### Success Criteria

- Selected tracks update with new values, percentages validated, and changes reflect in MongoDB.

### 5. Uploading Images
#### Description

Support uploading album art associated with albums.

#### Requirements

- Image Upload: UI for uploading images, specifying album association.
- Storage: Upload to R2 at albums/<album_id>/art.jpg, update art_path in Albums collection.
- Focus: Primarily album art, minimal track-specific image support.

#### Success Criteria

- Album art uploads and links correctly to albums, visible in the app and retrievable by the website.

### 6. Search Functionality
#### Description

Provide basic search to find tracks and albums.

#### Requirements

- UI: Search bar accepting text input.
- Backend: Query MongoDB with text indexes on title, album.name, and genre.
- Results: Display matching tracks and albums in the frontend.

#### Success Criteria

- Search returns relevant results based on input, displayed clearly in the UI.

## Technical Implementation
### Frontend

- Technology: Svelte
- Responsibilities: UI for file selection, bulk editing, image upload, and search; communicates with backend via Tauri commands.
- Local Storage: Utilizes Drizzle ORM with a local SQLite database (`@libsql/client`) for frontend-specific session and authentication state management, separate from the main MongoDB data store.

### Backend

- Technology: Rust with Tauri
- Libraries:
  - audio-metadata: Metadata extraction.
  - aws-sdk-rust: R2 interactions (S3-compatible).
  - `mongodb`: MongoDB operations (official Rust driver).
  - ffmpeg-rs: Audio transcoding.
- Responsibilities: File handling, transcoding, cloud storage, database updates.

### Integration

- Tauri: Bridges Svelte frontend and Rust backend for desktop functionality.

## Integration with Next.js Website
### Description

Ensure the database and storage are accessible by a Next.js website for streaming.

### Requirements

- Database Access: Next.js connects to the same MongoDB using separate read-only credentials.
- File Access: Website retrieves files from R2, selecting quality (low, medium, high) based on user settings.
- Security: Option for signed URLs if files are private.

### Success Criteria

- Website can query albums/tracks and stream appropriate quality files efficiently.

## Non-Functional Requirements
### Performance

- Goal: Handle personal library sizes (e.g., thousands of tracks) without significant delays.
- Approach: Use asynchronous operations for uploads, transcoding, and queries.

### Error Handling

- Goal: Provide clear feedback on failures.
- Approach: Display errors for invalid credentials, upload failures, or validation issues; log details for debugging.

### Usability

- Goal: Simple, intuitive interface for a single user.
- Approach: Clean Svelte UI with minimal steps for core actions.


## Assumptions

- Single user requires no multi-user features or local authentication.
- Initial R2 files are public; private access can be added later.
- Transcoding levels (256 kbps) suit streaming needs.

## Out of Scope

- Playback functionality within the app.
- Advanced bulk editing (e.g., pattern-based edits).
- Multi-user support or local authentication.
- Track-specific images beyond minimal support.

## Dependencies

- Cloudflare R2: Object storage.
- MongoDB: Database.
- Libraries: audio-metadata, aws-sdk-rust, `mongodb`, ffmpeg-rs, `keyring`.
- Frameworks: Svelte, Tauri, Rust.

## Timeline and Milestones
### Phase 1: Setup and Core Functionality (2-3 weeks)

- Configure Tauri/Svelte/Rust environment.
- Implement credential setup and validation.
- Build bulk upload with transcoding and metadata extraction.

### Phase 2: Editing and Search (2 weeks)

- Develop bulk editing for specified fields with validation.
- Add basic search functionality.

### Phase 3: Image Upload and Integration (1-2 weeks)

- Implement album art upload.
- Test MongoDB and R2 integration for website compatibility.

### Phase 4: Polish and Testing (1 week)

- Refine UI/UX, handle errors, and test end-to-end functionality.

### Total Estimated Time: 6-8 weeks 