import { chromium } from '@playwright/test';
import { mkdir, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';

const out = resolve(import.meta.dirname, '../artifacts/screenshots');
await mkdir(out, { recursive: true });
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1440, height: 960 } });
const errors = [];
page.on('pageerror', (error) => errors.push(error.message));
const started = performance.now();
await page.goto('http://127.0.0.1:1420/?review=large');
await page
  .getByRole('option', { name: 'Scan_00000.jpg', exact: true })
  .waitFor();
const readyMs = performance.now() - started;
const measurement = await page.evaluate(async () => {
  const scroll = document.querySelector('.files-scroll');
  const totalHeight = scroll.scrollHeight;
  const frames = [];
  let last = performance.now();
  let maxRendered = 0;
  for (let i = 0; i < 100; i++) {
    await new Promise(requestAnimationFrame);
    const now = performance.now();
    frames.push(now - last);
    last = now;
    scroll.scrollTop = ((totalHeight - scroll.clientHeight) * i) / 99;
    maxRendered = Math.max(
      maxRendered,
      document.querySelectorAll('.file-item').length,
    );
  }
  await new Promise(requestAnimationFrame);
  frames.sort((a, b) => a - b);
  return {
    totalHeight,
    maxRendered,
    frameMedianMs: frames[50],
    frameP95Ms: frames[95],
    frameMaxMs: frames.at(-1),
    endVisible: document
      .querySelector('.files-scroll')
      .textContent.includes('Scan_09999.jpg'),
  };
});
await page.screenshot({ path: resolve(out, '10000-files-light.png') });
await page.setViewportSize({ width: 900, height: 620 });
await page.getByRole('button', { name: 'Einstellungen', exact: true }).click();
await page.getByRole('button', { name: 'Dunkel', exact: true }).click();
await page.getByRole('button', { name: 'Fertig', exact: true }).click();
const overflow = await page.evaluate(
  () => document.body.scrollWidth > innerWidth,
);
await page.screenshot({ path: resolve(out, '10000-files-900-dark.png') });
const output = {
  context:
    'Isolated Chromium browser; real 10,000-entry Rust scan snapshot; not a native WebKit frame benchmark',
  readyMs,
  ...measurement,
  overflowAt900: overflow,
  pageErrors: errors,
};
await writeFile(
  resolve(out, '10000-files.json'),
  JSON.stringify(output, null, 2),
);
await browser.close();
if (
  errors.length ||
  overflow ||
  !measurement.endVisible ||
  measurement.maxRendered > 100
)
  throw new Error(JSON.stringify(output));
console.log(JSON.stringify(output));
