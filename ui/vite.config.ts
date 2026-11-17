/// <reference types="vitest/config" />
import react from '@vitejs/plugin-react'
import { defineConfig } from 'vite'

export default defineConfig({
  plugins: [react()],
  // Tauri expects a fixed port and shows Rust errors in the terminal.
  clearScreen: false,
  server: { port: 5173, strictPort: true },
  // Top-level await in main.tsx (dev preview); Tauri's webviews support it.
  build: { target: 'esnext' },
  test: { environment: 'jsdom', setupFiles: ['./src/test-setup.ts'] },
})
