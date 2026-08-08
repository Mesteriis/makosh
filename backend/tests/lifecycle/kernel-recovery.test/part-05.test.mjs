import assert from 'node:assert/strict';
import { copyFile, mkdtemp, rm, stat } from 'node:fs/promises';
import { spawnSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

import { runWithFileSizeLimit } from './support.mjs';

const backend = new URL('../../../', import.meta.url);

function kernel(dataDir, ...arguments_) {
  return spawnSync(
    'cargo',
    ['run', '-q', '-p', 'makosh-kernel', '--', '--data-dir', dataDir, ...arguments_],
    { cwd: backend, encoding: 'utf8' },
  );
}

function sqlite(storePath, sql) {
  const result = spawnSync('sqlite3', [storePath, sql], { encoding: 'utf8' });
  assert.equal(result.status, 0, result.stderr);
  return result.stdout;
}

test('an OS-level staged restore growth failure preserves the trustworthy target', async () => {
  const sourceDir = await mkdtemp(join(tmpdir(), 'makosh-restore-full-source-'));
  const targetDir = await mkdtemp(join(tmpdir(), 'makosh-restore-full-target-'));
  try {
    assert.equal(kernel(sourceDir, 'status').status, 0);
    const sourceStore = join(sourceDir, 'kernel-control-store.sqlite');
    const targetStore = join(targetDir, 'kernel-control-store.sqlite');
    for (const name of [
      '.makosh-installation-anchor-v1',
      '.makosh-recovery-fence-v1',
      'kernel-control-store.sqlite',
    ]) {
      await copyFile(join(sourceDir, name), join(targetDir, name));
    }
    sqlite(sourceStore, `
      CREATE TABLE makosh_kernel_restore_fault_padding (payload BLOB NOT NULL) STRICT;
      INSERT INTO makosh_kernel_restore_fault_padding VALUES (zeroblob(131072));
    `);

    const binary = fileURLToPath(new URL('target/debug/makosh-kernel', backend));
    const failed = runWithFileSizeLimit(
      binary,
      targetDir,
      (await stat(targetStore)).size,
      ['control-store', 'restore', '--source', sourceStore],
      'RESTORE\n',
    );
    assert.notEqual(failed.status, 0, failed.stdout);
    assert.match(failed.stderr, /prepare staged restore/);
    assert.equal(sqlite(targetStore, 'SELECT generation FROM makosh_kernel_control_store_metadata;'), '1\n');

    const status = kernel(targetDir, 'status');
    assert.equal(status.status, 0, status.stderr);
    assert.match(status.stdout, /^state=module_control_plane\ncontrol_store=trustworthy\n$/);
  } finally {
    await rm(sourceDir, { recursive: true, force: true });
    await rm(targetDir, { recursive: true, force: true });
  }
});
