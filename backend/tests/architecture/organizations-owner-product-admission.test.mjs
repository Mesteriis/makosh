import assert from 'node:assert/strict';
import { globSync } from 'node:fs';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import test from 'node:test';

const PROJECT_ROOT = resolve(import.meta.dirname, '../../..');
const absolute = (path) => resolve(PROJECT_ROOT, path);
const read = async (path) => readFile(absolute(path), 'utf8');

const packages = [
  'makosh-organizations-api',
  'makosh-organizations-assembly',
  'makosh-organizations-core',
  'makosh-organizations-persistence',
  'makosh-organizations-runtime',
];
const capabilities = [
  'organizations.client.v1',
  'organizations.lifecycle.event.v1',
  'organizations.storage.v1',
];

test('Task 17 records Organizations as implemented but not production-admitted', async () => {
  const policy = JSON.parse(await read('backend/architecture/policy.json'));
  assert.equal(policy.implementation.currentSlice, 'speech_to_text_whisper_admission_v1');
  assert.equal(policy.implementation.productionPackages.length, 283);
  assert.equal(policy.implementation.ownerInventory.businessCapabilities.length, 253);
  assert.deepEqual(
    policy.implementation.productionPackages
      .filter(({ owner }) => owner === 'organizations')
      .map(({ name }) => name)
      .sort(),
    [],
  );
  assert.deepEqual(
    policy.implementation.ownerInventory.businessCapabilities.filter((id) => id.startsWith('organizations.')),
    [],
  );
  assert.equal(policy.implementation.ownerInventory.domains.includes('organizations'), false);
});

test('Task 17 creates exactly five provider-neutral Organizations packages', async () => {
  const manifests = globSync('**/Cargo.toml', { cwd: absolute('backend'), exclude: ['target/**'] });
  assert.equal(manifests.length, 421);
  for (const name of packages) {
    const manifest = await read(`backend/src/${name.replace('makosh-', '')}/Cargo.toml`);
    assert.match(manifest, new RegExp(`name = "${name}"`));
    assert.doesNotMatch(manifest, /clearbit|crunchbase|linkedin|provider|credential/i);
  }
  const workspace = await read('backend/Cargo.toml');
  assert.equal(packages.every((name) => workspace.includes(`src/${name.replace('makosh-', '')}`)), true);
});

test('Task 17 owns exact lifecycle, provenance and sanitized event contracts', async () => {
  const [proto, admission] = await Promise.all([
    read('backend/src/organizations-api/proto/makosh/organizations/client/v1/organizations.proto'),
    read('backend/src/organizations-runtime/src/admission.rs'),
  ]);
  for (const rpc of [
    'Create', 'Update', 'SetState', 'AddSource', 'RemoveSource',
    'Get', 'List', 'Search', 'ListSources',
  ]) assert.match(proto, new RegExp(`rpc ${rpc}\\(`));
  for (const capability of capabilities) assert.equal(admission.includes(capability), true);
  assert.match(proto, /message OrganizationChangedV1/);
  assert.doesNotMatch(proto, /registration_number|vat|provider_account|credential|private_locator|arbitrary/i);
});

test('Task 17 storage is owner-local FORCE RLS and relay-safe', async () => {
  const [migration, repository] = await Promise.all([
    read('backend/src/organizations-persistence/migrations/0001_organizations_owner.sql'),
    read('backend/src/organizations-persistence/src/repository.rs'),
  ]);
  for (const table of [
    'organizations_records', 'organizations_sources', 'organizations_client_operations',
    'organizations_outbox',
  ]) {
    assert.match(migration, new RegExp(`CREATE TABLE makosh_data\\.${table}`));
    assert.match(migration, new RegExp(`ALTER TABLE makosh_data\\.${table} FORCE ROW LEVEL SECURITY`));
  }
  assert.match(repository, /set_config\('makosh\.logical_owner_id'/);
  assert.match(repository, /FOR UPDATE SKIP LOCKED/);
});

test('Task 17 compiles frontend and actual managed/release evidence', async () => {
  const [generator, surfaces, adapters, gateway, runner, flow, materializer] = await Promise.all([
    read('frontend/scripts/generate-proto.mjs'),
    read('frontend/src/platform/client-runtime/clientSurfaces.ts'),
    read('frontend/src/app/client-surfaces/compiledClientSurfaceAdapters.ts'),
    read('backend/src/api/gateway/session_contract/src/browser/client_bootstrap.rs'),
    read('backend/scripts/test-authenticated-storage.mjs'),
    read('backend/tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/organizations_managed_flow.rs'),
    read('backend/scripts/materialize-dev-release.sh'),
  ]);
  assert.equal(generator.includes('organizations-api'), true);
  assert.match(surfaces, /routeId: 'organizations'[\s\S]*adapterId: 'organizations-owner'/);
  assert.match(adapters, /'organizations-owner'/);
  assert.match(gateway, /Self::Organizations => Some\("organizations\.client\.v1"\)/);
  assert.equal(runner.includes('managed_organizations_lifecycle_replays_and_restarts_with_owner_rls'), true);
  assert.match(flow, /assert_review_owner_rls_v1/);
  assert.match(materializer, /makosh-organizations-runtime/);
  assert.deepEqual(globSync('src/domains/organizations/api/**/*', { cwd: absolute('frontend') }), []);
});
