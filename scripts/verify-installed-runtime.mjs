import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { constants } from 'node:fs';
import { copyFile, mkdtemp, readFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import sharp from 'sharp';

// Exercises the runtime shipped inside an installed package, using disposable media.
// Application-level plan/backup/restore behavior is covered by the source CI suite.
export async function verifyInstalledRuntime(runtime) {
  for (const notice of [
    'LICENSE',
    'THIRD-PARTY-NOTICES.md',
    'DEPENDENCY-LICENSES.txt',
  ]) {
    if (!(await readFile(join(runtime, '..', notice), 'utf8')).trim())
      throw new Error(`Installed license file is empty: ${notice}`);
  }
  const windows = process.platform === 'win32';
  const binary = join(
    runtime,
    windows ? 'exiftool/bin/exiftool.exe' : 'perl/bin/perl',
  );
  const prefix = windows ? [] : [join(runtime, 'exiftool/bin/exiftool')];
  const execute = (args) =>
    execFileSync(
      binary,
      [...prefix, '-config', '', '-charset', 'filename=UTF8', '-@', '-'],
      {
        input: `${args.join('\n')}\n`,
        encoding: 'utf8',
        timeout: 30000,
        maxBuffer: 4 * 1024 * 1024,
        env: {
          ...process.env,
          PATH: '/nonexistent',
          PERL5LIB: '',
          PERL5OPT: '',
          PERLLIB: '',
        },
      },
    );
  const manifest = JSON.parse(
    await readFile(join(runtime, 'manifest.json'), 'utf8'),
  );
  if (manifest.platform !== process.platform || manifest.arch !== process.arch)
    throw new Error('Installed runtime platform/architecture mismatch');
  const version = execute(['-ver']).trim();
  if (version !== manifest.exiftool)
    throw new Error(`Installed ExifTool version mismatch: ${version}`);
  const temporary = await mkdtemp(join(tmpdir(), 'tagryn-installed-runtime-'));
  try {
    const file = join(temporary, 'Install check – 東京.jpg');
    const backup = join(temporary, 'original.jpg');
    await sharp({
      create: { width: 16, height: 16, channels: 3, background: '#428070' },
    })
      .jpeg()
      .toFile(file);
    const hash = async (path) =>
      createHash('sha256')
        .update(await readFile(path))
        .digest('hex');
    const original = await hash(file);
    await copyFile(file, backup, constants.COPYFILE_EXCL);
    const type = execute(['-s3', '-FileType', file]).trim();
    if (type !== 'JPEG')
      throw new Error(`Installed runtime could not read JPEG: ${type}`);
    const title = '  Grüße – 東京 😀\nLiteral \\n $value @name  ';
    const escaped = [...Buffer.from(title)]
      .map((byte) => `\\x${byte.toString(16).padStart(2, '0')}`)
      .join('');
    execute(['-ec', '-overwrite_original', `-XMP-dc:Title=${escaped}`, file]);
    if (execute(['-b', '-XMP-dc:Title', file]) !== title)
      throw new Error('Installed runtime changed literal metadata bytes');
    if ((await hash(file)) === original)
      throw new Error('Installed runtime did not write metadata');
    if ((await hash(backup)) !== original)
      throw new Error('Fixture backup changed');
    await copyFile(backup, file);
    if ((await hash(file)) !== original)
      throw new Error('Fixture restoration failed');
    return {
      exiftool: version,
      licenseNoticesPresent: true,
      jpegRead: true,
      literalUnicodeWrite: true,
      fixtureRestoration: true,
      systemPathDisabled: true,
    };
  } finally {
    await rm(temporary, { recursive: true, force: true });
  }
}
