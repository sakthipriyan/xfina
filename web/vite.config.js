import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import path from 'path'
import { execSync } from 'child_process'

let commitHash = '';
try {
  commitHash = execSync('git rev-parse --short HEAD').toString().trim();
  const isDirty = execSync('git status --porcelain').toString().trim().length > 0;
  if (isDirty) commitHash += '*';
} catch (e) {
  // Ignore if git fails
}

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
    __COMMIT_HASH__: JSON.stringify(process.env.VITE_COMMIT_HASH || commitHash)
  }
})
