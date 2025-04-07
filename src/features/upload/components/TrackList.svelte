<script lang="ts">
  // Props and logic for displaying the track list and handling selections will go here.
  import type { UploadItemMetadata } from '$lib/types/catalog';
  import { createEventDispatcher } from 'svelte';

  export let items: UploadItemMetadata[] = [];
  export let selectedIndices: number[] = [];

  const dispatch = createEventDispatcher<{
    toggleSelection: number;
    selectAll: boolean;
    editTrack: number;
    startBulkEdit: void;
  }>();

  function handleSelectAllChange(event: Event) {
    const target = event.target as HTMLInputElement;
    dispatch('selectAll', target.checked);
  }

</script>

<div class="tracks-list-container">
  <!-- Markup for the track list, checkboxes, and buttons will be moved here -->
  <p>Track List Placeholder</p>
  <!-- Example: -->
   <div class="tracks-header">
     <div class="select-all-container">
       <input
         type="checkbox"
         id="select-all-tracks-list"
         checked={selectedIndices.length === items.length && items.length > 0}
         indeterminate={selectedIndices.length > 0 && selectedIndices.length < items.length}
         on:change={handleSelectAllChange}
       />
       <label for="select-all-tracks-list">Select All</label>
     </div>
     <!-- Add Bulk Edit button trigger -->
     {#if selectedIndices.length > 0}
        <button on:click={() => dispatch('startBulkEdit')}>Bulk Edit ({selectedIndices.length})</button>
     {/if}
   </div>
   <ul class="tracks-list">
    {#each items as item, index (item.original_path)}
      <li class:selected={selectedIndices.includes(index)}>
        <input
          type="checkbox"
          id={`track-select-${index}`}
          checked={selectedIndices.includes(index)}
          on:change={() => dispatch('toggleSelection', index)}
        />
        <span>{item.title || item.original_path.split('/').pop()}</span>
        <button on:click={() => dispatch('editTrack', index)}>Edit</button>
      </li>
    {/each}
   </ul>
</div>

<style>
  /* Styles for the track list will be moved or defined here */
  .tracks-list-container {
    /* Basic styling */
    padding: 1rem;
    border: 1px solid #ccc;
    margin-bottom: 1rem;
  }
  .tracks-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 0.5rem;
      padding-bottom: 0.5rem;
      border-bottom: 1px solid #eee;
  }
  .tracks-list {
    list-style: none;
    padding: 0;
    margin: 0;
  }
  .tracks-list li {
    display: flex;
    align-items: center;
    padding: 0.5rem 0;
    border-bottom: 1px solid #eee;
  }
   .tracks-list li:last-child {
      border-bottom: none;
   }
  .tracks-list li.selected {
    background-color: #e0e0e0;
  }
  .tracks-list li input[type="checkbox"] {
    margin-right: 0.5rem;
  }
   .tracks-list li span {
      flex-grow: 1;
      margin-right: 0.5rem;
   }
</style>