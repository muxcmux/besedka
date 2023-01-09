import { defineConfig } from "vite"
/** @type {import('vite').UserConfig} */

export default defineConfig({
  build: {
    rollupOptions: {
      input: {
        app: './src/main.ts',
        defaultTheme: './themes/default.css'
      }
    }
  }
})
