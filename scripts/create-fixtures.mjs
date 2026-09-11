import sharp from 'sharp';
import { mkdir, writeFile, cp } from 'node:fs/promises';
import { execFileSync } from 'node:child_process';
import { resolve, join } from 'node:path';
const root = resolve(import.meta.dirname, '..');
const directory = join(root, 'artifacts/fixtures');
const review = join(root, 'public/review-images');
await mkdir(directory, { recursive: true });
await mkdir(review, { recursive: true });
const runtime = join(root, 'src-tauri/resources/runtime');
const windows = process.platform === 'win32';
const binary = join(
  runtime,
  windows ? 'exiftool/bin/exiftool.exe' : 'perl/bin/perl',
);
const prefix = windows ? [] : [join(runtime, 'exiftool/bin/exiftool')];
const run = (args) =>
  execFileSync(binary, [...prefix, '-config', '', ...args], {
    encoding: 'utf8',
    maxBuffer: 16 * 1024 * 1024,
  });
const locations = ['Alpine Licht', 'Dolomiten', 'Stillwasser', 'Morgennebel'];
for (let index = 0; index < 14; index++) {
  const name = `${locations[index % locations.length]}_${String(index + 1).padStart(3, '0')}${index === 0 ? ' – Österreich' : ''}.jpg`;
  const tint = ['#ddc7a8', '#c8d6cb', '#abc4c4', '#bbc0c6'][index % 4];
  const sky = ['#7e9eab', '#8caaa3', '#719aaa', '#959ba8'][index % 4];
  const image = `<svg xmlns="http://www.w3.org/2000/svg" width="1800" height="1200" viewBox="0 0 1800 1200"><defs><linearGradient id="sky" x2="0" y2="1"><stop stop-color="${sky}"/><stop offset="1" stop-color="${tint}"/></linearGradient><linearGradient id="peak" x1="0" y1="0" x2="0" y2="1"><stop stop-color="#6e7f7d"/><stop offset="1" stop-color="#b9b3a0"/></linearGradient><linearGradient id="front" x2="0" y2="1"><stop stop-color="#435553"/><stop offset="1" stop-color="#233a37"/></linearGradient><filter id="grain"><feTurbulence type="fractalNoise" baseFrequency=".6" numOctaves="3" stitchTiles="stitch"/><feColorMatrix type="saturate" values="0"/><feComponentTransfer><feFuncA type="linear" slope=".06"/></feComponentTransfer><feBlend mode="soft-light" in2="SourceGraphic"/></filter><linearGradient id="haze" x2="0" y2="1"><stop stop-color="${tint}" stop-opacity="0"/><stop offset="1" stop-color="${tint}" stop-opacity=".55"/></linearGradient></defs><rect width="1800" height="1200" fill="url(#sky)"/><circle cx="${1350 - index * 27}" cy="280" r="64" fill="#f8e9c8" opacity=".55"/><path d="M0 745L117 604L181 640L333 375L405 479L457 442L622 639L765 461L915 512L1038 355L1080 401L1118 368L1372 659L1480 509L1580 551L1710 456L1800 580V1200H0Z" fill="#8a9892"/><path d="M0 993L132 808L248 784L408 532L495 694L574 663L684 784L817 554L906 596L1112 242L1177 347L1220 323L1360 625L1490 506L1559 567L1612 518L1800 835V1200H0Z" fill="url(#peak)"/><path d="M1112 242L1081 390L1099 423L1044 486L1090 476L1116 396L1177 347L1134 371Z" fill="#e5e1cf"/><path d="M1112 242L1220 323L1172 394L1245 446L1264 493L1134 371Z" fill="#d3d6cd"/><path d="M408 532L398 627L359 652L355 711L427 656L458 661Z M817 554L781 683L839 651L866 669L845 612Z" fill="#d9dcce" opacity=".8"/><path d="M1112 430L1162 634L1037 865L1131 644Z M1220 475L1300 708L1200 994L1280 713Z M408 698L445 773L314 1012Z" fill="#495b59" opacity=".4"/><path d="M0 1055Q171 884 309 952T619 1006Q754 979 921 838Q1070 699 1291 871T1800 955V1200H0Z" fill="#56716b"/><rect y="660" width="1800" height="540" fill="url(#haze)"/><path d="M0 1200V954L117 870L248 939L311 913L437 1002L591 1109L672 1057L786 1160L882 1200Z" fill="url(#front)"/><path d="M1189 1200L1330 1100L1455 1076L1571 964L1697 1022L1800 957V1200Z" fill="#344d43"/><rect width="1800" height="1200" filter="url(#grain)" opacity=".25"/></svg>`;
  const path = join(directory, name);
  await sharp(Buffer.from(image))
    .withIccProfile('srgb')
    .jpeg({ quality: 91 })
    .toFile(path);
  run([
    '-overwrite_original',
    '-Make=FUJIFILM',
    '-Model=X-T5',
    '-LensModel=XF16-55mmF2.8 R LM WR',
    `-FNumber=${index % 3 === 0 ? 8 : 5.6}`,
    '-ExposureTime=1/250',
    `-ISO=${index % 2 ? 200 : 125}`,
    '-FocalLength=35',
    '-DateTimeOriginal=2026:09:06 06:42:18',
    '-OffsetTimeOriginal=+02:00',
    `-XMP-dc:Title=${index ? locations[index % 4] : 'Erstes Licht am Großglockner'}`,
    '-XMP-dc:Description=Synthetic landscape fixture for Tagryn UI and metadata verification.',
    '-XMP-dc:Creator=Studio Nord',
    '-IFD0:Artist=Studio Nord',
    '-IPTC:By-line=Studio Nord',
    '-XMP-dc:Rights=© 2026 Studio Nord · Test fixture',
    `-XMP-xmp:Rating=${index % 6}`,
    '-XMP-dc:Subject=Alpen',
    '-XMP-dc:Subject+=Morgenlicht',
    '-XMP-dc:Subject+=Landschaft',
    ...(index % 3 === 0
      ? [
          '-GPSLatitude=47.0745',
          '-GPSLatitudeRef=N',
          '-GPSLongitude=12.6947',
          '-GPSLongitudeRef=E',
          '-XMP-exif:GPSLatitude#=47.0745',
          '-XMP-exif:GPSLongitude#=12.6947',
        ]
      : []),
    path,
  ]);
  await cp(path, join(review, name));
}
const sample = join(directory, 'Alpine Licht_001 – Österreich.jpg');
await sharp(sample).png().toFile(join(directory, 'Farbprobe.png'));
await sharp(sample).webp().toFile(join(directory, 'Web-Fassung.webp'));
await sharp(sample)
  .resize(6000, 4000)
  .tiff({ compression: 'lzw' })
  .toFile(join(directory, '24MP-Studiotest.tiff'));
await writeFile(
  join(directory, 'Beschädigt.jpg'),
  'This is intentionally not an image. Tagryn must show a read error.',
);
console.log(
  `Generated synthetic fixtures (not camera-original benchmarks): ${directory}`,
);
