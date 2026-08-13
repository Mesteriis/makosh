import assert from 'node:assert/strict';
import { globSync } from 'node:fs';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import test from 'node:test';

const PROJECT_ROOT = resolve(import.meta.dirname, '../../..');
const absolute = (path) => resolve(PROJECT_ROOT, path);
const read = async (path) => readFile(absolute(path), 'utf8');

const packages = [
  'makosh-projects-api',
  'makosh-projects-assembly',
  'makosh-projects-core',
  'makosh-projects-persistence',
  'makosh-projects-runtime',
];
const capabilities = [
  'projects.client.v1',
  'projects.lifecycle.event.v1',
  'projects.storage.v1',
];

test('Task 20 authority lifts only the Projects development freeze', async () => {
  const [policy, adr] = await Promise.all([
    read('backend/architecture/policy.json').then(JSON.parse),
    read('docs/adr/ADR-0405-projects-owner-expected-outcomes-and-product-boundary.md'),
  ]);
  assert.equal(policy.domains.developmentAllowlist.includes('projects'), true);
  assert.equal(policy.domains.blocked.includes('projects'), false);
  assert.deepEqual(policy.domains.blocked, []);
  assert.match(adr, /expected outcomes/i);
  assert.match(adr, /Graph.*projection/s);
});

test('Task 20 records Projects as implemented but not production-admitted', async () => {
  const policy = JSON.parse(await read('backend/architecture/policy.json'));
  assert.equal(policy.implementation.currentSlice, 'speech_to_text_whisper_admission_v1');
  assert.equal(policy.implementation.productionPackages.length, 283);
  assert.equal(policy.implementation.ownerInventory.businessCapabilities.length, 253);
  assert.deepEqual(
    policy.implementation.productionPackages.filter(({ owner }) => owner === 'projects'),
    [],
  );
  assert.deepEqual(
    policy.implementation.ownerInventory.businessCapabilities.filter((id) => id.startsWith('projects.')),
    [],
  );
});

test('Task 20 creates exactly five Projects packages', async () => {
  const manifests = globSync('**/Cargo.toml', { cwd: absolute('backend'), exclude: ['target/**'] });
  assert.equal(manifests.length, 421);
  for (const name of packages) {
    const manifest = await read(`backend/src/${name.replace('makosh-', '')}/Cargo.toml`);
    assert.match(manifest, new RegExp(`name = "${name}"`));
  }
});

test('Task 20 contract owns typed project outcome and reference lifecycle', async () => {
  const proto = await read('backend/src/projects-api/proto/makosh/projects/client/v1/projects.proto');
  for (const rpc of [
    'Create', 'Update', 'SetState', 'AddOutcome', 'UpdateOutcome',
    'SetOutcomeState', 'RemoveOutcome', 'AddReference', 'RemoveReference',
    'Get', 'List', 'ListOutcomes', 'ListReferences',
  ]) assert.match(proto, new RegExp(`rpc ${rpc}\\(`));
  assert.match(proto, /PROJECT_STATE_COMPLETED/);
  assert.match(proto, /PROJECT_OUTCOME_STATE_ACHIEVED/);
  assert.match(proto, /PROJECT_REFERENCE_KIND_DOCUMENT/);
  assert.doesNotMatch(proto, /raw_payload|private_locator|credential|arbitrary|graph_node/i);
});

test('Task 20 persistence is FORCE RLS and relay-safe', async () => {
  const [migration, repository] = await Promise.all([
    read('backend/src/projects-persistence/migrations/0001_projects_owner.sql'),
    read('backend/src/projects-persistence/src/repository.rs'),
  ]);
  for (const table of [
    'projects_records', 'projects_outcomes', 'projects_references',
    'projects_client_operations', 'projects_outbox',
  ]) {
    assert.match(migration, new RegExp(`CREATE TABLE makosh_data\\.${table}`));
    assert.match(migration, new RegExp(`ALTER TABLE makosh_data\\.${table} FORCE ROW LEVEL SECURITY`));
  }
  assert.match(repository, /set_config\('makosh\.logical_owner_id'/);
  assert.match(repository, /FOR UPDATE SKIP LOCKED/);
});

test('Task 20 replaces historical REST with one generated Projects surface', async () => {
  const [api, surfaces, layout, runner, materializer] = await Promise.all([
    read('frontend/src/domains/projects/api/projects.ts'),
    read('frontend/src/platform/client-runtime/clientSurfaces.ts'),
    read('frontend/src/app/layout/AppLayoutRoot.vue'),
    read('backend/scripts/test-authenticated-storage.mjs'),
    read('backend/scripts/materialize-dev-release.sh'),
  ]);
  assert.match(api, /getProjectsQueryClient/);
  assert.doesNotMatch(api, /\/api\/v1\/projects/);
  assert.match(surfaces, /adapterId: 'projects-owner'/);
  assert.match(layout, /ProjectsWorkspaceView/);
  assert.match(runner, /managed_projects_lifecycle_replays_and_restarts_with_owner_rls/);
  assert.match(materializer, /makosh-projects-runtime/);
});
