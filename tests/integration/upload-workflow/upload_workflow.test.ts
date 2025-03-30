import { expect, test, describe, beforeAll, afterAll } from 'vitest';
import { invoke } from '@tauri-apps/api/tauri';
import path from 'path';
import fs from 'fs';

// Mock data for testing
const TEST_CREDENTIALS = {
  r2: {
    bucketName: 'test-bucket',
    accessKey: 'test-access-key',
    secretKey: 'test-secret-key',
    endpoint: 'https://test-endpoint.r2.cloudflarestorage.com'
  },
  mongodb: {
    uri: 'mongodb://localhost:27017/test_music_library'
  }
};

describe('Music Upload Workflow', () => {
  const testTrackPaths = [
    path.join(__dirname, '../../fixtures/track1.mp3'),
    path.join(__dirname, '../../fixtures/track2.mp3')
  ];
  
  // Test tracklisting for verification
  let trackIds: string[] = [];
  let albumId: string;
  
  beforeAll(async () => {
    // Ensure test credentials are set
    await invoke('set_credentials', { credentials: TEST_CREDENTIALS });
    
    // Create test files if they don't exist (to ensure tests can run)
    createTestFilesIfNeeded();
  });
  
  test('should upload multiple tracks and create album', async () => {
    // Upload test tracks
    const result = await invoke('upload_tracks', { 
      filePaths: testTrackPaths 
    });
    
    expect(result).toHaveProperty('success', true);
    expect(result).toHaveProperty('trackIds');
    expect(result).toHaveProperty('albumId');
    
    // Store IDs for later tests
    trackIds = result.trackIds;
    albumId = result.albumId;
    
    // Verify proper length
    expect(trackIds.length).toBe(testTrackPaths.length);
  });
  
  test('should transcode uploaded tracks to multiple qualities', async () => {
    // For each track, check if the different quality versions exist in R2
    for (const trackId of trackIds) {
      const trackFiles = await invoke('get_track_files', { trackId });
      
      expect(trackFiles).toHaveProperty('original');
      expect(trackFiles).toHaveProperty('low');
      expect(trackFiles).toHaveProperty('medium');
      
      // Verify the files exist and are accessible
      const originalExists = await invoke('check_file_exists', { 
        path: trackFiles.original 
      });
      const lowExists = await invoke('check_file_exists', { 
        path: trackFiles.low 
      });
      const mediumExists = await invoke('check_file_exists', { 
        path: trackFiles.medium 
      });
      
      expect(originalExists).toBe(true);
      expect(lowExists).toBe(true);
      expect(mediumExists).toBe(true);
    }
  });
  
  test('should extract metadata and associate tracks with album', async () => {
    // Get album data
    const albumData = await invoke('get_album', { albumId });
    
    // Verify album has track references
    expect(albumData).toHaveProperty('track_ids');
    expect(albumData.track_ids.length).toBe(trackIds.length);
    
    // Verify all uploaded track IDs are associated with the album
    for (const trackId of trackIds) {
      expect(albumData.track_ids).toContain(trackId);
    }
    
    // Check each track has correct metadata
    for (const trackId of trackIds) {
      const trackData = await invoke('get_track', { trackId });
      
      expect(trackData).toHaveProperty('title');
      expect(trackData).toHaveProperty('album_id', albumId);
      expect(trackData).toHaveProperty('duration');
      expect(trackData.duration).toBeGreaterThan(0);
    }
  });
  
  afterAll(async () => {
    // Clean up - delete test tracks and album
    for (const trackId of trackIds) {
      await invoke('delete_track', { trackId });
    }
    
    await invoke('delete_album', { albumId });
  });
  
  // Helper function to create test files if needed
  function createTestFilesIfNeeded() {
    const fixturesDir = path.join(__dirname, '../../fixtures');
    
    // Create fixtures directory if it doesn't exist
    if (!fs.existsSync(fixturesDir)) {
      fs.mkdirSync(fixturesDir, { recursive: true });
    }
    
    // Create test MP3 files if they don't exist
    [
      { path: path.join(fixturesDir, 'track1.mp3'), title: 'Test Track 1' },
      { path: path.join(fixturesDir, 'track2.mp3'), title: 'Test Track 2' }
    ].forEach(file => {
      if (!fs.existsSync(file.path)) {
        // Generate a minimal valid MP3 file for testing
        // This is a simplified example - in reality, you would
        // need a proper MP3 file or a good mock
        const headerBytes = Buffer.from([
          0xFF, 0xFB, 0x90, 0x44, 0x00, 0x00, 0x00, 0x00,
          0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
        ]);
        fs.writeFileSync(file.path, headerBytes);
      }
    });
  }
}); 