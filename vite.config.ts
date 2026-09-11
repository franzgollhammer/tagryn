import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';
import tailwindcss from '@tailwindcss/vite';
import { readFile } from 'node:fs/promises';
import { resolve, basename } from 'node:path';

export default defineConfig({
  publicDir: 'public/app',
  plugins: [
    vue(),
    tailwindcss(),
    {
      name: 'local-review-fixtures',
      configureServer(server) {
        server.middlewares.use(async (req, res, next) => {
          const url = new URL(req.url ?? '/', 'http://localhost');
          const session = [
            '/review-session.json',
            '/review-large.json',
          ].includes(url.pathname);
          if (!session && !url.pathname.startsWith('/review-images/'))
            return next();
          try {
            const name = decodeURIComponent(
              url.pathname.slice('/review-images/'.length),
            );
            if (
              !session &&
              (basename(name) !== name || !name.endsWith('.jpg'))
            ) {
              res.statusCode = 400;
              res.end();
              return;
            }
            const file = session
              ? resolve('public', basename(url.pathname))
              : resolve('public/review-images', name);
            res.setHeader(
              'Content-Type',
              url.pathname.endsWith('.json')
                ? 'application/json'
                : 'image/jpeg',
            );
            res.end(await readFile(file));
          } catch {
            res.statusCode = 404;
            res.end();
          }
        });
      },
    },
  ],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: '127.0.0.1',
    watch: { ignored: ['**/src-tauri/**', '**/.build/**', '**/artifacts/**'] },
  },
  build: { target: ['es2022', 'safari16.4'] },
});
