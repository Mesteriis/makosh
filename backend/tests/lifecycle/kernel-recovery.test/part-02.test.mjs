import assert from 'node:assert/strict';
import { chmod, copyFile, mkdir, mkdtemp, readFile, rm, stat, symlink, writeFile } from 'node:fs/promises';
import { spawn, spawnSync } from 'node:child_process';
import { createConnection } from 'node:net';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';

import { waitForChildExit } from './support.mjs';

test('Kernel rejects a data directory that is readable by other users', async () => {
  const dataDir = await mkdtemp(join(tmpdir(), 'makosh-kernel-permissions-'));
  try {
    await chmod(dataDir, 0o755);
    const result = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'makosh-kernel', '--', '--data-dir', dataDir, 'status'],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.notEqual(result.status, 0);
    assert.match(result.stderr, /data directory must be owner-private/);
  } finally {
    await rm(dataDir, { recursive: true, force: true });
  }
});
test('Kernel rejects symlinked data and control-store paths', async () => {
  const root = await mkdtemp(join(tmpdir(), 'makosh-kernel-symlink-'));
  const target = join(root, 'target');
  const linkedDataDir = join(root, 'linked-data');
  const dataDir = join(root, 'data');
  try {
    await writeFile(target, 'not a store');
    await symlink(target, linkedDataDir);
    const linkedDirectory = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'makosh-kernel', '--', '--data-dir', linkedDataDir, 'status'],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.notEqual(linkedDirectory.status, 0);
    assert.match(linkedDirectory.stderr, /data directory must not be a symlink/);

    await writeFile(join(root, 'store-target'), 'not a sqlite database');
    await mkdir(dataDir, { mode: 0o700 });
    await symlink(join(root, 'store-target'), join(dataDir, 'kernel-control-store.sqlite'));
    const linkedStore = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'makosh-kernel', '--', '--data-dir', dataDir, 'status'],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.equal(linkedStore.status, 0, linkedStore.stderr);
    assert.match(linkedStore.stdout, /^state=recovery_only\ncontrol_store=unavailable\n$/);
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});



test('Kernel keeps a corrupt control store in restricted recovery_only', async () => {
  const dataDir = await mkdtemp(join(tmpdir(), 'makosh-kernel-corrupt-'));
  try {
    await writeFile(join(dataDir, 'kernel-control-store.sqlite'), 'not a sqlite database', { mode: 0o600 });
    const result = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'makosh-kernel', '--', '--data-dir', dataDir, 'status'],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.equal(result.status, 0, result.stderr);
    assert.match(result.stdout, /^state=recovery_only\ncontrol_store=unavailable\n$/);
  } finally {
    await rm(dataDir, { recursive: true, force: true });
  }
});



test('Installation anchor prevents a missing Store from becoming a new instance', async () => {
  const dataDir = await mkdtemp(join(tmpdir(), 'makosh-kernel-anchor-'));
  const storePath = join(dataDir, 'kernel-control-store.sqlite');
  try {
    const first = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'makosh-kernel', '--', '--data-dir', dataDir, 'status'],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.equal(first.status, 0, first.stderr);
    await stat(join(dataDir, '.makosh-installation-anchor-v1'));
    await rm(storePath);

    const second = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'makosh-kernel', '--', '--data-dir', dataDir, 'status'],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.equal(second.status, 0, second.stderr);
    assert.match(second.stdout, /^state=recovery_only\ncontrol_store=unavailable\n$/);
    await assert.rejects(stat(storePath));
  } finally {
    await rm(dataDir, { recursive: true, force: true });
  }
});



test('Offline restore requires confirmation, verifies the anchor and advances fences', async () => {
  const sourceDir = await mkdtemp(join(tmpdir(), 'makosh-kernel-restore-source-'));
  const targetDir = await mkdtemp(join(tmpdir(), 'makosh-kernel-restore-target-'));
  try {
    const sourceBootstrap = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'makosh-kernel', '--', '--data-dir', sourceDir, 'status'],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.equal(sourceBootstrap.status, 0, sourceBootstrap.stderr);
    await copyFile(
      join(sourceDir, '.makosh-installation-anchor-v1'),
      join(targetDir, '.makosh-installation-anchor-v1'),
    );
    await copyFile(
      join(sourceDir, '.makosh-recovery-fence-v1'),
      join(targetDir, '.makosh-recovery-fence-v1'),
    );
    await writeFile(join(targetDir, 'kernel-control-store.sqlite'), 'corrupt store', { mode: 0o600 });

    const unconfirmed = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'makosh-kernel', '--', '--data-dir', targetDir, 'control-store', 'restore', '--source', join(sourceDir, 'kernel-control-store.sqlite')],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8', input: 'no\n' },
    );
    assert.notEqual(unconfirmed.status, 0);
    assert.match(unconfirmed.stderr, /operation was not confirmed/);

    const restored = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'makosh-kernel', '--', '--data-dir', targetDir, 'control-store', 'restore', '--source', join(sourceDir, 'kernel-control-store.sqlite')],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8', input: 'RESTORE\n' },
    );
    assert.equal(restored.status, 0, restored.stderr);
    assert.match(restored.stdout, /offline_control_store_operation=restore\n/);
    assert.match(restored.stdout, /control_store_generation=2\n$/);

    const status = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'makosh-kernel', '--', '--data-dir', targetDir, 'status'],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.equal(status.status, 0, status.stderr);
    assert.match(status.stdout, /^state=module_control_plane\ncontrol_store=trustworthy\n$/);
  } finally {
    await rm(sourceDir, { recursive: true, force: true });
    await rm(targetDir, { recursive: true, force: true });
  }
});



test('Offline reset requires explicit data directory and confirmation', async () => {
  const dataDir = await mkdtemp(join(tmpdir(), 'makosh-kernel-reset-'));
  try {
    const bootstrap = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'makosh-kernel', '--', '--data-dir', dataDir, 'status'],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.equal(bootstrap.status, 0, bootstrap.stderr);

    const reset = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'makosh-kernel', '--', '--data-dir', dataDir, 'control-store', 'reset'],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8', input: 'RESET\n' },
    );
    assert.equal(reset.status, 0, reset.stderr);
    assert.match(reset.stdout, /offline_control_store_operation=reset\n/);
    assert.match(reset.stdout, /control_store_generation=2\n$/);

    const missingDataDir = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'makosh-kernel', '--', 'control-store', 'reset'],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8', input: 'RESET\n' },
    );
    assert.notEqual(missingDataDir.status, 0);
    assert.match(missingDataDir.stderr, /require an explicit absolute --data-dir/);
  } finally {
    await rm(dataDir, { recursive: true, force: true });
  }
});



test('Offline reset replaces a corrupt control store only through a new-instance ceremony', async () => {
  const dataDir = await mkdtemp(join(tmpdir(), 'makosh-kernel-reset-corrupt-'));
  const anchorPath = join(dataDir, '.makosh-installation-anchor-v1');
  const storePath = join(dataDir, 'kernel-control-store.sqlite');
  try {
    const bootstrap = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'makosh-kernel', '--', '--data-dir', dataDir, 'status'],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.equal(bootstrap.status, 0, bootstrap.stderr);
    const oldAnchor = await readFile(anchorPath, 'utf8');
    await writeFile(storePath, 'corrupt store', { mode: 0o600 });

    const reset = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'makosh-kernel', '--', '--data-dir', dataDir, 'control-store', 'reset'],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8', input: 'RESET\n' },
    );
    assert.equal(reset.status, 0, reset.stderr);
    assert.match(reset.stdout, /reset_mode=new_instance\n/);
    assert.match(reset.stdout, /control_store_generation=1\n$/);
    assert.notEqual(await readFile(anchorPath, 'utf8'), oldAnchor);

    const status = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'makosh-kernel', '--', '--data-dir', dataDir, 'status'],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.equal(status.status, 0, status.stderr);
    assert.match(status.stdout, /^state=module_control_plane\ncontrol_store=trustworthy\n$/);
  } finally {
    await rm(dataDir, { recursive: true, force: true });
  }
});



test('Recovery IPC uses a bounded protobuf length-delimited frame without waiting for EOF', async () => {
  const dataDir = await mkdtemp(join(tmpdir(), 'makosh-kernel-ipc-'));
  const kernel = spawn(
    'cargo',
    ['run', '-q', '-p', 'makosh-kernel', '--', '--data-dir', dataDir, 'serve'],
    { cwd: new URL('../../', import.meta.url), stdio: 'pipe' },
  );
  try {
    const socketPath = await new Promise((resolve, reject) => {
      const timeout = setTimeout(() => reject(new Error('Kernel did not publish its recovery socket')), 15_000);
      kernel.stdout.on('data', (chunk) => {
        const match = chunk.toString().match(/^recovery_socket=(.+)$/m);
        if (match) {
          clearTimeout(timeout);
          resolve(match[1]);
        }
      });
      kernel.once('exit', (code) => {
        clearTimeout(timeout);
        reject(new Error(`Kernel exited before IPC readiness: ${code}`));
      });
    });

    const response = await new Promise((resolve, reject) => {
      const socket = createConnection(socketPath);
      const chunks = [];
      socket.once('connect', () => socket.write(Buffer.from([0x02, 0x0a, 0x00])));
      socket.on('data', (chunk) => chunks.push(chunk));
      socket.once('end', () => resolve(Buffer.concat(chunks)));
      socket.once('error', reject);
    });

    assert.ok(response.length > 1);
    assert.equal(response[0], response.length - 1);
    assert.deepEqual([...response.subarray(1, 7)], [0x0a, 0x08, 0x0a, 0x06, 0x08, 0x03]);
  } finally {
    kernel.kill('SIGTERM');
    await waitForChildExit(kernel);
    await rm(dataDir, { recursive: true, force: true });
  }
});



test('Kernel handles SIGTERM by closing the recovery socket within a bounded interval', async () => {
  const dataDir = await mkdtemp(join(tmpdir(), 'makosh-kernel-sigterm-'));
  const kernel = spawn(
    'cargo',
    ['run', '-q', '-p', 'makosh-kernel', '--', '--data-dir', dataDir, 'serve'],
    { cwd: new URL('../../', import.meta.url), stdio: 'pipe' },
  );
  try {
    const socketPath = await new Promise((resolve, reject) => {
      const timeout = setTimeout(() => reject(new Error('Kernel did not publish its recovery socket')), 15_000);
      kernel.stdout.on('data', (chunk) => {
        const match = chunk.toString().match(/^recovery_socket=(.+)$/m);
        if (match) {
          clearTimeout(timeout);
          resolve(match[1]);
        }
      });
      kernel.once('exit', (code) => {
        clearTimeout(timeout);
        reject(new Error(`Kernel exited before IPC readiness: ${code}`));
      });
    });
    kernel.kill('SIGTERM');
    await Promise.race([
      waitForChildExit(kernel),
      new Promise((_, reject) => setTimeout(() => reject(new Error('Kernel did not stop after SIGTERM')), 2_000)),
    ]);
    await assert.rejects(stat(socketPath));
  } finally {
    if (kernel.exitCode === null && kernel.signalCode === null) {
      kernel.kill('SIGKILL');
      await waitForChildExit(kernel);
    }
    await rm(dataDir, { recursive: true, force: true });
  }
});



test('Recovery IPC isolates malformed clients and keeps the Kernel serving', async () => {
  const dataDir = await mkdtemp(join(tmpdir(), 'makosh-kernel-ipc-error-'));
  const kernel = spawn(
    'cargo',
    ['run', '-q', '-p', 'makosh-kernel', '--', '--data-dir', dataDir, 'serve'],
    { cwd: new URL('../../', import.meta.url), stdio: 'pipe' },
  );
  try {
    const socketPath = await new Promise((resolve, reject) => {
      const timeout = setTimeout(() => reject(new Error('Kernel did not publish its recovery socket')), 15_000);
      kernel.stdout.on('data', (chunk) => {
        const match = chunk.toString().match(/^recovery_socket=(.+)$/m);
        if (match) {
          clearTimeout(timeout);
          resolve(match[1]);
        }
      });
      kernel.once('exit', (code) => {
        clearTimeout(timeout);
        reject(new Error(`Kernel exited before IPC readiness: ${code}`));
      });
    });
    const socket = createConnection(socketPath);
    await new Promise((resolve, reject) => {
      socket.once('connect', () => {
        socket.end(Buffer.from([0x80, 0x80, 0x80, 0x80, 0x80]));
        resolve();
      });
      socket.once('error', reject);
    });
    await new Promise((resolve) => setTimeout(resolve, 100));
    assert.equal(kernel.exitCode, null);
    assert.ok((await stat(socketPath)).isSocket());
    kernel.kill('SIGTERM');
    await Promise.race([
      waitForChildExit(kernel),
      new Promise((_, reject) => setTimeout(() => reject(new Error('Kernel did not stop after SIGTERM')), 2_000)),
    ]);
    await assert.rejects(stat(socketPath));
  } finally {
    if (kernel.exitCode === null && kernel.signalCode === null) {
      kernel.kill('SIGKILL');
      await waitForChildExit(kernel);
    }
    await rm(dataDir, { recursive: true, force: true });
  }
});



test('Kernel removes an owner-owned stale recovery socket after SIGKILL', async () => {
  const dataDir = await mkdtemp(join(tmpdir(), 'makosh-kernel-stale-socket-'));
  const startKernel = () => spawn(
    'cargo',
    ['run', '-q', '-p', 'makosh-kernel', '--', '--data-dir', dataDir, 'serve'],
    { cwd: new URL('../../', import.meta.url), stdio: 'pipe' },
  );
  const waitForSocket = (kernel) => new Promise((resolve, reject) => {
    const timeout = setTimeout(() => reject(new Error('Kernel did not publish its recovery socket')), 15_000);
    kernel.stdout.on('data', (chunk) => {
      const match = chunk.toString().match(/^recovery_socket=(.+)$/m);
      if (match) {
        clearTimeout(timeout);
        resolve(match[1]);
      }
    });
    kernel.once('exit', (code) => {
      clearTimeout(timeout);
      reject(new Error(`Kernel exited before IPC readiness: ${code}`));
    });
  });

  let kernel = startKernel();
  try {
    const socketPath = await waitForSocket(kernel);
    kernel.kill('SIGKILL');
    await waitForChildExit(kernel);
    assert.ok((await stat(socketPath)).isSocket());

    kernel = startKernel();
    assert.equal(await waitForSocket(kernel), socketPath);
    kernel.kill('SIGTERM');
    await waitForChildExit(kernel);
    await assert.rejects(stat(socketPath));
  } finally {
    if (kernel.exitCode === null && kernel.signalCode === null) {
      kernel.kill('SIGKILL');
      await waitForChildExit(kernel);
    }
    await rm(dataDir, { recursive: true, force: true });
  }
});



test('Recovery IPC exports through SQLite backup and honours bounded shutdown', async () => {
  const dataDir = await mkdtemp(join(tmpdir(), 'makosh-kernel-export-'));
  const kernel = spawn(
    'cargo',
    ['run', '-q', '-p', 'makosh-kernel', '--', '--data-dir', dataDir, 'serve'],
    { cwd: new URL('../../', import.meta.url), stdio: 'pipe' },
  );
  try {
    const socketPath = await new Promise((resolve, reject) => {
      const timeout = setTimeout(() => reject(new Error('Kernel did not publish its recovery socket')), 15_000);
      kernel.stdout.on('data', (chunk) => {
        const match = chunk.toString().match(/^recovery_socket=(.+)$/m);
        if (match) {
          clearTimeout(timeout);
          resolve(match[1]);
        }
      });
      kernel.once('exit', (code) => {
        clearTimeout(timeout);
        reject(new Error(`Kernel exited before IPC readiness: ${code}`));
      });
    });
    const request = (frame) => new Promise((resolve, reject) => {
      const socket = createConnection(socketPath);
      const chunks = [];
      socket.once('connect', () => socket.write(frame));
      socket.on('data', (chunk) => chunks.push(chunk));
      socket.once('end', () => resolve(Buffer.concat(chunks)));
      socket.once('error', reject);
    });

    const exportResponse = await request(Buffer.from([0x02, 0x1a, 0x00]));
    assert.equal(exportResponse[1], 0x1a);
    const exportedStore = join(dataDir, 'recovery', 'control-store.sqlite');
    assert.ok((await stat(exportedStore)).size > 0);

    const shutdownResponse = await request(Buffer.from([0x02, 0x22, 0x00]));
    assert.deepEqual([...shutdownResponse], [0x02, 0x22, 0x00]);
    await waitForChildExit(kernel);
  } finally {
    if (kernel.exitCode === null) {
      kernel.kill('SIGTERM');
      await waitForChildExit(kernel);
    }
    await rm(dataDir, { recursive: true, force: true });
  }
});
