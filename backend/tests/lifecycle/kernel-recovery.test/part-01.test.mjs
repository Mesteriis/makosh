import assert from 'node:assert/strict';
import { chmod, copyFile, mkdir, mkdtemp, rm, stat, symlink, writeFile } from 'node:fs/promises';
import { spawn, spawnSync } from 'node:child_process';
import { createConnection } from 'node:net';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import test from 'node:test';

import { waitForChildExit } from './support.mjs';

test('Kernel reaches module_control_plane with an explicit private data directory', async () => {
  const dataDir = await mkdtemp(join(tmpdir(), 'makosh-kernel-recovery-'));
  try {
    const result = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'makosh-kernel', '--', '--data-dir', dataDir, 'status'],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );

    assert.equal(result.status, 0, result.stderr);
    assert.match(result.stdout, /^state=module_control_plane\ncontrol_store=trustworthy\n$/);
  } finally {
    await rm(dataDir, { recursive: true, force: true });
  }
});

test('incomplete browser Gateway admission fails before creating a control store', async () => {
  const dataDir = await mkdtemp(join(tmpdir(), 'makosh-kernel-browser-gateway-config-'));
  try {
    const result = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'makosh-kernel', '--', '--data-dir', dataDir, 'serve', '--browser-gateway-listen-address', '127.0.0.1:0'],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );

    assert.notEqual(result.status, 0);
    assert.match(result.stderr, /browser Gateway listener, origin, RP ID, certificate DER and private-key DER must be specified together/);
    await assert.rejects(stat(join(dataDir, 'kernel-control-store.sqlite')));
  } finally {
    await rm(dataDir, { recursive: true, force: true });
  }
});


test('one production Kernel serve owns every private control-plane socket', async () => {
  const dataDir = await mkdtemp(join(tmpdir(), 'makosh-kernel-control-plane-'));
  const kernel = spawn(
    'cargo',
    ['run', '-q', '-p', 'makosh-kernel', '--', '--data-dir', dataDir, 'serve'],
    { cwd: new URL('../../', import.meta.url), stdio: 'pipe' },
  );
  const expectedSockets = new Set([
    'recovery_socket',
    'owner_control_socket',
    'module_registration_socket',
    'external_runtime_session_socket',
  ]);
  try {
    const sockets = await new Promise((resolve, reject) => {
      const timeout = setTimeout(() => reject(new Error('Kernel did not publish control-plane sockets')), 15_000);
      let stdout = '';
      let stderr = '';
      kernel.stdout.on('data', (chunk) => {
        stdout += chunk.toString('utf8');
        const sockets = new Map([...stdout.matchAll(/^(recovery_socket|owner_control_socket|module_registration_socket|external_runtime_session_socket)=(.+)$/gm)].map(([, name, path]) => [name, path]));
        if ([...expectedSockets].every((name) => sockets.has(name))) {
          clearTimeout(timeout);
          resolve(sockets);
        }
      });
      kernel.stderr.on('data', (chunk) => { stderr += chunk.toString('utf8'); });
      kernel.once('exit', (code) => reject(new Error(`Kernel exited before control-plane readiness: ${code}: ${stderr}`)));
    });
    for (const name of expectedSockets) {
      assert.equal((await stat(sockets.get(name))).mode & 0o777, 0o600, name);
    }
    kernel.kill('SIGTERM');
    await Promise.race([
      waitForChildExit(kernel),
      new Promise((_, reject) => setTimeout(() => reject(new Error('Kernel did not stop after SIGTERM')), 2_000)),
    ]);
    for (const path of sockets.values()) {
      await assert.rejects(stat(path));
    }
  } finally {
    if (kernel.exitCode === null) {
      kernel.kill('SIGTERM');
      await waitForChildExit(kernel);
    }
    await rm(dataDir, { recursive: true, force: true });
  }
});


test('an untrusted Control Store exposes only the recovery socket', async () => {
  const dataDir = await mkdtemp(join(tmpdir(), 'makosh-kernel-recovery-sockets-'));
  let kernel;
  try {
    const initialized = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'makosh-kernel', '--', '--data-dir', dataDir, 'status'],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.equal(initialized.status, 0, initialized.stderr);
    await writeFile(join(dataDir, 'kernel-control-store.sqlite'), 'corrupt control store', { mode: 0o600 });
    kernel = spawn(
      'cargo',
      ['run', '-q', '-p', 'makosh-kernel', '--', '--data-dir', dataDir, 'serve'],
      { cwd: new URL('../../', import.meta.url), stdio: 'pipe' },
    );
    const recoverySocket = await new Promise((resolve, reject) => {
      const timeout = setTimeout(() => reject(new Error('Kernel did not publish its recovery socket')), 15_000);
      let stdout = '';
      kernel.stdout.on('data', (chunk) => {
        stdout += chunk.toString('utf8');
        const match = stdout.match(/^recovery_socket=(.+)$/m);
        if (match) { clearTimeout(timeout); resolve(match[1]); }
      });
      kernel.once('exit', (code) => reject(new Error(`Kernel exited before recovery readiness: ${code}`)));
    });
    const runtimeDir = dirname(recoverySocket);
    await new Promise((resolve) => setTimeout(resolve, 100));
    await assert.rejects(stat(join(runtimeDir, 'owner.sock')));
    await assert.rejects(stat(join(runtimeDir, 'reg.sock')));
    await assert.rejects(stat(join(runtimeDir, 'runtime.sock')));
  } finally {
    if (kernel?.exitCode === null) {
      kernel.kill('SIGTERM');
      await waitForChildExit(kernel);
    }
    await rm(dataDir, { recursive: true, force: true });
  }
});


test('file-backed adapter generates a key and enrolls exactly one local owner without development mode', async () => {
  const dataDir = await mkdtemp(join(tmpdir(), 'makosh-kernel-development-enrollment-'));
  try {
    const generated = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'makosh-kernel', '--', '--data-dir', dataDir, 'device-key-generate'],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.equal(generated.status, 0, generated.stderr);
    assert.match(generated.stdout, new RegExp(`^file_device_key=created\\nfile_device_key_path=${dataDir}/device-es256\\.key\\n$`));
    const generatedAgain = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'makosh-kernel', '--', '--data-dir', dataDir, 'device-key-generate'],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.equal(generatedAgain.status, 0, generatedAgain.stderr);
    assert.match(generatedAgain.stdout, /^file_device_key=existing\n/);

    const enrolled = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'makosh-kernel', '--', '--data-dir', dataDir, 'initial-owner-enroll', '--owner-id', 'dev-owner', '--device-id', 'dev-device'],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.equal(enrolled.status, 0, enrolled.stderr);
    assert.match(enrolled.stderr, /file-backed device signer/);
    assert.match(enrolled.stdout, /file_initial_owner_enrolled=true/);
    const key = await stat(join(dataDir, 'device-es256.key'));
    assert.equal(key.mode & 0o777, 0o600);

    const second = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'makosh-kernel', '--', '--data-dir', dataDir, 'initial-owner-enroll', '--owner-id', 'other-owner', '--device-id', 'other-device'],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.notEqual(second.status, 0);
    assert.match(second.stderr, /initial owner is already enrolled/);
  } finally {
    await rm(dataDir, { recursive: true, force: true });
  }
});


test('file-backed enrollment rejects malformed IDs before creating a key', async () => {
  const dataDir = await mkdtemp(join(tmpdir(), 'makosh-kernel-development-invalid-owner-'));
  try {
    const result = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'makosh-kernel', '--', '--data-dir', dataDir, 'initial-owner-enroll', '--owner-id', 'invalid owner', '--device-id', 'dev-device'],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.notEqual(result.status, 0);
    assert.match(result.stderr, /must be ASCII identifiers/);
    await assert.rejects(stat(join(dataDir, 'device-es256.key')));
    await assert.rejects(stat(join(dataDir, 'kernel-control-store.sqlite')));
  } finally {
    await rm(dataDir, { recursive: true, force: true });
  }
});


test('Kernel uses the OS-standard local data directory when no override is supplied', async () => {
  const home = await mkdtemp(join(tmpdir(), 'makosh-kernel-home-'));
  try {
    const result = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'makosh-kernel', '--', 'status'],
      {
        cwd: new URL('../../', import.meta.url),
        encoding: 'utf8',
        env: {
          ...process.env,
          HOME: home,
          CARGO_HOME: process.env.CARGO_HOME ?? '/Users/avm/.cargo',
          RUSTUP_HOME: process.env.RUSTUP_HOME ?? '/Users/avm/.rustup',
        },
      },
    );

    assert.equal(result.status, 0, result.stderr);
    assert.match(result.stdout, /^state=module_control_plane\ncontrol_store=trustworthy\n$/);
  } finally {
    await rm(home, { recursive: true, force: true });
  }
});


test('Kernel rejects a relative data directory without fallback', () => {
  const result = spawnSync(
    'cargo',
    ['run', '-q', '-p', 'makosh-kernel', '--', '--data-dir', 'relative-store', 'status'],
    { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
  );

  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /data directory must be an absolute path/);
});


test('Kernel rejects a second process for the same data directory', async () => {
  const dataDir = await mkdtemp(join(tmpdir(), 'makosh-kernel-lock-'));
  const first = spawn(
    'cargo',
    ['run', '-q', '-p', 'makosh-kernel', '--', '--data-dir', dataDir, 'serve'],
    { cwd: new URL('../../', import.meta.url), stdio: 'pipe' },
  );
  try {
    await new Promise((resolve, reject) => {
      const timeout = setTimeout(() => reject(new Error('first Kernel did not become ready')), 15_000);
      first.stdout.on('data', (chunk) => {
        if (chunk.toString().includes('recovery_socket=')) {
          clearTimeout(timeout);
          resolve();
        }
      });
      first.once('exit', (code) => {
        clearTimeout(timeout);
        reject(new Error(`first Kernel exited before readiness: ${code}`));
      });
    });
    const second = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'makosh-kernel', '--', '--data-dir', dataDir, 'status'],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.notEqual(second.status, 0);
    assert.match(second.stderr, /data directory is already in use/);
  } finally {
    first.kill('SIGTERM');
    await waitForChildExit(first);
    await rm(dataDir, { recursive: true, force: true });
  }
});


test('canonical data-directory aliases share one exclusive instance lock', async () => {
  const root = await mkdtemp(join(tmpdir(), 'makosh-kernel-alias-lock-'));
  const realRoot = join(root, 'real');
  const dataDir = join(realRoot, 'data');
  const aliasRoot = join(root, 'alias');
  await mkdir(realRoot, { mode: 0o700 });
  await symlink(realRoot, aliasRoot);
  const first = spawn(
    'cargo',
    ['run', '-q', '-p', 'makosh-kernel', '--', '--data-dir', dataDir, 'serve'],
    { cwd: new URL('../../', import.meta.url), stdio: 'pipe' },
  );
  try {
    await new Promise((resolve, reject) => {
      const timeout = setTimeout(() => reject(new Error('first Kernel did not become ready')), 15_000);
      first.stdout.on('data', (chunk) => {
        if (chunk.toString().includes('recovery_socket=')) {
          clearTimeout(timeout);
          resolve();
        }
      });
      first.once('exit', (code) => reject(new Error(`first Kernel exited before readiness: ${code}`)));
    });
    const second = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'makosh-kernel', '--', '--data-dir', join(aliasRoot, 'data'), 'status'],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.notEqual(second.status, 0);
    assert.match(second.stderr, /data directory is already in use/);
  } finally {
    first.kill('SIGTERM');
    await waitForChildExit(first);
    await rm(root, { recursive: true, force: true });
  }
});
