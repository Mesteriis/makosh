import assert from 'node:assert/strict';
import { globSync } from 'node:fs';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import test from 'node:test';

const PROJECT_ROOT = resolve(import.meta.dirname, '../../..');
const absolute = (path) => resolve(PROJECT_ROOT, path);
const read = (path) => readFile(absolute(path), 'utf8');

const exactTelegramPackages = [
  'makosh-telegram-api:contract',
  'makosh-telegram-assembly:assembly',
  'makosh-telegram-automation-api:contract',
  'makosh-telegram-automation-core:implementation',
  'makosh-telegram-automation-persistence:persistence',
  'makosh-telegram-call-media-contract:contract',
  'makosh-telegram-call-media-tgcalls:implementation',
  'makosh-telegram-calls-api:contract',
  'makosh-telegram-calls-core:implementation',
  'makosh-telegram-calls-persistence:persistence',
  'makosh-telegram-core:implementation',
  'makosh-telegram-delivery-intent-contract:contract',
  'makosh-telegram-persistence:persistence',
  'makosh-telegram-runtime:runtime',
  'makosh-telegram-tdlib:implementation',
];

const exactTelegramCapabilities = [
  'telegram.api-hash.credential-provisioning.v1',
  'telegram.authorization.realtime.v1',
  'telegram.authorization.v1',
  'telegram.automation.command.v1',
  'telegram.automation.query.v1',
  'telegram.blob.v1',
  'telegram.call-evidence.publish.v1',
  'telegram.calls.command.v1',
  'telegram.calls.query.v1',
  'telegram.calls.realtime.v1',
  'telegram.command.v1',
  'telegram.credentials.v1',
  'telegram.delivery-intent.v1',
  'telegram.events.v1',
  'telegram.lifecycle.v1',
  'telegram.query.v1',
  'telegram.realtime.v1',
  'telegram.reconfiguration.v1',
  'telegram.runtime.v1',
  'telegram.session-store-key.credential-provisioning.v1',
  'telegram.storage.v1',
];

const exactTelegramRoutes = [
  '/makosh.telegram.v1.TelegramAuthorizationService/Authorize',
  '/makosh.telegram.v1.TelegramLifecycleService/Execute',
  '/makosh.telegram.v1.TelegramOperationalService/ExecuteCommand',
  '/makosh.telegram.v1.TelegramOperationalService/ExecuteQuery',
  '/makosh.telegram.v1.TelegramRealtimeService/Replay',
  '/makosh.telegram.v1.TelegramReconfigurationService/Execute',
  '/makosh.telegram.automation.v1.TelegramAutomationQueryService/Query',
  '/makosh.telegram.automation.v1.TelegramAutomationCommandService/Execute',
  '/makosh.telegram.calls.v1.TelegramCallsQueryService/Query',
  '/makosh.telegram.calls.v1.TelegramCallsCommandService/Execute',
  '/makosh.telegram.calls.v1.TelegramCallsRealtimeService/Replay',
].sort();

const exactTelegramTables = [
  'telegram_accounts',
  'telegram_attachment_projections',
  'telegram_automation_mutation_receipts',
  'telegram_automation_policies',
  'telegram_automation_policy_chat_scopes',
  'telegram_automation_preview_receipts',
  'telegram_automation_template_variables',
  'telegram_automation_templates',
  'telegram_call_evidence_outbox',
  'telegram_call_local_mute',
  'telegram_call_media_projection',
  'telegram_call_media_state_history',
  'telegram_call_operation_history',
  'telegram_call_operations',
  'telegram_call_realtime_backfill_jobs',
  'telegram_call_realtime_events',
  'telegram_call_realtime_frames',
  'telegram_call_realtime_replay_cursor',
  'telegram_call_realtime_replay_order',
  'telegram_call_sessions',
  'telegram_call_state_history',
  'telegram_chat_avatar_projections',
  'telegram_chat_folder_projections',
  'telegram_chat_operational_states',
  'telegram_chat_position_projections',
  'telegram_chat_projections',
  'telegram_chat_states',
  'telegram_communications_outbox',
  'telegram_delivery_intent_inbox',
  'telegram_delivery_intent_jobs',
  'telegram_delivery_intent_result_outbox',
  'telegram_delivery_route_accounts',
  'telegram_delivery_route_conversations',
  'telegram_delivery_route_messages',
  'telegram_file_projections',
  'telegram_message_mutations',
  'telegram_message_projections',
  'telegram_message_reactions',
  'telegram_message_tombstones',
  'telegram_message_versions',
  'telegram_owner_scope',
  'telegram_participant_projections',
  'telegram_provider_event_journal',
  'telegram_runtime_operations',
  'telegram_runtime_reconfigurations',
  'telegram_topic_projections',
].sort();

test('Task 10 remains staged behind the external Telegram provider gate', async () => {
  const policy = JSON.parse(await read('backend/architecture/policy.json'));
  const implementation = policy.implementation;
  const packages = implementation.productionPackages
    .filter(({ owner }) => owner === 'telegram')
    .map(({ name, surface }) => `${name}:${surface}`);

  assert.equal(implementation.currentSlice, 'speech_to_text_whisper_admission_v1');
  assert.equal(implementation.productionPackages.length, 283);
  assert.equal(implementation.ownerInventory.businessCapabilities.length, 253);
  assert.equal(implementation.ownerInventory.integrations.includes('telegram'), false);
  assert.deepEqual(packages, ['makosh-telegram-delivery-intent-contract:contract']);
  assert.deepEqual(
    implementation.ownerInventory.businessCapabilities.filter((capability) =>
      exactTelegramCapabilities.includes(capability)),
    [],
  );
  assert.equal(exactTelegramPackages.length, 15);
});

test('Task 10 changes no Cargo package and exposes exactly the existing Telegram client boundary', async () => {
  const [workspace, mainContract, automationContract, callsContract, compiledAdapters] = await Promise.all([
    read('backend/Cargo.toml'),
    read('backend/src/telegram-api/src/client_contract.rs'),
    read('backend/src/telegram-automation-api/src/contract.rs'),
    read('backend/src/telegram-calls-api/src/contract.rs'),
    read('frontend/src/app/client-surfaces/compiledClientSurfaceAdapters.ts'),
  ]);
  const members = [...workspace.matchAll(/^\s*"([^"]+)",?$/gm)].map((match) => match[1]);
  const manifests = globSync('**/Cargo.toml', {
    cwd: absolute('backend'),
    exclude: ['target/**'],
  });
  const routes = [...`${mainContract}\n${automationContract}\n${callsContract}`.matchAll(/"(\/makosh\.telegram\.[^"]+)"/g)]
    .map((match) => match[1])
    .filter((route) => route.includes('Service/'))
    .sort();

  assert.equal(members.length, 420);
  assert.equal(manifests.length, 421);
  assert.deepEqual(routes, exactTelegramRoutes);
  assert.match(compiledAdapters, /'telegram-integration'/);
  assert.doesNotMatch(compiledAdapters, /telegram-bot/);
});

test('Task 10 requires immutable revision 10 FORCE RLS for every Telegram table', async () => {
  const [ownerRls, assembly] = await Promise.all([
    read('backend/src/telegram-persistence/src/owner_rls.rs'),
    read('backend/src/telegram-assembly/src/lib.rs'),
  ]);
  const tableInventory = ownerRls.match(/TELEGRAM_OWNER_RLS_TABLES_V1:[^=]+= \[([\s\S]*?)\n\];/);
  assert.ok(tableInventory, 'exact Telegram RLS table inventory');
  const rlsTables = [...tableInventory[1].matchAll(/"([a-z0-9_]+)"/g)]
    .map((match) => match[1])
    .sort();

  assert.match(ownerRls, /TELEGRAM_OWNER_RLS_STORAGE_REVISION_V1: u32 = 10/);
  assert.match(ownerRls, /runtime_principal_prefix/);
  assert.match(ownerRls, /current_user/);
  assert.match(ownerRls, /ENABLE ROW LEVEL SECURITY/);
  assert.match(ownerRls, /FORCE ROW LEVEL SECURITY/);
  assert.deepEqual(rlsTables, exactTelegramTables);
  assert.match(assembly, /telegram_storage_bundle_with_owner_rls_v10/);
  assert.match(assembly, /TELEGRAM_STORAGE_BUNDLE_REVISION_V10/);
});

test('Task 10 release keeps one Telegram module with four compiler-consumed artifacts', async () => {
  const [assembly, materializer, developmentAssembly, compilerTest] = await Promise.all([
    read('backend/src/telegram-assembly/src/lib.rs'),
    read('backend/scripts/materialize-dev-release.sh'),
    read('backend/development/assembly/src/main.rs'),
    read('backend/tests/architecture/release-distribution-compiler.test.mjs'),
  ]);

  for (const artifact of [
    'telegram.runtime.v1',
    'telegram.storage.v1',
    'telegram.tdjson.v1',
    'telegram.tgcalls.v1',
  ]) {
    assert.equal(assembly.includes(artifact), true, artifact);
    assert.equal(compilerTest.includes(artifact), true, artifact);
  }
  assert.match(materializer, /telegram\.release-artifacts\.json/);
  assert.match(
    materializer,
    /prepare_new_output_directory "Telegram call bridge" "\$tgcalls_root"/,
  );
  assert.match(materializer, /dev-native\/tgcalls-makosh/);
  assert.match(developmentAssembly, /const MODULE_PLAN: \[ModulePlanV1; 41\]/);
  assert.equal((developmentAssembly.match(/runtime_artifact_id: TELEGRAM_RUNTIME_ARTIFACT/g) ?? []).length, 1);
});

test('Task 10 admission requires managed bootstrap privacy and real TDLib plus tgcalls evidence', async () => {
  const [runner, flow, tgcallsBuilder, makefile] = await Promise.all([
    read('backend/scripts/test-authenticated-storage.mjs'),
    read('backend/tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/telegram_managed_flow.rs'),
    read('backend/scripts/build-telegram-tgcalls-bridge-macos.sh'),
    read('backend/Makefile'),
  ]);

  for (const testName of [
    'managed_telegram_runtime_uses_kernel_leases_and_event_only_communications_handoff',
    'managed_telegram_core_operational_projection_is_restart_safe',
    'managed_telegram_folder_reassignment_converges_after_partial_provider_failure',
    'managed_telegram_automation_route_is_durable_and_provider_side_effect_free',
    'managed_telegram_call_history_route_is_durable_and_replayable',
    'managed_telegram_runtime_bootstrap_fails_closed_and_stops_promptly',
    'managed_telegram_private_surfaces_reject_malformed_provider_output',
    'managed_telegram_real_tdlib_reaches_qr_authorization',
    'managed_telegram_real_tgcalls_audio_device_conformance',
  ]) {
    assert.equal(runner.includes(testName), true, testName);
    assert.equal(flow.includes(testName), true, testName);
  }
  assert.match(flow, /runtime_storage_credential_for_registration_v1/);
  assert.match(flow, /assert_supervised_telegram_child_output_is_private_v1/);
  assert.match(flow, /NOBYPASSRLS/);
  assert.match(flow, /managed_telegram_real_provider_prerequisites_are_exact/);
  assert.match(makefile, /telegram-admission-preflight:/);
  assert.match(makefile, /managed_telegram_real_provider_prerequisites_are_exact/);
  assert.match(tgcallsBuilder, /release_eligible=true/);
  assert.match(tgcallsBuilder, /AUDIO_CONFORMANCE_NAME/);
});
