import { defineConfig } from 'vitest/config';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
  plugins: [svelte({ hot: !process.env.VITEST })],
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: ['./tests/setup.ts'],
    include: ['./tests/**/*.test.ts'],
    deps: {
      inline: [/svelte/]
    },
    coverage: {
      provider: 'c8',
      reporter: ['text', 'json', 'html'],
      include: ['src/**/*.{ts,js,svelte}', 'src-tauri/src/**/*.rs']
    }
  },
  resolve: {
    alias: {
      '@': '/src',
      '@tauri': '/src-tauri'
    }
  }
}); 