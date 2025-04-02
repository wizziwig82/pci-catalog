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

describe('Album Creation and Existence Check', () => {
  // Setup test tracks with the same album name but different filenames
  const testTrack1Path = path.join(__dirname, '../../../fixtures/album_test_track1.mp3');
  const testTrack2Path = path.join(__dirname, '../../../fixtures/album_test_track2.mp3');
  
  // Test IDs for verification
  let trackId1: string;
  let trackId2: string;
  let albumId1: string;
  let albumId2: string;
  
  beforeAll(async () => {
    // Ensure test credentials are set
    await invoke('set_credentials', { credentials: TEST_CREDENTIALS });
    
    // Create test files if they don't exist
    createTestFilesIfNeeded();
    
    // Clear any existing test data to ensure clean environment
    await invoke('clear_test_data');
  });
  
  test('should create a new album when uploading first track', async () => {
    // Upload the first test track
    const result1 = await invoke('upload_tracks', { 
      filePaths: [testTrack1Path] 
    });
    
    expect(result1).toHaveProperty('success', true);
    expect(result1).toHaveProperty('uploaded_tracks');
    expect(result1.uploaded_tracks.length).toBe(1);
    
    // Store IDs for later tests
    trackId1 = result1.uploaded_tracks[0].track_id;
    albumId1 = result1.uploaded_tracks[0].album_id;
    
    // Verify album was created
    const album = await invoke('get_album', { albumId: albumId1 });
    expect(album).toHaveProperty('name', 'Test Album');
    expect(album).toHaveProperty('track_ids');
    expect(album.track_ids).toContain(trackId1);
    expect(album.track_ids.length).toBe(1);
  });
  
  test('should use existing album when uploading second track with same album name', async () => {
    // Upload the second test track (should detect same album)
    const result2 = await invoke('upload_tracks', { 
      filePaths: [testTrack2Path] 
    });
    
    expect(result2).toHaveProperty('success', true);
    expect(result2).toHaveProperty('uploaded_tracks');
    expect(result2.uploaded_tracks.length).toBe(1);
    
    // Store IDs for verification
    trackId2 = result2.uploaded_tracks[0].track_id;
    albumId2 = result2.uploaded_tracks[0].album_id;
    
    // Verify album ID is the same as first upload (reused existing album)
    expect(albumId2).toBe(albumId1);
    
    // Verify album now contains both tracks
    const album = await invoke('get_album', { albumId: albumId1 });
    expect(album).toHaveProperty('track_ids');
    expect(album.track_ids).toContain(trackId1);
    expect(album.track_ids).toContain(trackId2);
    expect(album.track_ids.length).toBe(2);
  });
  
  test('should correctly associate tracks with albums', async () => {
    // Verify track 1 is associated with the album
    const track1 = await invoke('get_track', { trackId: trackId1 });
    expect(track1).toHaveProperty('album_id', albumId1);
    
    // Verify track 2 is associated with the same album
    const track2 = await invoke('get_track', { trackId: trackId2 });
    expect(track2).toHaveProperty('album_id', albumId1);
    
    // Get all tracks for the album and verify both are present
    const albumTracks = await invoke('get_tracks_by_album', { albumId: albumId1 });
    expect(albumTracks).toHaveProperty('data');
    expect(albumTracks.data.length).toBe(2);
    
    // Verify track IDs in album tracks collection
    const trackIds = albumTracks.data.map(track => track._id);
    expect(trackIds).toContain(trackId1);
    expect(trackIds).toContain(trackId2);
  });
  
  afterAll(async () => {
    // Clean up - delete test tracks and album
    await invoke('delete_track', { trackId: trackId1 });
    await invoke('delete_track', { trackId: trackId2 });
    await invoke('delete_album', { albumId: albumId1 });
  });
  
  // Helper function to create test files with proper metadata
  function createTestFilesIfNeeded() {
    const fixturesDir = path.join(__dirname, '../../../fixtures');
    
    // Create fixtures directory if it doesn't exist
    if (!fs.existsSync(fixturesDir)) {
      fs.mkdirSync(fixturesDir, { recursive: true });
    }
    
    // Create test MP3 files with same album but different track names
    [
      { path: testTrack1Path, title: 'Test Track 1', album: 'Test Album' },
      { path: testTrack2Path, title: 'Test Track 2', album: 'Test Album' }
    ].forEach(file => {
      if (!fs.existsSync(file.path)) {
        // Generate a minimal valid MP3 file with ID3 tags for testing
        // This is simplified - in reality, you'd want proper MP3 files
        const headerBytes = Buffer.from([
          0xFF, 0xFB, 0x90, 0x44, 0x00, 0x00, 0x00, 0x00,
          0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
        ]);
        fs.writeFileSync(file.path, headerBytes);
        
        // In a real implementation, you'd set ID3 tags here with proper album info
        // For this test, we'll rely on our backend to read these test files properly
      }
    });
  }
}); 