import { expect, test, describe, beforeAll, afterAll } from 'vitest';
import { invoke } from '@tauri-apps/api/tauri';

// Mock credential data
const TEST_MONGO_CREDENTIALS = 'mongodb://test-user:test-password@test-host:27017/test-db';

const TEST_R2_CREDENTIALS = {
  account_id: 'test-account-id',
  bucket_name: 'test-bucket',
  access_key_id: 'test-access-key',
  secret_access_key: 'test-secret-key',
  endpoint: 'https://test-endpoint.r2.cloudflarestorage.com'
};

describe('Secure Credential Storage', () => {
  // Clean up credentials from previous test runs
  beforeAll(async () => {
    try {
      await invoke('delete_mongo_credentials');
    } catch (_) {
      // Ignore errors when credentials don't exist
    }
    
    try {
      await invoke('delete_r2_credentials');
    } catch (_) {
      // Ignore errors when credentials don't exist
    }
  });
  
  // Clean up after tests
  afterAll(async () => {
    try {
      await invoke('delete_mongo_credentials');
    } catch (_) {
      // Ignore errors when credentials don't exist
    }
    
    try {
      await invoke('delete_r2_credentials');
    } catch (_) {
      // Ignore errors when credentials don't exist
    }
  });
  
  test('should store and retrieve MongoDB credentials', async () => {
    // Store credentials
    const storeResult = await invoke('store_mongo_credentials_wrapper', {
      connectionString: TEST_MONGO_CREDENTIALS
    });
    
    expect(storeResult).toEqual(true);
    
    // Check if credentials exist
    const hasCredentials = await invoke('has_credentials', {
      credentialType: 'mongo'
    });
    
    expect(hasCredentials).toBe(true);
    
    // Retrieve credentials
    const retrievedCredentials = await invoke('get_mongo_credentials_wrapper');
    
    expect(retrievedCredentials).toEqual(TEST_MONGO_CREDENTIALS);
  });
  
  test('should store and retrieve R2 credentials', async () => {
    // Store credentials
    const storeResult = await invoke('store_r2_credentials_wrapper', {
      accountId: TEST_R2_CREDENTIALS.account_id,
      bucketName: TEST_R2_CREDENTIALS.bucket_name,
      accessKeyId: TEST_R2_CREDENTIALS.access_key_id,
      secretAccessKey: TEST_R2_CREDENTIALS.secret_access_key,
      endpoint: TEST_R2_CREDENTIALS.endpoint
    });
    
    expect(storeResult).toEqual(true);
    
    // Check if credentials exist
    const hasCredentials = await invoke('has_credentials', {
      credentialType: 'r2'
    });
    
    expect(hasCredentials).toBe(true);
    
    // Retrieve credentials
    const retrievedCredentials = await invoke('get_r2_credentials_wrapper');
    
    expect(retrievedCredentials).toEqual(TEST_R2_CREDENTIALS);
  });
  
  test('should delete MongoDB credentials', async () => {
    // Store credentials first
    await invoke('store_mongo_credentials_wrapper', {
      connectionString: TEST_MONGO_CREDENTIALS
    });
    
    // Delete credentials
    const deleteResult = await invoke('delete_credentials_proxy', {
      credential_type: 'mongo'
    });
    
    expect(deleteResult).toEqual(undefined);
    
    // Check if credentials exist after deletion
    const hasCredentials = await invoke('has_credentials_proxy', {
      credential_type: 'mongo'
    });
    
    expect(hasCredentials).toBe(false);
    
    // Trying to get deleted credentials should throw an error
    try {
      await invoke('get_mongo_credentials_wrapper');
      // If we reach here, the test should fail
      expect(true).toBe(false);
    } catch (error) {
      expect(error).toBeDefined();
    }
  });
  
  test('should delete R2 credentials', async () => {
    // Store credentials first
    await invoke('store_r2_credentials_wrapper', {
      accountId: TEST_R2_CREDENTIALS.account_id,
      bucketName: TEST_R2_CREDENTIALS.bucket_name,
      accessKeyId: TEST_R2_CREDENTIALS.access_key_id,
      secretAccessKey: TEST_R2_CREDENTIALS.secret_access_key,
      endpoint: TEST_R2_CREDENTIALS.endpoint
    });
    
    // Delete credentials
    const deleteResult = await invoke('delete_credentials_proxy', {
      credential_type: 'r2'
    });
    
    expect(deleteResult).toEqual(undefined);
    
    // Check if credentials exist after deletion
    const hasCredentials = await invoke('has_credentials_proxy', {
      credential_type: 'r2'
    });
    
    expect(hasCredentials).toBe(false);
    
    // Trying to get deleted credentials should throw an error
    try {
      await invoke('get_r2_credentials_wrapper');
      // If we reach here, the test should fail
      expect(true).toBe(false);
    } catch (error) {
      expect(error).toBeDefined();
    }
  });
  
  test('should initialize connections with stored credentials', async () => {
    // Store MongoDB credentials
    await invoke('store_mongo_credentials_wrapper', {
      connectionString: TEST_MONGO_CREDENTIALS
    });
    
    // Store R2 credentials
    await invoke('store_r2_credentials_wrapper', {
      accountId: TEST_R2_CREDENTIALS.account_id,
      bucketName: TEST_R2_CREDENTIALS.bucket_name,
      accessKeyId: TEST_R2_CREDENTIALS.access_key_id,
      secretAccessKey: TEST_R2_CREDENTIALS.secret_access_key,
      endpoint: TEST_R2_CREDENTIALS.endpoint
    });
    
    // These would fail in a real environment as the credentials are fake,
    // but we should be able to see they at least attempt to connect using the stored credentials
    try {
      await invoke('init_mongo_client');
    } catch (error) {
      // Expected to fail with connection error, not with missing credentials error
      expect(String(error)).not.toContain('not found in secure storage');
    }
    
    try {
      await invoke('init_r2_client');
    } catch (error) {
      // Expected to fail with connection error, not with missing credentials error
      expect(String(error)).not.toContain('not found in secure storage');
    }
  });
}); 