import { spawn, execFileSync } from 'node:child_process';
import { mkdtemp, writeFile } from 'node:fs/promises';
import { resolve, join } from 'node:path';

const root = resolve(import.meta.dirname, '..');
const binary = join(
  root,
  'src-tauri/target/release/bundle/macos/Tagryn.app/Contents/MacOS/tagryn',
);
const fixture = join(
  root,
  'artifacts/fixtures/Alpine Licht_001 – Österreich.jpg',
);
const runs = [];
for (let i = 0; i < Number(process.argv[2] ?? 3); i++) {
  const profile = await mkdtemp(join(root, '.build/native-timing-'));
  const child = spawn(binary, [fixture], {
    env: { ...process.env, PATH: '/nonexistent', TAGRYN_PROFILE_DIR: profile },
    stdio: ['ignore', 'pipe', 'pipe'],
  });
  let stderr = '';
  const ready = await new Promise((accept, reject) => {
    const timeout = setTimeout(() => {
      child.kill('SIGTERM');
      reject(
        new Error(`Native startup timed out (pid ${child.pid}): ${stderr}`),
      );
    }, 10000);
    child.once('error', reject);
    if (!process.argv.includes('--background'))
      setTimeout(() => {
        try {
          execFileSync('/usr/bin/osascript', [
            '-e',
            `tell application "System Events" to set frontmost of (first process whose unix id is ${child.pid}) to true`,
          ]);
        } catch (error) {
          clearTimeout(timeout);
          child.kill('SIGTERM');
          reject(error);
        }
      }, 150);
    child.stderr.on('data', (chunk) => {
      stderr += chunk.toString();
      const match = stderr.match(/TAGRYN_READY (\{[^\n]+\})/);
      if (match) {
        clearTimeout(timeout);
        accept(JSON.parse(match[1]));
      }
    });
    child.once('exit', (code) => {
      clearTimeout(timeout);
      reject(new Error(`App exited before ready (${code}): ${stderr}`));
    });
  });
  await new Promise((resolve) => setTimeout(resolve, 1200));
  let descendants = [];
  try {
    descendants = execFileSync('/usr/bin/pgrep', ['-P', String(child.pid)], {
      encoding: 'utf8',
    })
      .trim()
      .split(/\s+/);
  } catch {
    /* Worker count will remain explicit in the report. */
  }
  const memory = execFileSync(
    '/bin/ps',
    ['-p', [child.pid, ...descendants].join(','), '-o', 'pid=,rss=,comm='],
    { encoding: 'utf8' },
  )
    .trim()
    .split('\n')
    .map((line) => {
      const match = line.trim().match(/^(\d+)\s+(\d+)\s+(.+)$/);
      return {
        pid: Number(match[1]),
        rssKiB: Number(match[2]),
        executable: match[3].split('/').at(-1),
      };
    });
  const stopped = new Promise((resolve) =>
    child.once('exit', (code) => resolve(code)),
  );
  execFileSync('/usr/bin/osascript', [
    join(root, 'scripts/native-control.applescript'),
    'close-window',
    'q',
  ]);
  const timeout = setTimeout(() => {
    child.kill('SIGTERM');
  }, 10000);
  const exitCode = await stopped;
  clearTimeout(timeout);
  if (exitCode !== 0 || stderr.includes('Tagryn UI:'))
    throw new Error(`Native shutdown/diagnostic failed: ${stderr}`);
  runs.push({
    ...ready,
    memory,
    directProcessRssKiB: memory.reduce(
      (sum, process) => sum + process.rssKiB,
      0,
    ),
    normalQuit: exitCode === 0,
  });
}
const result = {
  context:
    'Release .app, one real JPEG fixture, fresh SQLite profile per run, warm OS caches, PATH=/nonexistent. The harness explicitly activates the native window after 150 ms; this overhead is included. UI readiness is not full-folder indexing. RSS includes application and direct ExifTool children only; shared/XPC WebKit processes are excluded and this is NOT total application memory.',
  runs,
};
await writeFile(
  join(root, 'artifacts/native-timing.json'),
  JSON.stringify(result, null, 2),
);
console.log(JSON.stringify(result));
