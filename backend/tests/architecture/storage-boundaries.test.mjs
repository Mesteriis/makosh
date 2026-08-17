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

test('Storage startup logging remains bounded and never serializes migration payloads', async () => {
  const runtime = await readFile(
    new URL('../../src/platform/storage/runtime/src/control/runtime.rs', import.meta.url),
    'utf8',
  );

  assert.match(runtime, /event = "storage\.configuration\.loaded"/);
  assert.match(runtime, /bindings\.desired_count = configuration\.desired_bindings\.len\(\)/);
  assert.match(runtime, /bundles\.desired_count = configuration\.desired_bundles\.len\(\)/);
  assert.doesNotMatch(runtime, /payload\.configuration = \?configuration/);
});

test('PgBouncer transaction-mode runtime clients disable persistent statement caching', async () => {
  const runtimeConnections = [
    '../../src/platform/scheduler/persistence/src/store/connection.rs',
    '../../src/mail-persistence/src/durable.rs',
    '../../src/mail-persons-sync-persistence/src/repository.rs',
    '../../src/telegram-persistence/src/durable.rs',
    '../../src/whatsapp-persistence/src/durable.rs',
    '../../src/zulip-persistence/src/lib.rs',
  ];

  for (const path of runtimeConnections) {
    const source = await readFile(new URL(path, import.meta.url), 'utf8');
    assert.match(
      source,
      /PgConnectOptions::new\(\)[\s\S]*?\.statement_cache_capacity\(0\)[\s\S]*?\.database\(/,
      path,
    );
  }
});

test('development PgBouncer tracks prepared statements while transaction pooling', async () => {
  const configuration = await readFile(
    new URL('../../development/authenticated/pgbouncer.ini', import.meta.url),
    'utf8',
  );

  assert.match(configuration, /^pool_mode = transaction$/m);
  assert.match(configuration, /^max_prepared_statements = [1-9][0-9]*$/m);
});

test('incremental Storage binding apply resolves only the new runtime credential', async () => {
  const [apply, bindings] = await Promise.all([
    readFile(
      new URL('../../src/platform/storage/runtime/src/control/apply.rs', import.meta.url),
      'utf8',
    ),
    readFile(
      new URL('../../src/platform/storage/runtime/src/admin/bindings.rs', import.meta.url),
      'utf8',
    ),
  ]);

  assert.match(
    apply,
    /resolve_runtime_credential\(channel, configuration, binding\.clone\(\)\)/,
  );
  assert.doesNotMatch(apply, /resolve_runtime_credentials\(channel, configuration\)/);
  assert.match(
    apply,
    /reconcile_authorized_roles\([\s\S]*std::slice::from_ref\(&runtime_credential\)/,
  );
  assert.match(
    apply,
    /apply_authorized_bindings\([\s\S]*&configuration\.desired_bindings/,
  );
  assert.match(bindings, /runtime_bindings: &\[StorageBindingV1\]/);
  assert.doesNotMatch(bindings, /RuntimeRoleCredentialV1/);
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

test('allows only the exact byte-bound Tasks lifecycle migration exception', async () => {
  const content = await readFile(
    new URL('../../src/tasks-persistence/migrations/0002_tasks_lifecycle_owner_rls.sql', import.meta.url),
    'utf8',
  );
  const exact = storageEntry(content, {
    path: 'src/tasks-persistence/migrations/0002_tasks_lifecycle_owner_rls.sql',
  });
  assert.equal(codes(validateStorageEntries(policy(), [exact])).has('forbidden_migration_construct'), false);

  for (const changed of [
    { ...exact, path: 'src/tasks-persistence/migrations/0002_tasks_lifecycle_alias.sql' },
    { ...exact, content: `${content}\n-- edited` },
  ]) {
    assert.equal(codes(validateStorageEntries(policy(), [changed])).has('forbidden_migration_construct'), true);
  }
});

test('allows only the exact byte-bound Knowledge lifecycle migration exception', async () => {
  const content = await readFile(
    new URL('../../src/knowledge-persistence/migrations/0002_knowledge_lifecycle_owner_rls.sql', import.meta.url),
    'utf8',
  );
  const exact = storageEntry(content, {
    path: 'src/knowledge-persistence/migrations/0002_knowledge_lifecycle_owner_rls.sql',
    packageName: 'makosh-knowledge-persistence',
    owner: 'knowledge',
  });
  assert.equal(codes(validateStorageEntries(policy(), [exact])).has('forbidden_migration_construct'), false);

  for (const changed of [
    { ...exact, path: 'src/knowledge-persistence/migrations/0002_knowledge_lifecycle_alias.sql' },
    { ...exact, content: `${content}\n-- edited` },
  ]) {
    assert.equal(codes(validateStorageEntries(policy(), [changed])).has('forbidden_migration_construct'), true);
  }
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
      SELECT * FROM makosh_data.persons_people;
      CREATE TABLE makosh_data.tasks_items (
        id UUID PRIMARY KEY,
        person_id UUID REFERENCES makosh_data.persons_people(id)
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
  'INSERT INTO makosh_data.persons_people (id) VALUES ($1);',
  'UPDATE makosh_data.persons_people SET name = $2 WHERE id = $1;',
  'DELETE FROM makosh_data.persons_people WHERE id = $1;',
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
      CREATE INDEX persons_items_id_idx ON makosh_data.tasks_items (id);
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
  ['DISABLE named trigger', 'ALTER TABLE makosh_data.tasks_items DISABLE TRIGGER tasks_touch;'],
  ['DISABLE ALL triggers', 'ALTER TABLE makosh_data.tasks_items DISABLE TRIGGER ALL;'],
  ['DISABLE USER triggers', 'ALTER TABLE makosh_data.tasks_items DISABLE TRIGGER USER;'],
  ['ENABLE REPLICA trigger', 'ALTER TABLE makosh_data.tasks_items ENABLE REPLICA TRIGGER tasks_touch;'],
  ['ENABLE ALWAYS trigger', 'ALTER TABLE makosh_data.tasks_items ENABLE ALWAYS TRIGGER tasks_touch;'],
  ['ALTER TRIGGER rename', 'ALTER TRIGGER tasks_touch ON makosh_data.tasks_items RENAME TO tasks_touch_bypass;'],
  ['SET session replication role', 'SET session_replication_role = replica;'],
  ['RESET session replication role', 'RESET session_replication_role;'],
  ['set_config session replication role local', "SELECT set_config('session_replication_role', 'replica', true);"],
  ['set_config session replication role session', "SELECT set_config('session_replication_role', 'replica', false);"],
  ['pg_catalog set_config', "SELECT pg_catalog.set_config('session_replication_role', 'replica', false);"],
  ['quoted set_config', "SELECT \"set_config\"('session_replication_role', 'replica', false);"],
  ['quoted pg_catalog set_config', "SELECT \"pg_catalog\".\"set_config\"('session_replication_role', 'replica', false);"],
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

const personsProfileGuard = `
  CREATE TABLE makosh_data.persons_profiles (logical_owner_id TEXT PRIMARY KEY);
  CREATE FUNCTION makosh_data.persons_reject_profile_history_mutation()
  RETURNS trigger LANGUAGE plpgsql AS $$
  BEGIN
    RAISE EXCEPTION 'persons profile history is immutable' USING ERRCODE = '55000';
  END;
  $$;
  CREATE TRIGGER persons_profiles_immutable
  BEFORE UPDATE OR DELETE ON makosh_data.persons_profiles
  FOR EACH ROW EXECUTE FUNCTION makosh_data.persons_reject_profile_history_mutation();
`;

function personsMigration(content = personsProfileGuard, overrides = {}) {
  return storageEntry(content, {
    path: 'src/persons-persistence/migrations/0001_persons.sql',
    packageName: 'makosh-persons-persistence',
    owner: 'persons',
    ...overrides,
  });
}

test('allows only the exact Persons profile-history immutability guard', () => {
  assert.deepEqual(validateStorageEntries(policy(), [personsMigration()]), []);
  for (const entry of [
    personsMigration(personsProfileGuard, { path: 'modules/persons/migrations/0001.sql' }),
    personsMigration(`${personsProfileGuard}\nCREATE FUNCTION makosh_data.persons_other() RETURNS void LANGUAGE sql AS $$ SELECT 1 $$;`),
    personsMigration(personsProfileGuard.replace('persons_profiles_immutable', 'persons_profiles_mutable')),
    personsMigration(personsProfileGuard.replace('makosh_data.persons_profiles\n', 'makosh_data.persons_current\n')),
    personsMigration(personsProfileGuard.replace("RAISE EXCEPTION 'persons profile history is immutable' USING ERRCODE = '55000';", 'RETURN OLD;')),
    personsMigration(personsProfileGuard.replace("RAISE EXCEPTION 'persons profile history is immutable' USING ERRCODE = '55000';", 'NULL;')),
    personsMigration(personsProfileGuard.replace('persons profile history is immutable', 'altered message')),
    personsMigration(personsProfileGuard.replace("ERRCODE = '55000'", "ERRCODE = 'P0001'")),
    personsMigration(personsProfileGuard.replace('BEGIN', 'BEGIN DELETE FROM makosh_data.persons_profiles;')),
    personsMigration(personsProfileGuard.replace('BEGIN', "BEGIN EXECUTE 'DELETE FROM makosh_data.persons_profiles';")),
    personsMigration(personsProfileGuard.replace('RETURNS trigger LANGUAGE plpgsql', 'RETURNS trigger SECURITY DEFINER LANGUAGE plpgsql')),
    personsMigration(personsProfileGuard.replace('LANGUAGE plpgsql AS', 'LANGUAGE plpgsql SET search_path = makosh_data AS')),
    personsMigration(`${personsProfileGuard}\nPREPARE persons_bypass AS DELETE FROM makosh_data.persons_profiles;`),
    personsMigration(`${personsProfileGuard}\nALTER TABLE makosh_data.persons_profiles DISABLE TRIGGER persons_profiles_immutable;`),
    personsMigration(`${personsProfileGuard}\nALTER TABLE makosh_data.persons_profiles DISABLE TRIGGER ALL;`),
    personsMigration(`${personsProfileGuard}\nALTER TABLE makosh_data.persons_profiles DISABLE TRIGGER USER;`),
    personsMigration(`${personsProfileGuard}\nALTER TABLE makosh_data.persons_profiles ENABLE REPLICA TRIGGER persons_profiles_immutable;`),
    personsMigration(`${personsProfileGuard}\nALTER TABLE makosh_data.persons_profiles ENABLE ALWAYS TRIGGER persons_profiles_immutable;`),
    personsMigration(`${personsProfileGuard}\nALTER TRIGGER persons_profiles_immutable ON makosh_data.persons_profiles RENAME TO persons_profiles_mutable;`),
    personsMigration(`${personsProfileGuard}\nSET session_replication_role = replica;`),
    personsMigration(`${personsProfileGuard}\nRESET session_replication_role;`),
    personsMigration(`${personsProfileGuard}\nSELECT set_config('session_replication_role', 'replica', true);`),
    personsMigration(`${personsProfileGuard}\nSELECT set_config('session_replication_role', 'replica', false);`),
    personsMigration(`${personsProfileGuard}\nSELECT pg_catalog.set_config('session_replication_role', 'replica', false);`),
    personsMigration(`${personsProfileGuard}\nSELECT "set_config"('session_replication_role', 'replica', false);`),
    personsMigration(`${personsProfileGuard}\nSELECT "pg_catalog"."set_config"('session_replication_role', 'replica', false);`),
  ]) {
    assert.ok(codes(validateStorageEntries(policy(), [entry])).has('forbidden_migration_construct'));
  }
});
