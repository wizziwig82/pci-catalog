<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  // MongoDB variables - MODIFIED for Connection String
  let mongoConnectionString = ''; 
  // Removed mongoUsername, mongoPassword, mongoHostname, mongoPort
  let mongoConnected = false;
  let mongoConnectionError = '';
  let mongoLoading = false;

  // R2 variables
  let r2AccountId = '';
  let r2AccessKeyId = '';
  let r2SecretAccessKey = '';
  let r2Connected = false;
  let r2ConnectionError = '';
  let r2Loading = false;

  onMount(async () => {
    await loadStoredCredentials();
  });

  async function loadStoredCredentials() {
    try {
      // Directly attempt to load MongoDB connection string
      // @ts-ignore - Tauri API typings might still be imperfect
      const connectionString = await invoke<string>('get_mongo_credentials');
      if (connectionString) {
        mongoConnectionString = connectionString;
        mongoConnected = true; // Assume connected if loaded, test separately
      }
    } catch (error: any) {
      // Ignore specific "not found" or empty file errors, log others
      const errorMsg = typeof error === 'string' ? error : JSON.stringify(error);
      if (!errorMsg.includes('not found') && !errorMsg.includes('empty')) {
        console.error('Error loading MongoDB credentials:', error);
      }
      mongoConnected = false; // Ensure disconnected state if load fails
    }

    try {
      // Directly attempt to load R2 credentials
      // @ts-ignore - Tauri API typings might still be imperfect for credential structure
      const credentials = await invoke<any>('get_r2_credentials');
      if (credentials) {
        r2AccountId = credentials.account_id;
        r2AccessKeyId = credentials.access_key_id;
        r2SecretAccessKey = credentials.secret_access_key;
        r2Connected = true; // Assume connected if loaded, test separately if needed
      }
    } catch (error) {
      // Ignore specific "not found" error, log others
      if (error !== 'R2 credentials not found') {
        console.error('Error loading R2 credentials:', error);
      }
    }
  }

  async function initMongoClient() {
    try {
      mongoLoading = true;
      mongoConnectionError = '';
      
      // Store the connection string
      // @ts-ignore - Tauri API typings not working correctly
      await invoke('store_mongo_credentials', {
        connectionString: mongoConnectionString // Pass connection string
      });
      
      // Initialize the MongoDB client (backend now uses the stored string)
      // @ts-ignore - Tauri API typings not working correctly
      const success = await invoke('init_mongo_client');
      
      if (success) {
        mongoConnected = true;
        mongoConnectionError = '';
      } else {
        mongoConnected = false;
        mongoConnectionError = 'Failed to connect to MongoDB using the provided string.';
      }
    } catch (error: any) {
      mongoConnected = false;
      mongoConnectionError = `Error connecting to MongoDB: ${typeof error === 'string' ? error : JSON.stringify(error)}`;
      console.error('MongoDB connection error:', error);
    } finally {
      mongoLoading = false;
    }
  }

  async function initR2Client() {
    try {
      r2Loading = true;
      r2ConnectionError = '';
      
      // Store the credentials
      // @ts-ignore - Tauri API typings not working correctly
      await invoke('store_r2_credentials', {
        accountId: r2AccountId,
        accessKeyId: r2AccessKeyId,
        secretAccessKey: r2SecretAccessKey
      });
      
      // Initialize the R2 client
      // @ts-ignore - Tauri API typings not working correctly
      const success = await invoke('init_r2_client');
      
      if (success) {
        r2Connected = true;
        r2ConnectionError = '';
      } else {
        r2Connected = false;
        r2ConnectionError = 'Failed to connect to R2';
      }
    } catch (error) {
      r2Connected = false;
      r2ConnectionError = `Error connecting to R2: ${error}`;
      console.error('R2 connection error:', error);
    } finally {
      r2Loading = false;
    }
  }

  async function testMongoConnection() {
    // Test function remains largely the same, backend handles the logic
    try {
      mongoLoading = true;
      mongoConnectionError = '';
      
      // Test the MongoDB connection
      // @ts-ignore - Tauri API typings not working correctly
      const success = await invoke('test_mongo_connection');
      
      if (success) {
        mongoConnected = true;
        mongoConnectionError = 'Connection successful!';
      } else {
        mongoConnected = false;
        mongoConnectionError = 'Failed to connect to MongoDB. Check the connection string and network.';
      }
    } catch (error: any) {
      mongoConnected = false;
      mongoConnectionError = `Error testing MongoDB connection: ${typeof error === 'string' ? error : JSON.stringify(error)}`;
      console.error('MongoDB connection test error:', error);
    } finally {
      mongoLoading = false;
    }
  }

  async function testR2Connection() {
    try {
      r2Loading = true;
      r2ConnectionError = '';
      
      // Test the R2 connection
      // @ts-ignore - Tauri API typings not working correctly
      const success = await invoke('test_r2_connection');
      
      if (success) {
        r2Connected = true;
        r2ConnectionError = '';
      } else {
        r2Connected = false;
        r2ConnectionError = 'Failed to connect to R2';
      }
    } catch (error) {
      r2Connected = false;
      r2ConnectionError = `Error testing R2 connection: ${error}`;
      console.error('R2 connection test error:', error);
    } finally {
      r2Loading = false;
    }
  }

  async function deleteMongoCredentials() {
    try {
      // @ts-ignore - Tauri API typings not working correctly
      await invoke('delete_credentials', { credential_type: 'mongo' });
      mongoConnectionString = ''; // Clear the variable
      mongoConnected = false;
      mongoConnectionError = ''; // Clear error message
    } catch (error) {
      console.error('Error deleting MongoDB credentials:', error);
      mongoConnectionError = 'Failed to delete credentials.'; // Provide feedback
    }
  }

  async function deleteR2Credentials() {
    try {
      // @ts-ignore - Tauri API typings not working correctly
      await invoke('delete_credentials', { credential_type: 'r2' });
      r2AccountId = '';
      r2AccessKeyId = '';
      r2SecretAccessKey = '';
      r2Connected = false;
    } catch (error) {
      console.error('Error deleting R2 credentials:', error);
    }
  }
</script>

<div class="container p-4">
  <h1 class="text-2xl font-bold mb-4">Settings</h1>
  
  <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
    <!-- MongoDB Settings -->
    <div class="bg-white dark:bg-gray-800 p-4 rounded shadow">
      <h2 class="text-xl font-semibold mb-3">MongoDB Connection</h2>
      
      <div class="mb-3">
        <label for="mongoConnectionString" class="block text-sm font-medium mb-1">Connection String</label>
        <!-- Use textarea for better visibility of long strings -->
        <textarea 
          id="mongoConnectionString" 
          bind:value={mongoConnectionString} 
          class="w-full p-2 border rounded font-mono text-sm" 
          rows="4"
          placeholder="mongodb+srv://user:<password>@cluster..."
        ></textarea>
         <!-- Removed old input fields -->
      </div>
            
      <div class="flex space-x-2 mb-2">
        <button 
          on:click={initMongoClient} 
          disabled={mongoLoading || !mongoConnectionString} 
          class="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:bg-gray-400"
        >
          {mongoLoading ? 'Saving & Connecting...' : 'Save & Connect'}
        </button>
        
        <button 
          on:click={testMongoConnection} 
          disabled={mongoLoading || !mongoConnected} 
          class="px-4 py-2 bg-gray-600 text-white rounded hover:bg-gray-700 disabled:bg-gray-400"
        >
          {mongoLoading ? 'Testing...' : 'Test Connection'}
        </button>

        <button 
          on:click={deleteMongoCredentials} 
          disabled={mongoLoading}
          class="px-4 py-2 bg-red-600 text-white rounded hover:bg-red-700 disabled:bg-gray-400"
        >
         Delete Credentials
        </button>
      </div>

      {#if mongoConnectionError}
        <p class="text-red-500 text-sm mt-2">{mongoConnectionError}</p>
      {:else if mongoConnected}
        <p class="text-green-500 text-sm mt-2">Connection Active</p>
      {/if}
    </div>

    <!-- R2 Settings -->
    <div class="bg-white dark:bg-gray-800 p-4 rounded shadow">
      <h2 class="text-xl font-semibold mb-3">R2 Credentials</h2>
      
      <div class="mb-2">
        <label for="r2AccountId" class="block text-sm font-medium mb-1">Account ID</label>
        <input type="text" id="r2AccountId" bind:value={r2AccountId} class="w-full p-2 border rounded" />
      </div>
      
      <div class="mb-2">
        <label for="r2AccessKeyId" class="block text-sm font-medium mb-1">Access Key ID</label>
        <input type="text" id="r2AccessKeyId" bind:value={r2AccessKeyId} class="w-full p-2 border rounded" />
      </div>
      
      <div class="mb-3">
        <label for="r2SecretAccessKey" class="block text-sm font-medium mb-1">Secret Access Key</label>
        <input type="password" id="r2SecretAccessKey" bind:value={r2SecretAccessKey} class="w-full p-2 border rounded" />
      </div>
      
      <div class="flex space-x-2 mb-2">
        <button 
          on:click={initR2Client} 
          disabled={r2Loading || (!r2AccountId || !r2AccessKeyId || !r2SecretAccessKey)}
          class="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:bg-gray-400"
        >
          {r2Loading ? 'Saving & Connecting...' : 'Save & Connect'}
        </button>

        <button 
          on:click={testR2Connection} 
          disabled={r2Loading || !r2Connected}
          class="px-4 py-2 bg-gray-600 text-white rounded hover:bg-gray-700 disabled:bg-gray-400"
        >
          {r2Loading ? 'Testing...' : 'Test Connection'}
        </button>

         <button 
          on:click={deleteR2Credentials} 
          disabled={r2Loading}
          class="px-4 py-2 bg-red-600 text-white rounded hover:bg-red-700 disabled:bg-gray-400"
        >
         Delete Credentials
        </button>
      </div>

      {#if r2ConnectionError}
        <p class="text-red-500 text-sm mt-2">{r2ConnectionError}</p>
      {:else if r2Connected}
        <p class="text-green-500 text-sm mt-2">Connection Active</p>
      {/if}
    </div>
  </div>
</div>

<style lang="postcss">
  /* The postcss lang attribute is needed to avoid the "Expected token }" error */
</style> 