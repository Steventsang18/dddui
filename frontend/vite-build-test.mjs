import { build } from 'vite';
try {
  await build({ configFile: 'vite.config.ts', logLevel: 'info' });
  console.log('BUILD SUCCESS');
} catch(e) {
  console.error('BUILD ERROR:', e.message);
  console.error(e.stack?.slice(0, 500));
}
