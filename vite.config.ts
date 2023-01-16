import { defineConfig } from 'vite'
import wasmPack from 'vite-plugin-wasm-pack';

export default defineConfig({
  build: {
    lib: {
      entry: './searcher/lib/request.ts',
      name: 'Request',
      fileName: 'request'
    }
  },
  plugins: [wasmPack(['./searcher'])]
})
