import '@testing-library/jest-dom';
import { vi } from 'vitest';
import { cleanup } from '@testing-library/svelte';

// Mock Tauri API functions
vi.mock('@tauri-apps/api/tauri', () => {
  return {
    invoke: vi.fn().mockImplementation((command, args) => {
      // Mock implementation for various commands
      switch (command) {
        case 'set_credentials':
          return Promise.resolve({ success: true });
        
        case 'upload_tracks':
          return Promise.resolve({
            success: true,
            trackIds: ['track-id-1', 'track-id-2'],
            albumId: 'album-id-1'
          });
          
        case 'get_track_files':
          return Promise.resolve({
            original: `tracks/${args.trackId}/original.mp3`,
            low: `tracks/${args.trackId}/low.mp3`,
            medium: `tracks/${args.trackId}/medium.mp3`
          });
          
        case 'check_file_exists':
          return Promise.resolve(true);
          
        case 'get_album':
          return Promise.resolve({
            name: 'Test Album',
            art_path: 'albums/album-id-1/art.jpg',
            track_ids: ['track-id-1', 'track-id-2']
          });
          
        case 'get_track':
          return Promise.resolve({
            title: 'Test Track',
            album_id: 'album-id-1',
            duration: 180,
            writers: ['Writer 1'],
            writer_percentages: [100],
            publishers: ['Publisher 1'],
            publisher_percentages: [100],
            genre: ['Rock'],
            instruments: ['Guitar'],
            mood: ['Energetic']
          });
          
        case 'delete_track':
        case 'delete_album':
          return Promise.resolve({ success: true });
          
        default:
          return Promise.reject(new Error(`Unknown command: ${command}`));
      }
    })
  };
});

// Clean up after each test
afterEach(() => {
  cleanup();
});

// Setup global variables that might be used in tests
global.ResizeObserver = vi.fn().mockImplementation(() => ({
  observe: vi.fn(),
  unobserve: vi.fn(),
  disconnect: vi.fn(),
}));

// Set up any other global mocks or configurations
// ... 