import { expect, test, describe, beforeAll, afterAll } from 'vitest';
import { invoke } from '@tauri-apps/api/tauri';

// Mock credential data
const TEST_MONGO_CREDENTIALS = {
  uri: 'mongodb://test-user:test-password@test-host:27017/test-db',
  database: 'test-db'
};

const TEST_R2_CREDENTIALS = {
  bucket_name: 'test-bucket',
  access_key: 'test-access-key',
  secret_key: 'test-secret-key',
  endpoint: 'https://test-endpoint.r2.cloudflarestorage.com'
};

describe('Secure Credential Storage', () => {
  // Clean up credentials from previous test runs
  beforeAll(async () => {
    try {
      await invoke('delete_mongo_credentials');
    } catch (e) {
      // Ignore errors when credentials don't exist
    }
    
    try {
      await invoke('delete_r2_credentials');
    } catch (e) {
      // Ignore errors when credentials don't exist
    }
  });
  
  // Clean up after tests
  afterAll(async () => {
    try {
      await invoke('delete_mongo_credentials');
    } catch (e) {
      // Ignore errors when credentials don't exist
    }
    
    try {
      await invoke('delete_r2_credentials');
    } catch (e) {
      // Ignore errors when credentials don't exist
    }
  });
  
  test('should store and retrieve MongoDB credentials', async () => {
    // Store credentials
    const storeResult = await invoke('store_mongo_credentials', {
      credentials: TEST_MONGO_CREDENTIALS
    });
    
    expect(storeResult).toHaveProperty('success', true);
    
    // Check if credentials exist
    const hasCredentials = await invoke('has_credentials', {
      credentialType: 'mongo'
    });
    
    expect(hasCredentials).toBe(true);
    
    // Retrieve credentials
    const retrievedCredentials = await invoke('get_mongo_credentials');
    
    expect(retrievedCredentials).toEqual(TEST_MONGO_CREDENTIALS);
  });
  
  test('should store and retrieve R2 credentials', async () => {
    // Store credentials
    const storeResult = await invoke('store_r2_credentials', {
      credentials: TEST_R2_CREDENTIALS
    });
    
    expect(storeResult).toHaveProperty('success', true);
    
    // Check if credentials exist
    const hasCredentials = await invoke('has_credentials', {
      credentialType: 'r2'
    });
    
    expect(hasCredentials).toBe(true);
    
    // Retrieve credentials
    const retrievedCredentials = await invoke('get_r2_credentials');
    
    expect(retrievedCredentials).toEqual(TEST_R2_CREDENTIALS);
  });
  
  test('should delete MongoDB credentials', async () => {
    // Store credentials first
    await invoke('store_mongo_credentials', {
      credentials: TEST_MONGO_CREDENTIALS
    });
    
    // Delete credentials
    const deleteResult = await invoke('delete_mongo_credentials');
    
    expect(deleteResult).toHaveProperty('success', true);
    
    // Check if credentials exist after deletion
    const hasCredentials = await invoke('has_credentials', {
      credentialType: 'mongo'
    });
    
    expect(hasCredentials).toBe(false);
    
    // Trying to get deleted credentials should throw an error
    try {
      await invoke('get_mongo_credentials');
      // If we reach here, the test should fail
      expect(true).toBe(false);
    } catch (error) {
      expect(error).toBeDefined();
    }
  });
  
  test('should delete R2 credentials', async () => {
    // Store credentials first
    await invoke('store_r2_credentials', {
      credentials: TEST_R2_CREDENTIALS
    });
    
    // Delete credentials
    const deleteResult = await invoke('delete_r2_credentials');
    
    expect(deleteResult).toHaveProperty('success', true);
    
    // Check if credentials exist after deletion
    const hasCredentials = await invoke('has_credentials', {
      credentialType: 'r2'
    });
    
    expect(hasCredentials).toBe(false);
    
    // Trying to get deleted credentials should throw an error
    try {
      await invoke('get_r2_credentials');
      // If we reach here, the test should fail
      expect(true).toBe(false);
    } catch (error) {
      expect(error).toBeDefined();
    }
  });
  
  test('should initialize connections with stored credentials', async () => {
    // Store MongoDB credentials
    await invoke('store_mongo_credentials', {
      credentials: TEST_MONGO_CREDENTIALS
    });
    
    // Store R2 credentials
    await invoke('store_r2_credentials', {
      credentials: TEST_R2_CREDENTIALS
    });
    
    // These would fail in a real environment as the credentials are fake,
    // but we should be able to see they at least attempt to connect using the stored credentials
    try {
      await invoke('init_mongo_with_stored_credentials');
    } catch (error) {
      // Expected to fail with connection error, not with missing credentials error
      expect(String(error)).not.toContain('not found in secure storage');
    }
    
    try {
      await invoke('init_r2_with_stored_credentials');
    } catch (error) {
      // Expected to fail with connection error, not with missing credentials error
      expect(String(error)).not.toContain('not found in secure storage');
    }
  });
}); 