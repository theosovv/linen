import { defineConfig } from 'vite';
import { resolve } from 'path';

export default defineConfig({
  root: 'playground',
  plugins: [
    // Здесь будет наш custom plugin для фреймворка
  ],
  resolve: {
    alias: {
      'linen': resolve(__dirname, 'src/index.ts'),
    }
  },
  server: {
    port: 3000,
    open: true
  }
});