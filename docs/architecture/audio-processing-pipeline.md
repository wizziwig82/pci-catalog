# Audio Processing Pipeline

**Date**: [Current Date]

This document outlines the architecture and implementation details for the audio processing pipeline in the PCI File Manager application.

## Overview

The audio processing pipeline handles the transformation of uploaded audio files, extraction of metadata, and generation of various formats for different use cases. The pipeline is designed to be modular, scalable, and capable of handling various audio formats efficiently.

## Pipeline Architecture

```
┌────────────┐     ┌─────────────┐     ┌────────────────┐     ┌─────────────┐
│            │     │             │     │                │     │             │
│   Upload   │────>│  Metadata   │────>│   Transcoding  │────>│   Storage   │
│   Module   │     │  Extraction │     │     Module     │     │   Module    │
│            │     │             │     │                │     │             │
└────────────┘     └─────────────┘     └────────────────┘     └─────────────┘
       │                  │                    │                    │
       │                  │                    │                    │
       v                  v                    v                    v
┌────────────┐     ┌─────────────┐     ┌────────────────┐     ┌─────────────┐
│            │     │             │     │                │     │             │
│  Progress  │     │  MongoDB    │     │   Waveform     │     │ R2 Storage  │
│  Tracking  │     │  Database   │     │   Generation   │     │ Integration │
│            │     │             │     │                │     │             │
└────────────┘     └─────────────┘     └────────────────┘     └─────────────┘
```

## Components

### 1. Upload Module

Handles the initial reception of audio files from the user interface.

**Responsibilities:**
- Validate file formats (MP3, WAV, FLAC, AAC)
- Check file integrity
- Validate file size against limits
- Stream files to temporary storage
- Pass files to the metadata extraction module

**Implementation:**

```javascript
// Example upload handling (pseudocode)
async function handleFileUpload(file) {
  // Validate file format
  const validFormats = ['mp3', 'wav', 'flac', 'aac', 'm4a'];
  const extension = file.name.split('.').pop().toLowerCase();
  
  if (!validFormats.includes(extension)) {
    throw new Error('Unsupported file format');
  }
  
  // Stream to temp location
  const tempPath = path.join(os.tmpdir(), `upload_${Date.now()}_${file.name}`);
  await streamToFile(file, tempPath);
  
  // Proceed to metadata extraction
  return processAudioFile(tempPath, file.name);
}
```

### 2. Metadata Extraction Module

Extracts metadata from audio files to populate the database.

**Responsibilities:**
- Extract technical metadata (format, bitrate, duration)
- Extract embedded metadata (ID3 tags, etc.)
- Generate audio fingerprint (optional)
- Extract BPM and musical key (if possible)
- Pass metadata to MongoDB

**Implementation:**

```javascript
// Example metadata extraction (pseudocode)
async function extractMetadata(filePath) {
  try {
    // Use music-metadata library to extract data
    const metadata = await mm.parseFile(filePath);
    
    return {
      title: metadata.common.title || path.basename(filePath, path.extname(filePath)),
      artist: metadata.common.artist || 'Unknown Artist',
      album: metadata.common.album || 'Unknown Album',
      duration: metadata.format.duration,
      bitrate: metadata.format.bitrate,
      sampleRate: metadata.format.sampleRate,
      bpm: metadata.common.bpm,
      key: metadata.common.key,
      genre: metadata.common.genre,
      year: metadata.common.year,
      originalFormat: metadata.format.container || path.extname(filePath).substring(1)
    };
  } catch (error) {
    console.error('Error extracting metadata:', error);
    throw new Error('Failed to extract metadata');
  }
}
```

### 3. Transcoding Module

Converts audio to various formats and quality levels.

**Responsibilities:**
- Transcode to specified formats (AAC in MP4 container)
- Generate multiple quality versions
- Optimize audio for streaming
- Maintain audio quality within specifications
- Handle transcoding errors gracefully

**FFmpeg Integration:**

```javascript
// Example transcoding function (pseudocode)
async function transcodeAudio(inputPath, outputPath, format, quality) {
  return new Promise((resolve, reject) => {
    // Define quality presets
    const qualityPresets = {
      high: { bitrate: '256k', sampleRate: '48000' },
      medium: { bitrate: '128k', sampleRate: '44100' },
      low: { bitrate: '64k', sampleRate: '44100' }
    };
    
    const preset = qualityPresets[quality];
    
    // Setup FFmpeg command
    const ffmpeg = spawn('ffmpeg', [
      '-i', inputPath,
      '-c:a', format === 'aac' ? 'aac' : format,
      '-b:a', preset.bitrate,
      '-ar', preset.sampleRate,
      '-movflags', 'faststart',  // Optimize for streaming
      outputPath
    ]);
    
    // Error handling
    ffmpeg.on('error', (err) => reject(err));
    ffmpeg.stderr.on('data', (data) => console.log(data.toString()));
    
    // Success handling
    ffmpeg.on('close', (code) => {
      if (code === 0) {
        resolve(outputPath);
      } else {
        reject(new Error(`FFmpeg process exited with code ${code}`));
      }
    });
  });
}
```

### 4. Waveform Generation

Creates visual waveform representations of audio files.

**Responsibilities:**
- Generate visual waveform data
- Save waveform data in efficient format
- Handle various audio durations
- Create different resolution waveforms for different views

**Implementation:**

```javascript
// Example waveform generation (pseudocode)
async function generateWaveform(audioPath, outputPath) {
  return new Promise((resolve, reject) => {
    // Use audiowaveform tool or FFmpeg
    const process = spawn('audiowaveform', [
      '-i', audioPath,
      '-o', outputPath,
      '--pixels-per-second', '20',
      '--bits', '8'
    ]);
    
    process.on('error', reject);
    process.on('close', (code) => {
      if (code === 0) {
        resolve(outputPath);
      } else {
        reject(new Error(`Waveform generation failed with code ${code}`));
      }
    });
  });
}
```

### 5. Storage Module

Handles uploading processed files to Cloudflare R2.

**Responsibilities:**
- Upload original files to R2
- Upload transcoded versions to R2
- Generate appropriate file paths
- Handle upload errors and retries
- Update database with storage URLs

**Implementation Details:**

```javascript
// Example R2 storage function (pseudocode)
async function storeAudioInR2(localFilePath, metadata, quality = 'original') {
  try {
    // Create artist and album folders if needed
    const artist = sanitizePathComponent(metadata.artist);
    const album = metadata.album ? sanitizePathComponent(metadata.album) : 'singles';
    
    // Define folder structure
    const fileType = quality === 'original' ? 'original' : 'transcoded';
    const folderPath = `${fileType}/${artist}/${album}/${metadata.trackId}`;
    
    // Define filename
    const extension = path.extname(localFilePath);
    const baseFilename = sanitizePathComponent(metadata.title);
    const filename = `${baseFilename}_${quality}${extension}`;
    
    // Full path in R2
    const r2Path = `${folderPath}/${filename}`;
    
    // Upload to R2
    const fileContent = fs.readFileSync(localFilePath);
    await r2Client.send(new PutObjectCommand({
      Bucket: AUDIO_BUCKET_NAME,
      Key: r2Path,
      Body: fileContent,
      ContentType: getContentType(extension)
    }));
    
    // Generate public URL
    const url = `https://${AUDIO_BUCKET_NAME}.${R2_DOMAIN}/${r2Path}`;
    
    return { url, path: r2Path };
  } catch (error) {
    console.error('R2 upload error:', error);
    throw new Error('Failed to upload file to storage');
  }
}
```

## Workflow Integration

### Processing Queue System

The audio processing pipeline uses a queue-based system to handle processing asynchronously:

```javascript
// Example queue implementation (pseudocode)
const audioProcessingQueue = new Queue('audio-processing', {
  redis: { host: 'localhost', port: 6379 }
});

// Add job to queue
async function queueAudioProcessing(filePath, metadata) {
  return audioProcessingQueue.add('process', {
    filePath,
    metadata,
    qualities: ['high', 'medium']
  });
}

// Process queue
audioProcessingQueue.process('process', async (job) => {
  const { filePath, metadata, qualities } = job.data;
  
  try {
    // Update job progress
    job.progress(10);
    
    // Extract metadata if not provided
    const fullMetadata = metadata || await extractMetadata(filePath);
    job.progress(20);
    
    // Store original file
    const originalStorage = await storeAudioInR2(filePath, fullMetadata);
    job.progress(40);
    
    // Transcode and store different qualities
    const transcodedVersions = [];
    
    for (const quality of qualities) {
      job.log(`Transcoding to ${quality} quality`);
      
      const transcodedPath = path.join(
        os.tmpdir(), 
        `transcoded_${quality}_${path.basename(filePath)}`
      );
      
      // Transcode the file
      await transcodeAudio(filePath, transcodedPath, 'aac', quality);
      
      // Store transcoded version
      const storageResult = await storeAudioInR2(
        transcodedPath, 
        fullMetadata, 
        quality
      );
      
      transcodedVersions.push({
        quality,
        format: 'aac',
        url: storageResult.url,
        path: storageResult.path
      });
      
      // Delete temporary transcoded file
      fs.unlinkSync(transcodedPath);
      
      // Update progress
      job.progress(40 + (50 * (qualities.indexOf(quality) + 1) / qualities.length));
    }
    
    // Generate waveform
    const waveformPath = path.join(
      os.tmpdir(),
      `waveform_${path.basename(filePath)}.json`
    );
    await generateWaveform(filePath, waveformPath);
    
    // Store waveform data
    const waveformData = fs.readFileSync(waveformPath, 'utf8');
    job.progress(95);
    
    // Delete temporary files
    fs.unlinkSync(filePath);
    fs.unlinkSync(waveformPath);
    
    // Return results
    job.progress(100);
    return {
      metadata: fullMetadata,
      originalFile: originalStorage,
      transcodedVersions,
      waveformData
    };
  } catch (error) {
    // Log and rethrow for queue error handling
    console.error('Audio processing failed:', error);
    throw error;
  }
});

// Handle completion
audioProcessingQueue.on('completed', async (job, result) => {
  // Update database with processing results
  await updateTrackWithProcessingResults(job.id, result);
});

// Handle failures
audioProcessingQueue.on('failed', async (job, error) => {
  console.error(`Job ${job.id} failed:`, error);
  await updateTrackWithFailureStatus(job.id, error.message);
});
```

## Progress Tracking

Progress tracking is implemented to provide users with real-time updates on file processing:

```javascript
// Example progress tracking integration (pseudocode)
function trackProcessingProgress(trackId) {
  const progressChannel = `track-progress:${trackId}`;
  
  // Socket.IO example
  io.on('connection', (socket) => {
    socket.on('subscribe-to-progress', (requestedTrackId) => {
      if (requestedTrackId === trackId) {
        socket.join(progressChannel);
      }
    });
  });
  
  // Update progress
  function updateProgress(progress, status) {
    io.to(progressChannel).emit('progress-update', {
      trackId,
      progress,
      status
    });
    
    // Also update in database
    return db.collection('tracks').updateOne(
      { _id: ObjectId(trackId) },
      { 
        $set: { 
          processingProgress: progress, 
          processingStatus: status 
        } 
      }
    );
  }
  
  return {
    updateProgress
  };
}
```

## Error Handling and Recovery

The pipeline includes robust error handling and recovery mechanisms:

1. **Temporary File Management**
   - All temporary files are tracked and cleaned up after use
   - Orphaned temporary files are automatically cleaned up by a scheduled task

2. **Processing Retries**
   - Failed transcoding jobs are automatically retried up to 3 times
   - Exponential backoff is applied between retry attempts

3. **Partial Success Handling**
   - If some quality versions fail but others succeed, the track is still considered usable
   - Failed versions are flagged for manual review

4. **Monitoring and Alerting**
   - Critical failures trigger alerts to the development team
   - System maintains logs of all processing operations

## Performance Considerations

1. **Resource Management**
   - Transcoding is CPU-intensive, limit concurrent processes based on available CPU cores
   - Memory usage is monitored and limited to prevent system instability
   - Disk I/O is optimized with buffered operations

2. **Scaling Options**
   - For high volume scenarios, transcoding can be offloaded to worker nodes
   - Job distribution can be managed via Redis or RabbitMQ
   - Cloud transcoding services can be integrated as fallback options

3. **Optimization Techniques**
   - Parallel processing of different quality versions where resources allow
   - Stream processing to minimize disk usage
   - Hardware acceleration where available (VAAPI, NVENC, etc.)

## System Requirements

For optimal performance of the audio processing pipeline:

1. **Hardware Recommendations**
   - CPU: 4+ cores (8+ cores recommended for high volume)
   - RAM: 8GB minimum (16GB+ recommended)
   - Storage: SSD for temporary processing files
   - Network: High bandwidth connection to R2 storage

2. **Software Dependencies**
   - FFmpeg 4.2+ with AAC, MP3, FLAC, and WAV support
   - Node.js 14+ with worker threads support
   - Redis for job queuing (optional)
   - audiowaveform tool for waveform generation

## Related Documents

- [R2 Storage Design](r2-storage-design.md)
- [Database Schema](database-schema.md)
- [API Endpoints Reference](../api/endpoints-reference.md) 