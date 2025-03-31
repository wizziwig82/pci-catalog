<script lang="ts">
  import { onMount } from 'svelte';
  
  function navigateToSettings() {
    window.location.href = '/settings';
  }
  
  function navigateToUpload() {
    window.location.href = '/upload';
  }
</script>

<main class="container">
  <h1>Music Library Manager</h1>
  
  <div class="welcome-section">
    <h2>Welcome to the Music Library Manager</h2>
    <p>This application helps you organize and manage your music library.</p>
    
    <div class="action-buttons">
      <button class="btn btn-upload" on:click={navigateToUpload}>
        Upload Music
      </button>
      <button class="btn btn-settings" on:click={navigateToSettings}>
        Database Settings
      </button>
    </div>
  </div>
</main>

<style lang="postcss">
  .container {
    max-width: 800px;
    margin: 0 auto;
    padding: 20px;
  }
  
  h1 {
    margin-bottom: 30px;
    color: #333;
  }
  
  .welcome-section {
    background-color: #f5f5f5;
    border-radius: 8px;
    padding: 30px;
    text-align: center;
  }
  
  .action-buttons {
    margin-top: 30px;
  }
  
  .btn {
    padding: 10px 20px;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 16px;
    transition: background-color 0.3s;
    margin: 0 10px;
  }
  
  .btn-settings {
    background-color: #6c757d;
    color: white;
  }
  
  .btn-settings:hover {
    background-color: #5a6268;
  }
  
  .btn-upload {
    background-color: #38a169;
    color: white;
  }
  
  .btn-upload:hover {
    background-color: #2f855a;
  }
</style>
