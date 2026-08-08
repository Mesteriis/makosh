import assert from 'node:assert/strict';
import { mkdtemp, rm, symlink, writeFile } from 'node:fs/promises';
import { spawn } from 'node:child_process';
import { createConnection } from 'node:net';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';

import { waitForChildExit } from './support.mjs';

const backend = new URL('../../../', import.meta.url);

test('Recovery IPC rejects oversized and truncated frames without stopping accept', async () => {
  const fixture = await RecoveryKernelFixture.start('framing');
  try {
    await sendAndClose(fixture.socketPath, Buffer.from([0x81, 0x80, 0x04]));
    await sendAndClose(fixture.socketPath, Buffer.from([0x02, 0x0a]));
    const response = await request(fixture.socketPath, Buffer.from([0x02, 0x0a, 0x00]));
    assert.ok(response.length > 1);
    assert.equal(fixture.child.exitCode, null);
  } finally {
    await fixture.stop();
  }
});

test('a stalled recovery client cannot extend bounded Kernel shutdown', async () => {
  const fixture = await RecoveryKernelFixture.start('stalled');
  const stalled = createConnection(fixture.socketPath);
  try {
    await new Promise((resolve, reject) => {
      stalled.once('connect', resolve);
      stalled.once('error', reject);
    });
    fixture.child.kill('SIGTERM');
    await Promise.race([
      waitForChildExit(fixture.child),
      new Promise((_, reject) => setTimeout(
        () => reject(new Error('stalled client exceeded bounded shutdown')),
        2_000,
      )),
    ]);
  } finally {
    stalled.destroy();
    await fixture.stop();
  }
});

test('Kernel rejects a symlink planted at the recovery socket path', async () => {
  const fixture = await RecoveryKernelFixture.start('socket-symlink');
  try {
    const target = join(fixture.dataDir, 'socket-symlink-target');
    fixture.child.kill('SIGKILL');
    await waitForChildExit(fixture.child);
    await rm(fixture.socketPath);
    await writeFile(target, 'must remain untouched', { mode: 0o600 });
    await symlink(target, fixture.socketPath);

    const restarted = spawn(
      'cargo',
      ['run', '-q', '-p', 'makosh-kernel', '--', '--data-dir', fixture.dataDir, 'serve'],
      { cwd: backend, stdio: 'pipe' },
    );
    const stderr = [];
    restarted.stderr.on('data', (chunk) => stderr.push(chunk));
    const exit = await waitForChildExit(restarted);
    assert.notEqual(exit.code, 0);
    assert.match(Buffer.concat(stderr).toString(), /recovery socket must not be a symlink/);
  } finally {
    await fixture.stop();
  }
});

class RecoveryKernelFixture {
  constructor(dataDir, child, socketPath) {
    this.dataDir = dataDir;
    this.child = child;
    this.socketPath = socketPath;
  }

  static async start(name) {
    const dataDir = await mkdtemp(join(tmpdir(), `makosh-recovery-${name}-`));
    const child = spawn(
      'cargo',
      ['run', '-q', '-p', 'makosh-kernel', '--', '--data-dir', dataDir, 'serve'],
      { cwd: backend, stdio: 'pipe' },
    );
    const socketPath = await waitForRecoverySocket(child);
    return new RecoveryKernelFixture(dataDir, child, socketPath);
  }

  async stop() {
    if (this.child.exitCode === null && this.child.signalCode === null) {
      this.child.kill('SIGKILL');
      await waitForChildExit(this.child);
    }
    await rm(this.dataDir, { recursive: true, force: true });
  }
}

function waitForRecoverySocket(child) {
  return new Promise((resolve, reject) => {
    const timeout = setTimeout(() => reject(new Error('Kernel recovery socket timeout')), 15_000);
    child.stdout.on('data', (chunk) => {
      const match = chunk.toString().match(/^recovery_socket=(.+)$/m);
      if (match) {
        clearTimeout(timeout);
        resolve(match[1]);
      }
    });
    child.once('exit', (code) => {
      clearTimeout(timeout);
      reject(new Error(`Kernel exited before recovery readiness: ${code}`));
    });
  });
}

function sendAndClose(socketPath, frame) {
  return new Promise((resolve, reject) => {
    const socket = createConnection(socketPath);
    socket.once('connect', () => socket.end(frame));
    socket.once('close', resolve);
    socket.once('error', reject);
  });
}

function request(socketPath, frame) {
  return new Promise((resolve, reject) => {
    const socket = createConnection(socketPath);
    const chunks = [];
    socket.once('connect', () => socket.write(frame));
    socket.on('data', (chunk) => chunks.push(chunk));
    socket.once('end', () => resolve(Buffer.concat(chunks)));
    socket.once('error', reject);
  });
}
