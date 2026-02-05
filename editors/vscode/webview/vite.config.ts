import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { resolve } from 'path';

export default defineConfig({
  plugins: [react()],
  root: resolve(__dirname, 'src'),
  build: {
    outDir: resolve(__dirname, 'dist'),
    emptyOutDir: true,
    rollupOptions: {
      input: resolve(__dirname, 'src/index.html'),
      output: {
        entryFileNames: 'index.js',
        assetFileNames: '[name].[ext]',
        chunkFileNames: '[name].js',
      },
    },
    sourcemap: true,
  },
  define: {
    'process.env': {},
  },
});
