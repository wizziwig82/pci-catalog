<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  // Define the writer structure expected by this component
  interface WriterEntry { // Removed 'export'
    name: string;
    percentage: number;
  }

  // Props
  export let writers: WriterEntry[] = []; // Use bind:writers for two-way binding

  // State
  let currentWriterInput = '';
  let validationError: string | null = null;

  const dispatch = createEventDispatcher<{
    change: WriterEntry[]; // Event dispatched when writers array changes internally
  }>();

  // --- Percentage Distribution Logic ---
  function distributePercentages(currentWriters: WriterEntry[]): WriterEntry[] {
    const count = currentWriters.length;
    if (count === 0) return [];

    let assignedPercentage = 0;
    const basePercentage = Math.floor(100 / count);
    const remainder = 100 % count;

    const updatedWriters = currentWriters.map((writer, index) => {
      let percentage = basePercentage;
      if (index < remainder) {
        percentage += 1; // Distribute remainder
      }
      assignedPercentage += percentage;
      return { ...writer, percentage };
    });

    // Final check to ensure sum is exactly 100 due to potential floating point issues (though less likely with floor/remainder)
     let finalSum = updatedWriters.reduce((sum, w) => sum + w.percentage, 0);
     if(finalSum !== 100 && updatedWriters.length > 0) {
         // Adjust the last element slightly if needed
         updatedWriters[updatedWriters.length - 1].percentage += (100 - finalSum);
     }

    return updatedWriters;
  }

  // --- Add/Remove Logic ---
  function addWriter() {
    const newName = currentWriterInput.trim();
    if (newName && !writers.some(w => w.name === newName)) {
      const newWriter: WriterEntry = { name: newName, percentage: 0 }; // Initial percentage 0, will be recalculated
      const updatedWriters = distributePercentages([...writers, newWriter]);
      writers = updatedWriters; // Update the bound prop
      dispatch('change', writers); // Notify parent of change
      currentWriterInput = ''; // Clear input
      validatePercentages(); // Re-validate after adding
    } else {
        currentWriterInput = ''; // Clear input even if duplicate/empty
    }
  }

  function removeWriter(indexToRemove: number) {
    const updatedWriters = distributePercentages(writers.filter((_, index) => index !== indexToRemove));
    writers = updatedWriters; // Update the bound prop
    dispatch('change', writers); // Notify parent of change
    validatePercentages(); // Re-validate after removing
  }

  // --- Validation ---
  function validatePercentages() {
    validationError = null;
    if (writers.length === 0) return; // No validation needed for empty list

    let sum = 0;
    for (const writer of writers) {
      const percentage = Number(writer.percentage); // Ensure it's treated as a number
      if (isNaN(percentage) || percentage < 0) {
        validationError = 'Percentages must be non-negative numbers.';
        return;
      }
      sum += percentage;
    }

    // Use a small tolerance for floating point comparisons
    if (Math.abs(sum - 100) > 0.01) {
      validationError = `Percentages must sum to 100%. Current sum: ${sum.toFixed(2)}%`;
    }
  }

  // Reactive validation whenever the writers array or its contents change
  $: if (writers) {
      validatePercentages();
      // Optional: Dispatch change event whenever internal percentage inputs modify the array
      // This might be redundant if parent relies on bind:writers
      // dispatch('change', writers);
  }

</script>

<div class="writer-editor">
  <div class="tags-input">
    <input
      type="text"
      bind:value={currentWriterInput}
      placeholder="Add writer name and press Enter"
      on:keydown={(e) => { if (e.key === 'Enter') { e.preventDefault(); addWriter(); } }}
    />
  </div>

  <div class="tags-list">
    {#each writers as writer, i (writer.name)}
      <div class="tag-item">
        <span class="writer-name">{writer.name}</span>
        <div class="percentage-input">
          <input
            type="number"
            min="0"
            max="100"
            step="0.01"
            bind:value={writer.percentage}
            on:input={validatePercentages}
            aria-label={`Percentage for ${writer.name}`}
          /> <!-- Allow decimals --><!-- Validate on manual input -->
          <span>%</span>
        </div>
        <button class="remove-tag" on:click={() => removeWriter(i)} aria-label={`Remove writer ${writer.name}`}>&times;</button>
      </div>
    {/each}
  </div>

  {#if validationError}
    <div class="validation-error" role="alert">
      {validationError}
    </div>
  {/if}
</div>

<style>
  .writer-editor {
    /* Styles for the writer editor section */
  }
  .tags-input {
    margin-bottom: 0.5rem;
  }
  .tags-input input {
     width: 100%;
     padding: 0.5rem;
     border: 1px solid #ccc;
     border-radius: 4px;
  }
  .tags-list {
    margin-bottom: 0.5rem;
  }
  .tag-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.25rem 0;
    border-bottom: 1px solid #eee;
  }
   .tag-item:last-child {
      border-bottom: none;
   }
  .writer-name {
    flex-grow: 1;
  }
  .percentage-input {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }
  .percentage-input input {
    width: 60px; /* Adjust as needed */
    padding: 0.25rem;
     border: 1px solid #ccc;
     border-radius: 4px;
     text-align: right;
  }
  .remove-tag {
    background: none;
    border: none;
    color: red;
    cursor: pointer;
    font-size: 1.2em;
    padding: 0 0.25rem;
  }
  .validation-error {
    color: red;
    font-size: 0.9em;
    margin-top: 0.5rem;
  }
</style>