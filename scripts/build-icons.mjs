import sharp from 'sharp';
import { mkdir, readFile, writeFile, cp } from 'node:fs/promises';
import { resolve, join } from 'node:path';
const root = resolve(import.meta.dirname, '..');
const out = join(root, 'src-tauri/icons');
await mkdir(out, { recursive: true });
await mkdir(join(root, 'public/app/brand'), { recursive: true });
const master = await readFile(join(root, 'assets/brand/tagryn.svg'));
const compact = await readFile(join(root, 'assets/brand/tagryn-small.svg'));
const sizes = [16, 24, 32, 48, 64, 128, 256, 512, 1024];
const buffers = new Map();
for (const size of sizes) {
  const png = await sharp(size <= 32 ? compact : master)
    .resize(size, size)
    .ensureAlpha()
    .png()
    .toBuffer();
  buffers.set(size, png);
  await writeFile(join(out, `${size}x${size}.png`), png);
  await writeFile(join(root, `public/app/brand/${size}.png`), png);
}
await writeFile(join(out, '128x128@2x.png'), buffers.get(256));
await writeFile(join(out, 'icon.png'), buffers.get(1024));
const icoSizes = [16, 24, 32, 48, 64, 128, 256];
const header = Buffer.alloc(6 + icoSizes.length * 16);
header.writeUInt16LE(1, 2);
header.writeUInt16LE(icoSizes.length, 4);
let offset = header.length;
icoSizes.forEach((size, i) => {
  const png = buffers.get(size);
  const p = 6 + i * 16;
  header[p] = size === 256 ? 0 : size;
  header[p + 1] = header[p];
  header.writeUInt16LE(1, p + 4);
  header.writeUInt16LE(32, p + 6);
  header.writeUInt32LE(png.length, p + 8);
  header.writeUInt32LE(offset, p + 12);
  offset += png.length;
});
await writeFile(
  join(out, 'icon.ico'),
  Buffer.concat([header, ...icoSizes.map((s) => buffers.get(s))]),
);
const chunks = [
  ['icp4', 16],
  ['icp5', 32],
  ['icp6', 64],
  ['ic07', 128],
  ['ic08', 256],
  ['ic09', 512],
  ['ic10', 1024],
].map(([type, size]) => {
  const data = buffers.get(size);
  const h = Buffer.alloc(8);
  h.write(type);
  h.writeUInt32BE(data.length + 8, 4);
  return Buffer.concat([h, data]);
});
const icnsHeader = Buffer.alloc(8);
icnsHeader.write('icns');
icnsHeader.writeUInt32BE(
  8 + chunks.reduce((sum, chunk) => sum + chunk.length, 0),
  4,
);
await writeFile(join(out, 'icon.icns'), Buffer.concat([icnsHeader, ...chunks]));
await writeFile(join(root, 'public/app/tagryn.svg'), master);
for (const name of [
  'tagryn.svg',
  'tagryn-small.svg',
  'study-b-layers.svg',
  'study-c-detail.svg',
  'wordmark.svg',
])
  await cp(
    join(root, 'assets/brand', name),
    join(root, 'public/app/brand', name),
  );
const svgData = (name) =>
  `data:image/svg+xml;base64,${Buffer.from(name).toString('base64')}`;
const studies = [
  master,
  await readFile(join(root, 'assets/brand/study-b-layers.svg')),
  await readFile(join(root, 'assets/brand/study-c-detail.svg')),
];
const sheet = `<svg xmlns="http://www.w3.org/2000/svg" width="1440" height="1000" viewBox="0 0 1440 1000"><rect width="1440" height="1000" fill="#F4F3EF"/><text x="64" y="70" fill="#20312B" font-family="Helvetica,Arial" font-size="28" font-weight="700">Tagryn</text><text x="64" y="106" fill="#647269" font-family="Helvetica,Arial" font-size="16">Identity studies · Image meets information</text>${studies.map((s, i) => `<image href="${svgData(s)}" x="${70 + i * 456}" y="145" width="210" height="210"/><text x="${80 + i * 456}" y="394" fill="#20312B" font-family="Helvetica,Arial" font-size="18">${['01  Detail frame · Selected', '02  Metadata layers', '03  Split view'][i]}</text>`).join('')}<path d="M64 432H1376" stroke="#D5DAD3"/><rect x="0" y="470" width="720" height="530" fill="#F4F3EF"/><rect x="720" y="470" width="720" height="530" fill="#141B19"/><image href="${svgData(master)}" x="172" y="500" width="350" height="350"/><image href="${svgData(master)}" x="892" y="500" width="350" height="350"/>${[0, 720].map((x) => [16, 32, 48, 64].map((s, i) => `<image href="${svgData(s <= 32 ? compact : master)}" x="${x + 204 + i * 70}" y="888" width="${s}" height="${s}"/>`).join('')).join('')}</svg>`;
await writeFile(join(root, 'assets/brand/specimen.svg'), sheet);
await sharp(Buffer.from(sheet))
  .png()
  .toFile(join(root, 'assets/brand/specimen.png'));
console.log('Tagryn SVG, PNG, ICO, ICNS and specimen assets generated.');
