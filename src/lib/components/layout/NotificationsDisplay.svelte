<script lang="ts">
  import { notifications } from '$lib/stores/notifications';
  import { fly } from 'svelte/transition';

  function getBackgroundColor(type: string): string {
    switch (type) {
      case 'error': return 'bg-red-500';
      case 'success': return 'bg-green-500';
      case 'info': return 'bg-blue-500';
      default: return 'bg-gray-500';
    }
  }
</script>

{#if $notifications.length > 0}
<div class="fixed bottom-4 right-4 z-50 flex flex-col space-y-2">
  {#each $notifications as notification (notification.id)}
    <div
      in:fly={{ y: 20, duration: 300 }}
      out:fly={{ x: 100, duration: 200 }}
      class="p-4 rounded-md text-white shadow-lg {getBackgroundColor(notification.type)}"
    >
      <div class="flex justify-between items-center">
        <span>{notification.message}</span>
        <button
          on:click={() => notifications.remove(notification.id)}
          class="ml-4 text-xl font-bold leading-none hover:text-gray-200"
          aria-label="Close notification"
        >
          &times;
        </button>
      </div> <!-- Missing closing div for flex container -->
      </div>
    <!-- Removed extra closing div -->
  {/each}
</div>
{/if}

<style>
  /* Add any additional styling if needed, Tailwind classes are used primarily */
</style>