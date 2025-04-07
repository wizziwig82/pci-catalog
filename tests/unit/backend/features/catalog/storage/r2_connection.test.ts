import { expect, test, describe, beforeAll, afterAll } from 'vitest';
import { initializeR2Client, uploadFile, downloadFile, deleteFile } from '../../../../src-tauri/src/storage/r2';

// Mock credentials for testing - these should come from test environment variables
const TEST_BUCKET = 'test-bucket';
const TEST_ACCESS_KEY = 'test-access-key';
const TEST_SECRET_KEY = 'test-secret-key';
const TEST_ENDPOINT = 'https://test-endpoint.r2.cloudflarestorage.com';

describe('R2 Storage Connection', () => {
  let r2Client: any;
  const testFileName = 'test-file.txt';
  const testFileContent = 'This is test content for R2 storage testing';

  beforeAll(async () => {
    // Initialize R2 client with test credentials
    r2Client = await initializeR2Client({
      bucketName: TEST_BUCKET,
      accessKey: TEST_ACCESS_KEY,
      secretKey: TEST_SECRET_KEY,
      endpoint: TEST_ENDPOINT
    });
  });

  test('should connect to R2 bucket successfully', async () => {
    const connection = await r2Client.testConnection();
    expect(connection.success).toBe(true);
  });

  test('should upload file to R2 bucket', async () => {
    const result = await uploadFile(
      r2Client,
      testFileName,
      Buffer.from(testFileContent),
      'text/plain'
    );
    expect(result.success).toBe(true);
    expect(result.path).toContain(testFileName);
  });

  test('should download file from R2 bucket', async () => {
    const result = await downloadFile(r2Client, testFileName);
    expect(result.success).toBe(true);
    expect(result.data.toString()).toBe(testFileContent);
  });

  test('should delete file from R2 bucket', async () => {
    const result = await deleteFile(r2Client, testFileName);
    expect(result.success).toBe(true);
  });

  afterAll(async () => {
    // Clean up any test files that might remain
    try {
      await deleteFile(r2Client, testFileName);
    } catch (error) {
      // Ignore errors during cleanup
    }
  });
}); 