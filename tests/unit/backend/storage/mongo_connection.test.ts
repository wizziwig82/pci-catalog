import { expect, test, describe, beforeAll, afterAll } from 'vitest';
import { 
  initializeMongoClient, 
  createAlbum, 
  getAlbum, 
  updateAlbum,
  deleteAlbum
} from '../../../../src-tauri/src/storage/mongodb';

// Mock credentials for testing - these should come from test environment variables
const TEST_MONGO_URI = 'mongodb://localhost:27017/test_music_library';

describe('MongoDB Connection', () => {
  let mongoClient: any;
  const testAlbumId = 'test-album-id';
  const testAlbumData = {
    name: 'Test Album',
    art_path: 'albums/test-album-id/art.jpg',
    track_ids: []
  };

  beforeAll(async () => {
    // Initialize MongoDB client with test credentials
    mongoClient = await initializeMongoClient({
      uri: TEST_MONGO_URI
    });
  });

  test('should connect to MongoDB successfully', async () => {
    const connection = await mongoClient.testConnection();
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
      await deleteAlbum(mongoClient, testAlbumId);
    } catch (error) {
      // Ignore errors during cleanup
    }
    
    // Close MongoDB connection
    await mongoClient.close();
  });
}); 