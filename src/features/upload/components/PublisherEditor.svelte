<script context="module" lang="ts">
  // Define the publisher structure expected by this component
  export interface PublisherEntry { // Moved to context="module" and EXPORTED
    name: string;
    percentage: number;
  }
</script>

<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  // Note: We don't need to import the type here anymore as it's defined in the module context

  // Props
  export let publishers: PublisherEntry[] = []; // Renamed from writers, use bind:publishers for two-way binding

  // State
  let currentPublisherInput = ''; // Renamed from currentWriterInput
  let validationError: string | null = null;

  const dispatch = createEventDispatcher<{
    change: PublisherEntry[]; // Event dispatched when publishers array changes internally
  }>();

  // --- Percentage Distribution Logic (Functionally identical to WriterEditor) ---
  function distributePercentages(currentPublishers: PublisherEntry[]): PublisherEntry[] { // Renamed parameter
    const count = currentPublishers.length;
    if (count === 0) return [];

    let assignedPercentage = 0;
    const basePercentage = Math.floor(100 / count);
    const remainder = 100 % count;

    const updatedPublishers = currentPublishers.map((publisher, index) => { // Renamed variable
      let percentage = basePercentage;
      if (index < remainder) {
        percentage += 1; // Distribute remainder
      }
      assignedPercentage += percentage;
      return { ...publisher, percentage }; // Renamed variable
    });

    // Final check to ensure sum is exactly 100
     let finalSum = updatedPublishers.reduce((sum, p) => sum + p.percentage, 0); // Renamed variable
     if(finalSum !== 100 && updatedPublishers.length > 0) {
         // Adjust the last element slightly if needed
         updatedPublishers[updatedPublishers.length - 1].percentage += (100 - finalSum);
     }

    return updatedPublishers;
  }

  // --- Add/Remove Logic ---
  function addPublisher() { // Renamed function
    const newName = currentPublisherInput.trim();
    if (newName && !publishers.some(p => p.name === newName)) { // Renamed variable
      const newPublisher: PublisherEntry = { name: newName, percentage: 0 }; // Renamed variable
      const updatedPublishers = distributePercentages([...publishers, newPublisher]); // Renamed variables
      publishers = updatedPublishers; // Update the bound prop, renamed variable
      dispatch('change', publishers); // Notify parent of change, renamed variable
      currentPublisherInput = ''; // Clear input, renamed variable
      validatePercentages(); // Re-validate after adding
    } else {
        currentPublisherInput = ''; // Clear input even if duplicate/empty, renamed variable
    }
  }

  function removePublisher(indexToRemove: number) { // Renamed function
    const updatedPublishers = distributePercentages(publishers.filter((_, index) => index !== indexToRemove)); // Renamed variable
    publishers = updatedPublishers; // Update the bound prop, renamed variable
    dispatch('change', publishers); // Notify parent of change, renamed variable
    validatePercentages(); // Re-validate after removing
  }

  // --- Validation (Functionally identical to WriterEditor) ---
  function validatePercentages() {
    validationError = null;
    if (publishers.length === 0) return; // No validation needed for empty list, renamed variable

    let sum = 0;
    for (const publisher of publishers) { // Renamed variable
      const percentage = Number(publisher.percentage); // Ensure it's treated as a number, renamed variable
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

  // Calling validatePercentages once initially to validate the initial state
  $: if (publishers) {
    // Avoid the recursive loop by not modifying the publishers array here
    validatePercentages();
  }
</script>

<div class="publisher-editor"> <!-- Renamed class -->
  <div class="tags-input">
    <input
      type="text"
      bind:value={currentPublisherInput}
      placeholder="Add publisher name and press Enter"
      on:keydown={(e: KeyboardEvent) => { if (e.key === 'Enter') { e.preventDefault(); addPublisher(); } }}
    />
  </div>

  <div class="tags-list">
    {#each publishers as publisher, i (publisher.name)} <!-- Renamed variables -->
      <div class="tag-item">
        <span class="publisher-name">{publisher.name}</span> <!-- Renamed class and variable -->
        <div class="percentage-input">
          <input
            type="number"
            min="0"
            max="100"
            step="0.01"
            bind:value={publisher.percentage}
            on:input={validatePercentages}
            aria-label={`Percentage for ${publisher.name}`}
          />
          <span>%</span>
        </div>
        <button class="remove-tag" on:click={() => removePublisher(i)} aria-label={`Remove publisher ${publisher.name}`}>&times;</button> <!-- Renamed function call and aria-label -->
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
  /* Renamed main class */
  .publisher-editor {
    /* Styles for the publisher editor section */
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
  /* Renamed specific class */
  .publisher-name {
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