import assert from 'node:assert/strict';
import { mkdtemp, readFile, rm, stat } from 'node:fs/promises';
import { spawn, spawnSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

import { runWithFileSizeLimit } from './kernel-recovery.test/support.mjs';

const backend = new URL('../../', import.meta.url);

function kernel(...arguments_) {
  return spawnSync('cargo', ['run', '-q', '-p', 'makosh-kernel', '--', ...arguments_], {
    cwd: backend,
    encoding: 'utf8',
  });
}

function sqlite(storePath, sql) {
  const result = spawnSync('sqlite3', [storePath, sql], { encoding: 'utf8' });
  assert.equal(result.status, 0, result.stderr);
  return result.stdout;
}

async function pristineDataDirectory(prefix) {
  const dataDir = await mkdtemp(join(tmpdir(), prefix));
  const bootstrap = kernel('--data-dir', dataDir, 'status');
  assert.equal(bootstrap.status, 0, bootstrap.stderr);
  return dataDir;
}

async function replaceWithVersionOne(dataDir, stopAtVersion) {
  const storePath = join(dataDir, 'kernel-control-store.sqlite');
  const anchor = await readFile(join(dataDir, '.makosh-installation-anchor-v1'), 'utf8');
  const instanceId = anchor.trim().split(':').at(-1);
  await rm(storePath);
  sqlite(storePath, versionOneSql(instanceId, stopAtVersion));
  return storePath;
}

function versionOneSql(instanceId, stopAtVersion) {
  return `
    CREATE TABLE makosh_kernel_control_store_metadata (
      singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
      schema_version INTEGER NOT NULL,
      instance_id TEXT NOT NULL,
      generation INTEGER NOT NULL CHECK (generation >= 1)
    ) STRICT;
    INSERT INTO makosh_kernel_control_store_metadata
      (singleton, schema_version, instance_id, generation)
      VALUES (1, 1, '${instanceId}', 1);
    CREATE TRIGGER stop_at_source_version
      BEFORE UPDATE OF schema_version ON makosh_kernel_control_store_metadata
      WHEN NEW.schema_version > ${stopAtVersion}
      BEGIN SELECT RAISE(ABORT, 'migration boundary reached'); END;
  `;
}

function assertRestricted(dataDir) {
  const status = kernel('--data-dir', dataDir, 'status');
  assert.equal(status.status, 0, status.stderr);
  assert.match(status.stdout, /^state=recovery_only\ncontrol_store=unavailable\n$/);
}

function assertTrustworthy(dataDir) {
  const status = kernel('--data-dir', dataDir, 'status');
  assert.equal(status.status, 0, status.stderr);
  assert.match(status.stdout, /^state=module_control_plane\ncontrol_store=trustworthy\n$/);
}

async function waitForPath(path) {
  for (let attempt = 0; attempt < 100; attempt += 1) {
    try {
      await stat(path);
      return;
    } catch (error) {
      if (error.code !== 'ENOENT') throw error;
    }
    await new Promise((resolve) => setTimeout(resolve, 10));
  }
  assert.fail(`timed out waiting for ${path}`);
}

test('every source schema version 1 through 33 migrates atomically to 34', async () => {
  for (let sourceVersion = 1; sourceVersion < 34; sourceVersion += 1) {
    const dataDir = await pristineDataDirectory(`makosh-migration-v${sourceVersion}-`);
    try {
      const storePath = await replaceWithVersionOne(dataDir, sourceVersion);
      assertRestricted(dataDir);
      assert.equal(sqlite(storePath, 'SELECT schema_version FROM makosh_kernel_control_store_metadata;'), `${sourceVersion}\n`);
      sqlite(storePath, 'DROP TRIGGER stop_at_source_version;');
      assertTrustworthy(dataDir);
      assert.equal(sqlite(storePath, 'SELECT schema_version FROM makosh_kernel_control_store_metadata;'), '34\n');
    } finally {
      await rm(dataDir, { recursive: true, force: true });
    }
  }
});

test('newer and partially incompatible schemas fail closed', async () => {
  for (const mutation of [
    'UPDATE makosh_kernel_control_store_metadata SET schema_version = 35;',
    'DROP TABLE makosh_kernel_initial_owner_identity;',
  ]) {
    const dataDir = await pristineDataDirectory('makosh-migration-reject-');
    try {
      sqlite(join(dataDir, 'kernel-control-store.sqlite'), mutation);
      assertRestricted(dataDir);
    } finally {
      await rm(dataDir, { recursive: true, force: true });
    }
  }
});

test('a process kill with an active rollback journal preserves an atomic source schema', async () => {
  const dataDir = await pristineDataDirectory('makosh-migration-hot-journal-');
  try {
    const storePath = await replaceWithVersionOne(dataDir, 18);
    sqlite(storePath, 'DROP TRIGGER stop_at_source_version;');
    const writer = spawn('sqlite3', [storePath], { stdio: ['pipe', 'ignore', 'pipe'] });
    writer.stdin.write(`
      PRAGMA journal_mode=DELETE;
      BEGIN IMMEDIATE;
      ALTER TABLE makosh_kernel_control_store_metadata ADD COLUMN interrupted_probe TEXT;
      UPDATE makosh_kernel_control_store_metadata SET schema_version = 2;
    `);
    await waitForPath(`${storePath}-journal`);
    writer.kill('SIGKILL');
    await new Promise((resolve) => writer.once('exit', resolve));

    assertTrustworthy(dataDir);
    assert.equal(sqlite(storePath, 'SELECT schema_version FROM makosh_kernel_control_store_metadata;'), '34\n');
    assert.equal(sqlite(storePath, "SELECT count(*) FROM pragma_table_info('makosh_kernel_control_store_metadata') WHERE name='interrupted_probe';"), '0\n');
  } finally {
    await rm(dataDir, { recursive: true, force: true });
  }
});

test('an OS-level database growth failure cannot expose a partial migration', async () => {
  const dataDir = await pristineDataDirectory('makosh-migration-file-limit-');
  try {
    const storePath = await replaceWithVersionOne(dataDir, 18);
    sqlite(storePath, 'DROP TRIGGER stop_at_source_version;');
    const binary = fileURLToPath(new URL('target/debug/makosh-kernel', backend));
    const limited = runWithFileSizeLimit(
      binary,
      dataDir,
      (await stat(storePath)).size,
      ['status'],
    );
    const remainedRestricted = limited.status !== 0
      || /^state=recovery_only\ncontrol_store=unavailable\n$/.test(limited.stdout);
    assert.equal(remainedRestricted, true, limited.stderr);

    const version = Number(sqlite(storePath, 'SELECT schema_version FROM makosh_kernel_control_store_metadata;').trim());
    assert.ok(version >= 1 && version < 34);
    assertTrustworthy(dataDir);
  } finally {
    await rm(dataDir, { recursive: true, force: true });
  }
});
