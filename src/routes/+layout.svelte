<script lang="ts">
	import '../app.css';
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';

	onMount(async () => {
		console.log('Layout mounted, attempting to initialize R2 client...');
		try {
			// Try to initialize client with existing credentials
			const success = await invoke<boolean>('init_r2_client');
			if (success) {
				console.log('R2 client initialized successfully.');
			} else {
				console.warn('init_r2_client command returned false.');
			}
		} catch (error) {
			// Only show error if it's not just missing credentials
			if (typeof error === 'string' && !error.includes('not set')) {
				console.error('Failed to initialize R2 client:', error);
			} else {
				console.log('R2 credentials not configured yet. Navigate to Settings to set up credentials.');
			}
		}
	});
</script>

<nav class="main-nav">
	<div class="nav-container">
		<a href="/" class="nav-logo">Music Library Manager</a>
		<div class="nav-links">
			<a href="/" class="nav-link">Home</a>
			<a href="/upload" class="nav-link">Upload</a>
			<a href="/settings" class="nav-link">Settings</a>
		</div>
	</div>
</nav>

<main>
	<slot></slot>
</main>

<style lang="postcss">
	.main-nav {
		background-color: #343a40;
		color: white;
		padding: 10px 0;
		box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
	}
	
	.nav-container {
		max-width: 1200px;
		margin: 0 auto;
		padding: 0 20px;
		display: flex;
		justify-content: space-between;
		align-items: center;
	}
	
	.nav-logo {
		font-size: 1.25rem;
		font-weight: bold;
		color: white;
		text-decoration: none;
	}
	
	.nav-links {
		display: flex;
		gap: 20px;
	}
	
	.nav-link {
		color: rgba(255, 255, 255, 0.75);
		text-decoration: none;
		padding: 5px 0;
		transition: color 0.3s;
	}
	
	.nav-link:hover {
		color: white;
	}
	
	main {
		min-height: calc(100vh - 58px);
	}
</style>
