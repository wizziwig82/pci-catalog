import { expect, test, describe } from 'vitest';
import { extractMetadata } from '../../../../src-tauri/src/metadata/extractor';
import path from 'path';

describe('Metadata Extraction', () => {
  const testMP3Path = path.join(__dirname, '../../../fixtures/test-track.mp3');
  const testWAVPath = path.join(__dirname, '../../../fixtures/test-track.wav');
  
  test('should extract basic metadata from MP3 file', async () => {
    const metadata = await extractMetadata(testMP3Path);
    
    expect(metadata).toBeDefined();
    expect(metadata.title).toBeDefined();
    expect(metadata.duration).toBeGreaterThan(0);
    expect(metadata.format).toBe('mp3');
  });
  
  test('should extract artist and album metadata from MP3 file', async () => {
    const metadata = await extractMetadata(testMP3Path);
    
    expect(metadata.artist).toBeDefined();
    expect(metadata.album).toBeDefined();
  });
  
  test('should extract metadata from WAV file', async () => {
    const metadata = await extractMetadata(testWAVPath);
    
    expect(metadata).toBeDefined();
    expect(metadata.duration).toBeGreaterThan(0);
    expect(metadata.format).toBe('wav');
  });
  
  test('should handle files with missing metadata', async () => {
    // Mock a file with minimal metadata
    const minimalMetadataFile = path.join(__dirname, '../../../fixtures/minimal-metadata.mp3');
    
    const metadata = await extractMetadata(minimalMetadataFile);
    
    expect(metadata).toBeDefined();
    expect(metadata.title).toBe('Unknown Title');
    expect(metadata.artist).toBe('Unknown Artist');
    expect(metadata.album).toBe('Unknown Album');
    expect(metadata.duration).toBeGreaterThan(0);
  });
  
  test('should throw error for unsupported file format', async () => {
    const textFilePath = path.join(__dirname, '../../../fixtures/not-audio.txt');
    
    await expect(extractMetadata(textFilePath)).rejects.toThrow();
  });
}); 