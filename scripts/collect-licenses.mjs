import { readdir, readFile, writeFile } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';
import { execFileSync } from 'node:child_process';

const root = resolve(import.meta.dirname, '..');
const sections = [
  'TAGRYN — THIRD-PARTY DEPENDENCY LICENSES\nGenerated from the locked build dependencies. Runtime ExifTool/Perl licenses are additionally included in runtime/.\nDevelopment tools are listed for transparency; they are not necessarily shipped in the application.',
];
async function notices(directory, explicit) {
  const names = (await readdir(directory)).filter((name) =>
    /^(licen[cs]e|copying|copyright|notice)([.-]|$)/i.test(name),
  );
  if (explicit && !names.includes(explicit)) names.push(explicit);
  const output = [];
  for (const name of names.sort()) {
    try {
      output.push(`${name}\n${await readFile(join(directory, name), 'utf8')}`);
    } catch {
      /* Some packages use a license directory rather than a text file. */
    }
  }
  return output.join('\n\n');
}
async function npmPackages(directory) {
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    if (!entry.isDirectory() || entry.name.startsWith('.')) continue;
    const path = join(directory, entry.name);
    if (entry.name.startsWith('@')) {
      await npmPackages(path);
      continue;
    }
    try {
      const pkg = JSON.parse(
        await readFile(join(path, 'package.json'), 'utf8'),
      );
      const license = await notices(path);
      sections.push(
        `npm: ${pkg.name}@${pkg.version}\nLicense: ${typeof pkg.license === 'string' ? pkg.license : JSON.stringify(pkg.license ?? 'See upstream package')}\nSource: ${typeof pkg.repository === 'string' ? pkg.repository : (pkg.repository?.url ?? pkg.homepage ?? 'https://www.npmjs.com/package/' + pkg.name)}\n\n${license}`,
      );
      try {
        await npmPackages(join(path, 'node_modules'));
      } catch {
        /* No nested packages. */
      }
    } catch {
      /* Not a package directory. */
    }
  }
}
await npmPackages(join(root, 'node_modules'));
const cargo = JSON.parse(
  execFileSync(
    'cargo',
    [
      'metadata',
      '--locked',
      '--format-version',
      '1',
      '--manifest-path',
      'src-tauri/Cargo.toml',
    ],
    { cwd: root, encoding: 'utf8', maxBuffer: 32 * 1024 * 1024 },
  ),
);
for (const pkg of cargo.packages.sort((a, b) => a.name.localeCompare(b.name))) {
  if (!pkg.source) continue;
  sections.push(
    `Rust: ${pkg.name}@${pkg.version}\nLicense: ${pkg.license ?? 'See included license file'}\nSource: ${pkg.repository ?? pkg.source}\n\n${await notices(dirname(pkg.manifest_path), pkg.license_file)}`,
  );
}
const output = join(root, 'docs/DEPENDENCY-LICENSES.txt');
await writeFile(output, sections.join('\n\n' + '='.repeat(78) + '\n\n') + '\n');
console.log(`Collected ${sections.length - 1} package notices: ${output}`);
