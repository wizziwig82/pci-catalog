<script lang="ts">
  import { onMount } from 'svelte';
  // import { invoke } from '@tauri-apps/api/core'; // No longer needed directly
  import { safeInvoke } from '$lib/utils/invokeWrapper'; // Import the wrapper
  import { showSuccessToast } from '$lib/stores/notifications'; // Import success toast

  // MongoDB variables - MODIFIED for Connection String
  let mongoConnectionString = ''; 
  // Removed mongoUsername, mongoPassword, mongoHostname, mongoPort
  let mongoConnected = false;
  let mongoConnectionError = '';
  let mongoLoading = false;

  // R2 variables
  let r2AccountId = '';
  let r2BucketName = '';
  let r2AccessKeyId = '';
  let r2SecretAccessKey = '';
  let r2Endpoint = '';
  let r2Connected = false;
  let r2ConnectionError = '';
  let r2Loading = false;

  onMount(async () => {
    await loadStoredCredentials();
  });

  async function loadStoredCredentials() {
    try {
      // Use safeInvoke to load MongoDB connection string
      const connectionString = await safeInvoke<string>('get_mongo_credentials_wrapper');
      if (connectionString !== null) { // Check for null (indicates error handled by wrapper)
        mongoConnectionString = connectionString;
        // Don't assume connected, let test/init confirm
        // mongoConnected = true;
      } else {
        // Error occurred (toast shown by safeInvoke), ensure disconnected state
        mongoConnected = false;
      }
    } catch (e) {
       // This catch block is less likely to be hit if safeInvoke handles errors,
       // but keep it as a fallback for unexpected issues during load.
       console.error("Unexpected error during MongoDB credential load:", e);
       mongoConnected = false;
    }

    try {
      // Use safeInvoke to load R2 credentials
      const credentials = await safeInvoke<any>('get_r2_credentials_wrapper'); // Assuming R2Credentials structure matches Rust
      if (credentials !== null) {
        r2AccountId = credentials.account_id ?? '';
        r2BucketName = credentials.bucket_name ?? '';
        r2AccessKeyId = credentials.access_key_id ?? '';
        r2SecretAccessKey = credentials.secret_access_key ?? '';
        r2Endpoint = credentials.endpoint ?? '';
        // Don't assume connected
        // r2Connected = true;
      } else {
        // Error occurred (toast shown by safeInvoke), ensure disconnected state
        r2Connected = false;
      }
    } catch (e) {
       console.error("Unexpected error during R2 credential load:", e);
       r2Connected = false;
    }
  }

  async function initMongoClient() {
    mongoLoading = true;
    mongoConnectionError = '';

    // Store credentials using safeInvoke
    const storeSuccess = await safeInvoke<boolean>('store_mongo_credentials_wrapper', {
      connectionString: mongoConnectionString
    });

    if (storeSuccess === null) {
      // Error storing (toast shown by safeInvoke)
      mongoLoading = false;
      return; // Stop if storing failed
    }

    // Initialize client using safeInvoke
    const initSuccess = await safeInvoke<boolean>('init_mongo_client');

    if (initSuccess) {
      mongoConnected = true;
      mongoConnectionError = ''; // Clear previous errors
      showSuccessToast('MongoDB connected successfully!');
    } else {
      // Error initializing (toast shown by safeInvoke)
      mongoConnected = false;
      // mongoConnectionError = 'Failed to initialize MongoDB client.'; // Error shown by toast
    }

    mongoLoading = false;
  }

  async function initR2Client() {
    r2Loading = true;
    r2ConnectionError = '';

    // Store credentials using safeInvoke
    const storeSuccess = await safeInvoke<boolean>('store_r2_credentials_wrapper', {
      accountId: r2AccountId,
      bucketName: r2BucketName, // Ensure key matches Rust command argument name
      accessKeyId: r2AccessKeyId, // Ensure key matches Rust command argument name
      secretAccessKey: r2SecretAccessKey, // Ensure key matches Rust command argument name
      endpoint: r2Endpoint
    });

     if (storeSuccess === null) {
      // Error storing (toast shown by safeInvoke)
      r2Loading = false;
      return; // Stop if storing failed
    }

    // Initialize client using safeInvoke
    const initSuccess = await safeInvoke<boolean>('init_r2_client');

    if (initSuccess) {
      r2Connected = true;
      r2ConnectionError = ''; // Clear previous errors
      showSuccessToast('R2 connected successfully!');
    } else {
      // Error initializing (toast shown by safeInvoke)
      r2Connected = false;
      // r2ConnectionError = 'Failed to initialize R2 client.'; // Error shown by toast
    }

    r2Loading = false;
  }

  async function testMongoConnection() {
    mongoLoading = true;
    mongoConnectionError = ''; // Clear previous status

    // Test connection using safeInvoke
    const success = await safeInvoke<boolean>('test_mongo_connection');

    if (success) {
      mongoConnected = true;
      mongoConnectionError = 'Connection successful!'; // Provide feedback in the UI
      // showSuccessToast('MongoDB connection test successful!'); // Optional toast
    } else {
      // Error testing (toast shown by safeInvoke)
      mongoConnected = false;
      // mongoConnectionError = 'Connection test failed.'; // Error shown by toast
    }

    mongoLoading = false;
  }

  async function testR2Connection() {
    r2Loading = true;
    r2ConnectionError = ''; // Clear previous status

    // Test connection using safeInvoke
    const success = await safeInvoke<boolean>('test_r2_connection');

    if (success) {
      r2Connected = true;
      r2ConnectionError = 'Connection successful!'; // Provide feedback in the UI
      // showSuccessToast('R2 connection test successful!'); // Optional toast
    } else {
      // Error testing (toast shown by safeInvoke)
      r2Connected = false;
      // r2ConnectionError = 'Connection test failed.'; // Error shown by toast
    }

    r2Loading = false;
  }

  async function deleteMongoCredentials() {
    // Use safeInvoke for deletion
    const success = await safeInvoke<void>('delete_credentials', { credential_type: 'mongo' });
    if (success !== null) { // Check if command itself succeeded (null means error)
      mongoConnectionString = '';
      mongoConnected = false;
      mongoConnectionError = '';
      showSuccessToast('MongoDB credentials deleted.');
    }
    // Error toast is shown by safeInvoke if deletion fails
  }

  async function deleteR2Credentials() {
    // Use safeInvoke for deletion
    const success = await safeInvoke<void>('delete_credentials', { credential_type: 'r2' });
     if (success !== null) {
      r2AccountId = '';
      r2BucketName = '';
      r2AccessKeyId = '';
      r2SecretAccessKey = '';
      r2Endpoint = '';
      r2Connected = false;
      r2ConnectionError = ''; // Clear error message
      showSuccessToast('R2 credentials deleted.');
    }
    // Error toast is shown by safeInvoke if deletion fails
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
        <label for="r2BucketName" class="block text-sm font-medium mb-1">Bucket Name</label>
        <input type="text" id="r2BucketName" bind:value={r2BucketName} class="w-full p-2 border rounded" />
      </div>
      
      <div class="mb-2">
        <label for="r2AccessKeyId" class="block text-sm font-medium mb-1">Access Key ID</label>
        <input type="text" id="r2AccessKeyId" bind:value={r2AccessKeyId} class="w-full p-2 border rounded" />
      </div>
      
      <div class="mb-3">
        <label for="r2SecretAccessKey" class="block text-sm font-medium mb-1">Secret Access Key</label>
        <input type="password" id="r2SecretAccessKey" bind:value={r2SecretAccessKey} class="w-full p-2 border rounded" />
      </div>
      
      <div class="mb-2">
        <label for="r2Endpoint" class="block text-sm font-medium mb-1">Endpoint</label>
        <input type="text" id="r2Endpoint" bind:value={r2Endpoint} class="w-full p-2 border rounded" 
          placeholder="https://your-account-id.r2.cloudflarestorage.com" />
        <p class="text-xs text-gray-500 mt-1">Format: https://your-account-id.r2.cloudflarestorage.com</p>
      </div>
      
      <div class="flex space-x-2 mb-2">
        <button 
          on:click={initR2Client} 
          disabled={r2Loading || (!r2AccountId || !r2BucketName || !r2AccessKeyId || !r2SecretAccessKey || !r2Endpoint)}
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