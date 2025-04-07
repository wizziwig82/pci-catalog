import adapter from '@sveltejs/adapter-static'; // Changed to static adapter
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

const config = {
	preprocess: [
		vitePreprocess(),
	],
	kit: {
		adapter: adapter(),
		alias: {
			// Define alias for feature modules
			'$features': 'src/features'
		}
	},
	extensions: ['.svelte']
};

export default config;
