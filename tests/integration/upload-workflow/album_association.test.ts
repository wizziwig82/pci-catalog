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

describe('Track-to-Album Association in Upload Workflow', () => {
  // Create multiple test track paths from "different" sources but same album
  const testTrackPaths = [
    // First batch of uploads
    path.join(__dirname, '../../fixtures/album_integration_track1.mp3'),
    path.join(__dirname, '../../fixtures/album_integration_track2.mp3'),
    // Second batch (simulating uploads at a different time)
    path.join(__dirname, '../../fixtures/album_integration_track3.mp3'),
  ];
  
  // Store IDs for verification
  let trackIds: string[] = [];
  let albumId: string;
  
  beforeAll(async () => {
    // Ensure test credentials are set
    await invoke('set_credentials', { credentials: TEST_CREDENTIALS });
    
    // Create test files with proper metadata
    createTestFilesIfNeeded();
    
    // Clean any existing test data
    await invoke('clear_test_data').catch(e => console.log('Clear test data failed, may be first run', e));
  });
  
  test('should create album from initial upload batch', async () => {
    // Upload first batch of test tracks (track1 and track2)
    const result = await invoke('upload_tracks', { 
      filePaths: testTrackPaths.slice(0, 2) 
    });
    
    // Type assertion to handle the result
    const uploadResult = result as {
      success: boolean;
      uploaded_tracks: Array<{
        track_id: string;
        album_id: string;
      }>;
    };
    
    expect(uploadResult.success).toBe(true);
    expect(uploadResult.uploaded_tracks.length).toBe(2);
    
    // Store IDs for verification
    trackIds.push(...uploadResult.uploaded_tracks.map(t => t.track_id));
    albumId = uploadResult.uploaded_tracks[0].album_id;
    
    // All tracks should reference the same album
    const uniqueAlbumIds = new Set(uploadResult.uploaded_tracks.map(t => t.album_id));
    expect(uniqueAlbumIds.size).toBe(1);
    
    // Verify album contains both tracks
    const albumData = await invoke('get_album', { albumId });
    const typedAlbumData = albumData as { track_ids: string[] };
    expect(typedAlbumData.track_ids.length).toBe(2);
    
    // All track IDs should be in the album.track_ids array
    trackIds.forEach(trackId => {
      expect(typedAlbumData.track_ids).toContain(trackId);
    });
  });
  
  test('should associate subsequent uploads with existing album', async () => {
    // Upload second batch (track3) - should be associated with the same album
    const result = await invoke('upload_tracks', { 
      filePaths: [testTrackPaths[2]] 
    });
    
    // Type assertion to handle the result
    const uploadResult = result as {
      success: boolean;
      uploaded_tracks: Array<{
        track_id: string;
        album_id: string;
      }>;
    };
    
    expect(uploadResult.success).toBe(true);
    expect(uploadResult.uploaded_tracks.length).toBe(1);
    
    const newTrackId = uploadResult.uploaded_tracks[0].track_id;
    const newAlbumId = uploadResult.uploaded_tracks[0].album_id;
    
    // Add the new track ID to our list
    trackIds.push(newTrackId);
    
    // The album ID should match the existing album
    expect(newAlbumId).toBe(albumId);
    
    // Get album data to verify all tracks are now associated
    const albumData = await invoke('get_album', { albumId });
    const typedAlbumData = albumData as { track_ids: string[], name: string };
    
    // Album should now have 3 tracks
    expect(typedAlbumData.track_ids.length).toBe(3);
    
    // All track IDs should be in the album.track_ids array
    trackIds.forEach(trackId => {
      expect(typedAlbumData.track_ids).toContain(trackId);
    });
    
    // Verify album name is still consistent
    expect(typedAlbumData.name).toBe('Integration Test Album');
  });
  
  test('should correctly associate all tracks with the album in the database', async () => {
    // Retrieve all tracks from the album
    const tracksResult = await invoke('get_tracks_by_album', { albumId });
    
    // Type assertion
    const typedTracksResult = tracksResult as {
      success: boolean;
      data: Array<{
        _id: string;
        album_id: string;
        title: string;
      }>;
    };
    
    expect(typedTracksResult.success).toBe(true);
    
    // Should have 3 tracks in total
    expect(typedTracksResult.data.length).toBe(3);
    
    // Each track should reference the same album ID
    typedTracksResult.data.forEach(track => {
      expect(track.album_id).toBe(albumId);
    });
    
    // Track IDs should match what we stored
    const returnedTrackIds = typedTracksResult.data.map(track => track._id);
    expect(returnedTrackIds.sort()).toEqual(trackIds.sort());
  });
  
  afterAll(async () => {
    // Clean up - delete test tracks and album
    for (const trackId of trackIds) {
      await invoke('delete_track', { trackId }).catch(e => 
        console.log(`Failed to delete track ${trackId}:`, e)
      );
    }
    
    await invoke('delete_album', { albumId }).catch(e => 
      console.log(`Failed to delete album ${albumId}:`, e)
    );
  });
  
  // Helper function to create test files with the same album metadata
  function createTestFilesIfNeeded() {
    const fixturesDir = path.join(__dirname, '../../fixtures');
    
    // Create fixtures directory if it doesn't exist
    if (!fs.existsSync(fixturesDir)) {
      fs.mkdirSync(fixturesDir, { recursive: true });
    }
    
    // Create test MP3 files with the same album but different track names
    [
      { path: testTrackPaths[0], title: 'Integration Test Track 1' },
      { path: testTrackPaths[1], title: 'Integration Test Track 2' },
      { path: testTrackPaths[2], title: 'Integration Test Track 3' },
    ].forEach(file => {
      if (!fs.existsSync(file.path)) {
        // Generate a minimal valid MP3 file for testing
        const headerBytes = Buffer.from([
          0xFF, 0xFB, 0x90, 0x44, 0x00, 0x00, 0x00, 0x00,
          0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
        ]);
        fs.writeFileSync(file.path, headerBytes);
        
        // In a real implementation, you'd set ID3 tags here with proper album info
        // For this test, we'll rely on our backend to read and properly assign 
        // the album name "Integration Test Album" for these files
      }
    });
  }
}); 