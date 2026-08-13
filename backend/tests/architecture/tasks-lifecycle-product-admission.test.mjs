import assert from 'node:assert/strict';
import { globSync } from 'node:fs';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import test from 'node:test';

const PROJECT_ROOT = resolve(import.meta.dirname, '../../..');
const absolute = (path) => resolve(PROJECT_ROOT, path);
const read = (path) => readFile(absolute(path), 'utf8');

const exactTasksRoutes = [
  '/makosh.tasks.client.v1.TasksCommandService/AddChecklistItem',
  '/makosh.tasks.client.v1.TasksCommandService/AddDependency',
  '/makosh.tasks.client.v1.TasksCommandService/Create',
  '/makosh.tasks.client.v1.TasksCommandService/RemoveChecklistItem',
  '/makosh.tasks.client.v1.TasksCommandService/RemoveDependency',
  '/makosh.tasks.client.v1.TasksCommandService/SetPriority',
  '/makosh.tasks.client.v1.TasksCommandService/SetState',
  '/makosh.tasks.client.v1.TasksCommandService/Update',
  '/makosh.tasks.client.v1.TasksCommandService/UpdateChecklistItem',
  '/makosh.tasks.client.v1.TasksQueryService/Get',
  '/makosh.tasks.client.v1.TasksQueryService/List',
].sort();

test('Task 14 remains staged behind Task 10 without changing admitted Tasks ownership', async () => {
  const policy = JSON.parse(await read('backend/architecture/policy.json'));
  const implementation = policy.implementation;

  assert.equal(implementation.currentSlice, 'speech_to_text_whisper_admission_v1');
  assert.equal(implementation.productionPackages.length, 283);
  assert.equal(implementation.ownerInventory.businessCapabilities.length, 253);
  assert.deepEqual(
    implementation.productionPackages.filter(({ owner }) => owner === 'tasks').map(({ name }) => name).sort(),
    [
      'makosh-tasks-assembly',
      'makosh-tasks-command-api',
      'makosh-tasks-core',
      'makosh-tasks-persistence',
      'makosh-tasks-runtime',
    ],
  );
});

test('Task 14 exposes exact typed lifecycle routes and sanitized events', async () => {
  const [api, proto] = await Promise.all([
    read('backend/src/tasks-command-api/src/lib.rs'),
    read('backend/src/tasks-command-api/proto/makosh/tasks/client/v1/tasks.proto'),
  ]);
  const routes = [...new Set(
    [...api.matchAll(/"(\/makosh\.tasks\.client\.v1\.[^"]+)"/g)]
      .map((match) => match[1])
      .filter((route) => route.includes('Service/')),
  )].sort();

  assert.deepEqual(routes, exactTasksRoutes);
  assert.match(api, /TASKS_CLIENT_CAPABILITY_ID_V1.*tasks\.client\.v1/s);
  assert.match(api, /TASKS_LIFECYCLE_EVENT_CAPABILITY_ID_V1.*tasks\.lifecycle\.event\.v1/s);
  const changed = proto.split('message TaskChangedV1')[1]?.split('}')[0] ?? '';
  assert.doesNotMatch(changed, /title|description|label|metadata|provider|locator|credential/);
});

test('Task 14 appends owner-local Tasks storage revision 2', async () => {
  const schema = await read('backend/src/tasks-persistence/src/schema.rs');
  const migration = await read('backend/src/tasks-persistence/migrations/0002_tasks_lifecycle_owner_rls.sql');

  assert.match(schema, /TASKS_STORAGE_BUNDLE_REVISION_V2: u32 = 2/);
  assert.match(schema, /0002_tasks_lifecycle_owner_rls\.sql/);
  assert.match(migration, /tasks_dependencies/);
  assert.match(migration, /tasks_checklist/);
  assert.match(migration, /tasks_client_operations/);
  assert.match(migration, /ENABLE ROW LEVEL SECURITY/);
  assert.match(migration, /FORCE ROW LEVEL SECURITY/);
  assert.match(migration, /current_setting\('makosh\.logical_owner_id'/);
});

test('Task 14 compiles Tasks and removes handwritten REST coupling', async () => {
  const [surfaces, adapters, generator, gateway] = await Promise.all([
    read('frontend/src/platform/client-runtime/clientSurfaces.ts'),
    read('frontend/src/app/client-surfaces/compiledClientSurfaceAdapters.ts'),
    read('frontend/scripts/generate-proto.mjs'),
    read('backend/src/api/gateway/session_contract/src/browser/client_bootstrap.rs'),
  ]);
  const legacyFiles = globSync('src/domains/tasks/api/**/*', { cwd: absolute('frontend') });

  assert.match(surfaces, /routeId: 'tasks'[\s\S]*adapterId: 'tasks-owner'/);
  assert.match(adapters, /'tasks-owner'/);
  assert.equal(generator.includes('tasks-command-api'), true);
  assert.match(gateway, /Self::Tasks => Some\("tasks\.client\.v1"\)/);
  assert.deepEqual(legacyFiles, []);
});

test('Task 14 requires actual lifecycle pagination RLS replay and restart evidence', async () => {
  const [runner, flow] = await Promise.all([
    read('backend/scripts/test-authenticated-storage.mjs'),
    read('backend/tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/task_candidate_managed_flow.rs'),
  ]);
  assert.equal(runner.includes('managed_tasks_lifecycle_replays_and_restarts_with_owner_rls'), true);
  assert.match(flow, /NOBYPASSRLS/);
  assert.match(flow, /dependency/i);
  assert.match(flow, /checklist/i);
  assert.match(flow, /next.*task|task.*cursor/i);
});
