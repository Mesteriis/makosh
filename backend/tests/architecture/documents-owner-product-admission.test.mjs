import assert from 'node:assert/strict';
import { globSync } from 'node:fs';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import test from 'node:test';

const PROJECT_ROOT = resolve(import.meta.dirname, '../../..');
const absolute = (path) => resolve(PROJECT_ROOT, path);
const read = async (path) => readFile(absolute(path), 'utf8');

const packages = [
  'makosh-documents-api',
  'makosh-documents-assembly',
  'makosh-documents-core',
  'makosh-documents-persistence',
  'makosh-documents-runtime',
];
const capabilities = [
  'documents.blob.v1',
  'documents.client.v1',
  'documents.lifecycle.event.v1',
  'documents.storage.v1',
];

test('Task 18 records Documents as implemented but not production-admitted', async () => {
  const policy = JSON.parse(await read('backend/architecture/policy.json'));
  assert.equal(policy.implementation.currentSlice, 'speech_to_text_whisper_admission_v1');
  assert.equal(policy.implementation.productionPackages.length, 283);
  assert.equal(policy.implementation.ownerInventory.businessCapabilities.length, 253);
  assert.deepEqual(
    policy.implementation.productionPackages
      .filter(({ owner }) => owner === 'documents')
      .map(({ name }) => name)
      .sort(),
    [],
  );
  assert.deepEqual(
    policy.implementation.ownerInventory.businessCapabilities.filter((id) => id.startsWith('documents.')),
    [],
  );
  assert.equal(policy.implementation.ownerInventory.domains.includes('documents'), false);
});

test('Task 18 creates exactly five byte-free Documents packages', async () => {
  const manifests = globSync('**/Cargo.toml', { cwd: absolute('backend'), exclude: ['target/**'] });
  assert.equal(manifests.length, 421);
  for (const name of packages) {
    const manifest = await read(`backend/src/${name.replace('makosh-', '')}/Cargo.toml`);
    assert.match(manifest, new RegExp(`name = "${name}"`));
  }
  const workspace = await read('backend/Cargo.toml');
  assert.equal(packages.every((name) => workspace.includes(`src/${name.replace('makosh-', '')}`)), true);
});

test('Task 18 owns exact lifecycle, custody and sanitized event contracts', async () => {
  const [proto, admission] = await Promise.all([
    read('backend/src/documents-api/proto/makosh/documents/client/v1/documents.proto'),
    read('backend/src/documents-runtime/src/admission.rs'),
  ]);
  for (const rpc of [
    'Create', 'Update', 'SetState', 'AttachBlob', 'ReleaseBlob', 'AddSource',
    'RemoveSource', 'Get', 'List', 'Search', 'ListSources',
  ]) assert.match(proto, new RegExp(`rpc ${rpc}\\(`));
  for (const capability of capabilities) assert.equal(admission.includes(capability), true);
  assert.match(proto, /message DocumentChangedV1/);
  assert.doesNotMatch(proto, /content_bytes|storage_path|private_locator|provider_account|arbitrary/i);
});

test('Task 18 storage is owner-local FORCE RLS and relay-safe', async () => {
  const [migration, repository] = await Promise.all([
    read('backend/src/documents-persistence/migrations/0001_documents_owner.sql'),
    read('backend/src/documents-persistence/src/repository.rs'),
  ]);
  for (const table of [
    'documents_records', 'documents_sources', 'documents_client_operations',
    'documents_blob_operations', 'documents_outbox',
  ]) {
    assert.match(migration, new RegExp(`CREATE TABLE makosh_data\\.${table}`));
    assert.match(migration, new RegExp(`ALTER TABLE makosh_data\\.${table} FORCE ROW LEVEL SECURITY`));
  }
  assert.match(repository, /set_config\('makosh\.logical_owner_id'/);
  assert.match(repository, /FOR UPDATE SKIP LOCKED/);
  assert.doesNotMatch(migration, /content_bytes|storage_path|raw_payload/i);
});

test('Task 18 compiles frontend and actual managed/release evidence', async () => {
  const [generator, surfaces, adapters, gateway, runner, flow, materializer] = await Promise.all([
    read('frontend/scripts/generate-proto.mjs'),
    read('frontend/src/platform/client-runtime/clientSurfaces.ts'),
    read('frontend/src/app/client-surfaces/compiledClientSurfaceAdapters.ts'),
    read('backend/src/api/gateway/session_contract/src/browser/client_bootstrap.rs'),
    read('backend/scripts/test-authenticated-storage.mjs'),
    read('backend/tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/documents_managed_flow.rs'),
    read('backend/scripts/materialize-dev-release.sh'),
  ]);
  assert.equal(generator.includes('documents-api'), true);
  assert.match(surfaces, /routeId: 'documents'[\s\S]*adapterId: 'documents-owner'/);
  assert.match(adapters, /'documents-owner'/);
  assert.match(gateway, /Self::Documents => Some\("documents\.client\.v1"\)/);
  assert.equal(runner.includes('managed_documents_lifecycle_custody_replays_and_restarts_with_owner_rls'), true);
  assert.match(flow, /assert_review_owner_rls_v1/);
  assert.match(materializer, /makosh-documents-runtime/);
  assert.deepEqual(globSync('src/domains/documents/api/**/*', { cwd: absolute('frontend') }), []);
});
