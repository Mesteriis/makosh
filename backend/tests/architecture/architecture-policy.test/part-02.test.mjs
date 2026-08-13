import assert from 'node:assert/strict';
import { mkdir, mkdtemp, rm, symlink, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';

import { validatePolicy } from '../../../scripts/lib/policy-schema.mjs';
import { collectSourceEntries } from '../../../scripts/lib/repository-scan.mjs';
import { validateSourceEntries } from '../../../scripts/lib/source-boundaries.mjs';
import { canonicalPolicyForTests as policy } from '../support/canonical-policy.mjs';

import { codes } from './support.mjs';

test('requires the exact canonical durable envelope policy', () => {
  const mutations = [
    (events) => { events.protocolPackage = 'makosh-events-contracts'; },
    (events) => { events.role = 'core'; },
    (events) => { events.owner = 'kernel'; },
    (events) => { events.surface = 'implementation'; },
    (events) => { events.serialization = 'json'; },
    (events) => { events.envelopeMajorVersion = 2; },
    (events) => { events.kinds = events.kinds.filter((kind) => kind !== 'ack'); },
    (events) => { events.kinds = ['command', 'event', 'observation', 'result', 'result']; },
    (events) => { events.kinds.push('dead_letter'); },
    (events) => { events.kindMetadata = 'flat_fields'; },
    (events) => { events.payloadBinding = 'protobuf_type_url'; },
    (events) => { events.payloadVisibility = 'decoded_by_event_hub'; },
    (events) => { events.outboxPublishMode = 'relay_reencodes'; },
    (events) => { events.clientEnvelopeReuseAllowed = true; },
    (events) => { events.brokerAckIsEnvelopeAck = true; },
    (events) => { events.unknownMajorVersion = 'best_effort'; },
    (events) => { events.automaticFormatFallbackEnabled = true; },
    (events) => { events.forbiddenPayloadData.pop(); },
    (events) => { events.forbiddenPayloadData.push('public_status'); },
    (events) => { events.forbiddenDependencies.pop(); },
    (events) => { events.forbiddenDependencies.push('reqwest'); },
    (events) => { events.unversionedCompatibilityAlias = true; },
  ];

  for (const mutate of mutations) {
    const invalid = policy();
    mutate(invalid.events);

    assert.ok(codes(validatePolicy(invalid)).has('events_protocol_policy'));
  }

  const missing = policy();
  delete missing.events;
  assert.ok(codes(validatePolicy(missing)).has('events_protocol_policy'));
});


test('requires an exact integration ingress package allowlist', () => {
  const invalid = policy();
  invalid.dependencies.integrationDomainContractPackages = ['communications'];

  assert.ok(codes(validatePolicy(invalid)).has('dependency_policy'));
});

test('requires an exact engine domain contract package allowlist', () => {
  const invalid = policy();
  invalid.dependencies.engineDomainContractPackages = ['communications'];

  assert.ok(codes(validatePolicy(invalid)).has('dependency_policy'));
});

test('requires an exact engine engine contract package allowlist', () => {
  const invalid = policy();
  invalid.dependencies.engineEngineContractPackages = ['attachment_archive_inspection'];

  assert.ok(codes(validatePolicy(invalid)).has('dependency_policy'));
});

test('requires an exact integration engine contract package allowlist', () => {
  const invalid = policy();
  invalid.dependencies.integrationEngineContractPackages = ['attachment_security'];

  assert.ok(codes(validatePolicy(invalid)).has('dependency_policy'));
});

test('requires an exact integration workflow contract package allowlist', () => {
  const invalid = policy();
  invalid.dependencies.integrationWorkflowContractPackages = ['call_transcription'];

  assert.ok(codes(validatePolicy(invalid)).has('dependency_policy'));
});



test('requires explicit compile-isolation policy', () => {
  const invalid = policy();
  invalid.compileIsolation.forbidSameOwnerRuntimeDependencies = false;

  assert.ok(codes(validatePolicy(invalid)).has('compile_isolation_policy'));
});



test('requires an explicit host-only provider execution registry', () => {
  const invalid = policy();
  invalid.integrations.hostOnlyProviderExecutionOwners = [];

  assert.ok(codes(validatePolicy(invalid)).has('integration_policy'));
});



test('requires the exact Storage Control topology and fail-closed lifecycle policy', () => {
  const canonical = policy().storage;
  assert.deepEqual(canonical.fixedSchemas, [
    'makosh_data',
    'makosh_platform',
    'makosh_extensions',
  ]);
  assert.ok(canonical.bindingFields.includes('role_epoch'));
  assert.ok(canonical.bindingFields.includes('storage_bundle_digest'));
  assert.deepEqual(canonical.testSupportPostgresClientAllowlist, [
    'makosh-communication-delayed-delivery-testkit:dev:sqlx',
    'makosh-events-jetstream-testkit:dev:sqlx',
    'makosh-kernel-recovery-testkit:dev:sqlx',
    'makosh-scheduler-testkit:dev:sqlx',
  ]);

  const mutations = [
    (storage) => { storage.runtimeComponent = 'storage_manager'; },
    (storage) => { storage.managementMode = 'external_or_managed'; },
    (storage) => { storage.clusterTopology = 'one_cluster_per_module'; },
    (storage) => { storage.databaseTopology = 'one_database_per_module'; },
    (storage) => { storage.fixedSchemas = ['public']; },
    (storage) => { storage.runtimePath = 'direct_postgresql'; },
    (storage) => { storage.poolMode = 'session'; },
    (storage) => { storage.moduleSelfMigrationsEnabled = true; },
    (storage) => { storage.destructiveMigrationsEnabled = true; },
    (storage) => { storage.kernelSqlProxyEnabled = true; },
    (storage) => { storage.regexOnlyValidationAllowed = true; },
    (storage) => { storage.crossOwnerBusinessSqlEnabled = true; },
    (storage) => { storage.directSharedTechnicalDmlEnabled = true; },
    (storage) => { storage.sharedTechnicalFunctions.pop(); },
    (storage) => { storage.pgbouncerSoleBudgetBoundary = true; },
    (storage) => { storage.bindingFields.pop(); },
    (storage) => { storage.revocationSequence.pop(); },
    (storage) => { storage.forbiddenProtocolDependencies.pop(); },
    (storage) => { storage.testSupportPostgresClientAllowlist = []; },
  ];

  for (const mutate of mutations) {
    const invalid = policy();
    mutate(invalid.storage);
    assert.ok(codes(validatePolicy(invalid)).has('storage_policy'));
  }

  const missing = policy();
  delete missing.storage;
  assert.ok(codes(validatePolicy(missing)).has('storage_policy'));
});



test('requires explicit production and test-only workspace roots', () => {
  const invalid = policy();
  delete invalid.tests;

  assert.ok(codes(validatePolicy(invalid)).has('test_layout_policy'));
});



test('requires explicit source roots and extensions', () => {
  const invalid = policy();
  invalid.source.roots = [];

  assert.ok(codes(validatePolicy(invalid)).has('source_policy'));
});



test('requires an explicit single-layout policy', () => {
  const invalid = policy();
  delete invalid.layout;

  assert.ok(codes(validatePolicy(invalid)).has('layout_policy'));
});



for (const owner of ['decisions']) {
  test(`rejects a production path for blocked domain ${owner}`, () => {
    const blockedPolicy = policy();
    blockedPolicy.domains.developmentAllowlist = blockedPolicy.domains.developmentAllowlist.filter(
      (value) => value !== owner,
    );
    blockedPolicy.domains.blocked.push(owner);
    const violations = validateSourceEntries(blockedPolicy, [
      { path: `modules/${owner}/src/lib.rs`, content: '' },
    ]);

    assert.ok(codes(violations).has('blocked_source_owner'));
  });
}



test('rejects singular aliases of blocked owners in production paths', () => {
  const blockedPolicy = policy();
  blockedPolicy.domains.developmentAllowlist = blockedPolicy.domains.developmentAllowlist.filter(
    (owner) => owner !== 'projects',
  );
  blockedPolicy.domains.blocked.push('projects');
  const violations = validateSourceEntries(blockedPolicy, [
    { path: 'modules/project/src/lib.rs', content: '' },
  ]);

  assert.ok(codes(violations).has('blocked_source_owner'));
});



for (const owner of ['graph', 'timeline', 'search', 'context']) {
  test(`rejects a production path for blocked projection ${owner}`, () => {
    const violations = validateSourceEntries(policy(), [
      { path: `crates/makosh-${owner}-runtime/src/lib.rs`, content: '' },
    ]);

    assert.ok(codes(violations).has('blocked_projection'));
  });
}



test('rejects SQL ownership for a blocked domain', () => {
  const blockedPolicy = policy();
  blockedPolicy.domains.developmentAllowlist = blockedPolicy.domains.developmentAllowlist.filter(
    (owner) => owner !== 'projects',
  );
  blockedPolicy.domains.blocked.push('projects');
  const violations = validateSourceEntries(blockedPolicy, [
    {
      path: 'modules/tasks/migrations/0001.sql',
      content: 'CREATE TABLE projects (id UUID PRIMARY KEY);',
    },
  ]);

  assert.ok(codes(violations).has('blocked_sql_owner'));
});

test('Persons decision evidence exception is exact and does not admit a Decisions owner', () => {
  const blockedPolicy = policy();
  blockedPolicy.domains.developmentAllowlist = blockedPolicy.domains.developmentAllowlist.filter(
    (owner) => owner !== 'decisions',
  );
  blockedPolicy.domains.blocked.push('decisions');
  const exact = validateSourceEntries(blockedPolicy, [{
    path: 'src/persons-persistence/migrations/0002_persons_durable.sql',
    content: 'CREATE TABLE makosh_data.persons_decision_receipts (decision_id BYTEA);',
  }]);
  assert.ok(!codes(exact).has('blocked_sql_owner'));

  for (const entry of [
    {
      path: 'src/persons-persistence/migrations/0003_decisions.sql',
      content: 'CREATE TABLE makosh_data.persons_decision_receipts (decision_id BYTEA);',
    },
    {
      path: 'src/persons-persistence/migrations/0002_persons_durable.sql',
      content: 'CREATE TABLE makosh_data.decisions (decision_id BYTEA);',
    },
  ]) {
    assert.ok(codes(validateSourceEntries(blockedPolicy, [entry])).has('blocked_sql_owner'));
  }
});



test('does not treat SQL comments as ownership declarations', () => {
  const violations = validateSourceEntries(policy(), [
    {
      path: 'modules/tasks/migrations/0001.sql',
      content: '-- CREATE TABLE projects (id UUID);\nCREATE TABLE tasks (id UUID);',
    },
  ]);

  assert.deepEqual(violations, []);
});



test('allows source paths for enabled domains including Knowledge and AI', () => {
  const violations = validateSourceEntries(policy(), [
    { path: 'modules/communications/src/lib.rs', content: '' },
    { path: 'modules/persons/src/lib.rs', content: '' },
    { path: 'modules/organizations/src/lib.rs', content: '' },
    { path: 'modules/tasks/src/lib.rs', content: '' },
    { path: 'modules/calendar/src/lib.rs', content: '' },
    { path: 'modules/documents/src/lib.rs', content: '' },
    { path: 'modules/knowledge/src/lib.rs', content: '' },
    { path: 'modules/ai/src/lib.rs', content: '' },
  ]);

  assert.deepEqual(violations, []);
});



test('does not reject a cohesive production source file by line count', () => {
  const violations = validateSourceEntries(policy(), [{
    path: 'src/platform/kernel/src/main.rs',
    content: Array.from({ length: 801 }, () => 'let value = 1;').join('\n'),
  }]);

  assert.ok(!codes(violations).has('production_source_too_large'));
});



test('does not infer ownership from ordinary filenames below an allowed owner', () => {
  const violations = validateSourceEntries(policy(), [
    { path: 'modules/tasks/src/request_context.rs', content: '' },
    { path: 'crates/makosh-documents-runtime/src/search_adapter.rs', content: '' },
  ]);

  assert.deepEqual(violations, []);
});



for (const directory of ['tests', 'fixtures', 'snapshots']) {
  test(`rejects package-local ${directory} directories in production source`, () => {
    const violations = validateSourceEntries(policy(), [
      { path: `src/domains/tasks/runtime/${directory}/api.rs`, content: '' },
    ]);

    assert.ok(codes(violations).has('test_in_production_source'));
  });
}



for (const path of [
  'src/domains/tasks/implementation/src/task_test.rs',
  'src/domains/tasks/implementation/src/test_task.rs',
]) {
  test(`rejects production test filename ${path}`, () => {
    const violations = validateSourceEntries(policy(), [{ path, content: '' }]);

    assert.ok(codes(violations).has('test_in_production_source'));
  });
}



test('rejects snapshot files in production source', () => {
  const violations = validateSourceEntries(policy(), [{
    path: 'src/domains/tasks/implementation/src/state.snap',
    content: '',
  }]);

  assert.ok(codes(violations).has('test_in_production_source'));
});



test('allows cfg(test) modules because they are excluded from production builds', () => {
  const violations = validateSourceEntries(policy(), [{
    path: 'src/domains/tasks/implementation/src/lib.rs',
    content: '#[cfg(test)]\nmod tests;',
  }]);

  assert.deepEqual(violations, []);
});



test('allows compound cfg(test) modules because they are excluded from production builds', () => {
  const violations = validateSourceEntries(policy(), [{
    path: 'src/domains/tasks/implementation/src/lib.rs',
    content: '#[cfg(all(test, feature = "slow"))]\nmod slow_tests;',
  }]);

  assert.deepEqual(violations, []);
});



test('does not treat cfg(test) text inside a Rust comment as an inline test', () => {
  const violations = validateSourceEntries(policy(), [{
    path: 'src/domains/tasks/implementation/src/lib.rs',
    content: '// #[cfg(test)] is forbidden by ADR-0211',
  }]);

  assert.deepEqual(violations, []);
});



test('filesystem scan reads Rust content and exposes nested test paths to policy', async (context) => {
  const root = await mkdtemp(join(tmpdir(), 'makosh-architecture-'));
  context.after(() => rm(root, { recursive: true, force: true }));
  const packageSource = join(root, 'src', 'domains', 'tasks', 'implementation', 'src');
  await mkdir(join(packageSource, 'tests'), { recursive: true });
  await writeFile(join(packageSource, 'lib.rs'), '#[cfg(test)]\nmod tests;\n');
  await writeFile(join(packageSource, 'tests', 'api.rs'), 'fn helper() {}\n');
  await symlink(join(root, 'outside-source'), join(packageSource, 'linked-source'));

  const entries = await collectSourceEntries(root, policy());
  const violations = validateSourceEntries(policy(), entries);

  assert.match(
    entries.find(({ path }) => path.endsWith('/lib.rs'))?.content ?? '',
    /cfg\(test\)/u,
  );
  assert.ok(entries.some(({ path }) => path.includes('/tests/')));
  assert.ok(entries.some(({ isSymbolicLink }) => isSymbolicLink === true));
  assert.ok(codes(violations).has('source_symlink'));
  assert.ok(codes(violations).has('source_symlink'));
});
