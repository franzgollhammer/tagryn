import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import sharp from 'sharp';
test('bundle uses Tagryn branding and valid platform icon containers', async () => {
  const config = JSON.parse(
    await readFile(new URL('../src-tauri/tauri.conf.json', import.meta.url)),
  );
  assert.equal(config.productName, 'Tagryn');
  assert.ok(config.bundle.icon.includes('icons/icon.icns'));
  assert.ok(config.bundle.icon.includes('icons/icon.ico'));
  const icns = await readFile(
    new URL('../src-tauri/icons/icon.icns', import.meta.url),
  );
  assert.equal(icns.subarray(0, 4).toString(), 'icns');
  assert.equal(icns.readUInt32BE(4), icns.length);
  const ico = await readFile(
    new URL('../src-tauri/icons/icon.ico', import.meta.url),
  );
  assert.equal(ico.readUInt16LE(2), 1);
  assert.equal(ico.readUInt16LE(4), 7);
  for (const size of [16, 32, 128, 256, 512, 1024]) {
    const meta = await sharp(
      await readFile(
        new URL(`../src-tauri/icons/${size}x${size}.png`, import.meta.url),
      ),
    ).metadata();
    assert.equal(meta.width, size);
    assert.equal(meta.channels, 4);
  }
});
