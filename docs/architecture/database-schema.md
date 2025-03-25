# MongoDB Database Schema

**Date**: [Current Date]

This document outlines the MongoDB database schema for the PCI File Manager application.

## Collections

### 1. Tracks

The main collection for storing audio files metadata.

```javascript
{
  _id: ObjectId,            // MongoDB automatically generated ID
  title: String,            // Required: Title of the track
  artist: String,           // Required: Artist name
  album: String,            // Optional: Album name
  duration: Number,         // Required: Duration in seconds (auto-extracted)
  genre: [String],          // Optional: Array of genres
  releaseYear: Number,      // Optional: Year of release
  bpm: Number,              // Optional: Beats per minute
  key: String,              // Optional: Musical key
  tags: [String],           // Optional: Array of tags for categorization
  
  // File information
  audioUrl: String,         // Required: URL to the audio file in R2
  originalFormat: String,   // Required: Original file format
  transcodedVersions: [     // Optional: Transcoded versions
    {
      format: String,       // Format of the transcoded version
      quality: String,      // Quality setting (e.g., "high", "medium", "low")
      url: String,          // URL to the transcoded file in R2
      size: Number          // File size in bytes
    }
  ],
  
  // Cover art
  coverArt: {
    url: String,            // URL to the cover art in R2
    width: Number,          // Image width in pixels
    height: Number,         // Image height in pixels
    format: String          // Image format
  },
  
  // Metadata
  dateAdded: Date,          // Required: Date when track was added
  lastModified: Date,       // Required: Date when track was last modified
  addedBy: ObjectId,        // Optional: Reference to user who added the track
  modifiedBy: ObjectId,     // Optional: Reference to user who last modified the track
  isPublic: Boolean,        // Optional: Whether the track is publicly accessible
  
  // Analytics (optional)
  playCount: Number,        // Number of times the track has been played
  downloadCount: Number     // Number of times the track has been downloaded
}
```

### 2. Users

Collection for storing user information for authentication and authorization.

```javascript
{
  _id: ObjectId,            // MongoDB automatically generated ID
  username: String,         // Required: Username for login
  email: String,            // Required: Email address
  passwordHash: String,     // Required: Hashed password
  role: String,             // Required: User role (admin, editor, viewer)
  firstName: String,        // Optional: First name
  lastName: String,         // Optional: Last name
  dateCreated: Date,        // Required: Date when user was created
  lastLogin: Date,          // Optional: Date of last login
  isActive: Boolean,        // Required: Whether the user account is active
  
  // Permissions (optional, for fine-grained control)
  permissions: {
    canUpload: Boolean,
    canEdit: Boolean,
    canDelete: Boolean,
    canManageUsers: Boolean
  }
}
```

### 3. Playlists (Optional)

Collection for organizing tracks into playlists.

```javascript
{
  _id: ObjectId,            // MongoDB automatically generated ID
  name: String,             // Required: Name of the playlist
  description: String,      // Optional: Description of the playlist
  tracks: [ObjectId],       // Required: Array of track IDs
  createdBy: ObjectId,      // Required: Reference to user who created the playlist
  dateCreated: Date,        // Required: Date when playlist was created
  lastModified: Date,       // Required: Date when playlist was last modified
  isPublic: Boolean,        // Optional: Whether the playlist is publicly accessible
  coverArt: String          // Optional: URL to the playlist cover art in R2
}
```

### 4. ActivityLog (Optional)

Collection for tracking system activity for auditing and analytics.

```javascript
{
  _id: ObjectId,            // MongoDB automatically generated ID
  userId: ObjectId,         // Required: Reference to user who performed the action
  action: String,           // Required: Type of action (upload, edit, delete, etc.)
  resourceType: String,     // Required: Type of resource affected (track, user, playlist)
  resourceId: ObjectId,     // Required: ID of the resource affected
  details: Object,          // Optional: Additional details about the action
  timestamp: Date,          // Required: Timestamp of the action
  ipAddress: String         // Optional: IP address of the user
}
```

## Indexes

For optimal performance, the following indexes should be created:

### Tracks Collection
- `title`: Text index for search
- `artist`: Text index for search
- `album`: Text index for search
- `tags`: Index for filtering
- `dateAdded`: Index for sorting
- `lastModified`: Index for sorting

### Users Collection
- `username`: Unique index
- `email`: Unique index
- `role`: Index for filtering

### Playlists Collection
- `name`: Text index for search
- `createdBy`: Index for filtering
- `tracks`: Index for lookups

### ActivityLog Collection
- `userId`: Index for filtering
- `action`: Index for filtering
- `resourceType` + `resourceId`: Compound index
- `timestamp`: Index for sorting and filtering

## Relationships

- **Tracks → Users**: Many-to-one relationship through `addedBy` and `modifiedBy` fields
- **Playlists → Tracks**: Many-to-many relationship through the `tracks` array
- **Playlists → Users**: Many-to-one relationship through the `createdBy` field
- **ActivityLog → Users**: Many-to-one relationship through the `userId` field
- **ActivityLog → Resources**: Many-to-one relationship through the `resourceType` and `resourceId` fields

## Schema Validation

MongoDB schema validation should be implemented to ensure data integrity. Example validation rules:

```javascript
// Tracks collection validation
{
  validator: {
    $jsonSchema: {
      bsonType: "object",
      required: ["title", "artist", "duration", "audioUrl", "dateAdded", "lastModified"],
      properties: {
        title: {
          bsonType: "string",
          description: "Title must be a string and is required"
        },
        artist: {
          bsonType: "string",
          description: "Artist must be a string and is required"
        },
        // Add validation for other fields
      }
    }
  }
}
```

## Data Migration Considerations

When implementing schema changes:

1. Use a staged approach for data migration
2. Create backup collections before major schema changes
3. Implement version control for schema changes
4. Test migrations on a staging environment before production

## Performance Considerations

1. Use appropriate data types for storage efficiency
2. Consider embedding vs. referencing based on query patterns
3. Monitor index usage and optimize as needed
4. Implement pagination for large collections
5. Use projection to limit returned fields for better performance

This schema provides a solid foundation for the music catalog management application while allowing for future expansion and optimization. 