import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

// Vite config for the ModelGate dashboard SPA.
//
// `base: './'` lets the bundle run at any mount path — the
// modelgate-web Rust crate embeds `dist/` and serves it at `/`, but
// `base: './'` keeps the assets portable if that ever changes.
//
// The dev proxy forwards /api/* to the Rust server on 9378 so `npm run
// dev` + `smctl gate web` can run side by side during frontend work.
export default defineConfig({
  plugins: [react()],
  base: './',
  server: {
    port: 5173,
    proxy: {
      '/api': {
        target: 'http://127.0.0.1:9378',
        changeOrigin: false,
        ws: false,
      },
    },
  },
  build: {
    outDir: 'dist',
    sourcemap: true,
    // Hard budget: per-asset warning at 500 KB, total budget per the
    // web-server spec is 2 MB. CI can enforce the 2 MB total later.
    chunkSizeWarningLimit: 512,
  },
});
