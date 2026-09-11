import { createHash } from 'node:crypto';
import {
  mkdir,
  writeFile,
  readFile,
  cp,
  stat,
  chmod,
  readdir,
  mkdtemp,
} from 'node:fs/promises';
import { spawn } from 'node:child_process';
import { resolve, join } from 'node:path';
import { availableParallelism } from 'node:os';

const root = resolve(import.meta.dirname, '..');
const build = join(root, '.build');
const runtime = join(root, 'src-tauri/resources/runtime');
await mkdir(build, { recursive: true });
await mkdir(runtime, { recursive: true });
const windows = process.platform === 'win32';
const deploymentTarget = process.platform === 'darwin' ? '13.3' : null;
const run = (cmd, args, cwd = root) =>
  new Promise((ok, fail) => {
    const child = spawn(cmd, args, {
      cwd,
      stdio: 'inherit',
      shell: false,
      env: {
        ...process.env,
        ...(deploymentTarget
          ? { MACOSX_DEPLOYMENT_TARGET: deploymentTarget }
          : {}),
      },
    });
    child.on('error', fail);
    child.on('exit', (code) =>
      code === 0 ? ok() : fail(new Error(`${cmd} exited ${code}`)),
    );
  });
async function download(url, dest, algorithm, expected) {
  let data;
  try {
    data = await readFile(dest);
  } catch {
    const response = await fetch(url);
    if (!response.ok)
      throw new Error(`Download failed: ${response.status} ${url}`);
    data = Buffer.from(await response.arrayBuffer());
  }
  const actual = createHash(algorithm)
    .update(data)
    .digest(algorithm === 'sha512' ? 'base64' : 'hex');
  if (actual !== expected) throw new Error(`Integrity mismatch for ${url}`);
  await writeFile(dest, data);
}
const pkg = windows ? 'exiftool-vendored.exe' : 'exiftool-vendored.pl';
const exifSha = windows
  ? 'U0l0CApIGO+AjpFYCiNCApQsMdlf6sqCyEl4wlKKqvwVAEVcvMRw91k7zj32OPFZE+/+XlLFsc6876MpOqBLiw=='
  : '5B80JT9phZSUsc7zz77rQ6DfW0YssTpJ5qW/1bOH1N8XQq/jcn+9eRW9HFwhVT9uoIliw9Fg007ST3pUzQIzyw==';
const exifArchive = join(build, `${pkg}-13.59.2.tgz`);
await download(
  `https://registry.npmjs.org/${pkg}/-/${pkg}-13.59.2.tgz`,
  exifArchive,
  'sha512',
  exifSha,
);
const exifDir = join(runtime, 'exiftool');
await mkdir(exifDir, { recursive: true });
await run('tar', ['-xzf', exifArchive, '--strip-components=1', '-C', exifDir]);
if (!windows) {
  const perlBin = join(runtime, 'perl/bin/perl');
  let built = false;
  try {
    const manifest = JSON.parse(
      await readFile(join(runtime, 'manifest.json'), 'utf8'),
    );
    built =
      (await stat(perlBin)).isFile() &&
      manifest.platform === process.platform &&
      manifest.arch === process.arch &&
      manifest.perl === '5.44.0' &&
      manifest.runtimeRevision === 2 &&
      manifest.deploymentTarget === deploymentTarget;
  } catch {
    /* First build. */
  }
  if (!built) {
    const archive = join(build, 'perl-5.44.0.tar.gz');
    await download(
      'https://www.cpan.org/src/5.0/perl-5.44.0.tar.gz',
      archive,
      'sha256',
      '3b855066b92491cb40e86affb1ca57d1a388aa43e51b91c7806a32c2f65f96c3',
    );
    const source = await mkdtemp(join(build, 'perl-build-'));
    await run('tar', ['-xzf', archive, '--strip-components=1', '-C', source]);
    await run(
      'sh',
      [
        'Configure',
        '-des',
        `-Dprefix=${join(runtime, 'perl')}`,
        '-Duserelocatableinc',
        '-Uuseshrplib',
        '-Uinstallusrbinperl',
        '-Dman1dir=none',
        '-Dman3dir=none',
        ...(deploymentTarget
          ? [
              `-Accflags=-mmacosx-version-min=${deploymentTarget}`,
              `-Aldflags=-mmacosx-version-min=${deploymentTarget}`,
            ]
          : []),
      ],
      source,
    );
    await run('make', [`-j${Math.min(4, availableParallelism())}`], source);
    await run('make', ['install'], source);
    for (const license of ['Artistic', 'Copying', 'README'])
      await cp(join(source, license), join(runtime, `Perl-${license}`));
  }
  await chmod(perlBin, 0o755);
  await run(perlBin, [join(exifDir, 'bin/exiftool'), '-ver']);
} else {
  await run(join(exifDir, 'bin/exiftool.exe'), ['-ver']);
}
await writeFile(
  join(runtime, 'manifest.json'),
  JSON.stringify(
    {
      exiftool: '13.59',
      distribution: '13.59.2',
      perl: windows ? 'Packaged upstream Windows runtime' : '5.44.0',
      platform: process.platform,
      arch: process.arch,
      deploymentTarget,
      runtimeRevision: 2,
      exiftoolSha512: exifSha,
    },
    null,
    2,
  ),
);
// Tauri recopies resources on incremental builds. Upstream read-only license/module files otherwise make the next build fail.
async function writableResources(path) {
  for (const item of await readdir(path, { withFileTypes: true })) {
    const full = join(path, item.name);
    if (item.isDirectory()) await writableResources(full);
    else if (item.isFile()) await chmod(full, (await stat(full)).mode | 0o200);
  }
}
if (!windows) await writableResources(runtime);
console.log(`Tagryn runtime ready: ${runtime}`);
