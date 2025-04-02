<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { stringToTags } from '../stores/tagData';
  
  // Event dispatcher to communicate with parent components
  const dispatch = createEventDispatcher<{
    tagsChanged: { tags: string[] };
  }>();
  
  // Component props
  export let tagOptions: string[] = [];
  export let selectedTagsString: string = '';
  export let placeholder: string = "Add custom tag (press Enter)";
  export let showTextInput: boolean = true;
  
  // Internal state
  let selectedTags: string[] = stringToTags(selectedTagsString);
  let customTagInput: string = '';
  
  // Update selected tags whenever the input string changes
  $: {
    selectedTags = stringToTags(selectedTagsString);
  }
  
  // Toggle a tag's selection
  function toggleTag(tag: string) {
    const index = selectedTags.indexOf(tag);
    if (index >= 0) {
      // Remove tag if already selected
      selectedTags = [...selectedTags.slice(0, index), ...selectedTags.slice(index + 1)];
    } else {
      // Add tag if not selected
      selectedTags = [...selectedTags, tag];
    }
    
    // Dispatch event to parent
    dispatchTagsChanged();
  }
  
  // Add a custom tag
  function addCustomTag(e: KeyboardEvent) {
    const target = e.target as HTMLInputElement;
    if (e.key === 'Enter' && target.value.trim()) {
      const newTag = target.value.trim();
      
      // Check if tag already exists to avoid duplicates
      if (!selectedTags.includes(newTag)) {
        selectedTags = [...selectedTags, newTag];
        dispatchTagsChanged();
      }
      
      // Clear the input
      target.value = '';
      customTagInput = '';
    }
  }
  
  // Remove a tag from the selected list
  function removeTag(tag: string) {
    selectedTags = selectedTags.filter(t => t !== tag);
    dispatchTagsChanged();
  }
  
  // Dispatch the tagsChanged event
  function dispatchTagsChanged() {
    dispatch('tagsChanged', { tags: selectedTags });
  }
</script>

<div class="tag-selector">
  <div class="tag-options-container">
    <div class="tag-options">
      {#each tagOptions as tag}
        <button 
          class="tag-option" 
          class:selected={selectedTags.includes(tag)} 
          on:click={() => toggleTag(tag)}
        >
          {tag}
        </button>
      {/each}
    </div>
  </div>
  
  {#if showTextInput}
    <div class="custom-tag-input">
      <input 
        type="text" 
        placeholder={placeholder} 
        on:keydown={addCustomTag}
        bind:value={customTagInput}
      />
    </div>
  {/if}
  
  <div class="selected-tags">
    {#each selectedTags as tag}
      <div class="selected-tag">
        <span>{tag}</span>
        <button class="remove-tag" on:click={() => removeTag(tag)}>×</button>
      </div>
    {/each}
  </div>
</div>

<style lang="postcss">
  .tag-selector {
    width: 100%;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Open Sans', 'Helvetica Neue', sans-serif;
    margin-bottom: 16px;
  }
  
  .tag-options-container {
    max-height: 240px; /* Increased height to reduce scrolling */
    overflow-y: auto;
    border: 1px solid #e2e8f0;
    border-radius: 4px;
    padding: 6px;
    margin-bottom: 8px;
    background-color: #f9fafb;
  }
  
  .tag-options {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(65px, 1fr)); /* Smaller minimum width for more columns */
    gap: 4px;
  }
  
  .tag-option {
    font-size: 9px; /* Smaller font size */
    padding: 3px 4px; /* Smaller padding */
    border-radius: 3px;
    border: 1px solid #d1d5db;
    background-color: #ffffff;
    cursor: pointer;
    transition: all 0.2s;
    text-align: center;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    height: 20px; /* Fixed height for uniformity */
    line-height: 1;
  }
  
  .tag-option:hover {
    border-color: #4299e1;
  }
  
  .tag-option.selected {
    background-color: #4299e1;
    color: white;
    border-color: #3182ce;
  }
  
  .custom-tag-input input {
    width: 100%;
    padding: 6px;
    border: 1px solid #e2e8f0;
    border-radius: 4px;
    font-size: 13px;
    margin-bottom: 8px;
  }
  
  .custom-tag-input input:focus {
    outline: none;
    border-color: #4299e1;
    box-shadow: 0 0 0 2px rgba(66, 153, 225, 0.2);
  }
  
  .selected-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-top: 8px;
  }
  
  .selected-tag {
    display: flex;
    align-items: center;
    background-color: #edf2f7;
    border-radius: 4px;
    padding: 3px 6px;
    font-size: 11px;
  }
  
  .remove-tag {
    margin-left: 4px;
    background: none;
    border: none;
    color: #a0aec0;
    cursor: pointer;
    font-size: 12px;
    line-height: 1;
    padding: 0 2px;
  }
  
  .remove-tag:hover {
    color: #e53e3e;
  }
</style> 