import { render, fireEvent, screen } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import BulkEditForm from '../../../../src/components/BulkEditForm.svelte';

describe('BulkEditForm', () => {
  const mockTracks = [
    { 
      id: 'track1', 
      title: 'Track 1', 
      writers: ['Writer 1'], 
      writer_percentages: [100],
      publishers: ['Publisher 1'],
      publisher_percentages: [100],
      genre: ['Rock'],
      instruments: ['Guitar', 'Drums'],
      mood: ['Energetic']
    },
    { 
      id: 'track2', 
      title: 'Track 2', 
      writers: ['Writer 2', 'Writer 3'], 
      writer_percentages: [60, 40],
      publishers: ['Publisher 2'],
      publisher_percentages: [100],
      genre: ['Pop'],
      instruments: ['Piano', 'Synth'],
      mood: ['Upbeat']
    }
  ];

  const mockSaveChanges = vi.fn();

  beforeEach(() => {
    mockSaveChanges.mockReset();
  });

  it('should render selected tracks count', () => {
    render(BulkEditForm, { 
      props: { 
        selectedTracks: mockTracks,
        onSave: mockSaveChanges
      } 
    });
    
    expect(screen.getByText('Editing 2 tracks')).toBeInTheDocument();
  });

  it('should display writer fields correctly', () => {
    render(BulkEditForm, { 
      props: { 
        selectedTracks: mockTracks,
        onSave: mockSaveChanges
      } 
    });
    
    // Writers section should be visible
    expect(screen.getByText('Writers')).toBeInTheDocument();
    
    // Add writer button should be visible
    expect(screen.getByText('Add Writer')).toBeInTheDocument();
  });

  it('should validate writer percentages sum to 100', async () => {
    render(BulkEditForm, { 
      props: { 
        selectedTracks: mockTracks,
        onSave: mockSaveChanges
      } 
    });
    
    // Get writer fields
    const writerInput = screen.getByLabelText('Writer Name');
    const percentageInput = screen.getByLabelText('Percentage');
    const addButton = screen.getByText('Add Writer');
    
    // Add a writer with invalid percentage
    await fireEvent.input(writerInput, { target: { value: 'New Writer' } });
    await fireEvent.input(percentageInput, { target: { value: '50' } });
    await fireEvent.click(addButton);
    
    // Try to save changes
    const saveButton = screen.getByText('Save Changes');
    await fireEvent.click(saveButton);
    
    // Should show validation error
    expect(screen.getByText('Writer percentages must sum to 100%')).toBeInTheDocument();
    
    // Save function should not be called due to validation error
    expect(mockSaveChanges).not.toHaveBeenCalled();
  });

  it('should call onSave with updated data when valid', async () => {
    render(BulkEditForm, { 
      props: { 
        selectedTracks: mockTracks,
        onSave: mockSaveChanges
      } 
    });
    
    // Update genre
    const genreInput = screen.getByLabelText('Genre');
    await fireEvent.input(genreInput, { target: { value: 'Jazz' } });
    
    // Add a writer with correct percentage
    const writerInput = screen.getByLabelText('Writer Name');
    const percentageInput = screen.getByLabelText('Percentage');
    await fireEvent.input(writerInput, { target: { value: 'Sole Writer' } });
    await fireEvent.input(percentageInput, { target: { value: '100' } });
    
    // Save changes
    const saveButton = screen.getByText('Save Changes');
    await fireEvent.click(saveButton);
    
    // Save function should be called with correct data
    expect(mockSaveChanges).toHaveBeenCalledTimes(1);
    const saveData = mockSaveChanges.mock.calls[0][0];
    
    expect(saveData.genre).toEqual(['Jazz']);
    expect(saveData.writers).toEqual(['Sole Writer']);
    expect(saveData.writer_percentages).toEqual([100]);
  });
}); 