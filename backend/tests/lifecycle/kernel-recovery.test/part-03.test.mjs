import assert from 'node:assert/strict';
import { mkdir, mkdtemp, readFile, rm } from 'node:fs/promises';
import { spawnSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';

const backend = new URL('../../', import.meta.url);

function command(executable, arguments_) {
  return spawnSync(executable, arguments_, { cwd: backend, encoding: 'utf8' });
}

function kernel(dataDir) {
  return command('cargo', ['run', '-q', '-p', 'makosh-kernel', '--', '--data-dir', dataDir, 'status']);
}

test('Kernel migrates a canonical v8 pending settings revision without declaring it current', async () => {
  const root = await mkdtemp(join(tmpdir(), 'makosh-kernel-settings-migration-'));
  const dataDir = join(root, 'data');
  const storePath = join(dataDir, 'kernel-control-store.sqlite');
  try {
    await mkdir(dataDir, { mode: 0o700 });
    assert.equal(kernel(dataDir).status, 0);
    const anchor = await readFile(join(dataDir, '.makosh-installation-anchor-v1'), 'utf8');
    await rm(storePath);
    createVersionOneFixture(storePath, anchor.trim().split(':').at(-1));
    const prepareV8 = kernel(dataDir);
    assert.equal(prepareV8.status, 0, prepareV8.stderr);
    assert.match(prepareV8.stdout, /^state=recovery_only\ncontrol_store=unavailable\n$/);
    insertPendingSettingsFixture(storePath);

    const migrated = kernel(dataDir);
    assert.equal(migrated.status, 0, migrated.stderr);
    assert.match(migrated.stdout, /^state=module_control_plane\ncontrol_store=trustworthy\n$/);
    assert.equal(readMigrationState(storePath), '31\npending_validation\n');
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

function createVersionOneFixture(storePath, instanceId) {
  const sql = `
    CREATE TABLE makosh_kernel_control_store_metadata (
      singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
      schema_version INTEGER NOT NULL,
      instance_id TEXT NOT NULL,
      generation INTEGER NOT NULL CHECK (generation >= 1)
    ) STRICT;
    INSERT INTO makosh_kernel_control_store_metadata
      (singleton, schema_version, instance_id, generation)
      VALUES (1, 1, '${instanceId}', 1);
    CREATE TRIGGER stop_at_v8
      BEFORE UPDATE OF schema_version ON makosh_kernel_control_store_metadata
      WHEN NEW.schema_version > 8
      BEGIN SELECT RAISE(ABORT, 'v8 fixture ready'); END;
  `;
  const result = command('sqlite3', [storePath, sql]);
  assert.equal(result.status, 0, result.stderr);
}

function insertPendingSettingsFixture(storePath) {
  const sql = `
    DROP TRIGGER stop_at_v8;
    INSERT INTO makosh_kernel_module_registration
      (registration_id, module_id, owner_id, descriptor_sha256, state, grant_epoch)
      VALUES ('registration-1', 'module-1', 'owner-1', zeroblob(32), 'pending', 1);
    INSERT INTO makosh_kernel_settings_schema_binding
      (registration_id, schema_major, schema_revision, schema_sha256, desired_revision, effective_revision)
      VALUES ('registration-1', 1, 1, zeroblob(32), 2, 1);
  `;
  const result = command('sqlite3', [storePath, sql]);
  assert.equal(result.status, 0, result.stderr);
}

function readMigrationState(storePath) {
  const sql = 'SELECT schema_version FROM makosh_kernel_control_store_metadata; SELECT apply_state FROM makosh_kernel_settings_schema_binding;';
  const result = command('sqlite3', [storePath, sql]);
  assert.equal(result.status, 0, result.stderr);
  return result.stdout;
}
