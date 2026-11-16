import {defineConfig} from 'vite';
import react from '@vitejs/plugin-react';
import {astryxStylex} from '@astryxdesign/build/vite';
import path from 'node:path';
import {fileURLToPath} from 'node:url';

const root = path.dirname(fileURLToPath(import.meta.url));

export default defineConfig({
  base: './',
  plugins: [
    ...astryxStylex({
      stylexOptions: {
        dev: process.env.NODE_ENV !== 'production',
        runtimeInjection: false,
        treeshakeCompensation: true,
        classNamePrefix: 'p',
        aliases: {
          '@astryxdesign/core/theme/tokens.stylex': [
            path.join(root, 'node_modules/@astryxdesign/core/src/theme/tokens.stylex.ts'),
          ],
        },
        unstable_moduleResolution: {type: 'commonJS'},
      },
      stylexPrefix: 'x',
    }),
    react(),
  ],
});
