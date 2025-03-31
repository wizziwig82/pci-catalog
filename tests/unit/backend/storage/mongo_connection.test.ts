import { expect, test, describe, beforeAll, afterAll } from 'vitest';
import { 
  initialize_mongo_client as initializeMongoClient, 
  create_album as createAlbum, 
  get_album as getAlbum, 
  update_album as updateAlbum,
  delete_album as deleteAlbum,
  create_track as createTrack,
  get_track as getTrack,
  update_track as updateTrack,
  delete_track as deleteTrack,
  search_tracks as searchTracks,
  get_tracks_by_album as getTracksByAlbum
} from '../../../../src-tauri/src/storage/mongodb';

// Define types that match the Rust structs for TypeScript tests
type Writer = {
  name: string;
  percentage: number;
};

type Publisher = {
  name: string;
  percentage: number;
};

type TrackPaths = {
  original: string;
  medium: string;
  low: string;
};

type Track = {
  title: string;
  album_id: string;
  filename: string;
  duration: number;
  comments?: string;
  writers: Writer[];
  publishers: Publisher[];
  genre?: string;
  instruments: string[];
  mood?: string;
  path: TrackPaths;
};

type Album = {
  name: string;
  art_path?: string;
  track_ids: string[];
};

// Mock credentials for testing - these should come from test environment variables
const TEST_MONGO_URI = 'mongodb://localhost:27017/test_music_library';

describe('MongoDB Connection', () => {
  let mongoClient: any;
  const testAlbumId = 'test-album-id';
  const testAlbumData: Album = {
    name: 'Test Album',
    art_path: 'albums/test-album-id/art.jpg',
    track_ids: []
  };
  
  const testTrackId = 'test-track-id';
  const testTrackData: Track = {
    title: 'Test Track',
    album_id: testAlbumId,
    filename: 'test-track.mp3',
    duration: 180.5,
    comments: 'Test comments',
    writers: [
      { name: 'Writer 1', percentage: 60 },
      { name: 'Writer 2', percentage: 40 }
    ],
    publishers: [
      { name: 'Publisher 1', percentage: 100 }
    ],
    genre: 'Test Genre',
    instruments: ['guitar', 'piano'],
    mood: 'Upbeat',
    path: {
      original: 'tracks/original/test-track.mp3',
      medium: 'tracks/medium/test-track.mp3',
      low: 'tracks/low/test-track.mp3'
    }
  };

  beforeAll(async () => {
    // Initialize MongoDB client with test credentials
    mongoClient = await initializeMongoClient({
      uri: TEST_MONGO_URI
    });
  });

  test('should connect to MongoDB successfully', async () => {
    const connection = await mongoClient.test_connection();
    expect(connection.success).toBe(true);
  });

  test('should create an album in MongoDB', async () => {
    const result = await createAlbum(mongoClient, testAlbumId, testAlbumData);
    expect(result.success).toBe(true);
    expect(result.id).toBe(testAlbumId);
  });

  test('should retrieve an album from MongoDB', async () => {
    const result = await getAlbum(mongoClient, testAlbumId);
    expect(result.success).toBe(true);
    expect(result.data.name).toBe(testAlbumData.name);
    expect(result.data.art_path).toBe(testAlbumData.art_path);
  });

  test('should update an album in MongoDB', async () => {
    const updatedData = {
      ...testAlbumData,
      name: 'Updated Test Album'
    };
    const result = await updateAlbum(mongoClient, testAlbumId, updatedData);
    expect(result.success).toBe(true);
    
    // Verify update
    const getResult = await getAlbum(mongoClient, testAlbumId);
    expect(getResult.data.name).toBe('Updated Test Album');
  });

  // Track tests
  test('should create a track in MongoDB', async () => {
    const result = await createTrack(mongoClient, testTrackId, testTrackData);
    expect(result.success).toBe(true);
    expect(result.id).toBe(testTrackId);
  });

  test('should retrieve a track from MongoDB', async () => {
    const result = await getTrack(mongoClient, testTrackId);
    expect(result.success).toBe(true);
    expect(result.data.title).toBe(testTrackData.title);
    expect(result.data.album_id).toBe(testTrackData.album_id);
  });

  test('should update a track in MongoDB', async () => {
    const updatedData = {
      ...testTrackData,
      title: 'Updated Test Track',
      genre: 'Updated Genre'
    };
    const result = await updateTrack(mongoClient, testTrackId, updatedData);
    expect(result.success).toBe(true);
    
    // Verify update
    const getResult = await getTrack(mongoClient, testTrackId);
    expect(getResult.data.title).toBe('Updated Test Track');
    expect(getResult.data.genre).toBe('Updated Genre');
  });
  
  test('should retrieve tracks by album from MongoDB', async () => {
    // Update the album to include the track
    const updatedAlbum = {
      ...testAlbumData,
      name: 'Updated Test Album',
      track_ids: [testTrackId]
    };
    await updateAlbum(mongoClient, testAlbumId, updatedAlbum);
    
    // Get tracks by album
    const result = await getTracksByAlbum(mongoClient, testAlbumId);
    expect(result.success).toBe(true);
    expect(result.data.length).toBeGreaterThan(0);
    expect(result.data[0].title).toBe('Updated Test Track');
  });
  
  test('should search tracks in MongoDB', async () => {
    // Search for tracks
    const result = await searchTracks(mongoClient, 'Updated');
    expect(result.success).toBe(true);
    expect(result.data.length).toBeGreaterThan(0);
    
    // Search should find our track
    const trackFound = result.data.some((track: Track) => track.title === 'Updated Test Track');
    expect(trackFound).toBe(true);
  });
  
  test('should delete a track from MongoDB', async () => {
    const result = await deleteTrack(mongoClient, testTrackId);
    expect(result.success).toBe(true);
    
    // Verify deletion
    const getResult = await getTrack(mongoClient, testTrackId);
    expect(getResult.success).toBe(false);
  });

  test('should delete an album from MongoDB', async () => {
    const result = await deleteAlbum(mongoClient, testAlbumId);
    expect(result.success).toBe(true);
    
    // Verify deletion
    const getResult = await getAlbum(mongoClient, testAlbumId);
    expect(getResult.success).toBe(false);
  });

  afterAll(async () => {
    // Clean up any test data that might remain
    try {
      await deleteTrack(mongoClient, testTrackId);
      await deleteAlbum(mongoClient, testAlbumId);
    } catch (error) {
      // Ignore errors during cleanup
    }
    
    // Close MongoDB connection
    await mongoClient.close();
  });
}); 