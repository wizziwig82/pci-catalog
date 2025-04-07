import { vi } from 'vitest';

// Mock SvelteKit runtime module $app/environment FIRST
vi.mock('$app/environment', () => ({
	browser: true, // Simulate browser environment for onMount
	dev: false,
	building: false,
	version: 'test',
}));

import '@testing-library/jest-dom/vitest';

// required for svelte5 + jsdom as jsdom does not support matchMedia
// Check if window is defined before attempting to modify it
if (typeof window !== 'undefined') {
	Object.defineProperty(window, 'matchMedia', {
		writable: true,
		enumerable: true,
		value: vi.fn().mockImplementation((query) => ({
			matches: false,
			media: query,
			onchange: null,
			addEventListener: vi.fn(),
			removeEventListener: vi.fn(),
			dispatchEvent: vi.fn()
		}))
	});
} else {
	console.warn('Vitest setup: window object not found, skipping matchMedia mock.');
}

// add more mocks here if you need them
