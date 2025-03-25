# Cloudflare R2 Storage Design

**Date**: [Current Date]

This document outlines the storage architecture for audio files and related assets in Cloudflare R2 for the PCI File Manager application.

## Bucket Organization

### Primary Buckets

1. **`pci-audio-files`** - Main bucket for storing original and transcoded audio files
2. **`pci-cover-art`** - Bucket for storing album/track artwork images
3. **`pci-app-assets`** - Bucket for application assets (if needed)

### Environment-Specific Buckets

For development and testing purposes, we recommend using environment-specific bucket prefixes:
- `dev-pci-audio-files`
- `staging-pci-audio-files`
- `prod-pci-audio-files`

## File Structure and Naming Conventions

### Audio Files

Audio files will be organized using the following path structure:

```
/{fileType}/{artistName}/{albumName}/{trackId}/{filename}
```

Where:
- `{fileType}` is either "original" or "transcoded"
- `{artistName}` is a URL-safe version of the artist name
- `{albumName}` is a URL-safe version of the album name (use "singles" if no album)
- `{trackId}` is the MongoDB ObjectId of the track
- `{filename}` follows the pattern: `{trackTitle}_{quality}.{extension}`

Example paths:
```
/original/john_doe/best_album/60a2b3c4d5e6f7g8h9i0j1k2/track_title_original.mp3
/transcoded/john_doe/best_album/60a2b3c4d5e6f7g8h9i0j1k2/track_title_high.mp3
/transcoded/john_doe/best_album/60a2b3c4d5e6f7g8h9i0j1k2/track_title_medium.mp3
```

### Cover Art

Cover art will be organized using the following path structure:

```
/{entityType}/{entityId}/{size}/{filename}
```

Where:
- `{entityType}` is either "track" or "album"
- `{entityId}` is the MongoDB ObjectId of the entity
- `{size}` is the image size variant (original, thumbnail, small, medium, large)
- `{filename}` follows the pattern: `{entityName}_{size}.{extension}`

Example paths:
```
/track/60a2b3c4d5e6f7g8h9i0j1k2/original/track_title_original.jpg
/track/60a2b3c4d5e6f7g8h9i0j1k2/thumbnail/track_title_thumbnail.jpg
/album/60a2b3c4d5e6f7g8h9i0j1k2/medium/album_name_medium.jpg
```

## File Format Standards

### Audio Formats

Original files will be stored in their uploaded format. Transcoded versions will be created in the following formats:

1. **High Quality**
   - Format: AAC
   - Container: MP4
   - Bitrate: 256 kbps
   - Sample Rate: 48 kHz

2. **Medium Quality**
   - Format: AAC
   - Container: MP4
   - Bitrate: 128 kbps
   - Sample Rate: 44.1 kHz

3. **Low Quality (Optional)**
   - Format: AAC
   - Container: MP4
   - Bitrate: 64 kbps
   - Sample Rate: 44.1 kHz

### Image Formats

Cover art will be stored in the following formats:

1. **Original** - As uploaded by the user (maximum dimensions: 3000x3000px)
2. **Large** - JPEG, 1200x1200px
3. **Medium** - JPEG, 600x600px
4. **Small** - JPEG, 300x300px
5. **Thumbnail** - JPEG, 150x150px

All resized images will be square, using center cropping if the original has a different aspect ratio.

## Multi-part Upload Implementation

For large audio files, a multi-part upload strategy will be employed:

1. Client requests a multi-part upload initiation from the backend
2. Backend initiates a multi-part upload with R2 and returns upload ID and pre-signed URLs for each part
3. Client uploads file parts directly to R2 using pre-signed URLs
4. After all parts are uploaded, client notifies backend
5. Backend completes the multi-part upload in R2

Implementation details:
- Part size: 5MB (configurable based on file size)
- Maximum concurrent uploads: 3
- Client-side retry mechanism for failed part uploads

## Access Control

### Public vs. Private Access

Most audio files and cover art will need public access for streaming/display. To achieve this:

1. Set appropriate public access policies on the buckets
2. Use R2 Object Metadata to control access for specific files

For files that require restricted access:
1. Set these objects as private
2. Generate time-limited signed URLs when access is needed

### Custom Domain

For better user experience and branding, we'll use a custom domain for R2 assets:
- `audio.example.com` for audio files
- `images.example.com` for cover art

This requires setting up:
1. DNS CNAME records pointing to R2 endpoints
2. Custom SSL certificates for the domains

## Lifecycle Management

### Retention Policy

1. **Original Files** - Retained indefinitely
2. **Unused Transcoded Files** - If usage analytics shows no access for 6 months, consider deleting
3. **Orphaned Files** - Files with no corresponding MongoDB record should be identified and purged

### Versioning

For audio file replacement:
1. Keep the original file with a `.backup` suffix for 30 days
2. Replace the current file with the new version
3. Automate cleanup of backup files after retention period

## Monitoring and Analytics

To ensure optimal storage usage and performance:

1. **Usage Metrics**
   - Track total storage used per bucket
   - Monitor bandwidth usage
   - Track access patterns to identify popular content

2. **Cost Management**
   - Implement usage quotas per user/organization
   - Configure alerts for unusual storage growth
   - Periodic cleanup jobs for temporary/unused files

## Implementation Considerations

### Initialization

During application setup, automated scripts should:
1. Create required buckets if they don't exist
2. Configure bucket access policies
3. Set up CORS configurations to allow uploads from the application domain

### Error Handling

For R2 operations, implement robust error handling:
1. Network failures during upload/download
2. Bucket access permission issues
3. Storage quota exceeded scenarios
4. File corruption detection via checksums

### Backup Strategy

Critical metadata files should be backed up regularly. Consider:
1. Cross-region replication for critical content
2. Regular metadata exports to offline storage
3. Operational logs backup

## R2 SDK Integration

The application will use the official Cloudflare R2 SDK for Node.js. Key integration points:

```javascript
// Example integration points (pseudocode)
const { S3Client } = require("@aws-sdk/client-s3");
const { Upload } = require("@aws-sdk/lib-storage");

// Initialize R2 client
const r2Client = new S3Client({
  region: "auto",
  endpoint: `https://${accountId}.r2.cloudflarestorage.com`,
  credentials: {
    accessKeyId: process.env.R2_ACCESS_KEY_ID,
    secretAccessKey: process.env.R2_SECRET_ACCESS_KEY,
  },
});

// Functions to implement:
// - uploadAudioFile(file, metadata)
// - getSignedUrl(objectKey, expirationTime)
// - initiateMultipartUpload(key, contentType)
// - completeMultipartUpload(uploadId, key, parts)
// - generateThumbnails(imageFile)
```

## Related Documents

- [Audio Processing Pipeline](audio-processing-pipeline.md)
- [API Endpoints Reference](../api/endpoints-reference.md)
- [Cloudflare R2 Integration Guide](../guides/cloudflare-r2-integration.md) 