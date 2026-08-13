import assert from 'node:assert/strict';
import { globSync } from 'node:fs';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import test from 'node:test';

const PROJECT_ROOT = resolve(import.meta.dirname, '../../..');
const absolute = (path) => resolve(PROJECT_ROOT, path);
const read = (path) => readFile(absolute(path), 'utf8');

const exactWhatsAppPackages = [
  'makosh-whatsapp-api:contract',
  'makosh-whatsapp-assembly:assembly',
  'makosh-whatsapp-core:implementation',
  'makosh-whatsapp-delivery-intent-contract:contract',
  'makosh-whatsapp-persistence:persistence',
  'makosh-whatsapp-runtime:runtime',
];

const exactWhatsAppCapabilities = [
  'whatsapp.blob.v1',
  'whatsapp.command.v1',
  'whatsapp.delivery-intent.v1',
  'whatsapp.events.v1',
  'whatsapp.host_bridge.v1',
  'whatsapp.operational.query.v1',
  'whatsapp.operational.realtime.v1',
  'whatsapp.query.v1',
  'whatsapp.storage.v1',
];

const exactWhatsAppRoutes = [
  '/makosh.whatsapp.operational.realtime.v1.WhatsAppOperationalRealtimeService/Replay',
  '/makosh.whatsapp.operational.v1.WhatsAppOperationalQueryService/Query',
  '/makosh.whatsapp.v1.WhatsAppCommandService/ExecuteCommand',
  '/makosh.whatsapp.v1.WhatsAppQueryService/GetOperationStatus',
].sort();

const exactWhatsAppTables = [
  'whatsapp_communications_outbox',
  'whatsapp_delivery_intent_inbox',
  'whatsapp_delivery_intent_jobs',
  'whatsapp_delivery_intent_result_outbox',
  'whatsapp_delivery_route_accounts',
  'whatsapp_delivery_route_conversations',
  'whatsapp_delivery_route_messages',
  'whatsapp_host_observations',
  'whatsapp_operational_controls',
  'whatsapp_operational_dialogs',
  'whatsapp_operational_events',
  'whatsapp_operational_messages',
  'whatsapp_operational_participants',
  'whatsapp_operational_runtime_status',
  'whatsapp_operational_tombstones',
  'whatsapp_owner_scope',
  'whatsapp_provider_commands',
].sort();

test('Task 11 remains staged behind the external WhatsApp provider gate', async () => {
  const policy = JSON.parse(await read('backend/architecture/policy.json'));
  const implementation = policy.implementation;
  const packages = implementation.productionPackages
    .filter(({ owner }) => owner === 'whatsapp')
    .map(({ name, surface }) => `${name}:${surface}`);

  assert.equal(implementation.currentSlice, 'speech_to_text_whisper_admission_v1');
  assert.equal(implementation.productionPackages.length, 283);
  assert.equal(implementation.ownerInventory.businessCapabilities.length, 253);
  assert.equal(implementation.ownerInventory.integrations.includes('telegram'), false);
  assert.equal(implementation.ownerInventory.integrations.includes('whatsapp'), false);
  assert.deepEqual(packages, ['makosh-whatsapp-delivery-intent-contract:contract']);
  assert.deepEqual(
    implementation.ownerInventory.businessCapabilities.filter((capability) =>
      exactWhatsAppCapabilities.includes(capability)),
    [],
  );
  assert.equal(exactWhatsAppPackages.length, 6);
});

test('Task 11 adds no Cargo package and keeps four generated WhatsApp client routes', async () => {
  const [workspace, contract, compiledAdapters] = await Promise.all([
    read('backend/Cargo.toml'),
    read('backend/src/whatsapp-api/src/client_contract.rs'),
    read('frontend/src/app/client-surfaces/compiledClientSurfaceAdapters.ts'),
  ]);
  const members = [...workspace.matchAll(/^\s*"([^"]+)",?$/gm)].map((match) => match[1]);
  const manifests = globSync('**/Cargo.toml', {
    cwd: absolute('backend'),
    exclude: ['target/**'],
  });
  const routes = [...contract.matchAll(/"(\/makosh\.whatsapp\.[^"]+)"/g)]
    .map((match) => match[1])
    .filter((route) => route.includes('Service/'))
    .sort();

  assert.equal(members.length, 420);
  assert.equal(manifests.length, 421);
  assert.deepEqual(routes, exactWhatsAppRoutes);
  assert.match(compiledAdapters, /'whatsapp-integration'/);
  assert.doesNotMatch(compiledAdapters, /\/api\/v1\/whatsapp/);
});

test('Task 11 requires immutable revision 5 FORCE RLS for every WhatsApp table', async () => {
  const [ownerRls, assembly] = await Promise.all([
    read('backend/src/whatsapp-persistence/src/owner_rls.rs'),
    read('backend/src/whatsapp-assembly/src/lib.rs'),
  ]);
  const tableInventory = ownerRls.match(/WHATSAPP_OWNER_RLS_TABLES_V1:[^=]+= \[([\s\S]*?)\n\];/);
  assert.ok(tableInventory, 'exact WhatsApp RLS table inventory');
  const tables = [...tableInventory[1].matchAll(/"([a-z0-9_]+)"/g)]
    .map((match) => match[1])
    .sort();

  assert.match(ownerRls, /WHATSAPP_OWNER_RLS_STORAGE_REVISION_V1: u32 = 5/);
  assert.match(ownerRls, /runtime_principal_prefix/);
  assert.match(ownerRls, /current_user/);
  assert.match(ownerRls, /ENABLE ROW LEVEL SECURITY/);
  assert.match(ownerRls, /FORCE ROW LEVEL SECURITY/);
  assert.deepEqual(tables, exactWhatsAppTables);
  assert.match(assembly, /whatsapp_storage_bundle_with_owner_rls_v5/);
  assert.match(assembly, /WHATSAPP_STORAGE_BUNDLE_REVISION_V5/);
});

test('Task 11 release keeps one host-bridged WhatsApp module and two artifacts', async () => {
  const [assembly, materializer, developmentAssembly, compilerTest] = await Promise.all([
    read('backend/src/whatsapp-assembly/src/lib.rs'),
    read('backend/scripts/materialize-dev-release.sh'),
    read('backend/development/assembly/src/main.rs'),
    read('backend/tests/architecture/release-distribution-compiler.test.mjs'),
  ]);

  for (const artifact of ['whatsapp.runtime.v1', 'whatsapp.storage.v1']) {
    assert.equal(assembly.includes(artifact), true, artifact);
    assert.equal(compilerTest.includes(artifact), true, artifact);
  }
  assert.match(materializer, /whatsapp\.release-artifacts\.json/);
  assert.match(developmentAssembly, /const MODULE_PLAN: \[ModulePlanV1; 41\]/);
  assert.equal((developmentAssembly.match(/runtime_artifact_id: WHATSAPP_RUNTIME_ARTIFACT/g) ?? []).length, 1);
  assert.match(developmentAssembly, /runtime_artifact_id: WHATSAPP_RUNTIME_ARTIFACT,[\s\S]*?request_host_bridge: true/);
});

test('Task 11 requires managed bootstrap, privacy, replay and RLS evidence', async () => {
  const [runner, managedFlow, eventFlow] = await Promise.all([
    read('backend/scripts/test-authenticated-storage.mjs'),
    read('backend/tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/whatsapp_managed_flow.rs'),
    read('backend/tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/whatsapp_event_flow.rs'),
  ]);

  for (const testName of [
    'managed_whatsapp_runtime_uses_signed_kernel_admission_and_host_route_fencing',
    'managed_whatsapp_runtime_delivers_live_command_and_event_only_communications_handoff',
    'managed_whatsapp_runtime_bootstrap_fails_closed_and_stops_promptly',
    'managed_whatsapp_private_surfaces_reject_malformed_host_output',
  ]) {
    assert.equal(runner.includes(testName), true, testName);
    assert.equal(`${managedFlow}\n${eventFlow}`.includes(testName), true, testName);
  }
  assert.match(`${managedFlow}\n${eventFlow}`, /NOBYPASSRLS/);
  assert.match(`${managedFlow}\n${eventFlow}`, /runtime_storage_credential_for_registration_v1/);
  assert.match(`${managedFlow}\n${eventFlow}`, /supervised.*(?:stdout|stderr)/i);
});
