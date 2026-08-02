import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import path from 'path'

// https://vite.dev/config/
export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  base: process.env.VITE_BASE_URL || '/',
  build: {
    outDir: '../docs',
    emptyOutDir: true
  },
  define: {
    __COMMIT_HASH__: JSON.stringify(process.env.VITE_COMMIT_HASH || '')
  }
})
