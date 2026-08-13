import assert from 'node:assert/strict';
import { globSync } from 'node:fs';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import test from 'node:test';

const PROJECT_ROOT = resolve(import.meta.dirname, '../../..');
const absolute = (path) => resolve(PROJECT_ROOT, path);
const read = async (path) => readFile(absolute(path), 'utf8');

const packages = [
  'makosh-relationships-api',
  'makosh-relationships-assembly',
  'makosh-relationships-core',
  'makosh-relationships-persistence',
  'makosh-relationships-runtime',
];
const capabilities = [
  'relationships.client.v1',
  'relationships.lifecycle.event.v1',
  'relationships.storage.v1',
];

test('Task 19 development authority is present without production admission', async () => {
  const [policy, adr] = await Promise.all([
    read('backend/architecture/policy.json').then(JSON.parse),
    read('docs/adr/ADR-0404-confirmed-relationships-owner-and-personas-boundary.md'),
  ]);
  assert.equal(policy.domains.developmentAllowlist.includes('relationships'), true);
  assert.equal(policy.domains.blocked.includes('relationships'), false);
  assert.equal(policy.domains.blocked.includes('obligations'), false);
  assert.equal(policy.domains.developmentAllowlist.includes('decisions'), true);
  assert.match(adr, /Graph.*projection/s);
  assert.match(adr, /Person.*Organization/s);
});

test('Task 19 records Relationships as implemented but not production-admitted', async () => {
  const policy = JSON.parse(await read('backend/architecture/policy.json'));
  assert.equal(policy.implementation.currentSlice, 'speech_to_text_whisper_admission_v1');
  assert.equal(policy.implementation.productionPackages.length, 283);
  assert.equal(policy.implementation.ownerInventory.businessCapabilities.length, 253);
  assert.deepEqual(
    policy.implementation.productionPackages
      .filter(({ owner }) => owner === 'relationships')
      .map(({ name }) => name)
      .sort(),
    [],
  );
  assert.deepEqual(
    policy.implementation.ownerInventory.businessCapabilities.filter((id) => id.startsWith('relationships.')),
    [],
  );
  assert.equal(policy.implementation.ownerInventory.domains.includes('relationships'), false);
});

test('Task 19 creates exactly five Relationships packages', async () => {
  const manifests = globSync('**/Cargo.toml', { cwd: absolute('backend'), exclude: ['target/**'] });
  assert.equal(manifests.length, 421);
  for (const name of packages) {
    const manifest = await read(`backend/src/${name.replace('makosh-', '')}/Cargo.toml`);
    assert.match(manifest, new RegExp(`name = "${name}"`));
  }
});

test('Task 19 contracts are typed temporal confirmed and evidence bounded', async () => {
  const proto = await read('backend/src/relationships-api/proto/makosh/relationships/client/v1/relationships.proto');
  for (const rpc of [
    'Create', 'UpdateValidity', 'End', 'Reactivate', 'AddEvidence',
    'RemoveEvidence', 'Get', 'ListForParticipant', 'ListEvidence',
  ]) assert.match(proto, new RegExp(`rpc ${rpc}\\(`));
  assert.match(proto, /RELATIONSHIP_PARTICIPANT_KIND_PERSON/);
  assert.match(proto, /RELATIONSHIP_PARTICIPANT_KIND_ORGANIZATION/);
  assert.match(proto, /RELATIONSHIP_STATE_CONFIRMED/);
  assert.match(proto, /RELATIONSHIP_STATE_ENDED/);
  assert.doesNotMatch(proto, /confidence|trust_score|raw_payload|private_locator|arbitrary/i);
});

test('Task 19 persistence is FORCE RLS and relay-safe', async () => {
  const [migration, repository] = await Promise.all([
    read('backend/src/relationships-persistence/migrations/0001_relationships_owner.sql'),
    read('backend/src/relationships-persistence/src/repository.rs'),
  ]);
  for (const table of [
    'relationships_records', 'relationships_evidence',
    'relationships_client_operations', 'relationships_outbox',
  ]) {
    assert.match(migration, new RegExp(`CREATE TABLE makosh_data\\.${table}`));
    assert.match(migration, new RegExp(`ALTER TABLE makosh_data\\.${table} FORCE ROW LEVEL SECURITY`));
  }
  assert.match(repository, /set_config\('makosh\.logical_owner_id'/);
  assert.match(repository, /FOR UPDATE SKIP LOCKED/);
});

test('Task 19 activates only the bounded Personas Relationships client', async () => {
  const [personas, surfaces, runner, materializer] = await Promise.all([
    read('frontend/src/domains/personas/api/personas.ts'),
    read('frontend/src/platform/client-runtime/clientSurfaces.ts'),
    read('backend/scripts/test-authenticated-storage.mjs'),
    read('backend/scripts/materialize-dev-release.sh'),
  ]);
  assert.match(personas, /getRelationshipsQueryClient/);
  assert.doesNotMatch(personas, /relationships_unavailable/);
  assert.match(surfaces, /adapterId: 'relationships-owner'/);
  assert.match(runner, /managed_relationships_lifecycle_replays_and_restarts_with_owner_rls/);
  assert.match(materializer, /makosh-relationships-runtime/);
  assert.deepEqual(globSync('src/graph-*', { cwd: absolute('backend') }).sort(), [
    'src/graph-api',
    'src/graph-assembly',
    'src/graph-core',
    'src/graph-persistence',
    'src/graph-runtime',
  ]);
});
