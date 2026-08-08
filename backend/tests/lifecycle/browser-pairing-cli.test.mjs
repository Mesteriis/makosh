import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';

import { kernelCommand, startKernel, stopKernel } from './support/production-kernel-ipc.mjs';

const backendRoot = new URL('../../', import.meta.url);

test('browser pairing CLI requires local confirmation and creates an owner-approved ceremony', async () => {
  const root = await mkdtemp(join(tmpdir(), 'makosh-browser-pairing-cli-'));
  const dataDir = join(root, 'data');
  let kernel;
  try {
    const enrolled = kernelCommand(
      dataDir, 'initial-owner-enroll', '--owner-id', 'owner-browser-cli',
      '--device-id', 'device-browser-cli',
    );
    assert.equal(enrolled.status, 0, `${enrolled.stdout}\n${enrolled.stderr}`);
    kernel = await startKernel(dataDir);

    const rejected = browserPairCommand(dataDir, 'n\n');
    assert.notEqual(rejected.status, 0);
    assert.match(rejected.stderr, /browser pairing was not confirmed/);

    const approved = browserPairCommand(dataDir, 'y\n');
    assert.equal(approved.status, 0, approved.stderr);
    assert.match(approved.stdout, /^browser_pairing_id=[0-9a-f]{64}$/m);
    assert.match(approved.stdout, /^browser_pairing_state=approved_pending_registration$/m);
  } finally {
    if (kernel) await stopKernel(kernel.server);
    await rm(root, { recursive: true, force: true });
  }
});

function browserPairCommand(dataDir, input) {
  return spawnSync(
    'cargo',
    ['run', '-q', '-p', 'makosh-kernel', '--', '--data-dir', dataDir, 'browser-pairing', 'create'],
    { cwd: backendRoot, encoding: 'utf8', input, env: { ...process.env, RUSTC_WRAPPER: '' } },
  );
}
