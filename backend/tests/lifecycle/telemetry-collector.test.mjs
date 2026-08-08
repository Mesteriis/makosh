import assert from 'node:assert/strict';
import { execFileSync, spawn } from 'node:child_process';
import { mkdtemp, readdir, readFile, rm, stat } from 'node:fs/promises';
import net from 'node:net';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';

const BACKEND = path.resolve(import.meta.dirname, '../..');
const COLLECTOR = path.join(BACKEND, 'target/debug/makosh-telemetry-collector');

test('collector accepts one bounded private telemetry frame into a segment', async () => {
  execFileSync('cargo', ['+1.97.0', 'build', '-p', 'makosh-telemetry-collector'], {
    cwd: BACKEND,
    stdio: 'ignore',
  });
  const root = await mkdtemp(path.join(os.tmpdir(), 'makosh-telemetry-'));
  const dataDir = path.join(root, 'data');
  const runtimeDir = path.join(root, 'runtime');
  const child = spawn(COLLECTOR, ['serve', '--data-dir', dataDir, '--runtime-dir', runtimeDir], { stdio: 'ignore' });
  try {
    const socket = path.join(runtimeDir, 'telemetry.sock');
    await waitForSocket(socket);
    await writeFrame(socket, '1000|Lifecycle|Info|runtime-42|module.lifecycle|runtime.start|-|trace-42|0\n');
    const files = await waitForSegment(dataDir);
    const content = await readFile(path.join(dataDir, files[0]), 'utf8');
    assert.match(content, /runtime\.start/);
    assert.doesNotMatch(content, /private|@/);
  } finally {
    child.kill('SIGTERM');
    await rm(root, { recursive: true, force: true });
  }
});

test('collector reserves bounded diagnostic capacity after normal producer quota', async () => {
  execFileSync('cargo', ['+1.97.0', 'build', '-p', 'makosh-telemetry-collector'], {
    cwd: BACKEND,
    stdio: 'ignore',
  });
  const root = await mkdtemp(path.join(os.tmpdir(), 'makosh-telemetry-'));
  const dataDir = path.join(root, 'data');
  const runtimeDir = path.join(root, 'runtime');
  const child = spawn(COLLECTOR, ['serve', '--data-dir', dataDir, '--runtime-dir', runtimeDir], { stdio: 'ignore' });
  try {
    const socket = path.join(runtimeDir, 'telemetry.sock');
    await waitForSocket(socket);
    for (let index = 0; index < 105; index += 1) {
      await writeFrame(socket, `1000|Log|Info|runtime-42|module.lifecycle|runtime.log|-|-|${index}\n`);
    }
    for (let index = 0; index < 11; index += 1) {
      await writeFrame(socket, `1000|Log|Error|runtime-42|module.lifecycle|runtime.failure|io_error|-|${index}\n`);
    }
    const files = await waitForSegment(dataDir);
    const content = await readFile(path.join(dataDir, files[0]), 'utf8');
    assert.equal(content.trim().split('\n').length, 110);
  } finally {
    child.kill('SIGTERM');
    await rm(root, { recursive: true, force: true });
  }
});

test('collector restart retains finalized segments after an unclean stop', async () => {
  const root = await mkdtemp(path.join(os.tmpdir(), 'makosh-telemetry-'));
  const dataDir = path.join(root, 'data');
  const runtimeDir = path.join(root, 'runtime');
  let child = spawnCollector(dataDir, runtimeDir);
  try {
    const socket = path.join(runtimeDir, 'telemetry.sock');
    await waitForSocket(socket);
    await writeFrame(socket, '1000|Lifecycle|Info|runtime-42|module.lifecycle|runtime.before_crash|-|-|0\n');
    await waitForSegment(dataDir);
    child.kill('SIGKILL');
    await waitForExit(child);
    child = spawnCollector(dataDir, runtimeDir);
    await waitForSocket(socket);
    await writeFrame(socket, '1001|Lifecycle|Info|runtime-42|module.lifecycle|runtime.after_restart|-|-|0\n');
    const [segment] = await waitForSegment(dataDir);
    const content = await readFile(path.join(dataDir, segment), 'utf8');
    assert.match(content, /runtime\.before_crash/);
    assert.match(content, /runtime\.after_restart/);
  } finally {
    child.kill('SIGTERM');
    await rm(root, { recursive: true, force: true });
  }
});

test('collector inherited launch rejects a missing settings schema contract', async () => {
  const root = await mkdtemp(path.join(os.tmpdir(), 'makosh-telemetry-'));
  const child = spawn(COLLECTOR, [
    'serve-inherited',
    '--data-dir', path.join(root, 'data'),
    '--runtime-dir', path.join(root, 'runtime'),
    '--descriptor-path', '/dev/null',
    '--settings-schema-path', '',
  ], { stdio: 'ignore' });
  try {
    assert.notEqual(await waitForExit(child), 0);
  } finally {
    child.kill('SIGTERM');
    await rm(root, { recursive: true, force: true });
  }
});

function spawnCollector(dataDir, runtimeDir) {
  return spawn(COLLECTOR, ['serve', '--data-dir', dataDir, '--runtime-dir', runtimeDir], { stdio: 'ignore' });
}

function waitForExit(child) {
  return new Promise((resolve) => child.once('exit', resolve));
}

async function waitForSocket(socket) {
  for (let attempt = 0; attempt < 100; attempt += 1) {
    try {
      await stat(socket);
      if (await socketAcceptsConnections(socket)) return;
    } catch (error) {
      if (error.code !== 'ENOENT') throw error;
    }
    await new Promise((resolve) => setTimeout(resolve, 20));
  }
  throw new Error('collector socket did not appear');
}

function socketAcceptsConnections(socket) {
  return new Promise((resolve) => {
    const client = net.createConnection(socket);
    client.once('connect', () => { client.destroy(); resolve(true); });
    client.once('error', () => resolve(false));
  });
}

function writeFrame(socket, frame) {
  return new Promise((resolve, reject) => {
    const client = net.createConnection(socket);
    client.once('error', reject);
    client.once('connect', () => client.end(frame));
    client.once('close', resolve);
  });
}

async function waitForSegment(dataDir) {
  for (let attempt = 0; attempt < 100; attempt += 1) {
    try {
      const files = (await readdir(dataDir)).filter((name) => name.endsWith('.segment'));
      if (files.length === 1) return files;
    } catch (error) {
      if (error.code !== 'ENOENT') throw error;
    }
    await new Promise((resolve) => setTimeout(resolve, 20));
  }
  throw new Error('collector segment did not appear');
}
