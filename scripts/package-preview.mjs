import { execFileSync, spawn } from 'node:child_process';
import { createHash } from 'node:crypto';
import { closeSync, openSync, readFileSync } from 'node:fs';
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

// hdiutil/diskimages-helper can hang when child output is a pipe.
// Keep output in a regular file, as create-dmg's hdiutil wrapper does.
function diskImage(args) {
  const logPath = join(temporary, 'hdiutil.log');
  const log = openSync(logPath, 'w');
  try {
    run('/usr/bin/hdiutil', args, {
      stdio: ['ignore', log, log],
      timeout: args[0] === 'detach' ? 45000 : 180000,
    });
  } catch (error) {
    throw new Error(
      `hdiutil ${args[0]} failed: ${error.message}\n${readFileSync(logPath, 'utf8').slice(-8192)}`,
      { cause: error },
    );
  } finally {
    closeSync(log);
  }
}

function detachImage(mount) {
  try {
    diskImage(['detach', mount]);
  } catch (error) {
    if (!/Resource busy|DiskArbitration|ETIMEDOUT/.test(error.message))
      throw error;
    // Only our disposable image is mounted here; copying has already finished.
    // Verification of the compressed image and copied signatures still follows.
    diskImage(['detach', '-force', mount]);
  }
}

async function verifyStartup(command, args, profile) {
  const child = spawn(command, args, {
    detached: platform !== 'win32',
    stdio: ['ignore', 'pipe', 'pipe'],
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
  let profileCreated = false;
  child.on('error', (value) => {
    error = value;
  });
  const observeOutput = (chunk) => {
    stderr = (stderr + chunk).slice(-8192);
    frontendReady ||= stderr.includes('TAGRYN_READY ');
    frontendError ||= stderr.includes('Tagryn UI:');
  };
  child.stdout.on('data', observeOutput);
  child.stderr.on('data', observeOutput);
  try {
    let initialized = false;
    for (let attempt = 0; attempt < 30; attempt++) {
      if (error) throw error;
      if (child.exitCode !== null || child.signalCode !== null)
        throw new Error(`Installed app exited during startup: ${stderr}`);
      try {
        profileCreated = (await stat(join(profile, 'tagryn.sqlite'))).size > 0;
        initialized = profileCreated && frontendReady;
      } catch {
        /* Startup is still in progress. */
      }
      if (initialized) break;
      await sleep(1000);
    }
    if (!initialized) {
      if (platform === 'linux') {
        const diagnostics = join(root, 'artifacts/preview-diagnostics');
        await mkdir(diagnostics, { recursive: true });
        let processInfo = '';
        try {
          processInfo = run('ps', ['-axo', 'pid,ppid,stat,comm'])
            .split('\n')
            .filter((line) => /tagryn|WebKit|Xvfb|dbus/.test(line))
            .join('\n');
          const pid = run('pgrep', ['-x', 'tagryn']).trim().split('\n')[0];
          const entries = (
            await readFile(`/proc/${pid}/environ`, 'utf8')
          ).split('\0');
          const displayEnv = { ...process.env };
          for (const key of ['DISPLAY', 'XAUTHORITY']) {
            const entry = entries.find((value) => value.startsWith(`${key}=`));
            if (entry) displayEnv[key] = entry.slice(key.length + 1);
          }
          run(
            'import',
            ['-window', 'root', join(diagnostics, 'linux-startup.png')],
            { env: displayEnv },
          );
        } catch (diagnosticError) {
          processInfo += `\nDiagnostic capture: ${diagnosticError.message}`;
        }
        await writeFile(
          join(diagnostics, 'linux-startup.json'),
          JSON.stringify(
            { profileCreated, frontendReady, stderr, processInfo },
            null,
            2,
          ),
        );
      }
      throw new Error(
        `Installed app startup timed out (profile=${profileCreated}, frontend=${frontendReady}): ${stderr}`,
      );
    }
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
    // Separate image creation, copying, and compression. hdiutil's combined
    // -srcfolder path can hang or report Resource busy on Intel CI hosts.
    const writableImage = join(temporary, 'writable.dmg');
    const writableMount = join(temporary, 'writable-mount');
    const sizeKiB = Number(run('/usr/bin/du', ['-sk', app]).split(/\s/)[0]);
    const sizeMiB = Math.ceil((sizeKiB / 1024) * 1.3) + 32;
    await mkdir(writableMount);
    diskImage([
      'create',
      '-size',
      `${sizeMiB}m`,
      '-fs',
      'HFS+',
      '-volname',
      'Tagryn',
      writableImage,
    ]);
    diskImage([
      'attach',
      '-nobrowse',
      '-mountpoint',
      writableMount,
      writableImage,
    ]);
    try {
      run('/usr/bin/ditto', [app, join(writableMount, 'Tagryn.app')]);
      await symlink('/Applications', join(writableMount, 'Applications'));
    } finally {
      detachImage(writableMount);
    }
    packagePath = join(output, `Tagryn_${version}_macos_${arch}.dmg`);
    diskImage([
      'convert',
      writableImage,
      '-format',
      'UDZO',
      '-ov',
      '-o',
      packagePath,
    ]);
    diskImage(['verify', packagePath]);
    const mount = join(temporary, 'mounted');
    await mkdir(mount);
    diskImage([
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
      detachImage(mount);
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
        [
          '-a',
          'dbus-run-session',
          '--',
          'sh',
          '-c',
          'openbox --sm-disable >/dev/null 2>&1 & exec /usr/bin/tagryn',
        ],
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
