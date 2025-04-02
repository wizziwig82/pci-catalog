# Database Catalog Page Implementation Plan

## Overview
This plan outlines the implementation steps for Phase 2.1 of the Music Library Manager - the Database Catalog Page. The goal is to leverage existing code while adding new functionality to display, sort, edit, and manage tracks stored in MongoDB.

## 1. Create a New Route for the Catalog Page
- [x] Create a new route under `src/routes/catalog/+page.svelte`
- [x] Update the navigation in `src/routes/+layout.svelte` to include a link

## 2. Leverage Existing Code and Components
- [x] Reuse Track and Album data models from `src-tauri/src/storage/mongodb.rs`
- [ ] Reuse metadata editing functionality from the upload page
- [ ] Reuse the tag selector components for instruments and mood tags

## 3. Implement the Database Catalog UI
- [x] Create a table component to display tracks with:
  - [x] Column headers for all relevant metadata fields
  - [ ] Sortable columns with click-to-sort functionality
  - [ ] Row selection capabilities (single and multi-select)
  - [ ] A toolbar with action buttons (edit, delete, replace audio)

## 4. Implement Catalog Data Loading
- [x] Add Rust functions to:
  - [x] Fetch all tracks from MongoDB
  - [x] Support sorting and filtering of tracks
  - [x] Support pagination if the dataset becomes large

## 5. Add Track Selection and Editing Functionality
- [ ] Implement:
  - [ ] Single track selection (clicking a row)
  - [ ] Multi-track selection (checkboxes or Ctrl/Shift+click)
  - [ ] Reuse the existing metadata editing interface for consistency
  - [ ] Reuse validation for writer and publisher percentages

## 6. Implement Audio File Replacement
- [ ] Add functionality to:
  - [ ] Select a new audio file for an existing track
  - [ ] Transcode the new file using existing transcoding functionality
  - [ ] Replace the file in R2 and update the path in MongoDB

## 7. Implement Track Deletion
- [ ] Add functionality to:
  - [ ] Delete selected tracks from MongoDB
  - [ ] Delete corresponding files from R2
  - [ ] Update any affected albums (removing deleted track IDs)

## 8. Add Testing for New Functionality
- [ ] Write tests for:
  - [ ] Component tests for the catalog table UI
  - [ ] Integration tests for the track management functionality

## Implementation Sequence
1. [x] Create the new route and update navigation
2. [x] Set up database fetching functionality in Rust
3. [x] Implement table component with sorting
4. [ ] Add track selection and editing interfaces
5. [ ] Implement audio file replacement
6. [ ] Add track deletion functionality
7. [ ] Write comprehensive tests

## Review Points
We'll stop and test after completing each of the following milestones:
1. [x] New route creation and navigation
2. [x] Basic table display with data loading
3. [ ] Sorting functionality
4. [ ] Track selection and metadata editing
5. [ ] Audio file replacement
6. [ ] Track deletion

This approach ensures we can identify and fix any issues early in the development process. 

## Progress Notes
- Fixed MongoDB data model mismatch issues by creating a dedicated `TrackDocument` struct that matches the MongoDB schema
- Implemented a proper mapping between MongoDB documents and our application models
- Added MongoDB connection testing functionality to diagnose connection issues
- Enhanced logging to provide better visibility into MongoDB operations 