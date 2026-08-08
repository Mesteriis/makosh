import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

import { validateStorageEntries } from '../../scripts/lib/storage-boundaries.mjs';
import { canonicalPolicyForTests as policy } from './support/canonical-policy.mjs';

function storageEntry(content, overrides = {}) {
  return {
    path: 'modules/tasks/migrations/0001.sql',
    content,
    packageName: 'makosh-tasks-persistence',
    role: 'domain',
    owner: 'tasks',
    surface: 'persistence',
    ...overrides,
  };
}

function codes(violations) {
  return new Set(violations.map(({ code }) => code));
}

test('Storage quarantines policy-invalid owner bundles before bootstrap grants', async () => {
  const [admission, runtime] = await Promise.all([
    readFile(
      new URL('../../src/platform/storage/runtime/src/control/admission.rs', import.meta.url),
      'utf8',
    ),
    readFile(
      new URL('../../src/platform/storage/runtime/src/control/runtime.rs', import.meta.url),
      'utf8',
    ),
  ]);

  assert.match(admission, /admit_storage_bundle\(bundle\)\.is_ok\(\)/);
  assert.match(admission, /desired_bindings[\s\S]*admissible_bundle_keys/);
  assert.match(admission, /desired_bundles[\s\S]*referenced_bundle_keys/);
  assert.match(
    runtime,
    /quarantine_invalid_desired_bindings\(&configuration\)[\s\S]*bootstrap_platform_services/,
  );
});

test('allows schema-qualified owner-local SQL and additive migrations', () => {
  const violations = validateStorageEntries(policy(), [
    storageEntry(`
      CREATE TYPE makosh_data.tasks_state AS ENUM ('open', 'closed');
      CREATE TABLE makosh_data.tasks_items (
        id UUID PRIMARY KEY,
        state makosh_data.tasks_state NOT NULL,
        parent_id UUID REFERENCES makosh_data.tasks_items(id) ON UPDATE CASCADE
      );
      CREATE INDEX tasks_items_id_idx ON makosh_data.tasks_items (id);
      ALTER TABLE makosh_data.tasks_items ADD COLUMN title TEXT;
      UPDATE makosh_data.tasks_items SET title = 'owned' WHERE id = $1;
      SELECT * FROM makosh_data.tasks_items;
    `),
  ]);

  assert.deepEqual(violations, []);
});

test('allows platform persistence only in makosh_platform', () => {
  const violations = validateStorageEntries(policy(), [
    storageEntry(`
      CREATE TABLE makosh_platform.events_outbox (id UUID PRIMARY KEY);
      SELECT * FROM makosh_platform.events_outbox;
    `, {
      packageName: 'makosh-events-postgres',
      role: 'platform',
      owner: 'events',
    }),
  ]);

  assert.deepEqual(violations, []);
});

for (const sqlitePackage of [
  {
    packageName: 'makosh-kernel-control-store-sqlite',
    role: 'core',
    owner: 'kernel',
    table: 'kernel_registrations',
  },
  {
    packageName: 'makosh-vault-store-sqlcipher',
    role: 'platform',
    owner: 'vault',
    table: 'vault_secrets',
  },
]) {
  test(`keeps ${sqlitePackage.packageName} outside PostgreSQL schema rules`, () => {
    const violations = validateStorageEntries(policy(), [
      storageEntry(`CREATE TABLE ${sqlitePackage.table} (id TEXT PRIMARY KEY);`, sqlitePackage),
    ]);

    assert.deepEqual(violations, []);
  });
}

test('retains owner-prefix enforcement for the exact SQLite packages', () => {
  const violations = validateStorageEntries(policy(), [
    storageEntry('CREATE TABLE secrets (id TEXT PRIMARY KEY);', {
      packageName: 'makosh-vault-store-sqlcipher',
      role: 'platform',
      owner: 'vault',
    }),
  ]);

  assert.ok(codes(violations).has('unowned_sql_identifier'));
});

test('does not exempt an unregistered SQLite-looking package from PostgreSQL schemas', () => {
  const violations = validateStorageEntries(policy(), [
    storageEntry('CREATE TABLE tasks_items (id TEXT PRIMARY KEY);', {
      packageName: 'makosh-tasks-sqlite',
    }),
  ]);

  assert.ok(codes(violations).has('unqualified_sql_identifier'));
});

test('rejects SQL files outside a persistence surface', () => {
  const violations = validateStorageEntries(policy(), [
    storageEntry('SELECT * FROM makosh_data.tasks_items;', {
      packageName: 'makosh-tasks-runtime',
      surface: 'runtime',
    }),
  ]);

  assert.ok(codes(violations).has('sql_outside_persistence'));
});

test('rejects raw cross-owner SQL reads and foreign keys', () => {
  const violations = validateStorageEntries(policy(), [
    storageEntry(`
      SELECT * FROM makosh_data.contacts_people;
      CREATE TABLE makosh_data.tasks_items (
        id UUID PRIMARY KEY,
        contact_id UUID REFERENCES makosh_data.contacts_people(id)
      );
    `),
  ]);

  assert.ok(codes(violations).has('cross_owner_sql'));
});

test('prevents AI persistence from reading another owner table for context', () => {
  const violations = validateStorageEntries(policy(), [
    storageEntry('SELECT * FROM makosh_data.tasks_items;', {
      path: 'modules/ai/persistence/src/sql/context.sql',
      packageName: 'makosh-ai-persistence',
      owner: 'ai',
    }),
  ]);

  assert.ok(codes(violations).has('cross_owner_sql'));
});

for (const statement of [
  'INSERT INTO makosh_data.contacts_people (id) VALUES ($1);',
  'UPDATE makosh_data.contacts_people SET name = $2 WHERE id = $1;',
  'DELETE FROM makosh_data.contacts_people WHERE id = $1;',
]) {
  test(`rejects raw cross-owner DML: ${statement.split(' ')[0]}`, () => {
    const violations = validateStorageEntries(policy(), [
      storageEntry(statement, { path: 'modules/tasks/src/sql/write.sql' }),
    ]);

    assert.ok(codes(violations).has('cross_owner_sql'));
  });
}

test('rejects unqualified PostgreSQL identifiers', () => {
  const violations = validateStorageEntries(policy(), [
    storageEntry('CREATE TABLE tasks_items (id UUID PRIMARY KEY);'),
  ]);

  assert.ok(codes(violations).has('unqualified_sql_identifier'));
});

test('rejects a cross-owner PostgreSQL index name while keeping its table qualified', () => {
  const violations = validateStorageEntries(policy(), [
    storageEntry(`
      CREATE INDEX contacts_items_id_idx ON makosh_data.tasks_items (id);
    `),
  ]);

  assert.ok(codes(violations).has('cross_owner_sql'));
});

for (const schema of ['public', 'private', 'makosh_platform']) {
  test(`rejects ${schema} for a domain raw table access`, () => {
    const violations = validateStorageEntries(policy(), [
      storageEntry(`SELECT * FROM ${schema}.tasks_items;`),
    ]);

    assert.ok(codes(violations).has('forbidden_sql_schema'));
  });
}

test('rejects makosh_data for platform raw table access', () => {
  const violations = validateStorageEntries(policy(), [
    storageEntry('SELECT * FROM makosh_data.events_outbox;', {
      packageName: 'makosh-events-postgres',
      role: 'platform',
      owner: 'events',
    }),
  ]);

  assert.ok(codes(violations).has('forbidden_sql_schema'));
});

test('rejects owner-prefixed SQL for an unsupported PostgreSQL role', () => {
  const violations = validateStorageEntries(policy(), [
    storageEntry('SELECT * FROM makosh_data.gateway_state;', {
      packageName: 'makosh-gateway-persistence',
      role: 'api',
      owner: 'gateway',
    }),
  ]);

  assert.ok(codes(violations).has('unsupported_sql_role'));
});

test('allows a versioned makosh_platform technical function instead of raw platform DML', () => {
  const violations = validateStorageEntries(policy(), [
    storageEntry(`
      INSERT INTO makosh_data.tasks_items (id) VALUES ($1);
      SELECT makosh_platform.events_append_outbox_v1($1, $2);
      SELECT * FROM makosh_platform.events_accept_inbox_v1($1);
    `, { path: 'modules/tasks/src/sql/create_task.sql' }),
  ]);

  assert.deepEqual(violations, []);
});

test('rejects unversioned, unlisted and non-platform-owned makosh_platform functions', () => {
  const violations = validateStorageEntries(policy(), [
    storageEntry(`
      SELECT makosh_platform.events_append_outbox($1);
      SELECT makosh_platform.storage_claim_v2($1);
      SELECT makosh_platform.contacts_lookup_v1($1);
    `, { path: 'modules/tasks/src/sql/create_task.sql' }),
  ]);

  assert.ok(codes(violations).has('invalid_platform_function'));
});

test('rejects SQL not associated with a workspace package', () => {
  const violations = validateStorageEntries(policy(), [
    storageEntry('CREATE TABLE makosh_data.tasks_items (id UUID PRIMARY KEY);', {
      packageName: null,
      role: null,
      owner: null,
      surface: null,
    }),
  ]);

  assert.ok(codes(violations).has('orphan_sql'));
});

test('ignores ownership-looking SQL inside comments', () => {
  const violations = validateStorageEntries(policy(), [
    storageEntry(`
      -- SELECT * FROM contacts_people;
      /* CREATE TABLE projects_items (id UUID); */
      CREATE TABLE makosh_data.tasks_items (
        id UUID PRIMARY KEY,
        note TEXT DEFAULT 'DROP TABLE makosh_data.contacts_people'
      );
    `),
  ]);

  assert.deepEqual(violations, []);
});

const forbiddenMigrationCases = [
  ['DROP', 'DROP TABLE makosh_data.tasks_items;'],
  ['TRUNCATE', 'TRUNCATE TABLE makosh_data.tasks_items;'],
  ['rename', 'ALTER TABLE makosh_data.tasks_items RENAME TO tasks_old;'],
  ['destructive ALTER TYPE', 'ALTER TABLE makosh_data.tasks_items ALTER COLUMN id TYPE TEXT;'],
  ['ALTER COLUMN DEFAULT', 'ALTER TABLE makosh_data.tasks_items ALTER COLUMN id SET DEFAULT gen_random_uuid();'],
  ['ROLE', 'CREATE ROLE tasks_runtime;'],
  ['DATABASE', 'CREATE DATABASE tasks;'],
  ['SCHEMA', 'CREATE SCHEMA tasks;'],
  ['EXTENSION', 'CREATE EXTENSION pg_trgm;'],
  ['GRANT', 'GRANT SELECT ON makosh_data.tasks_items TO tasks_runtime;'],
  ['REVOKE', 'REVOKE SELECT ON makosh_data.tasks_items FROM tasks_runtime;'],
  ['DO block', 'DO $$ BEGIN NULL; END $$;'],
  ['dynamic EXECUTE', "EXECUTE 'DELETE FROM makosh_data.tasks_items';"],
  ['prepared SQL', 'PREPARE remove_task AS DELETE FROM makosh_data.tasks_items;'],
  ['function', 'CREATE FUNCTION makosh_data.tasks_touch_v1() RETURNS void AS $$ BEGIN END $$ LANGUAGE plpgsql;'],
  ['trigger', 'CREATE TRIGGER tasks_touch BEFORE UPDATE ON makosh_data.tasks_items EXECUTE FUNCTION makosh_data.tasks_touch_v1();'],
  ['FDW', 'CREATE FOREIGN TABLE makosh_data.tasks_remote (id UUID) SERVER remote;'],
  ['COPY PROGRAM', "COPY makosh_data.tasks_items FROM PROGRAM 'cat /tmp/tasks';"],
  ['ALTER SYSTEM', "ALTER SYSTEM SET shared_buffers = '1GB';"],
  ['CONCURRENTLY', 'CREATE INDEX CONCURRENTLY tasks_id_idx ON makosh_data.tasks_items (id);'],
  ['TABLESPACE', 'ALTER TABLE makosh_data.tasks_items SET TABLESPACE fast_space;'],
  ['LOAD', "LOAD 'unsafe_extension';"],
  ['VACUUM', 'VACUUM makosh_data.tasks_items;'],
  ['REINDEX', 'REINDEX TABLE makosh_data.tasks_items;'],
  ['CLUSTER', 'CLUSTER makosh_data.tasks_items;'],
  ['BEGIN', 'BEGIN; SELECT * FROM makosh_data.tasks_items; COMMIT;'],
  ['START TRANSACTION', 'START TRANSACTION;'],
  ['COMMIT', 'COMMIT;'],
  ['ROLLBACK', 'ROLLBACK;'],
  ['END', 'END TRANSACTION;'],
  ['ABORT', 'ABORT;'],
  ['SAVEPOINT', 'SAVEPOINT before_tasks;'],
  ['RELEASE SAVEPOINT', 'RELEASE SAVEPOINT before_tasks;'],
];

for (const [name, sql] of forbiddenMigrationCases) {
  test(`rejects ${name} in a V1 migration`, () => {
    const violations = validateStorageEntries(policy(), [storageEntry(sql)]);

    assert.ok(codes(violations).has('forbidden_migration_construct'));
  });
}

for (const path of [
  'modules/tasks/migrations/0002.down.sql',
  'modules/tasks/migrations/0002_down.sql',
  'modules/tasks/migrations/down/0002.sql',
]) {
  test(`rejects down migration file ${path}`, () => {
    const violations = validateStorageEntries(policy(), [
      storageEntry('SELECT * FROM makosh_data.tasks_items;', { path }),
    ]);

    assert.ok(codes(violations).has('down_migration'));
  });
}

test('migration heuristics ignore forbidden words in comments and literals', () => {
  const violations = validateStorageEntries(policy(), [
    storageEntry(`
      -- DROP TABLE makosh_data.tasks_items;
      -- BEGIN; LOAD 'unsafe'; SAVEPOINT ignored;
      ALTER TABLE makosh_data.tasks_items
        ADD COLUMN note TEXT DEFAULT
          'TRUNCATE, ALTER SYSTEM, TABLESPACE, COMMIT and ROLLBACK are documentation';
    `),
  ]);

  assert.deepEqual(violations, []);
});
