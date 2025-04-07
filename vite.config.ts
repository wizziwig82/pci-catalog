import { paraglideVitePlugin } from '@inlang/paraglide-js';
import { svelteTesting } from '@testing-library/svelte/vite';
import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import path from 'node:path'; // Use node: prefix
import { fileURLToPath } from 'node:url'; // Import fileURLToPath

export default defineConfig({
	plugins: [
		tailwindcss(),
		sveltekit(),
		paraglideVitePlugin({
			project: './project.inlang',
			outdir: './src/lib/paraglide'
		}),
		svelteTesting(), // Add testing library plugin for Svelte
	],
	// Vitest configuration consolidated here
	test: {
		globals: true,
		environment: 'jsdom', // Enable JSDOM for testing Svelte components
		setupFiles: ['./vitest-setup-client.ts'], // Use only client setup for jsdom environment
		include: ['./tests/**/*.test.ts', 'src/**/*.{test,spec}.{js,ts}'], // Include tests and src tests
		// Consider if specific excludes are needed if not using workspaces
		deps: {
		  inline: [/svelte/]
		},
		coverage: {
		  provider: 'v8',
		  reporter: ['text', 'json', 'html'],
		  include: ['src/**/*.{ts,js,svelte}'] // Focus coverage on src
		}
	},
	// Prevent vite from obscuring Rust errors
	clearScreen: false,
	// Tauri expects a fixed port, fail if that port is not available
	server: {
		strictPort: true,
	},
	// To make use of `TAURI_PLATFORM`, `TAURI_DEBUG`, etc
	envPrefix: ['VITE_', 'TAURI_'],
	build: {
		// Tauri supports es2021
		target: ['es2021', 'chrome100', 'safari14'],
		// Don't minify for debug builds
		minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
		// Produce sourcemaps for debug builds
		sourcemap: !!process.env.TAURI_DEBUG,
	},
	// Configure optimizedDeps to include @tauri-apps/api
	optimizeDeps: {
		include: ['@tauri-apps/api']
	},
	// Configure resolve aliases
	resolve: {
		alias: {
			// Use import.meta.url for robust path resolution
			'@': path.resolve(path.dirname(fileURLToPath(import.meta.url)), './src'),
			'$lib': path.resolve(path.dirname(fileURLToPath(import.meta.url)), './src/lib'), // Ensure $lib alias is here
			'$features': path.resolve(path.dirname(fileURLToPath(import.meta.url)), './src/features') // Add $features alias
		}
	}
});
