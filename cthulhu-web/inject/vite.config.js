import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import path from 'path'
// https://vitejs.dev/config/
export default defineConfig({
  plugins: [vue()],
  base: 'https://web.cthulhu.server/inject/',
  build: {
    outDir: 'dist/',
    assetsDir: 'assets',
    manifest: false,
    resolve: {
      alias: {
        "/@": path.resolve(__dirname, "./src"),
      },
    },
    rollupOptions: {
      input: {
        inject: '/src/main.js'
      },
      output: {
        entryFileNames: `[name].js`,
        chunkFileNames: `[name].js`,
        assetFileNames: `[name].[ext]`,
      }
    }
  },
})
