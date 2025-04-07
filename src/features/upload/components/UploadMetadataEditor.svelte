<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { UploadItemMetadata } from '$lib/types/catalog'; // Keep this type
  // Removed TagSelector and related imports - will be handled in child components if needed
  // import { instrumentTags, moodTags, tagsToString } from '$lib/stores/tagData';
  // import { tick } from 'svelte'; // Removed tick, focus management might move to children

  // Import the new child components
  import TrackList from './TrackList.svelte';
  import IndividualMetadataForm from './IndividualMetadataForm.svelte';
  import BulkMetadataForm from './BulkMetadataForm.svelte';
  // Import type definition from BulkMetadataForm if needed for event handling
  // We need to define or export BulkEditFields in BulkMetadataForm.svelte first.
  // For now, let's use a local definition or 'any' temporarily.
  interface BulkEditFieldsPlaceholder {
      album?: string | null;
      artist?: string | null;
      genre?: string | null;
      writers?: { name: string; percentage: number }[] | null;
      // Add other fields as needed
  }


  // Props
  export let uploadItemsMetadata: UploadItemMetadata[] = [];

  // Dispatcher for events
  const dispatch = createEventDispatcher<{
    cancel: void;
    finalize: void;
    metadataUpdated: { updatedMetadata: UploadItemMetadata[] };
  }>();

  // --- State Migrated from +page.svelte ---
  let selectedTrackIndex = -1;
  let bulkEditMode = false;
  let selectedTrackIndices: number[] = [];

  // Removed state related to bulk edit fields, individual edit fields, intermediate strings, etc.
  // This state will be managed within the child components (IndividualMetadataForm, BulkMetadataForm).
  // Container only needs: uploadItemsMetadata, selectedTrackIndex, bulkEditMode, selectedTrackIndices.
// --- Functions Migrated from +page.svelte ---
  // --- Functions Migrated from +page.svelte ---

  // Function to edit a specific track
  function editTrack(index: number) {
    selectedTrackIndex = index;
    bulkEditMode = false;
    // Initialize local state for individual editor
    // No need to initialize form state here anymore, just set the index/mode.
    // The IndividualMetadataForm will handle its own initialization based on the passed trackData prop.
  }

  // Function to toggle track selection for bulk editing
  function toggleTrackSelection(index: number) {
    const idx = selectedTrackIndices.indexOf(index);
    if (idx === -1) {
      selectedTrackIndices = [...selectedTrackIndices, index];
    } else {
      selectedTrackIndices = selectedTrackIndices.filter(i => i !== index);
    }
  }

  // Function to select/deselect all tracks
  function selectAllTracks(select: boolean) {
    if (select) {
      selectedTrackIndices = [...Array(uploadItemsMetadata.length).keys()];
    } else {
      selectedTrackIndices = [];
    }
  }

  // Function to enable bulk edit mode
  function startBulkEdit() {
    bulkEditMode = true;
    selectedTrackIndex = -1;

    // No need to initialize bulk fields here.
    // The BulkMetadataForm component will handle its own initialization.
  }

  // Event handler for when BulkMetadataForm emits 'apply'
  function handleApplyBulkEdits(event: CustomEvent<{ bulkData: BulkEditFieldsPlaceholder; modifiedFields: Set<keyof BulkEditFieldsPlaceholder> }>) {
      const { bulkData, modifiedFields } = event.detail;

      if (selectedTrackIndices.length === 0) return; // Should not happen if button is disabled, but good check

      let updatedMetadata = [...uploadItemsMetadata]; // Create a copy

      for (const index of selectedTrackIndices) {
          let currentItem = { ...updatedMetadata[index] }; // Copy item to modify

          // Apply changes only for fields that were modified in the bulk form
          if (modifiedFields.has('album') && bulkData.album !== undefined) {
              currentItem.album = bulkData.album ?? undefined; // Handle null/empty string based on BulkForm logic
          }
          if (modifiedFields.has('artist') && bulkData.artist !== undefined) {
              currentItem.artist = bulkData.artist ?? undefined;
          }
          if (modifiedFields.has('genre') && bulkData.genre !== undefined) {
              currentItem.genre = bulkData.genre ?? undefined;
          }
           // Update for the new writer structure
          if (modifiedFields.has('writers') && bulkData.writers !== undefined && bulkData.writers !== null) {
              // Assuming BulkMetadataForm provides the final validated array
              currentItem.writers = JSON.parse(JSON.stringify(bulkData.writers)); // Deep copy
          }
          // TODO: Add logic for other bulk-editable fields (instruments, mood, etc.) based on modifiedFields

          updatedMetadata[index] = currentItem; // Update the item in the copied array
      }

      // Dispatch event with updated data
      dispatch('metadataUpdated', { updatedMetadata });

      // Exit bulk edit mode
      bulkEditMode = false;
      selectedTrackIndices = []; // Clear selection after applying
  }

  // Event handler for when IndividualMetadataForm emits 'save'
  function handleSaveIndividualEdit(event: CustomEvent<UploadItemMetadata>) {
      const updatedTrackData = event.detail;

      if (selectedTrackIndex < 0) return; // Should not happen

      let updatedMetadata = [...uploadItemsMetadata];
      // Find the original item by path (or a better ID if available) and update it
      // Assuming original_path is unique for items being uploaded
      const originalIndex = updatedMetadata.findIndex(item => item.original_path === updatedTrackData.original_path);

      if (originalIndex !== -1) {
          updatedMetadata[originalIndex] = updatedTrackData; // Replace with the edited data
          dispatch('metadataUpdated', { updatedMetadata }); // Dispatch the full updated list
      } else {
          console.error("Could not find original track to update:", updatedTrackData);
          // Handle error case - maybe notify user?
      }

      selectedTrackIndex = -1; // Close the individual editor
  }

  // Function to cancel editing and go back
  function cancelEditing() {
    selectedTrackIndex = -1;
    bulkEditMode = false; // Ensure bulk mode is off
    selectedTrackIndices = []; // Clear selections
    dispatch('cancel'); // Notify parent
  }

  // Function to finalize metadata (just closes editor and notifies parent)
  function finalizeMetadata() {
    selectedTrackIndex = -1;
    bulkEditMode = false;
    selectedTrackIndices = [];
    dispatch('finalize'); // Notify parent
  }

  // --- Removed Tag Handling and Writer Input Functions ---
  // This logic is now delegated to the child form components (IndividualMetadataForm, BulkMetadataForm)
  // and potentially the WriterEditor component.

  // --- Removed Reactive Statements for String <-> Array Conversion ---
  // This conversion logic is no longer needed with the new writer structure and delegation to child components.

  // --- Helper function to get selected items for Bulk Form ---
  // This could be done reactively too, but a function might be clearer
  function getSelectedItems(): UploadItemMetadata[] {
      return selectedTrackIndices.map(index => uploadItemsMetadata[index]).filter(item => !!item);
  }


  // Reset local state when the component is destroyed or metadata changes externally
  // $: if (uploadItemsMetadata) { // This might cause infinite loops if not careful
  //   selectedTrackIndex = -1;
  //   bulkEditMode = false;
  //   selectedTrackIndices = [];
  // }

</script>

<div class="metadata-editor p-4 border-gray-300 rounded-md shadow-sm">
  <div class="editor-header mb-4 flex justify-between items-center pb-4 border-b border-gray-200">
    <h3 class="text-xl font-semibold">Edit Metadata</h3>
    <div class="editor-actions flex gap-2">
      <!-- Finalize and Cancel buttons are always visible -->
      <button class="save-button px-4 py-2 rounded text-sm font-medium transition-colors duration-150 bg-blue-600 text-white hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500" on:click={finalizeMetadata}>Finalize Metadata</button>
      <button class="cancel-button px-4 py-2 rounded text-sm font-medium transition-colors duration-150 bg-gray-200 text-gray-700 hover:bg-gray-300 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-gray-400" on:click={cancelEditing}>Cancel</button>
    </div>
  </div>

  {#if bulkEditMode}
    <!-- Render Bulk Edit Form -->
    <BulkMetadataForm
      selectedItems={getSelectedItems()}
      on:apply={handleApplyBulkEdits}
      on:cancel={() => bulkEditMode = false}
    />
  {:else if selectedTrackIndex >= 0 && uploadItemsMetadata[selectedTrackIndex]}
    <!-- Render Individual Edit Form -->
    <!-- Pass a deep copy to prevent accidental modification -->
    <IndividualMetadataForm
      trackData={JSON.parse(JSON.stringify(uploadItemsMetadata[selectedTrackIndex]))}
      on:save={handleSaveIndividualEdit}
      on:cancel={() => selectedTrackIndex = -1}
    />
  {:else}
    <!-- Render Track List -->
    <TrackList
      items={uploadItemsMetadata}
      selectedIndices={selectedTrackIndices}
      on:toggleSelection={(e) => toggleTrackSelection(e.detail)}
      on:selectAll={(e) => selectAllTracks(e.detail)}
      on:editTrack={(e) => editTrack(e.detail)}
      on:startBulkEdit={startBulkEdit}
    />
  {/if}
</div>

<!-- Styles remain largely the same, but some might be moved to child components later -->
<style lang="postcss">
  .metadata-editor {
    border: 1px solid #e5e7eb;
    max-width: 900px; /* Or adjust as needed */
    margin: 1rem auto;
  }

  .editor-header {
    /* @apply flex justify-between items-center pb-4 border-b border-gray-200; */ /* Classes moved inline */
  }

  .editor-header h3 {
    /* @apply text-xl font-semibold; */ /* Classes moved inline */
  }

  .editor-actions {
    /* @apply flex gap-2; */ /* Classes moved inline */
  }

  /* General Button Styles (Apply to all buttons unless overridden) */
  button {
    /* Removed @apply and replaced with direct CSS properties */
    padding: 0.5rem 1rem;
    border-radius: 0.25rem;
    font-size: 0.875rem;
    font-weight: 500;
    transition-property: color, background-color, border-color, text-decoration-color, fill, stroke;
    transition-timing-function: cubic-bezier(0.4, 0, 0.2, 1);
    transition-duration: 150ms;
  }

  .save-button { /* Apply button style might be defined in BulkMetadataForm */
    /* @apply bg-blue-600 text-white hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500; */ /* Classes moved inline */
  }

  .cancel-button {
    /* @apply bg-gray-200 text-gray-700 hover:bg-gray-300 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-gray-400; */ /* Classes moved inline */
  }

  /* Keep general panel styles if needed, or move them too */
   /* These might be defined within the child components now */
   /*
   .bulk-edit-panel h4, .individual-edit-panel h4 {
       @apply text-lg font-medium mb-4 text-gray-800;
   }
   */

</style>