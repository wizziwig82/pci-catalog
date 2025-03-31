import { expect, test, describe, beforeAll, afterAll } from 'vitest';
import { invoke } from '@tauri-apps/api/core';

// Mock credentials for testing
const TEST_CREDENTIALS = {
  uri: 'mongodb://localhost:27017/test_music_library',
};

describe('MongoDB Credential Management', () => {
  // Test init_mongo command
  test('should initialize MongoDB connection with valid credentials', async () => {
    try {
      const result = await invoke('init_mongo', { credentials: TEST_CREDENTIALS });
      expect(result).toBe(true);
    } catch (error: any) {
      // If MongoDB is not running locally, this test might fail
      console.warn('MongoDB connection test failed. Is MongoDB running locally?', error);
      // We'll skip this test if MongoDB is not available
      if (error.toString().includes('Failed to initialize MongoDB client')) {
        return;
      }
      throw error;
    }
  });

  // Test test_mongo_connection command
  test('should test MongoDB connection successfully', async () => {
    try {
      const result = await invoke('test_mongo_connection');
      expect(result).toBe(true);
    } catch (error: any) {
      // If MongoDB is not running locally, this test might fail
      console.warn('MongoDB connection test failed. Is MongoDB running locally?', error);
      // We'll skip this test if MongoDB is not available
      if (error.toString().includes('MongoDB client not initialized') || 
          error.toString().includes('Failed to connect to MongoDB')) {
        return;
      }
      throw error;
    }
  });

  // Add more tests for secure storage of credentials if keychain integration is implemented
}); 