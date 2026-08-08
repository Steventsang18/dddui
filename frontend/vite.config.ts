import { readFileSync } from 'fs';
import { resolve } from 'path';
import UnoCSS from 'unocss/vite';
import unoConfig from './uno.config.ts';
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { viteStaticCopy } from 'vite-plugin-static-copy';

const rootPackageJson = JSON.parse(readFileSync(resolve(__dirname, 'package.json'), 'utf-8')) as {
  version: string;
};

const rendererRoot = resolve('packages/desktop/src/renderer');

// Icon Park transform plugin (replaces webpack icon-park-loader)
function iconParkPlugin() {
  return {
    name: 'vite-plugin-icon-park',
    enforce: 'pre' as const,
    transform(source: string, id: string) {
      if (!id.endsWith('.tsx') || id.includes('node_modules')) return null;
      if (!source.includes('@icon-park/react')) return null;
      const transformedSource = source.replace(
        /import\s+\{\s+([a-zA-Z, ]*)\s+\}\s+from\s+['"]@icon-park\/react['"](;?)/g,
        function (str, match) {
          if (!match) return str;
          const components = match.split(',');
          const importComponent = str.replace(
            match,
            components.map((key: string) => `${key} as _${key.trim()}`).join(', ')
          );
          const hoc = `import IconParkHOC from '@renderer/components/IconParkHOC';
          ${components.map((key: string) => `const ${key.trim()} = IconParkHOC(_${key.trim()})`).join(';\n')}`;
          return importComponent + ';' + hoc;
        }
      );
      if (transformedSource !== source) return { code: transformedSource, map: null } as { code: string; map: null };
      return null;
    },
  };
}

export default defineConfig(({ mode }) => {
  const isDevelopment = mode === 'development';

  return {
    root: rendererRoot,
    base: './',
    publicDir: resolve('public'),
    appType: 'mpa',
    server: {
      port: 5173,
      // Proxy API and WebSocket to the Rust backend
      proxy: {
        '/api': {
          target: 'http://127.0.0.1:3080',
          changeOrigin: true,
        },
        '/ws': {
          target: 'ws://127.0.0.1:3080',
          ws: true,
        },
      },
    },
    resolve: {
      alias: {
        '@': resolve('packages/desktop/src'),
        '@common': resolve('packages/desktop/src/common'),
        '@renderer': rendererRoot,
        // Force ESM version of streamdown
        streamdown: resolve('node_modules/streamdown/dist/index.js'),
      },
      extensions: ['.ts', '.tsx', '.js', '.jsx', '.css'],
      dedupe: [
        'react',
        'react-dom',
        'react-router-dom',
        '@codemirror/state',
        '@codemirror/view',
        '@codemirror/language',
        '@lezer/highlight',
      ],
    },
    plugins: [
      react(),
      UnoCSS(unoConfig),
      iconParkPlugin(),
      ...(!isDevelopment
        ? [
        viteStaticCopy({
          structured: false,
          targets: [
            {
              src: 'assets/logos/**',
              dest: 'static/images',
            },
          ],
        }),
          ]
        : []),
    ],
    build: {
      target: 'es2022',
      sourcemap: false,
      minify: !isDevelopment,
      reportCompressedSize: false,
      chunkSizeWarningLimit: 1500,
      cssCodeSplit: true,
      rollupOptions: {
        input: {
          index: resolve(rendererRoot, 'index.html'),
          pet: resolve(rendererRoot, 'pet/pet.html'),
          'pet-hit': resolve(rendererRoot, 'pet/pet-hit.html'),
          'pet-confirm': resolve(rendererRoot, 'pet/pet-confirm.html'),
        },
        external: ['node:crypto', 'crypto'],
        onwarn(warning, warn) {
          if (warning.code === 'EVAL') return;
          warn(warning);
        },
        output: {
          manualChunks(id: string) {
            if (!id.includes('node_modules')) return undefined;
            // Merge highlight + markdown (circular: highlight ↔ markdown)
            if (
              id.includes('/react-syntax-highlighter/') ||
              id.includes('/refractor/') ||
              id.includes('/highlight.js/') ||
              id.includes('/react-markdown/') ||
              id.includes('/remark-') ||
              id.includes('/rehype-') ||
              id.includes('/unified/') ||
              id.includes('/mdast-') ||
              id.includes('/hast-') ||
              id.includes('/micromark')
            )
              return 'vendor-markdown';
            if (
              id.includes('/monaco-editor/') ||
              id.includes('/@monaco-editor/') ||
              id.includes('/codemirror/') ||
              id.includes('/@codemirror/')
            )
              return 'vendor-editor';
            if (id.includes('/@arco-design/'))
              return 'vendor-arco';
            if (id.includes('/react-dom/') || id.includes('/react/'))
              return 'vendor-react';
            if (id.includes('/katex/')) return 'vendor-katex';
            if (id.includes('/@icon-park/')) return 'vendor-icons';
            if (id.includes('/diff2html/')) return 'vendor-diff';
            return undefined;
          },
        },
      },
    },
    define: {
      'process.env.NODE_ENV': JSON.stringify(mode),
      __APP_VERSION__: JSON.stringify(rootPackageJson.version),
      global: 'globalThis',
    },
    optimizeDeps: {
      include: [
        'react',
        'react-dom',
        'react-router-dom',
        'react-i18next',
        'i18next',
        '@arco-design/web-react',
        '@icon-park/react',
        'react-markdown',
        'react-syntax-highlighter',
        'react-virtuoso',
        'classnames',
        'swr',
        'eventemitter3',
        'katex',
        'diff2html',
        'remark-gfm',
        'remark-math',
        'remark-breaks',
        'rehype-raw',
        'rehype-katex',
        '@uiw/react-codemirror',
        '@codemirror/lang-markdown',
        '@codemirror/language',
      ],
    },
  };
});
