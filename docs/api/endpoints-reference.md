# API Endpoints Reference

**Date**: [Current Date]

This document provides a comprehensive reference for all API endpoints in the PCI File Manager application.

## Base URL

All endpoints are relative to the base API URL:

- **Development**: `http://localhost:3000/api`
- **Production**: `https://app.example.com/api`

## Authentication

Unless specified otherwise, all endpoints require authentication via JWT token in the Authorization header:

```
Authorization: Bearer <your_jwt_token>
```

## Endpoint Categories

- [Authentication](#authentication-endpoints)
- [Tracks](#tracks-endpoints)
- [File Management](#file-management-endpoints)
- [Users](#users-endpoints)
- [Playlists](#playlists-endpoints)
- [System](#system-endpoints)

## Authentication Endpoints

### Login

Authenticates a user and returns a JWT token.

- **URL**: `/auth/login`
- **Method**: `POST`
- **Auth Required**: No
- **Permissions**: None

**Request Body**:

```json
{
  "username": "admin",
  "password": "your_password"
}
```

**Response**:

```json
{
  "success": true,
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refreshToken": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "user": {
    "id": "60a2b3c4d5e6f7g8h9i0j1k2",
    "username": "admin",
    "email": "admin@example.com",
    "role": "admin"
  }
}
```

**Error Responses**:

- `401 Unauthorized`: Invalid credentials
- `403 Forbidden`: Account is locked or disabled
- `429 Too Many Requests`: Rate limit exceeded (too many login attempts)

### Refresh Token

Generates a new access token using a refresh token.

- **URL**: `/auth/refresh`
- **Method**: `POST`
- **Auth Required**: No
- **Permissions**: None

**Request Body**:

```json
{
  "refreshToken": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

**Response**:

```json
{
  "success": true,
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

**Error Responses**:

- `401 Unauthorized`: Invalid refresh token
- `403 Forbidden`: Refresh token expired or revoked

### Logout

Invalidates the current token and any associated refresh tokens.

- **URL**: `/auth/logout`
- **Method**: `POST`
- **Auth Required**: Yes
- **Permissions**: None

**Response**:

```json
{
  "success": true,
  "message": "Successfully logged out"
}
```

## Tracks Endpoints

### Get All Tracks

Retrieves a paginated list of tracks with filtering and sorting options.

- **URL**: `/tracks`
- **Method**: `GET`
- **Auth Required**: Yes
- **Permissions**: View Tracks

**Query Parameters**:

- `page` (integer, default: 1): Page number
- `limit` (integer, default: 20, max: 100): Number of items per page
- `sort` (string, default: "dateAdded"): Field to sort by
- `order` (string, default: "desc"): Sort order, either "asc" or "desc"
- `search` (string): Search term for title, artist, album
- `artist` (string): Filter by artist
- `album` (string): Filter by album
- `genre` (string): Filter by genre
- `year` (integer): Filter by release year

**Response**:

```json
{
  "tracks": [
    {
      "id": "60a2b3c4d5e6f7g8h9i0j1k2",
      "title": "Track Title",
      "artist": "Artist Name",
      "album": "Album Name",
      "duration": 240,
      "audioUrl": "https://audio.example.com/path/to/file.mp3",
      "coverArt": {
        "url": "https://images.example.com/path/to/image.jpg",
        "width": 600,
        "height": 600
      },
      "dateAdded": "2023-06-15T10:30:00Z"
    },
    // More tracks...
  ],
  "pagination": {
    "total": 154,
    "page": 1,
    "limit": 20,
    "pages": 8
  }
}
```

### Get Track by ID

Retrieves detailed information about a specific track.

- **URL**: `/tracks/:id`
- **Method**: `GET`
- **Auth Required**: Yes
- **Permissions**: View Tracks

**URL Parameters**:

- `id` (string): Track ID

**Response**:

```json
{
  "id": "60a2b3c4d5e6f7g8h9i0j1k2",
  "title": "Track Title",
  "artist": "Artist Name",
  "album": "Album Name",
  "duration": 240,
  "genre": ["Rock", "Alternative"],
  "releaseYear": 2023,
  "bpm": 120,
  "key": "C Major",
  "tags": ["energetic", "guitar"],
  "audioUrl": "https://audio.example.com/path/to/file.mp3",
  "originalFormat": "mp3",
  "transcodedVersions": [
    {
      "quality": "high",
      "format": "aac",
      "url": "https://audio.example.com/path/to/file_high.m4a",
      "size": 9876543
    },
    {
      "quality": "medium",
      "format": "aac",
      "url": "https://audio.example.com/path/to/file_medium.m4a",
      "size": 5432109
    }
  ],
  "coverArt": {
    "url": "https://images.example.com/path/to/image.jpg",
    "width": 600,
    "height": 600,
    "format": "jpg"
  },
  "dateAdded": "2023-06-15T10:30:00Z",
  "lastModified": "2023-06-15T10:30:00Z",
  "addedBy": "60a2b3c4d5e6f7g8h9i0j1k2",
  "modifiedBy": "60a2b3c4d5e6f7g8h9i0j1k2",
  "isPublic": true,
  "playCount": 42,
  "downloadCount": 12
}
```

**Error Responses**:

- `404 Not Found`: Track not found

### Create Track

Creates a new track record (metadata only, use File Upload endpoints for audio files).

- **URL**: `/tracks`
- **Method**: `POST`
- **Auth Required**: Yes
- **Permissions**: Upload Tracks

**Request Body**:

```json
{
  "title": "New Track Title",
  "artist": "Artist Name",
  "album": "Album Name",
  "genre": ["Rock", "Alternative"],
  "releaseYear": 2023,
  "bpm": 120,
  "key": "C Major",
  "tags": ["energetic", "guitar"],
  "isPublic": true
}
```

**Response**:

```json
{
  "id": "60a2b3c4d5e6f7g8h9i0j1k2",
  "title": "New Track Title",
  "message": "Track created successfully",
  "uploadUrl": "https://upload.example.com/presigned-url"
}
```

**Error Responses**:

- `400 Bad Request`: Validation error
- `403 Forbidden`: Insufficient permissions

### Update Track

Updates an existing track's metadata.

- **URL**: `/tracks/:id`
- **Method**: `PUT`
- **Auth Required**: Yes
- **Permissions**: Edit Tracks

**URL Parameters**:

- `id` (string): Track ID

**Request Body**:

```json
{
  "title": "Updated Track Title",
  "artist": "Updated Artist Name",
  "album": "Updated Album Name",
  "genre": ["Rock", "Alternative"],
  "releaseYear": 2023,
  "bpm": 120,
  "key": "C Major",
  "tags": ["energetic", "guitar"],
  "isPublic": true
}
```

**Response**:

```json
{
  "id": "60a2b3c4d5e6f7g8h9i0j1k2",
  "message": "Track updated successfully"
}
```

**Error Responses**:

- `400 Bad Request`: Validation error
- `403 Forbidden`: Insufficient permissions
- `404 Not Found`: Track not found

### Delete Track

Deletes a track and associated files.

- **URL**: `/tracks/:id`
- **Method**: `DELETE`
- **Auth Required**: Yes
- **Permissions**: Delete Tracks

**URL Parameters**:

- `id` (string): Track ID

**Response**:

```json
{
  "id": "60a2b3c4d5e6f7g8h9i0j1k2",
  "message": "Track deleted successfully"
}
```

**Error Responses**:

- `403 Forbidden`: Insufficient permissions
- `404 Not Found`: Track not found

### Batch Update Tracks

Updates multiple tracks at once.

- **URL**: `/tracks/batch`
- **Method**: `PUT`
- **Auth Required**: Yes
- **Permissions**: Edit Tracks

**Request Body**:

```json
{
  "ids": ["60a2b3c4d5e6f7g8h9i0j1k2", "70a2b3c4d5e6f7g8h9i0j1k3"],
  "updates": {
    "artist": "Common Artist",
    "album": "Common Album",
    "genre": ["Common Genre"],
    "releaseYear": 2023,
    "isPublic": true
  }
}
```

**Response**:

```json
{
  "updatedCount": 2,
  "message": "2 tracks updated successfully"
}
```

**Error Responses**:

- `400 Bad Request`: Validation error
- `403 Forbidden`: Insufficient permissions

### Get Track Waveform

Retrieves waveform data for a track.

- **URL**: `/tracks/:id/waveform`
- **Method**: `GET`
- **Auth Required**: Yes
- **Permissions**: View Tracks

**URL Parameters**:

- `id` (string): Track ID

**Query Parameters**:

- `resolution` (integer, default: 800): Number of data points to return

**Response**:

```json
{
  "trackId": "60a2b3c4d5e6f7g8h9i0j1k2",
  "duration": 240,
  "resolution": 800,
  "waveform": [0.1, 0.2, 0.3, 0.5, 0.8, 0.4, /* ... */]
}
```

**Error Responses**:

- `404 Not Found`: Track not found

## File Management Endpoints

### Initiate File Upload

Initiates a new file upload and returns pre-signed URLs for direct upload to R2.

- **URL**: `/uploads/initiate`
- **Method**: `POST`
- **Auth Required**: Yes
- **Permissions**: Upload Tracks

**Request Body**:

```json
{
  "filename": "track.mp3",
  "fileSize": 12345678,
  "contentType": "audio/mp3",
  "trackId": "60a2b3c4d5e6f7g8h9i0j1k2", // Optional, for replacing existing track
  "multipart": true // For large files using multipart upload
}
```

**Response (Single Upload)**:

```json
{
  "uploadId": "upload_123456789",
  "uploadUrl": "https://upload.example.com/presigned-url",
  "trackId": "60a2b3c4d5e6f7g8h9i0j1k2",
  "expiresAt": "2023-06-15T11:30:00Z"
}
```

**Response (Multipart Upload)**:

```json
{
  "uploadId": "upload_123456789",
  "r2UploadId": "r2-multipart-upload-id",
  "parts": [
    {
      "partNumber": 1,
      "uploadUrl": "https://upload.example.com/part-1-presigned-url"
    },
    {
      "partNumber": 2,
      "uploadUrl": "https://upload.example.com/part-2-presigned-url"
    }
    // More parts...
  ],
  "trackId": "60a2b3c4d5e6f7g8h9i0j1k2",
  "expiresAt": "2023-06-15T11:30:00Z"
}
```

**Error Responses**:

- `400 Bad Request`: Validation error
- `403 Forbidden`: Insufficient permissions

### Complete Multipart Upload

Completes a multipart upload after all parts have been uploaded.

- **URL**: `/uploads/:uploadId/complete`
- **Method**: `POST`
- **Auth Required**: Yes
- **Permissions**: Upload Tracks

**URL Parameters**:

- `uploadId` (string): Upload ID

**Request Body**:

```json
{
  "r2UploadId": "r2-multipart-upload-id",
  "parts": [
    {
      "partNumber": 1,
      "etag": "\"abc123\""
    },
    {
      "partNumber": 2,
      "etag": "\"def456\""
    }
    // More parts...
  ]
}
```

**Response**:

```json
{
  "trackId": "60a2b3c4d5e6f7g8h9i0j1k2",
  "message": "Upload completed successfully",
  "processingStatus": "queued",
  "processingId": "process_123456789"
}
```

**Error Responses**:

- `400 Bad Request`: Invalid upload ID or parts
- `403 Forbidden`: Insufficient permissions

### Get Upload Status

Checks the status of an ongoing upload or processing job.

- **URL**: `/uploads/:uploadId/status`
- **Method**: `GET`
- **Auth Required**: Yes
- **Permissions**: Upload Tracks

**URL Parameters**:

- `uploadId` (string): Upload ID

**Response**:

```json
{
  "uploadId": "upload_123456789",
  "trackId": "60a2b3c4d5e6f7g8h9i0j1k2",
  "status": "processing",
  "progress": 75,
  "stage": "transcoding",
  "message": "Transcoding to high quality",
  "estimatedTimeRemaining": 45, // seconds
  "startedAt": "2023-06-15T10:30:00Z",
  "updatedAt": "2023-06-15T10:45:00Z"
}
```

**Error Responses**:

- `404 Not Found`: Upload not found

### Upload Cover Art

Uploads cover art for a track.

- **URL**: `/tracks/:id/cover`
- **Method**: `POST`
- **Auth Required**: Yes
- **Permissions**: Edit Tracks
- **Content-Type**: `multipart/form-data`

**URL Parameters**:

- `id` (string): Track ID

**Form Data**:

- `image`: The image file (JPEG or PNG, max 5MB)

**Response**:

```json
{
  "trackId": "60a2b3c4d5e6f7g8h9i0j1k2",
  "coverArt": {
    "url": "https://images.example.com/path/to/image.jpg",
    "width": 600,
    "height": 600,
    "format": "jpg"
  },
  "message": "Cover art uploaded successfully"
}
```

**Error Responses**:

- `400 Bad Request`: Invalid image format or size
- `403 Forbidden`: Insufficient permissions
- `404 Not Found`: Track not found

## Users Endpoints

### Get Current User

Retrieves information about the authenticated user.

- **URL**: `/users/me`
- **Method**: `GET`
- **Auth Required**: Yes
- **Permissions**: None

**Response**:

```json
{
  "id": "60a2b3c4d5e6f7g8h9i0j1k2",
  "username": "admin",
  "email": "admin@example.com",
  "firstName": "Admin",
  "lastName": "User",
  "role": "admin",
  "permissions": {
    "canUpload": true,
    "canEdit": true,
    "canDelete": true,
    "canManageUsers": true
  },
  "dateCreated": "2023-01-01T00:00:00Z",
  "lastLogin": "2023-06-15T10:00:00Z"
}
```

### Get All Users

Retrieves a list of all users (admin only).

- **URL**: `/users`
- **Method**: `GET`
- **Auth Required**: Yes
- **Permissions**: Manage Users

**Response**:

```json
{
  "users": [
    {
      "id": "60a2b3c4d5e6f7g8h9i0j1k2",
      "username": "admin",
      "email": "admin@example.com",
      "role": "admin",
      "isActive": true,
      "dateCreated": "2023-01-01T00:00:00Z",
      "lastLogin": "2023-06-15T10:00:00Z"
    },
    // More users...
  ]
}
```

**Error Responses**:

- `403 Forbidden`: Insufficient permissions

### Create User

Creates a new user (admin only).

- **URL**: `/users`
- **Method**: `POST`
- **Auth Required**: Yes
- **Permissions**: Manage Users

**Request Body**:

```json
{
  "username": "newuser",
  "email": "newuser@example.com",
  "password": "securePassword123",
  "firstName": "New",
  "lastName": "User",
  "role": "editor",
  "permissions": {
    "canUpload": true,
    "canEdit": true,
    "canDelete": false,
    "canManageUsers": false
  }
}
```

**Response**:

```json
{
  "id": "60a2b3c4d5e6f7g8h9i0j1k2",
  "username": "newuser",
  "email": "newuser@example.com",
  "role": "editor",
  "message": "User created successfully"
}
```

**Error Responses**:

- `400 Bad Request`: Validation error (e.g., username already exists)
- `403 Forbidden`: Insufficient permissions

## Playlists Endpoints

### Get All Playlists

Retrieves a list of playlists.

- **URL**: `/playlists`
- **Method**: `GET`
- **Auth Required**: Yes
- **Permissions**: View Tracks

**Response**:

```json
{
  "playlists": [
    {
      "id": "60a2b3c4d5e6f7g8h9i0j1k2",
      "name": "Rock Playlist",
      "description": "A collection of rock tracks",
      "trackCount": 25,
      "coverArt": "https://images.example.com/path/to/image.jpg",
      "createdBy": "admin",
      "dateCreated": "2023-06-01T12:00:00Z"
    },
    // More playlists...
  ]
}
```

### Create Playlist

Creates a new playlist.

- **URL**: `/playlists`
- **Method**: `POST`
- **Auth Required**: Yes
- **Permissions**: View Tracks

**Request Body**:

```json
{
  "name": "New Playlist",
  "description": "A new playlist description",
  "tracks": ["60a2b3c4d5e6f7g8h9i0j1k2", "70a2b3c4d5e6f7g8h9i0j1k3"],
  "isPublic": true
}
```

**Response**:

```json
{
  "id": "60a2b3c4d5e6f7g8h9i0j1k2",
  "name": "New Playlist",
  "message": "Playlist created successfully"
}
```

**Error Responses**:

- `400 Bad Request`: Validation error
- `403 Forbidden`: Insufficient permissions

## System Endpoints

### Get System Status

Retrieves system status information (admin only).

- **URL**: `/system/status`
- **Method**: `GET`
- **Auth Required**: Yes
- **Permissions**: System Config

**Response**:

```json
{
  "status": "healthy",
  "version": "1.0.0",
  "uptime": 345600, // seconds
  "storage": {
    "totalSize": 1099511627776, // bytes (1TB)
    "usedSize": 536870912000, // bytes (500GB)
    "percentUsed": 48.83
  },
  "processingQueue": {
    "active": 2,
    "waiting": 5,
    "completed": 1423,
    "failed": 12
  },
  "lastBackup": "2023-06-14T00:00:00Z"
}
```

**Error Responses**:

- `403 Forbidden`: Insufficient permissions

### Get Logs

Retrieves system logs (admin only).

- **URL**: `/system/logs`
- **Method**: `GET`
- **Auth Required**: Yes
- **Permissions**: System Config

**Query Parameters**:

- `level` (string, default: "error"): Log level (error, warn, info, debug)
- `limit` (integer, default: 100): Number of log entries to return
- `startDate` (string): Start date in ISO format
- `endDate` (string): End date in ISO format

**Response**:

```json
{
  "logs": [
    {
      "timestamp": "2023-06-15T10:45:23Z",
      "level": "error",
      "message": "Failed to transcode file: corrupt input file",
      "trackId": "60a2b3c4d5e6f7g8h9i0j1k2",
      "userId": "70a2b3c4d5e6f7g8h9i0j1k3"
    },
    // More logs...
  ],
  "count": 42,
  "level": "error"
}
```

**Error Responses**:

- `403 Forbidden`: Insufficient permissions

## Electron-Specific Endpoints

These endpoints are specific to the Electron application and not meant for external use.

### Get App Configuration

Retrieves configuration for the Electron application.

- **URL**: `/electron/config`
- **Method**: `GET`
- **Auth Required**: Yes
- **Permissions**: None

**Response**:

```json
{
  "uploadConcurrency": 3,
  "maxFileSize": 1073741824, // bytes (1GB)
  "supportedFormats": ["mp3", "wav", "flac", "aac", "m4a"],
  "tempDirectory": "/path/to/temp",
  "defaultTranscodingOptions": {
    "formats": ["aac"],
    "qualities": ["high", "medium"]
  },
  "uiSettings": {
    "theme": "dark",
    "animations": true,
    "language": "en"
  }
}
```

## Error Handling

All endpoints follow a consistent error response format:

```json
{
  "error": true,
  "code": "VALIDATION_ERROR",
  "message": "User-friendly error message",
  "details": {
    "field": "specific field with error",
    "reason": "specific reason for error"
  }
}
```

Common error codes:

- `UNAUTHORIZED`: Authentication required or failed
- `FORBIDDEN`: Insufficient permissions
- `NOT_FOUND`: Resource not found
- `VALIDATION_ERROR`: Invalid request data
- `INTERNAL_ERROR`: Server error
- `RATE_LIMITED`: Too many requests

## Versioning

The API follows semantic versioning. The current version is v1.

To specify a version, use the `Accept` header:

```
Accept: application/json; version=1
```

## Rate Limiting

API requests are subject to rate limiting to prevent abuse. The current limits are:

- 100 requests per minute for authenticated users
- 20 requests per minute for unauthenticated requests

Rate limit headers are included in all responses:

```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1623760932
```

## Related Documents

- [Authentication Flow](../architecture/authentication-flow.md)
- [Database Schema](../architecture/database-schema.md)
- [R2 Storage Design](../architecture/r2-storage-design.md) 