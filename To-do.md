# Music Management App Development To-Do List

This to-do list outlines the steps to build a music management app, covering setup, core features, editing, search, image integration, and testing. Tasks are listed in the order they should be completed.

## Phase 1: Setup and Core Functionality
### 1.1 Set Up Development Environment

- [x] Install Rust, Tauri, and Svelte on the development machine.
- [x] Create a new Tauri project using Svelte as the frontend framework.
- [x] Set up version control (e.g., Git) and initialize a repository.

### 1.2 Configure Cloudflare R2 and MongoDB

- [ ] Create a Cloudflare R2 bucket and obtain access keys.
- [x] Set up a MongoDB database with collections for Albums and Tracks.
- [x] Define the schema for Albums (e.g., name, art_path, album_id) and Tracks (e.g., title, album_id, metadata fields).
- [x] Write tests for R2 connection (upload/download test file) (in tests/unit/backend/storage).
- [x] Write tests for MongoDB connection and CRUD operations (in tests/unit/backend/storage).

### 1.3 Implement Credential Configuration

- [x] Create a settings page in Svelte for entering R2 and MongoDB credentials.
- [x] Use Tauri to securely store credentials in the macOS keychain (or equivalent for other platforms).
- [x] Implement validation to test connections to R2 and MongoDB.
- [x] Write tests for credential storage and retrieval (in tests/integration/credential-management).
- [x] Write tests for connection validation with both valid and invalid credentials (in tests/integration/credential-management).

### 1.4 Develop Bulk Upload Functionality with Metadata Editing

- [x] Create a UI component in Svelte for selecting multiple music files.
- [x] Implement file reading and metadata extraction (e.g., using a library like audio-metadata).
- [x] Set up transcoding with ffmpeg to generate medium quality and original versions of each track.
- [x] Modify upload workflow to pause after metadata extraction and transcoding.
- [x] Create UI for reviewing and editing extracted metadata before final upload.
- [x] Implement bulk editing capability for multiple tracks in the upload page.
- [x] Implement individual track metadata editing in the upload page.
- [x] Add a final upload step to send data to R2 and MongoDB after metadata editing is complete.
- [x] Upload transcoded files to R2 with organized paths (e.g., /tracks/{quality}/{filename}).
- [x] Store metadata in MongoDB, linking tracks to albums via album_id.
- [x] Write unit tests for metadata extraction (in tests/unit/backend/metadata).
- [x] Write unit tests for transcoding functionality (in tests/unit/backend/transcoding).
- [x] Write integration tests for the upload workflow (in tests/integration/upload-workflow).
- [ ] Update integration tests for the new upload workflow with metadata editing (in tests/integration/upload-workflow).

### 1.5 Handle Album Creation

- [x] Implement logic to check if an album exists in MongoDB based on metadata; create a new album if it doesn't.
- [x] Ensure tracks are correctly associated with albums using album_id.
- [x] Write tests for album existence check and creation (in tests/unit/backend/metadata).
- [x] Write tests for track-to-album association (in tests/integration/upload-workflow).

## Phase 2: Editing and Search
### 2.1 Implement Database Catalog Page

- [ ] Create a new tab/page in the UI for displaying the uploaded catalog.
- [ ] Design and implement a table component to display tracks and their metadata.
- [ ] Add column sorting functionality for all metadata fields.
- [ ] Implement track selection (individual and multiple) for editing.
- [ ] Add edit mode for selected tracks with metadata editing capabilities.
- [ ] Implement functionality to replace audio files for existing tracks.
- [ ] Add functionality to delete records from both MongoDB and R2.
- [ ] Write component tests for the catalog table UI (in tests/unit/frontend/components).
- [ ] Write integration tests for the track management functionality (in tests/integration/catalog-management).

### 2.2 Implement Bulk Editing in Database Page

- [ ] Integrate the bulk editing UI with the database catalog page.
- [ ] Allow editing fields like Writers, Publishers, Album, Genre, Instruments, and Mood for selected tracks.
- [ ] Add validation to ensure writer_percentages and publisher_percentages sum to 100%.
- [ ] Develop backend logic in Rust to update the corresponding MongoDB documents.
- [ ] Write unit tests for percentage validation logic (in tests/unit/backend/metadata).
- [ ] Write component tests for the bulk editing UI (in tests/unit/frontend/components).
- [ ] Write integration tests for the editing workflow (in tests/integration/editing-workflow).

### 2.3 Add Individual Metadata Editing in Database Page

- [ ] Enhance the database page with dedicated UI for editing individual track metadata.
- [ ] Allow editing of title and comments for individual tracks.
- [ ] Display filename and duration in the UI, ensuring they are read-only.
- [ ] Write tests for field editability constraints (in tests/unit/frontend/components).
- [ ] Write tests for individual metadata updates (in tests/integration/editing-workflow).

### 2.4 Develop Search Functionality for Database Page

- [ ] Add a search bar to the database page in the Svelte frontend.
- [ ] Implement text indexing in MongoDB for fields like title, album.name, and genre.
- [ ] Create backend queries in Rust to search MongoDB and return matching tracks and albums.
- [ ] Display search results in the catalog table with highlighting for matched terms.
- [ ] Allow filtering of the catalog based on search results.
- [ ] Write unit tests for search query construction (in tests/unit/backend/metadata).
- [ ] Write integration tests for search functionality and result display (in tests/integration/search-functionality).
- [ ] Write performance tests with larger datasets (in tests/integration/search-functionality).

## Phase 3: Image Upload and Integration
### 3.1 Implement Album Art Upload

- [ ] Create a UI component in Svelte for uploading album art and associating it with an album.
- [ ] Upload the image to Cloudflare R2 and store the art_path in the Albums collection in MongoDB.
- [ ] Ensure the app can retrieve and display the album art from R2.
- [ ] Integrate album art display in the database catalog page.
- [ ] Write tests for image upload UI component (in tests/unit/frontend/components).
- [ ] Write tests for R2 image storage and retrieval (in tests/unit/backend/storage).
- [ ] Write integration tests for album art management (in tests/integration/album-art-management).

### 3.2 Test Integration with Next.js Website

- [ ] Set up a basic Next.js project that connects to the same MongoDB database and R2 bucket.
- [ ] Verify that the Next.js site can query albums and tracks from MongoDB.
- [ ] Test streaming of tracks from R2 at different quality levels (low, medium, original).
- [ ] Write tests for Next.js database access (in tests/e2e/website-integration).
- [ ] Write tests for streaming functionality at different quality levels (in tests/e2e/website-integration).

## Phase 4: Polish and Testing
### 4.1 Refine UI/UX

- [ ] Ensure the Svelte interface is clean, intuitive, and responsive across all pages (Upload, Database, Settings).
- [ ] Implement consistent navigation between pages.
- [ ] Add tooltips or help text for complex fields (e.g., percentage inputs).
- [ ] Create loading indicators for database operations and file uploads.
- [ ] Write UI responsiveness tests (in tests/unit/frontend/components).
- [ ] Write accessibility tests (in tests/unit/frontend/components).

### 4.2 Implement Error Handling

- [ ] Display user-friendly error messages for issues like invalid credentials or upload failures.
- [ ] Implement error handling for database operations in the catalog page.
- [ ] Log detailed errors in the backend for debugging purposes.
- [ ] Write tests for error message display (in tests/unit/frontend/components).
- [ ] Write tests for error logging functionality (in tests/unit/backend).

### 4.3 Conduct End-to-End Testing

- [ ] Test bulk upload with 100 tracks, verifying metadata editing, transcoding, and final upload.
- [ ] Test database catalog viewing, sorting, and filtering with a large dataset.
- [ ] Test bulk editing and individual editing in both the upload and database pages.
- [ ] Verify search functionality works with a large dataset.
- [ ] Test audio file replacement and record deletion functionality.
- [ ] Confirm album art uploads and displays correctly in the app.
- [ ] Ensure the Next.js website integrates seamlessly with the app's data.
- [ ] Write comprehensive end-to-end workflow tests covering the complete user journey (in tests/e2e/complete-workflows).

## Additional Tasks

- [ ] Implement logging in Rust for debugging and monitoring app behavior.
- [ ] Add progress indicators in the Svelte UI for long operations like bulk uploads.
- [ ] Handle edge cases, such as missing metadata or invalid file formats.
- [ ] Implement data export functionality for backup purposes.
- [ ] Add keyboard shortcuts for common operations in the database catalog page.
- [ ] Write tests for edge case handling (in tests/unit/backend). 