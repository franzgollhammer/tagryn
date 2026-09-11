import { execFileSync, spawn } from 'node:child_process';
import { createHash } from 'node:crypto';
import {
  cp,
  mkdir,
  mkdtemp,
  readFile,
  readdir,
  realpath,
  rm,
  stat,
  symlink,
  writeFile,
} from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { basename, join, resolve } from 'node:path';
import { verifyInstalledRuntime } from './verify-installed-runtime.mjs';

const root = resolve(import.meta.dirname, '..');
const repository = process.env.GITHUB_REPOSITORY || 'franzgollhammer/tagryn';
const sourceRunId = process.env.TAGRYN_SOURCE_RUN || '34597693265';
const tag = process.env.TAGRYN_RELEASE_TAG || 'v0.1.0-alpha.1';
if (
  !/^\d+$/.test(sourceRunId) ||
  !/^v\d+\.\d+\.\d+(?:-[A-Za-z0-9.-]+)?$/.test(tag)
)
  throw new Error('Invalid source run or release tag');
const platform = process.platform;
const arch = process.arch;
const target = `${platform}-${arch}`;
const artifactNames = {
  'darwin-arm64': 'tagryn-macos-15',
  'darwin-x64': 'tagryn-macos-15-intel',
  'win32-x64': 'tagryn-windows-2025',
  'linux-x64': 'tagryn-ubuntu-22.04',
};
const artifactName = artifactNames[target];
if (!artifactName) throw new Error(`No verified artifact for ${target}`);
if (platform !== 'darwin' && process.env.GITHUB_ACTIONS !== 'true')
  throw new Error(
    'Windows/Linux installer checks must run on disposable GitHub runners',
  );
const run = (command, args, options = {}) =>
  execFileSync(command, args, {
    cwd: root,
    encoding: 'utf8',
    timeout: 180000,
    maxBuffer: 8 * 1024 * 1024,
    ...options,
  });
const api = (path) =>
  JSON.parse(run('gh', ['api', `repos/${repository}/${path}`]));
const sourceRun = api(`actions/runs/${sourceRunId}`);
let tagObject = api(`git/ref/tags/${tag}`).object;
for (let depth = 0; tagObject.type === 'tag' && depth < 5; depth++)
  tagObject = api(`git/tags/${tagObject.sha}`).object;
if (
  tagObject.type !== 'commit' ||
  sourceRun.head_sha !== tagObject.sha ||
  sourceRun.status !== 'completed' ||
  sourceRun.conclusion !== 'success' ||
  sourceRun.path !== '.github/workflows/verify.yml'
) {
  throw new Error(
    'Source run must be a successful Verify Tagryn run of the exact release tag',
  );
}
const configFile = api(`contents/src-tauri/tauri.conf.json?ref=${tag}`);
const config = JSON.parse(Buffer.from(configFile.content, 'base64').toString());
const artifact = api(
  `actions/runs/${sourceRunId}/artifacts?per_page=100`,
).artifacts.find((item) => item.name === artifactName);
if (!artifact || artifact.expired)
  throw new Error(`Source artifact missing or expired: ${artifactName}`);
// Tauri deliberately rejects executable paths containing symlinks on macOS.
const temporary = await realpath(
  await mkdtemp(join(tmpdir(), 'tagryn-preview-')),
);
const output = join(root, 'artifacts/preview');
await mkdir(output, { recursive: true });
const sha256 = async (path) =>
  createHash('sha256')
    .update(await readFile(path))
    .digest('hex');
const sleep = (ms) => new Promise((done) => setTimeout(done, ms));

async function verifyStartup(command, args, profile) {
  const child = spawn(command, args, {
    detached: platform !== 'win32',
    stdio: ['ignore', 'ignore', 'pipe'],
    env: {
      ...process.env,
      TAGRYN_PROFILE_DIR: profile,
      WEBKIT_DISABLE_DMABUF_RENDERER: '1',
    },
  });
  let error;
  let stderr = '';
  let frontendReady = false;
  let frontendError = false;
  child.on('error', (value) => {
    error = value;
  });
  child.stderr.on('data', (chunk) => {
    stderr = (stderr + chunk).slice(-8192);
    frontendReady ||= stderr.includes('TAGRYN_READY ');
    frontendError ||= stderr.includes('Tagryn UI:');
  });
  try {
    let initialized = false;
    for (let attempt = 0; attempt < 30; attempt++) {
      if (error) throw error;
      if (child.exitCode !== null || child.signalCode !== null)
        throw new Error(`Installed app exited during startup: ${stderr}`);
      try {
        initialized =
          (await stat(join(profile, 'tagryn.sqlite'))).size > 0 &&
          frontendReady;
      } catch {
        /* Startup is still in progress. */
      }
      if (initialized) break;
      await sleep(1000);
    }
    if (!initialized)
      throw new Error(
        `Installed app did not initialize its isolated profile: ${stderr}`,
      );
    await sleep(3000);
    if (frontendError)
      throw new Error(`Installed frontend reported an error: ${stderr}`);
    if (child.exitCode !== null || child.signalCode !== null)
      throw new Error(
        `Installed app stopped after profile initialization: ${stderr}`,
      );
    return {
      isolatedProfileCreated: true,
      frontendReady: true,
      processRemainedRunning: true,
    };
  } finally {
    if (child.pid && child.exitCode === null && child.signalCode === null) {
      if (platform === 'win32')
        run('taskkill.exe', ['/PID', String(child.pid), '/T', '/F']);
      else process.kill(-child.pid, 'SIGTERM');
      for (
        let attempt = 0;
        attempt < 20 && child.exitCode === null && child.signalCode === null;
        attempt++
      )
        await sleep(250);
      if (
        platform !== 'win32' &&
        child.exitCode === null &&
        child.signalCode === null
      )
        process.kill(-child.pid, 'SIGKILL');
    }
  }
}

let packagePath;
let verification;
try {
  const archivePath = join(temporary, 'source.zip');
  await writeFile(
    archivePath,
    run(
      'gh',
      ['api', `repos/${repository}/actions/artifacts/${artifact.id}/zip`],
      { encoding: 'buffer', maxBuffer: 256 * 1024 * 1024 },
    ),
  );
  const sourceDigest = `sha256:${await sha256(archivePath)}`;
  if (artifact.digest && artifact.digest !== sourceDigest)
    throw new Error('Downloaded artifact digest differs from GitHub metadata');
  const extracted = join(temporary, 'extracted');
  await mkdir(extracted);
  if (platform === 'darwin')
    run('/usr/bin/ditto', ['-x', '-k', archivePath, extracted]);
  else if (platform === 'linux')
    run('unzip', ['-q', archivePath, '-d', extracted]);
  else
    run(
      'powershell.exe',
      [
        '-NoProfile',
        '-NonInteractive',
        '-Command',
        'Expand-Archive -LiteralPath $env:TAGRYN_ZIP -DestinationPath $env:TAGRYN_EXTRACT',
      ],
      {
        env: {
          ...process.env,
          TAGRYN_ZIP: archivePath,
          TAGRYN_EXTRACT: extracted,
        },
      },
    );
  const bundles = join(extracted, 'src-tauri/target/release/bundle');
  const version = tag.slice(1);
  const install = join(temporary, 'installed');
  await mkdir(install);

  if (platform === 'darwin') {
    const app = join(root, 'src-tauri/target/release/bundle/macos/Tagryn.app');
    await mkdir(join(app, '..'), { recursive: true });
    if (await stat(app).catch(() => null))
      throw new Error(
        'Use a clean packaging worktree; macOS output already exists',
      );
    run('/usr/bin/ditto', [join(bundles, 'macos/Tagryn.app'), app]);
    // ExifTool does not use Perl's optional DBM bindings. Some CI hosts build
    // these against Homebrew dylibs, which must not leak into a portable app.
    const runtimeRoot = join(app, 'Contents/Resources/runtime');
    const excludedModules = ['DB_File', 'GDBM_File', 'NDBM_File'];
    async function assertNoDbmImports(directory) {
      for (const entry of await readdir(directory, { withFileTypes: true })) {
        const path = join(directory, entry.name);
        if (entry.isDirectory()) await assertNoDbmImports(path);
        else if (entry.name.endsWith('.pm') || entry.name === 'exiftool') {
          if (
            /\b(?:DB_File|GDBM_File|NDBM_File|dbmopen)\b/.test(
              await readFile(path, 'utf8'),
            )
          )
            throw new Error(
              'ExifTool now references a database module; review runtime packaging',
            );
        }
      }
    }
    await assertNoDbmImports(join(runtimeRoot, 'exiftool'));
    const removedModules = [];
    async function omitOptionalDbm(directory) {
      for (const entry of await readdir(directory, { withFileTypes: true })) {
        const path = join(directory, entry.name);
        if (
          (entry.isDirectory() &&
            basename(directory) === 'auto' &&
            excludedModules.includes(entry.name)) ||
          (entry.isFile() &&
            excludedModules.some((name) => entry.name === `${name}.pm`))
        ) {
          await rm(path, { recursive: entry.isDirectory() });
          removedModules.push(path.slice(runtimeRoot.length + 1));
        } else if (entry.isDirectory()) await omitOptionalDbm(path);
      }
    }
    await omitOptionalDbm(join(runtimeRoot, 'perl/lib'));
    const runtimeManifestPath = join(runtimeRoot, 'manifest.json');
    const runtimeManifest = JSON.parse(
      await readFile(runtimeManifestPath, 'utf8'),
    );
    runtimeManifest.previewPackaging = {
      omittedOptionalDbmFiles: removedModules,
    };
    await writeFile(
      runtimeManifestPath,
      `${JSON.stringify(runtimeManifest, null, 2)}\n`,
    );
    for (const binary of [
      'Contents/MacOS/tagryn',
      'Contents/Resources/runtime/perl/bin/perl',
    ]) {
      if (
        run('/usr/bin/lipo', ['-archs', join(app, binary)]).trim() !==
        (arch === 'arm64' ? 'arm64' : 'x86_64')
      )
        throw new Error(`Unexpected Mach-O architecture: ${binary}`);
    }
    run(process.execPath, ['scripts/verify-bundle.mjs', '--adhoc-sign']);
    const stage = join(temporary, 'dmg-stage');
    await mkdir(stage);
    run('/usr/bin/ditto', [app, join(stage, 'Tagryn.app')]);
    await symlink('/Applications', join(stage, 'Applications'));
    packagePath = join(output, `Tagryn_${version}_macos_${arch}.dmg`);
    run('/usr/bin/hdiutil', [
      'create',
      '-volname',
      'Tagryn',
      '-srcfolder',
      stage,
      '-format',
      'UDZO',
      '-ov',
      packagePath,
    ]);
    run('/usr/bin/hdiutil', ['verify', packagePath]);
    const mount = join(temporary, 'mounted');
    await mkdir(mount);
    run('/usr/bin/hdiutil', [
      'attach',
      '-readonly',
      '-nobrowse',
      '-mountpoint',
      mount,
      packagePath,
    ]);
    try {
      run('/usr/bin/ditto', [
        join(mount, 'Tagryn.app'),
        join(install, 'Tagryn.app'),
      ]);
    } finally {
      run('/usr/bin/hdiutil', ['detach', mount]);
    }
    const installedApp = join(install, 'Tagryn.app');
    run('/usr/bin/codesign', ['--verify', '--deep', '--strict', installedApp]);
    const runtime = await verifyInstalledRuntime(
      join(installedApp, 'Contents/Resources/runtime'),
    );
    const startup = await verifyStartup(
      join(installedApp, 'Contents/MacOS/tagryn'),
      ['-ApplePersistenceIgnoreState', 'YES'],
      join(temporary, 'profile'),
    );
    verification = {
      diskImageVerified: true,
      copiedBundleSignatureVerified: true,
      signing: 'ad-hoc; no Developer ID or notarization',
      runtime,
      startup,
    };
  } else if (platform === 'win32') {
    const installer = join(
      bundles,
      `nsis/Tagryn_${config.version}_x64-setup.exe`,
    );
    packagePath = join(output, `Tagryn_${version}_windows_x64-setup.exe`);
    await cp(installer, packagePath);
    run(
      'powershell.exe',
      [
        '-NoProfile',
        '-NonInteractive',
        '-Command',
        '$tagrynProcess = Start-Process -FilePath $env:TAGRYN_INSTALLER -ArgumentList @("/S", ("/D=" + $env:TAGRYN_INSTALL_DIR)) -Wait -PassThru; exit $tagrynProcess.ExitCode',
      ],
      {
        env: {
          ...process.env,
          TAGRYN_INSTALLER: packagePath,
          TAGRYN_INSTALL_DIR: install,
        },
      },
    );
    try {
      const runtime = await verifyInstalledRuntime(join(install, 'runtime'));
      const startup = await verifyStartup(
        join(install, 'tagryn.exe'),
        [],
        join(temporary, 'profile'),
      );
      verification = {
        silentInstallationPassed: true,
        originalInstallerBytesPreserved:
          (await sha256(packagePath)) === (await sha256(installer)),
        signing: 'unsigned',
        runtime,
        startup,
      };
    } finally {
      const uninstaller = (await readdir(install)).find((name) =>
        /^uninstall.*\.exe$/i.test(name),
      );
      if (uninstaller) run(join(install, uninstaller), ['/S']);
    }
  } else {
    const installer = join(bundles, `deb/Tagryn_${config.version}_amd64.deb`);
    packagePath = join(output, `Tagryn_${version}_linux_amd64.deb`);
    await cp(installer, packagePath);
    run('sudo', ['apt-get', 'install', '-y', packagePath]);
    try {
      const runtime = await verifyInstalledRuntime('/usr/lib/Tagryn/runtime');
      const startup = await verifyStartup(
        'xvfb-run',
        ['-a', 'dbus-run-session', '--', '/usr/bin/tagryn'],
        join(temporary, 'profile'),
      );
      verification = {
        debInstallationPassed: true,
        originalInstallerBytesPreserved:
          (await sha256(packagePath)) === (await sha256(installer)),
        signing: 'unsigned',
        runtime,
        startup,
      };
    } finally {
      run('sudo', ['apt-get', 'remove', '-y', 'tagryn']);
    }
  }
  const info = {
    schema: 1,
    releaseTag: tag,
    applicationCommit: tagObject.sha,
    sourceRun: `https://github.com/${repository}/actions/runs/${sourceRunId}`,
    sourceArtifact: {
      id: artifact.id,
      name: artifactName,
      digest: sourceDigest,
    },
    packagingCommit:
      process.env.GITHUB_SHA || run('git', ['rev-parse', 'HEAD']).trim(),
    packagingRun: process.env.GITHUB_RUN_ID
      ? `https://github.com/${repository}/actions/runs/${process.env.GITHUB_RUN_ID}`
      : null,
    target,
    asset: basename(packagePath),
    sha256: await sha256(packagePath),
    verification,
  };
  await writeFile(
    join(output, `build-info-${target}.json`),
    `${JSON.stringify(info, null, 2)}\n`,
  );
  console.log(JSON.stringify(info, null, 2));
} finally {
  await rm(temporary, { recursive: true, force: true });
}
