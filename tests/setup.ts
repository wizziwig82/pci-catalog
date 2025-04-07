import '@testing-library/jest-dom';
import { vi } from 'vitest';
import { cleanup } from '@testing-library/svelte';

// Removed global Tauri API mock (mock safeInvoke in individual tests instead)
// Removed global afterEach(cleanup) (handle in individual tests or client setup)

// Setup global variables that might be used in tests
global.ResizeObserver = vi.fn().mockImplementation(() => ({
  observe: vi.fn(),
  unobserve: vi.fn(),
  disconnect: vi.fn(),
}));

// Set up any other global mocks or configurations
// ... 