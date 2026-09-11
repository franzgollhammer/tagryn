import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { readFile, readdir, open, stat, writeFile } from 'node:fs/promises';
import { resolve, join, relative } from 'node:path';

if (process.platform !== 'darwin')
  throw new Error('This focused bundle check requires macOS.');
const root = resolve(import.meta.dirname, '..');
const app = join(root, 'src-tauri/target/release/bundle/macos/Tagryn.app');
const resources = join(app, 'Contents/Resources');
const run = (bin, args) =>
  execFileSync(bin, args, { encoding: 'utf8', maxBuffer: 4 * 1024 * 1024 });
const plist = JSON.parse(
  run('/usr/bin/plutil', [
    '-convert',
    'json',
    '-o',
    '-',
    join(app, 'Contents/Info.plist'),
  ]),
);
if (
  plist.CFBundleIdentifier !== 'app.tagryn.desktop' ||
  plist.CFBundleName !== 'Tagryn' ||
  plist.CFBundleIconFile !== 'icon.icns'
)
  throw new Error('Bundle branding mismatch');
const executables = await readdir(join(app, 'Contents/MacOS'));
if (executables.length !== 1 || executables[0] !== 'tagryn')
  throw new Error(`Unexpected product binaries: ${executables}`);
const hash = async (path) =>
  createHash('sha256')
    .update(await readFile(path))
    .digest('hex');
const iconHash = await hash(join(resources, 'icon.icns'));
if (iconHash !== (await hash(join(root, 'src-tauri/icons/icon.icns'))))
  throw new Error('Bundled icon differs from the final custom icon');
let bytes = 0;
const nativeFiles = [];
async function inspect(directory) {
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) {
      await inspect(path);
      continue;
    }
    if (!entry.isFile()) continue;
    bytes += (await stat(path)).size;
    const file = await open(path);
    const header = Buffer.alloc(4);
    await file.read(header, 0, 4, 0);
    await file.close();
    if (
      ['cffaedfe', 'cefaedfe', 'feedfacf', 'feedface', 'cafebabe'].includes(
        header.toString('hex'),
      )
    )
      nativeFiles.push(path);
  }
}
await inspect(app);
const deployment = [];
for (const path of nativeFiles) {
  const nonSystemLibraries = run('/usr/bin/otool', ['-L', path])
    .split('\n')
    .slice(1)
    .map((line) => line.trim())
    .filter(
      (line) =>
        line.startsWith('/') &&
        !line.startsWith('/System/') &&
        !line.startsWith('/usr/lib/'),
    );
  if (nonSystemLibraries.length)
    throw new Error(
      `Unbundled absolute library dependency in ${relative(app, path)}: ${nonSystemLibraries.join(', ')}`,
    );
  const output = run('/usr/bin/otool', ['-l', path]);
  const version =
    output.match(/\bminos\s+([\d.]+)/)?.[1] ??
    output.match(/LC_VERSION_MIN_MACOSX[\s\S]*?version ([\d.]+)/)?.[1];
  if (
    !version ||
    Number(version.split('.')[0]) > 13 ||
    (version.startsWith('13.') && Number(version.split('.')[1]) > 3)
  )
    throw new Error(
      `Deployment target exceeds macOS 13.3: ${relative(app, path)} (${version})`,
    );
  deployment.push({ file: relative(app, path), minimumMacOS: version });
  if (process.argv.includes('--adhoc-sign'))
    run('/usr/bin/codesign', [
      '--force',
      '--sign',
      '-',
      '--timestamp=none',
      path,
    ]);
}
if (process.argv.includes('--adhoc-sign'))
  run('/usr/bin/codesign', ['--force', '--sign', '-', '--timestamp=none', app]);
run('/usr/bin/codesign', ['--verify', '--deep', '--strict', app]);
const perl = join(resources, 'runtime/perl/bin/perl');
const exiftool = join(resources, 'runtime/exiftool/bin/exiftool');
const version = execFileSync(perl, [exiftool, '-ver'], {
  encoding: 'utf8',
  env: {
    ...process.env,
    PATH: '/nonexistent',
    PERL5LIB: '',
    PERL5OPT: '',
    PERLLIB: '',
  },
}).trim();
if (version !== '13.59') throw new Error('Bundled engine version mismatch');
const result = {
  result: 'passed',
  bundle: 'Tagryn.app',
  architecture: process.arch,
  identifier: plist.CFBundleIdentifier,
  minimumMacOS: plist.LSMinimumSystemVersion,
  executables,
  iconSha256: iconHash,
  totalFileBytes: bytes,
  exiftool: version,
  nonSystemAbsoluteLibraryDependencies: 0,
  nativeFiles: deployment,
  signing:
    'Local ad-hoc signature verified; no Developer ID and no notarization',
};
await writeFile(
  join(root, 'artifacts/bundle-check.json'),
  JSON.stringify(result, null, 2),
);
console.log(
  JSON.stringify({
    ...result,
    nativeFiles: `${nativeFiles.length} Mach-O files checked`,
  }),
);
