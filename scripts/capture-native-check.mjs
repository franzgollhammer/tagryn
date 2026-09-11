import { spawn, execFileSync } from 'node:child_process';
import { mkdtemp, readdir } from 'node:fs/promises';
import { resolve, join } from 'node:path';

const root = resolve(import.meta.dirname, '..');
let alreadyRunning = false;
try {
  execFileSync('/usr/bin/pgrep', ['-x', 'tagryn'], { stdio: 'pipe' });
  alreadyRunning = true;
} catch {
  /* No existing application is the required state. */
}
if (alreadyRunning)
  throw new Error(
    'Close existing Tagryn windows before native acceptance capture.',
  );
const profile = await mkdtemp(join(root, '.build/native-capture-'));
const fixtureRoot = join(root, 'artifacts/fixtures');
const fixtures = (await readdir(fixtureRoot))
  .filter((name) =>
    /^(Alpine|Dolomiten|Morgennebel|Stillwasser).*\.jpg$/.test(name),
  )
  .sort()
  .reverse()
  .map((name) => join(fixtureRoot, name));
if (!fixtures.length) throw new Error('Generate disposable fixtures first.');
const app = spawn(
  join(
    root,
    'src-tauri/target/release/bundle/macos/Tagryn.app/Contents/MacOS/tagryn',
  ),
  fixtures,
  {
    env: { ...process.env, PATH: '/nonexistent', TAGRYN_PROFILE_DIR: profile },
    stdio: ['ignore', 'ignore', 'pipe'],
  },
);
const pause = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
let stderr = '';
app.stderr.on('data', (chunk) => {
  stderr += chunk.toString();
});
const exited = new Promise((resolve) => app.once('exit', resolve));
const control = (...args) =>
  execFileSync(
    '/usr/bin/osascript',
    [join(root, 'scripts/native-control.applescript'), ...args],
    { encoding: 'utf8' },
  );
try {
  await pause(150);
  execFileSync('/usr/bin/osascript', [
    '-e',
    `tell application "System Events" to set frontmost of (first process whose unix id is ${app.pid}) to true`,
  ]);
  for (let i = 0; !stderr.includes('TAGRYN_READY') && i < 100; i++)
    await pause(100);
  if (!stderr.includes('TAGRYN_READY'))
    throw new Error(`Native UI did not initialize: ${stderr}`);
  await pause(2000);
  for (const [theme, label] of [
    ['light', 'Hell'],
    ['dark', 'Dunkel'],
  ]) {
    control('click', 'Einstellungen');
    await pause(250);
    control('click', label);
    control('click', 'Fertig');
    await pause(350);
    execFileSync(
      '/usr/bin/swift',
      [
        join(root, 'scripts/capture-native.swift'),
        join(root, `artifacts/screenshots/native-${theme}.png`),
      ],
      { encoding: 'utf8' },
    );
  }
  control('close-window', 'q');
  const quitTimeout = setTimeout(() => app.kill('SIGTERM'), 10000);
  const code = await exited;
  clearTimeout(quitTimeout);
  if (code !== 0 || stderr.includes('Tagryn UI:'))
    throw new Error(`Native acceptance did not end normally: ${stderr}`);
  console.log(
    JSON.stringify({
      result: 'passed',
      fixtures: fixtures.length,
      themes: ['light', 'dark'],
      profile,
      normalQuit: true,
    }),
  );
} catch (error) {
  execFileSync(
    '/usr/bin/swift',
    [
      join(root, 'scripts/capture-native.swift'),
      join(root, '.build/native-capture-failure.png'),
    ],
    { encoding: 'utf8' },
  );
  throw error;
} finally {
  if (app.exitCode === null) app.kill('SIGTERM');
}
