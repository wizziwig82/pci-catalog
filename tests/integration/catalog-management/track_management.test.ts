import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, fireEvent, cleanup } from '@testing-library/svelte';
import CatalogPage from '../../../src/routes/catalog/+page.svelte'; // Adjust path as needed
import { safeInvoke } from '$lib/utils/invokeWrapper';

// --- Mocks ---
// Mock Tauri APIs (similar to unit test, but might need more specific command mocks)
vi.mock('$lib/utils/invokeWrapper', () => ({
  safeInvoke: vi.fn(),
}));
vi.mock('$app/environment', () => ({ // Mock environment for onMount
  browser: true,
  dev: false,
  building: false,
  version: 'test',
}));
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));
vi.mock('$app/navigation', () => ({
  goto: vi.fn(),
}));
vi.mock('$app/stores', () => ({
  page: { subscribe: vi.fn() },
}));
vi.mock('$lib/stores/notifications', () => ({
  showSuccessToast: vi.fn(),
  showErrorToast: vi.fn(),
}));

// Mock Data (reuse or adapt from unit test)
const mockTracks = [
  {
    id: 'track-integ-1',
    title: 'Integ Test Track 1',
    album_id: 'album-integ-1',
    album_name: 'Integ Test Album',
    duration: 150,
    genre: ['Integration'],
    filename: 'integ1.mp3',
    writers: ['Integ Writer A'], writer_percentages: [100],
    publishers: ['Integ Pub X'], publisher_percentages: [100],
    instruments: ['Test'], mood: ['Testing'], comments: 'Integ track 1',
    path: { original: 'path/orig1.mp3', medium: 'path/med1.mp3' },
  },
   {
    id: 'track-integ-2',
    title: 'Integ Test Track 2',
    album_id: 'album-integ-1',
    album_name: 'Integ Test Album',
    duration: 190,
    genre: ['Test'],
    filename: 'integ2.mp3',
     writers: ['Integ Writer B'], writer_percentages: [100],
     publishers: ['Integ Pub Y'], publisher_percentages: [100],
     instruments: ['Mock'], mood: ['Mocking'], comments: 'Integ track 2',
    path: { original: 'path/orig2.mp3', medium: 'path/med2.mp3' },
  },
];

const mockTrackListResponse = {
  success: true,
  tracks: mockTracks,
  total_count: mockTracks.length,
};

// --- Test Suite ---
describe('Integration Tests: Track Management in Catalog', () => {

  beforeEach(() => {
    vi.clearAllMocks();
    // Setup default mocks for invoke calls
    (safeInvoke as vi.Mock).mockImplementation(async (command: string, args: any) => {
      console.log(`Mock safeInvoke called: ${command}`, args);
      if (command === 'init_mongo_client') return true;
      if (command === 'fetch_all_tracks') return structuredClone(mockTrackListResponse);
      if (command === 'update_track_metadata') {
        // Simulate successful update, maybe log payload
        console.log('Mock update_track_metadata payload:', args.payload);
        return true;
      }
      if (command === 'delete_tracks') {
         console.log('Mock delete_tracks IDs:', args.trackIds);
         // Simulate deletion by returning fewer tracks on next fetch
         mockTrackListResponse.tracks = mockTrackListResponse.tracks.filter(t => !args.trackIds.includes(t.id));
         mockTrackListResponse.total_count = mockTrackListResponse.tracks.length;
         return true;
      }
       if (command === 'select_audio_files') {
         return ['/fake/path/replacement-integ.wav'];
       }
       if (command === 'transcode_audio_batch') {
         return [{ success: true, input_path: args.filePaths[0], medium_quality_path: '/fake/path/transcoded_replacement-integ.mp3' }];
       }
       if (command === 'replace_track_audio') {
         console.log('Mock replace_track_audio:', args);
         // Simulate path update for next fetch? Difficult without real backend state.
         // For now, just return success.
         return true;
       }
      console.warn(`Mock safeInvoke received unhandled command: ${command}`);
      return null;
    });
  });

  afterEach(() => {
    cleanup();
    // Reset mock data if modified by tests (like delete)
     mockTrackListResponse.tracks = structuredClone(mockTracks); // Deep clone original data
     mockTrackListResponse.total_count = mockTracks.length;
  });

  it('should load and display tracks', async () => {
    render(CatalogPage);
    expect(await screen.findByText('Integ Test Track 1')).toBeInTheDocument();
    expect(screen.getByText('Integ Test Track 2')).toBeInTheDocument();
    expect(safeInvoke).toHaveBeenCalledWith('fetch_all_tracks', expect.anything());
  });

  it('should allow editing a single track title', async () => {
    render(CatalogPage);
    const track1Title = 'Integ Test Track 1';
    const updatedTitle = 'Integ Test Track 1 - Updated';
    await screen.findByText(track1Title); // Wait for load

    // Select the first track row
    await fireEvent.click(screen.getByText(track1Title).closest('tr')!);

    // Click Edit
    await fireEvent.click(screen.getByText('Edit Selected'));

    // Wait for editor and find the title input (assuming label association)
    const titleInput = await screen.findByLabelText('Title') as HTMLInputElement;
    expect(titleInput).toBeInTheDocument();
    expect(titleInput.value).toBe(track1Title);

    // Change title
    await fireEvent.input(titleInput, { target: { value: updatedTitle } });
    expect(titleInput.value).toBe(updatedTitle);

    // Click Save Changes (within the editor component)
    // Need to find the save button within the editor's context
    const editorElement = screen.getByText(`Edit Track: ${track1Title}`).closest('.metadata-editor');
    expect(editorElement).toBeInTheDocument();
    const saveButton = editorElement!.querySelector('button.save-button'); // Find save button within editor
    expect(saveButton).toBeInTheDocument();
    await fireEvent.click(saveButton!);

    // Verify safeInvoke call for update
    expect(safeInvoke).toHaveBeenCalledWith('update_track_metadata', {
      trackId: mockTracks[0].id,
      payload: expect.objectContaining({ title: updatedTitle })
    });

    // Optional: Verify UI updates if mock simulates data refresh
    // expect(await screen.findByText(updatedTitle)).toBeInTheDocument(); // Depends on mock refreshing data
  });

  it('should allow deleting a selected track', async () => {
    // Mock window.confirm
    window.confirm = vi.fn(() => true); // Auto-confirm deletion

    render(CatalogPage);
    const track2Title = 'Integ Test Track 2';
    await screen.findByText(track2Title); // Wait for load

    // Select the second track row
    await fireEvent.click(screen.getByText(track2Title).closest('tr')!);

    // Click Delete Selected button
    const deleteButton = screen.getByText(/Delete Selected \(1\)/);
    expect(deleteButton).not.toBeDisabled();
    await fireEvent.click(deleteButton);

    // Verify confirmation was called
    expect(window.confirm).toHaveBeenCalled();

    // Verify safeInvoke call for delete
    expect(safeInvoke).toHaveBeenCalledWith('delete_tracks', {
      trackIds: [mockTracks[1].id] // Expecting ID of the second track
    });

    // Verify the track is removed from the UI (mock simulates this)
    // Need to wait for the component to potentially re-render after mock update
    await vi.waitFor(() => {
        expect(screen.queryByText(track2Title)).not.toBeInTheDocument();
    });
    expect(screen.getByText('Integ Test Track 1')).toBeInTheDocument(); // Ensure first track remains
  });

  it('should handle the replace audio flow', async () => {
    render(CatalogPage);
    const track1Title = 'Integ Test Track 1';
    const track1Id = mockTracks[0].id;
    const fakeSelectedFile = '/fake/path/replacement-integ.wav';
    const fakeTranscodedFile = '/fake/path/transcoded_replacement-integ.mp3';

    await screen.findByText(track1Title); // Wait for load

    // Select the first track row
    await fireEvent.click(screen.getByText(track1Title).closest('tr')!);

    // Click Replace Audio button
    const replaceButton = screen.getByText('Replace Audio');
    expect(replaceButton).not.toBeDisabled();
    await fireEvent.click(replaceButton);

    // Verify select_audio_files was called
    expect(safeInvoke).toHaveBeenCalledWith('select_audio_files');

    // Verify transcode_audio_batch was called with the selected file
    // Need to wait for the async flow after file selection
    await vi.waitFor(() => {
      expect(safeInvoke).toHaveBeenCalledWith('transcode_audio_batch', expect.objectContaining({
        filePaths: [fakeSelectedFile]
      }));
    });

    // Verify replace_track_audio was called with the transcoded file path
    await vi.waitFor(() => {
      expect(safeInvoke).toHaveBeenCalledWith('replace_track_audio', {
        trackId: track1Id,
        newMediumQualityPath: fakeTranscodedFile
      });
    });
  });

  // TODO: Add integration test for editing a track
  // TODO: Add integration test for deleting a track
  // TODO: Add integration test for replacing audio

});