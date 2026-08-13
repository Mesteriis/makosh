import assert from 'node:assert/strict';
import { globSync } from 'node:fs';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import test from 'node:test';

const PROJECT_ROOT = resolve(import.meta.dirname, '../../..');
const absolute = (path) => resolve(PROJECT_ROOT, path);
const read = (path) => readFile(absolute(path), 'utf8');

const exactKnowledgeRoutes = [
  '/makosh.knowledge.client.v1.KnowledgeCommandService/AddSource',
  '/makosh.knowledge.client.v1.KnowledgeCommandService/Create',
  '/makosh.knowledge.client.v1.KnowledgeCommandService/RemoveSource',
  '/makosh.knowledge.client.v1.KnowledgeCommandService/SetState',
  '/makosh.knowledge.client.v1.KnowledgeCommandService/Update',
  '/makosh.knowledge.client.v1.KnowledgeQueryService/Get',
  '/makosh.knowledge.client.v1.KnowledgeQueryService/List',
  '/makosh.knowledge.client.v1.KnowledgeQueryService/ListSources',
  '/makosh.knowledge.client.v1.KnowledgeQueryService/Search',
].sort();

test('Task 15 remains staged behind Task 10 without changing admitted Knowledge ownership', async () => {
  const policy = JSON.parse(await read('backend/architecture/policy.json'));
  const implementation = policy.implementation;

  assert.equal(implementation.currentSlice, 'speech_to_text_whisper_admission_v1');
  assert.equal(implementation.productionPackages.length, 283);
  assert.equal(implementation.ownerInventory.businessCapabilities.length, 253);
  assert.deepEqual(
    implementation.productionPackages.filter(({ owner }) => owner === 'knowledge').map(({ name }) => name).sort(),
    [
      'makosh-knowledge-assembly',
      'makosh-knowledge-command-api',
      'makosh-knowledge-core',
      'makosh-knowledge-persistence',
      'makosh-knowledge-runtime',
    ],
  );
});

test('Task 15 exposes nine exact typed routes and one sanitized event', async () => {
  const [api, proto] = await Promise.all([
    read('backend/src/knowledge-command-api/src/lib.rs'),
    read('backend/src/knowledge-command-api/proto/makosh/knowledge/client/v1/knowledge.proto'),
  ]);
  const routes = [...new Set(
    [...api.matchAll(/"(\/makosh\.knowledge\.client\.v1\.[^"]+)"/g)]
      .map((match) => match[1])
      .filter((route) => route.includes('Service/')),
  )].sort();

  assert.deepEqual(routes, exactKnowledgeRoutes);
  assert.match(api, /KNOWLEDGE_CLIENT_CAPABILITY_ID_V1.*knowledge\.client\.v1/s);
  assert.match(api, /KNOWLEDGE_LIFECYCLE_EVENT_CAPABILITY_ID_V1.*knowledge\.lifecycle\.event\.v1/s);
  const changed = proto.split('message KnowledgeNoteChangedV1')[1]?.split('}')[0] ?? '';
  assert.doesNotMatch(changed, /title|body|excerpt|source_record|evidence|metadata|provider|locator|credential/);
});

test('Task 15 appends owner-local Knowledge storage revision 2', async () => {
  const schema = await read('backend/src/knowledge-persistence/src/schema.rs');
  const migration = await read('backend/src/knowledge-persistence/migrations/0002_knowledge_lifecycle_owner_rls.sql');

  assert.match(schema, /KNOWLEDGE_STORAGE_BUNDLE_REVISION_V2: u32 = 2/);
  assert.match(schema, /0002_knowledge_lifecycle_owner_rls\.sql/);
  assert.match(migration, /knowledge_sources/);
  assert.match(migration, /knowledge_client_operations/);
  assert.match(migration, /ENABLE ROW LEVEL SECURITY/);
  assert.match(migration, /FORCE ROW LEVEL SECURITY/);
  assert.match(migration, /current_setting\('makosh\.logical_owner_id'/);
});

test('Task 15 compiles Knowledge and removes handwritten REST Graph and Notes coupling', async () => {
  const [surfaces, adapters, generator, gateway] = await Promise.all([
    read('frontend/src/platform/client-runtime/clientSurfaces.ts'),
    read('frontend/src/app/client-surfaces/compiledClientSurfaceAdapters.ts'),
    read('frontend/scripts/generate-proto.mjs'),
    read('backend/src/api/gateway/session_contract/src/browser/client_bootstrap.rs'),
  ]);
  const legacyFiles = [
    ...globSync('src/domains/knowledge/api/**/*', { cwd: absolute('frontend') }),
    ...globSync('src/domains/notes/api/**/*', { cwd: absolute('frontend') }),
  ];

  assert.match(surfaces, /routeId: 'knowledge'[\s\S]*adapterId: 'knowledge-owner'/);
  assert.match(adapters, /'knowledge-owner'/);
  assert.equal(generator.includes('knowledge-command-api'), true);
  assert.match(gateway, /Self::Knowledge => Some\("knowledge\.client\.v1"\)/);
  assert.deepEqual(legacyFiles, []);
});

test('Task 15 requires actual lifecycle search pagination RLS replay and restart evidence', async () => {
  const [runner, flow] = await Promise.all([
    read('backend/scripts/test-authenticated-storage.mjs'),
    read('backend/tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/note_candidate_managed_flow.rs'),
  ]);
  assert.equal(runner.includes('managed_knowledge_lifecycle_search_replays_and_restarts_with_owner_rls'), true);
  assert.match(flow, /NOBYPASSRLS/);
  assert.match(flow, /AddSource|add_source/);
  assert.match(flow, /Search|search/);
  assert.match(flow, /next.*note|note.*cursor/i);
});
