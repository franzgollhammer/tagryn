import { execFileSync } from 'node:child_process';
import { resolve, join } from 'node:path';
import { mkdtemp, cp, readFile, stat } from 'node:fs/promises';
import { tmpdir } from 'node:os';

const runtime = resolve(import.meta.dirname, '../src-tauri/resources/runtime');
const manifest = JSON.parse(
  await readFile(join(runtime, 'manifest.json'), 'utf8'),
);
const destination = await mkdtemp(join(tmpdir(), 'tagryn-runtime-'));
await cp(runtime, destination, { recursive: true });
const windows = process.platform === 'win32';
const bin = join(
  destination,
  windows ? 'exiftool/bin/exiftool.exe' : 'perl/bin/perl',
);
const args = windows
  ? ['-ver']
  : [join(destination, 'exiftool/bin/exiftool'), '-ver'];
const version = execFileSync(bin, args, {
  env: {
    ...process.env,
    PATH: windows ? process.env.SystemRoot : '/nonexistent',
    PERL5LIB: '',
    PERL5OPT: '',
    PERLLIB: '',
  },
  encoding: 'utf8',
}).trim();
if (version !== manifest.exiftool)
  throw new Error(`Expected ${manifest.exiftool}, received ${version}`);
let metadataRead = 'not run; generate fixtures for the additional check';
const fixture = resolve(
  import.meta.dirname,
  '../artifacts/fixtures/Alpine Licht_001 – Österreich.jpg',
);
try {
  await stat(fixture);
  const output = execFileSync(
    bin,
    [
      ...(windows ? [] : [join(destination, 'exiftool/bin/exiftool')]),
      '-config',
      '',
      '-j',
      '-G1',
      fixture,
    ],
    {
      env: {
        ...process.env,
        PATH: windows ? process.env.SystemRoot : '/nonexistent',
        PERL5LIB: '',
        PERL5OPT: '',
        PERLLIB: '',
      },
      encoding: 'utf8',
      maxBuffer: 32 * 1024 * 1024,
    },
  );
  const document = JSON.parse(output)[0];
  if (document['IFD0:Model'] !== 'X-T5')
    throw new Error('Relocated runtime did not read the real camera fixture');
  metadataRead = 'passed';
} catch (error) {
  if (error.code !== 'ENOENT') throw error;
}
console.log(
  JSON.stringify({
    relocatedRuntime: destination,
    version,
    systemPerlRequired: false,
    metadataRead,
  }),
);
