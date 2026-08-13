import assert from 'node:assert/strict';
import { globSync } from 'node:fs';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import test from 'node:test';

const PROJECT_ROOT = resolve(import.meta.dirname, '../../..');
const absolute = (path) => resolve(PROJECT_ROOT, path);
const read = (path) => readFile(absolute(path), 'utf8');

const exactZulipPackages = [
  'makosh-zulip-api:contract',
  'makosh-zulip-assembly:assembly',
  'makosh-zulip-core:implementation',
  'makosh-zulip-delivery-intent-contract:contract',
  'makosh-zulip-http:implementation',
  'makosh-zulip-persistence:persistence',
  'makosh-zulip-runtime:runtime',
];

const exactZulipCapabilities = [
  'zulip.account.lifecycle.v1',
  'zulip.api-key.credential-provisioning.v1',
  'zulip.blob.v1',
  'zulip.command.v1',
  'zulip.credentials.v1',
  'zulip.delivery-intent.v1',
  'zulip.events.v1',
  'zulip.operational.query.v1',
  'zulip.operational.realtime.v1',
  'zulip.query.v1',
  'zulip.storage.v1',
];

const exactZulipRoutes = [
  '/makosh.zulip.account.v1.ZulipAccountLifecycleService/Apply',
  '/makosh.zulip.operational.realtime.v1.ZulipOperationalRealtimeService/Replay',
  '/makosh.zulip.operational.v1.ZulipOperationalQueryService/Query',
  '/makosh.zulip.v1.ZulipCommandService/ExecuteCommand',
  '/makosh.zulip.v1.ZulipQueryService/GetOperationStatus',
].sort();

const exactZulipTables = [
  'zulip_account_credential_bindings',
  'zulip_command_operations',
  'zulip_command_queue',
  'zulip_communications_outbox',
  'zulip_delivery_intent_inbox',
  'zulip_delivery_intent_jobs',
  'zulip_delivery_intent_result_outbox',
  'zulip_delivery_route_accounts',
  'zulip_delivery_route_conversations',
  'zulip_delivery_route_messages',
  'zulip_operational_account_state',
  'zulip_operational_attachments',
  'zulip_operational_conversations',
  'zulip_operational_events',
  'zulip_operational_message_mutations',
  'zulip_operational_messages',
  'zulip_operational_reactions',
  'zulip_owner_scope',
  'zulip_provider_cursor',
].sort();

test('Task 12 remains staged behind the external Zulip provider gate', async () => {
  const policy = JSON.parse(await read('backend/architecture/policy.json'));
  const implementation = policy.implementation;
  const packages = implementation.productionPackages
    .filter(({ owner }) => owner === 'zulip')
    .map(({ name, surface }) => `${name}:${surface}`);

  assert.equal(implementation.currentSlice, 'speech_to_text_whisper_admission_v1');
  assert.equal(implementation.productionPackages.length, 283);
  assert.equal(implementation.ownerInventory.businessCapabilities.length, 253);
  for (const owner of ['telegram', 'whatsapp', 'zulip']) {
    assert.equal(implementation.ownerInventory.integrations.includes(owner), false);
  }
  assert.deepEqual(packages, ['makosh-zulip-delivery-intent-contract:contract']);
  assert.deepEqual(
    implementation.ownerInventory.businessCapabilities.filter((capability) =>
      exactZulipCapabilities.includes(capability)),
    [],
  );
  assert.equal(exactZulipPackages.length, 7);
});

test('Task 12 adds no Cargo package and keeps five generated Zulip client routes', async () => {
  const [workspace, contract, compiledAdapters] = await Promise.all([
    read('backend/Cargo.toml'),
    read('backend/src/zulip-api/src/client_contract.rs'),
    read('frontend/src/app/client-surfaces/compiledClientSurfaceAdapters.ts'),
  ]);
  const members = [...workspace.matchAll(/^\s*"([^"]+)",?$/gm)].map((match) => match[1]);
  const manifests = globSync('**/Cargo.toml', {
    cwd: absolute('backend'),
    exclude: ['target/**'],
  });
  const routes = [...contract.matchAll(/"(\/makosh\.zulip\.[^"]+)"/g)]
    .map((match) => match[1])
    .filter((route) => route.includes('Service/'))
    .sort();

  assert.equal(members.length, 420);
  assert.equal(manifests.length, 421);
  assert.deepEqual(routes, exactZulipRoutes);
  assert.match(compiledAdapters, /'zulip-integration'/);
  assert.doesNotMatch(compiledAdapters, /\/api\/v1\/zulip/);
});

test('Task 12 requires immutable revision 7 FORCE RLS for every Zulip table', async () => {
  const [ownerRls, assembly] = await Promise.all([
    read('backend/src/zulip-persistence/src/owner_rls.rs'),
    read('backend/src/zulip-assembly/src/lib.rs'),
  ]);
  const tableInventory = ownerRls.match(/ZULIP_OWNER_RLS_TABLES_V1:[^=]+= \[([\s\S]*?)\n\];/);
  assert.ok(tableInventory, 'exact Zulip RLS table inventory');
  const tables = [...tableInventory[1].matchAll(/"([a-z0-9_]+)"/g)]
    .map((match) => match[1])
    .sort();

  assert.match(ownerRls, /ZULIP_OWNER_RLS_STORAGE_REVISION_V1: u32 = 7/);
  assert.match(ownerRls, /runtime_principal_prefix/);
  assert.match(ownerRls, /current_user/);
  assert.match(ownerRls, /ENABLE ROW LEVEL SECURITY/);
  assert.match(ownerRls, /FORCE ROW LEVEL SECURITY/);
  assert.deepEqual(tables, exactZulipTables);
  assert.match(assembly, /zulip_storage_bundle_with_owner_rls_v7/);
  assert.match(assembly, /ZULIP_STORAGE_BUNDLE_REVISION_V7/);
});

test('Task 12 release keeps one direct-provider Zulip module and two artifacts', async () => {
  const [assembly, materializer, developmentAssembly, compilerTest] = await Promise.all([
    read('backend/src/zulip-assembly/src/lib.rs'),
    read('backend/scripts/materialize-dev-release.sh'),
    read('backend/development/assembly/src/main.rs'),
    read('backend/tests/architecture/release-distribution-compiler.test.mjs'),
  ]);

  for (const artifact of ['zulip.runtime.v1', 'zulip.storage.v1']) {
    assert.equal(assembly.includes(artifact), true, artifact);
    assert.equal(compilerTest.includes(artifact), true, artifact);
  }
  assert.match(materializer, /zulip\.release-artifacts\.json/);
  assert.match(developmentAssembly, /const MODULE_PLAN: \[ModulePlanV1; 41\]/);
  assert.equal((developmentAssembly.match(/runtime_artifact_id: ZULIP_RUNTIME_ARTIFACT/g) ?? []).length, 1);
  assert.match(developmentAssembly, /runtime_artifact_id: ZULIP_RUNTIME_ARTIFACT,[\s\S]*?request_host_bridge: false/);
});

test('Task 12 requires managed bootstrap, privacy, replay and RLS evidence', async () => {
  const [runner, managedFlow, eventFlow] = await Promise.all([
    read('backend/scripts/test-authenticated-storage.mjs'),
    read('backend/tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/zulip_managed_flow.rs'),
    read('backend/tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/zulip_event_flow.rs'),
  ]);

  for (const testName of [
    'managed_zulip_runtime_uses_kernel_leases_and_route_specific_admission',
    'managed_zulip_runtime_delivers_live_command_and_event_only_communications_handoff',
    'managed_zulip_runtime_bootstrap_fails_closed_and_stops_promptly',
    'managed_zulip_private_surfaces_reject_malformed_provider_output',
  ]) {
    assert.equal(runner.includes(testName), true, testName);
    assert.equal(`${managedFlow}\n${eventFlow}`.includes(testName), true, testName);
  }
  assert.match(`${managedFlow}\n${eventFlow}`, /NOBYPASSRLS/);
  assert.match(`${managedFlow}\n${eventFlow}`, /runtime_storage_credential_for_registration_v1/);
  assert.match(`${managedFlow}\n${eventFlow}`, /supervised.*(?:stdout|stderr)/i);
});
