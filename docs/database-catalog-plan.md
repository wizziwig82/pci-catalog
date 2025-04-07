# Database Catalog Page Implementation Plan

## Overview
This plan outlines the implementation steps for Phase 2.1 of the Music Library Manager - the Database Catalog Page. The goal is to leverage existing code while adding new functionality to display, sort, edit, and manage tracks stored in MongoDB.

## 1. Create a New Route for the Catalog Page
- [x] Create a new route under `src/routes/catalog/+page.svelte`
- [x] Update the navigation in `src/routes/+layout.svelte` to include a link

## 2. Leverage Existing Code and Components
- [x] Reuse Track and Album data models from `src-tauri/src/storage/mongodb.rs`
- [x] Reuse metadata editing functionality from the upload page (Completed: 2025-04-04)
- [x] Reuse the tag selector components for instruments and mood tags (Verified: 2025-04-05)

## 3. Implement the Database Catalog UI
- [x] Create a table component to display tracks with:
  - [x] Column headers for all relevant metadata fields
  - [x] Sortable columns with click-to-sort functionality (Verified: 2025-04-05)
  - [x] Row selection capabilities (single and multi-select) (Verified: 2025-04-05)
  - [x] A toolbar with action buttons (edit, delete, replace audio) (Completed: 2025-04-04)

## 4. Implement Catalog Data Loading
- [x] Add Rust functions to:
  - [x] Fetch all tracks from MongoDB
  - [x] Support sorting and filtering of tracks
  - [x] Support pagination if the dataset becomes large

## 5. Add Track Selection and Editing Functionality
- [ ] Implement:
  - [x] Single track selection (clicking a row) (Completed: 2025-04-04)
  - [x] Multi-track selection (checkboxes or Ctrl/Shift+click) (Checkboxes implemented: 2025-04-05)
  - [x] Reuse the existing metadata editing interface for consistency (Completed: 2025-04-04)
  - [x] Reuse validation for writer and publisher percentages (Verified: 2025-04-05)

## 6. Implement Audio File Replacement
- [ ] Add functionality to:
  - [x] Select a new audio file for an existing track (Completed: 2025-04-04)
  - [x] Transcode the new file using existing transcoding functionality (Completed: 2025-04-04)
  - [x] Replace the file in R2 and update the path in MongoDB (Completed: 2025-04-04)

## 7. Implement Track Deletion
- [ ] Add functionality to:
  - [x] Delete selected tracks from MongoDB (Completed: 2025-04-04)
  - [x] Delete corresponding files from R2 (Completed: 2025-04-04)
  - [x] Update any affected albums (removing deleted track IDs) (Completed: 2025-04-04)

## 8. Add Testing for New Functionality
- [ ] Write tests for:
  - [x] Component tests for the catalog table UI (Added basic tests: 2025-04-05)
  - [x] Integration tests for the track management functionality (Completed: 2025-04-04)

## Implementation Sequence
1. [x] Create the new route and update navigation
2. [x] Set up database fetching functionality in Rust
3. [x] Implement table component with sorting
4. [x] Add track selection and editing interfaces (Completed: 2025-04-04)
5. [x] Implement audio file replacement (Completed: 2025-04-04)
6. [x] Add track deletion functionality (Completed: 2025-04-04)
7. [x] Write comprehensive tests (Completed: 2025-04-04)

## Review Points
We'll stop and test after completing each of the following milestones:
1. [x] New route creation and navigation
2. [x] Basic table display with data loading
3. [x] Sorting functionality (Verified: 2025-04-05)
4. [x] Track selection and metadata editing (Completed: 2025-04-04)
5. [x] Audio file replacement (Completed: 2025-04-04)
6. [x] Track deletion (Completed: 2025-04-04)

This approach ensures we can identify and fix any issues early in the development process. 

## Progress Notes
- Fixed MongoDB data model mismatch issues by creating a dedicated `TrackDocument` struct that matches the MongoDB schema
- Implemented a proper mapping between MongoDB documents and our application models
- Added MongoDB connection testing functionality to diagnose connection issues
- Enhanced logging to provide better visibility into MongoDB operations 