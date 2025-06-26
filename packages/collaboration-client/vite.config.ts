import { resolve } from 'path';
import { defineConfig } from 'vite';
import dts from 'vite-plugin-dts';

// https://vitejs.dev/config/
export default defineConfig(({ command }) => {
  if (command === 'serve') {
    // Dev server config
    return {
      root: 'example',
      server: {
        port: 3000 //
      }
    };
  } else {
    // Build config
    return {
      build: {
        lib: {
          entry: resolve(__dirname, 'src/index.ts'),
          name: 'CollaborationClient',
          fileName: 'index',
          formats: ['es', 'cjs', 'umd'],
        },
        rollupOptions: {
          external: ['yjs'],
          output: {
            globals: {
              yjs: 'Y',
            },
          },
        },
      },
      plugins: [dts()],
    };
  }
});