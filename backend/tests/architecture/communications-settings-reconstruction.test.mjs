import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);
const INVENTORY_PATH = new URL(
  'architecture/communications-settings-reconstruction.json',
  BACKEND_ROOT,
);
const POLICY_PATH = new URL('architecture/policy.json', BACKEND_ROOT);
const ADR_PATH = new URL(
  'docs/adr/ADR-0282-full-communications-and-settings-capability-reconstruction.md',
  PROJECT_ROOT,
);
const TELEGRAM_AUTOMATION_ADR_PATH = new URL(
  'docs/adr/ADR-0283-telegram-automation-management-and-preview-boundary.md',
  PROJECT_ROOT,
);
const TELEGRAM_CALLS_ADR_PATH = new URL(
  'docs/adr/ADR-0284-telegram-one-to-one-audio-calls-operational-boundary.md',
  PROJECT_ROOT,
);
const TELEGRAM_REALTIME_ADR_PATH = new URL(
  'docs/adr/ADR-0287-telegram-operational-realtime-replay-boundary.md',
  PROJECT_ROOT,
);
const TELEGRAM_FOLDER_ADR_PATH = new URL(
  'docs/adr/ADR-0289-telegram-folder-reassignment-convergence-boundary.md',
  PROJECT_ROOT,
);
const TELEGRAM_RECONFIGURATION_ADR_PATH = new URL(
  'docs/adr/ADR-0290-telegram-account-runtime-reconfiguration-boundary.md',
  PROJECT_ROOT,
);
const TELEGRAM_CLIENT_PROTO_PATH = new URL(
  'src/telegram-api/proto/makosh/telegram/v1/client.proto',
  BACKEND_ROOT,
);
const TELEGRAM_CLIENT_CONTRACT_PATH = new URL(
  'src/telegram-api/src/client_contract.rs',
  BACKEND_ROOT,
);
const TELEGRAM_RUNTIME_ADMISSION_PATH = new URL(
  'src/telegram-runtime/src/admission.rs',
  BACKEND_ROOT,
);
const TELEGRAM_TDLIB_PATH = new URL('src/telegram-tdlib/src/lib.rs', BACKEND_ROOT);
const TELEGRAM_PERSISTENCE_PATH = new URL(
  'src/telegram-persistence/src/durable.rs',
  BACKEND_ROOT,
);
const TELEGRAM_PROJECTION_CACHE_PATH = new URL(
  'src/telegram-runtime/src/projection_cache.rs',
  BACKEND_ROOT,
);
const TELEGRAM_RUNTIME_PATH = new URL('src/telegram-runtime/src/lib.rs', BACKEND_ROOT);
const TELEGRAM_RUNTIME_PROCESS_PATH = new URL(
  'src/telegram-runtime/src/process.rs',
  BACKEND_ROOT,
);
const TELEGRAM_TDJSON_FIXTURE_PATH = new URL(
  'tests/fixtures/telegram-tdjson/tdjson.c',
  BACKEND_ROOT,
);
const TELEGRAM_MANAGED_FLOW_PATH = new URL(
  'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/telegram_managed_flow.rs',
  BACKEND_ROOT,
);
const WHATSAPP_OPERATIONAL_ADR_PATH = new URL(
  'docs/adr/ADR-0286-whatsapp-operational-read-and-realtime-boundary.md',
  PROJECT_ROOT,
);
const WHATSAPP_FRONTEND_GENERATED_READ_PATH = new URL(
  'frontend/src/gen/makosh/whatsapp/operational/v1/client_pb.ts',
  PROJECT_ROOT,
);
const WHATSAPP_FRONTEND_GENERATED_REPLAY_PATH = new URL(
  'frontend/src/gen/makosh/whatsapp/operational/realtime/v1/client_pb.ts',
  PROJECT_ROOT,
);
const WHATSAPP_FRONTEND_READ_CLIENT_PATH = new URL(
  'frontend/src/integrations/whatsapp/api/whatsAppOperationalReadClient.ts',
  PROJECT_ROOT,
);
const WHATSAPP_FRONTEND_READ_GATEWAY_PATH = new URL(
  'frontend/src/integrations/whatsapp/api/whatsAppOperationalReadGateway.ts',
  PROJECT_ROOT,
);
const WHATSAPP_FRONTEND_REPLAY_CLIENT_PATH = new URL(
  'frontend/src/integrations/whatsapp/api/whatsAppOperationalRealtimeClient.ts',
  PROJECT_ROOT,
);
const WHATSAPP_FRONTEND_REPLAY_GATEWAY_PATH = new URL(
  'frontend/src/integrations/whatsapp/api/whatsAppOperationalReplayGateway.ts',
  PROJECT_ROOT,
);
const WHATSAPP_FRONTEND_ACCOUNTS_PATH = new URL(
  'frontend/src/integrations/whatsapp/queries/whatsAppOperationalAccounts.ts',
  PROJECT_ROOT,
);
const WHATSAPP_FRONTEND_READ_CONTROLLER_PATH = new URL(
  'frontend/src/integrations/whatsapp/queries/useWhatsAppOperationalRead.ts',
  PROJECT_ROOT,
);
const WHATSAPP_FRONTEND_REPLAY_CONTROLLER_PATH = new URL(
  'frontend/src/integrations/whatsapp/queries/useWhatsAppOperationalReplay.ts',
  PROJECT_ROOT,
);
const WHATSAPP_FRONTEND_READ_PANEL_PATH = new URL(
  'frontend/src/integrations/whatsapp/presentation/WhatsAppOperationalReadPanel.vue',
  PROJECT_ROOT,
);
const WHATSAPP_FRONTEND_REPLAY_PANEL_PATH = new URL(
  'frontend/src/integrations/whatsapp/presentation/WhatsAppOperationalReplayPanel.vue',
  PROJECT_ROOT,
);
const WHATSAPP_FRONTEND_ROUTE_PATH = new URL(
  'frontend/src/integrations/whatsapp/views/WhatsAppOperationalRoute.vue',
  PROJECT_ROOT,
);
const FRONTEND_APP_LAYOUT_PATH = new URL(
  'frontend/src/app/layout/AppLayoutRoot.vue',
  PROJECT_ROOT,
);
const ZULIP_OPERATIONAL_ADR_PATH = new URL(
  'docs/adr/ADR-0291-zulip-account-history-query-and-replay-boundary.md',
  PROJECT_ROOT,
);
const ZULIP_OPERATIONAL_PROTO_PATH = new URL(
  'src/zulip-api/proto/makosh/zulip/operational/v1/client.proto',
  BACKEND_ROOT,
);
const ZULIP_REALTIME_PROTO_PATH = new URL(
  'src/zulip-api/proto/makosh/zulip/operational/realtime/v1/client.proto',
  BACKEND_ROOT,
);
const ZULIP_CLIENT_CONTRACT_PATH = new URL(
  'src/zulip-api/src/client_contract.rs',
  BACKEND_ROOT,
);
const ZULIP_HISTORY_HTTP_PATH = new URL('src/zulip-http/src/history.rs', BACKEND_ROOT);
const ZULIP_OPERATIONAL_PERSISTENCE_PATH = new URL(
  'src/zulip-persistence/src/operational.rs',
  BACKEND_ROOT,
);
const ZULIP_SCHEMA_PATH = new URL('src/zulip-persistence/src/schema.rs', BACKEND_ROOT);
const ZULIP_RUNTIME_PATH = new URL('src/zulip-runtime/src/lib.rs', BACKEND_ROOT);
const ZULIP_MANAGED_FLOW_PATH = new URL(
  'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/zulip_event_flow.rs',
  BACKEND_ROOT,
);
const ZULIP_FRONTEND_GENERATED_READ_PATH = new URL(
  'frontend/src/gen/makosh/zulip/operational/v1/client_pb.ts',
  PROJECT_ROOT,
);
const ZULIP_FRONTEND_GENERATED_REPLAY_PATH = new URL(
  'frontend/src/gen/makosh/zulip/operational/realtime/v1/client_pb.ts',
  PROJECT_ROOT,
);
const ZULIP_FRONTEND_READ_CLIENT_PATH = new URL(
  'frontend/src/integrations/zulip/api/zulipOperationalReadClient.ts',
  PROJECT_ROOT,
);
const ZULIP_FRONTEND_READ_GATEWAY_PATH = new URL(
  'frontend/src/integrations/zulip/api/zulipOperationalReadGateway.ts',
  PROJECT_ROOT,
);
const ZULIP_FRONTEND_REPLAY_CLIENT_PATH = new URL(
  'frontend/src/integrations/zulip/api/zulipOperationalRealtimeClient.ts',
  PROJECT_ROOT,
);
const ZULIP_FRONTEND_REPLAY_GATEWAY_PATH = new URL(
  'frontend/src/integrations/zulip/api/zulipOperationalReplayGateway.ts',
  PROJECT_ROOT,
);
const ZULIP_FRONTEND_ACCOUNTS_PATH = new URL(
  'frontend/src/integrations/zulip/queries/zulipOperationalAccounts.ts',
  PROJECT_ROOT,
);
const ZULIP_FRONTEND_READ_CONTROLLER_PATH = new URL(
  'frontend/src/integrations/zulip/queries/useZulipOperationalRead.ts',
  PROJECT_ROOT,
);
const ZULIP_FRONTEND_REPLAY_CONTROLLER_PATH = new URL(
  'frontend/src/integrations/zulip/queries/useZulipOperationalReplay.ts',
  PROJECT_ROOT,
);
const ZULIP_FRONTEND_READ_PANEL_PATH = new URL(
  'frontend/src/integrations/zulip/presentation/ZulipOperationalReadPanel.vue',
  PROJECT_ROOT,
);
const ZULIP_FRONTEND_MESSAGE_ROW_PATH = new URL(
  'frontend/src/integrations/zulip/presentation/ZulipMessageRow.vue',
  PROJECT_ROOT,
);
const ZULIP_FRONTEND_REPLAY_PANEL_PATH = new URL(
  'frontend/src/integrations/zulip/presentation/ZulipOperationalReplayPanel.vue',
  PROJECT_ROOT,
);
const ZULIP_FRONTEND_ROUTE_PATH = new URL(
  'frontend/src/integrations/zulip/views/ZulipOperationalRoute.vue',
  PROJECT_ROOT,
);

const ALLOWED_ROLES = new Set(['app', 'domain', 'engine', 'integration', 'platform', 'workflow']);
const ALLOWED_STATES = new Set(['implemented', 'planned']);
const BUSINESS_OWNER_ROLES = new Set(['domain', 'engine', 'integration', 'workflow']);
const FORBIDDEN_BUSINESS_OWNERS = new Set(['core', 'gateway', 'kernel', 'settings']);

test('ADR-0282 keeps an exact complete reconstruction inventory', async () => {
  const [inventorySource, policySource, adrSource] = await Promise.all([
    readFile(INVENTORY_PATH, 'utf8'),
    readFile(POLICY_PATH, 'utf8'),
    readFile(ADR_PATH, 'utf8'),
  ]);
  const inventory = JSON.parse(inventorySource);
  const policy = JSON.parse(policySource);

  assert.equal(inventory.version, 1);
  assert.equal(inventory.adr, 'ADR-0282');
  assert.equal(inventory.status, 'complete');
  assert.equal(inventory.completionGate, 'communications_settings_reconstruction_complete_v1');
  assert.equal(inventory.legacyAuthorityAllowed, false);
  assert.ok(inventory.slices.length > 20);

  const gates = inventory.slices.map(({ gate }) => gate);
  assert.equal(new Set(gates).size, gates.length, 'reconstruction gates must be unique');
  assert.ok(inventory.slices.every(({ state }) => ALLOWED_STATES.has(state)));
  assert.equal(inventory.slices.length, 91);
  assert.ok(inventory.slices.every(({ state }) => state === 'implemented'));

  for (const slice of inventory.slices) {
    assert.ok(ALLOWED_ROLES.has(slice.role), `unknown owner role for ${slice.gate}`);
    assert.ok(slice.owner.length > 0, `missing owner for ${slice.gate}`);
    assert.ok(Array.isArray(slice.dependsOn), `missing dependencies for ${slice.gate}`);
    assert.match(adrSource, new RegExp(`\\b${slice.gate}\\b`), `${slice.gate} is absent from ADR-0282`);
    if (BUSINESS_OWNER_ROLES.has(slice.role)) {
      assert.ok(
        !FORBIDDEN_BUSINESS_OWNERS.has(slice.owner),
        `${slice.gate} assigns business behavior to ${slice.owner}`,
      );
    }
  }

  const activeCapabilities = new Set(policy.implementation.ownerInventory.businessCapabilities);
  const knownDependencies = new Set([
    ...activeCapabilities,
    ...Object.keys(policy.phaseGates.requires),
    ...gates,
  ]);
  for (const slice of inventory.slices) {
    for (const dependency of slice.dependsOn) {
      assert.ok(
        knownDependencies.has(dependency),
        `${slice.gate} has an unknown dependency ${dependency}`,
      );
    }
  }
  assert.ok(
    gates.every((gate) => !activeCapabilities.has(gate)),
    'reconstruction slice must not be active before an exact production admission gate',
  );
  assert.ok(
    !Object.hasOwn(policy.phaseGates.requires, inventory.completionGate),
    'the aggregate reconstruction marker must not become a generic production capability',
  );
});

test('provider operational slices remain separate integrations', async () => {
  const inventory = JSON.parse(await readFile(INVENTORY_PATH, 'utf8'));
  const providerSlices = new Map(
    inventory.slices
      .filter(({ gate }) => gate.endsWith('_full_operational_v1'))
      .map((slice) => [slice.owner, slice]),
  );

  assert.deepEqual([...providerSlices.keys()].sort(), ['telegram', 'whatsapp', 'zulip']);
  assert.ok([...providerSlices.values()].every(({ role }) => role === 'integration'));

  for (const owner of ['mail', 'telegram', 'whatsapp', 'zulip']) {
    const ownerSlices = inventory.slices.filter((slice) => slice.owner === owner);
    assert.ok(ownerSlices.length > 0, `${owner} must have an independent reconstruction slice`);
    assert.ok(ownerSlices.every(({ role }) => role === 'integration'));
  }
});

test('Telegram completion remains closed behind its independent capability slices', async () => {
  const [
    inventorySource,
    automationAdrSource,
    callsAdrSource,
    realtimeAdrSource,
    folderAdrSource,
    reconfigurationAdrSource,
    clientProtoSource,
    clientContractSource,
    runtimeAdmissionSource,
    tdlibSource,
    telegramPersistenceSource,
    telegramProjectionCacheSource,
    telegramRuntimeSource,
    telegramRuntimeProcessSource,
    tdjsonFixtureSource,
    managedFlowSource,
  ] = await Promise.all([
    readFile(INVENTORY_PATH, 'utf8'),
    readFile(TELEGRAM_AUTOMATION_ADR_PATH, 'utf8'),
    readFile(TELEGRAM_CALLS_ADR_PATH, 'utf8'),
    readFile(TELEGRAM_REALTIME_ADR_PATH, 'utf8'),
    readFile(TELEGRAM_FOLDER_ADR_PATH, 'utf8'),
    readFile(TELEGRAM_RECONFIGURATION_ADR_PATH, 'utf8'),
    readFile(TELEGRAM_CLIENT_PROTO_PATH, 'utf8'),
    readFile(TELEGRAM_CLIENT_CONTRACT_PATH, 'utf8'),
    readFile(TELEGRAM_RUNTIME_ADMISSION_PATH, 'utf8'),
    readFile(TELEGRAM_TDLIB_PATH, 'utf8'),
    readFile(TELEGRAM_PERSISTENCE_PATH, 'utf8'),
    readFile(TELEGRAM_PROJECTION_CACHE_PATH, 'utf8'),
    readFile(TELEGRAM_RUNTIME_PATH, 'utf8'),
    readFile(TELEGRAM_RUNTIME_PROCESS_PATH, 'utf8'),
    readFile(TELEGRAM_TDJSON_FIXTURE_PATH, 'utf8'),
    readFile(TELEGRAM_MANAGED_FLOW_PATH, 'utf8'),
  ]);
  const inventory = JSON.parse(inventorySource);
  const telegramSlices = new Map(
    inventory.slices
      .filter(({ owner }) => owner === 'telegram')
      .map((slice) => [slice.gate, slice]),
  );
  const requiredTelegramGates = [
    'telegram_automation_v1',
    'telegram_call_history_v1',
    'telegram_call_media_v1',
    'telegram_call_signaling_v1',
    'telegram_calls_operational_v1',
    'telegram_core_operational_v1',
    'telegram_folder_reassignment_v1',
    'telegram_runtime_reconfiguration_v1',
    'telegram_tdlib_user_qr_identity_v1',
  ];
  const fullGate = telegramSlices.get('telegram_full_operational_v1');

  assert.deepEqual(
    [...telegramSlices.keys()].filter((gate) => gate !== 'telegram_full_operational_v1').sort(),
    requiredTelegramGates,
  );
  assert.deepEqual(
    [...fullGate.dependsOn].sort(),
    requiredTelegramGates.filter((gate) => !gate.startsWith('telegram_call_')),
  );
  assert.ok([...telegramSlices.values()].every(({ role }) => role === 'integration'));

  const automationGate = telegramSlices.get('telegram_automation_v1');
  assert.equal(automationGate.state, 'implemented');
  assert.equal(telegramSlices.get('telegram_core_operational_v1').state, 'implemented');
  assert.equal(
    telegramSlices.get('telegram_folder_reassignment_v1').state,
    'implemented',
  );
  assert.equal(
    telegramSlices.get('telegram_runtime_reconfiguration_v1').state,
    'implemented',
  );
  assert.equal(fullGate.state, 'implemented');
  assert.equal(telegramSlices.get('telegram_calls_operational_v1').state, 'implemented');
  assert.equal(telegramSlices.get('telegram_call_signaling_v1').state, 'implemented');
  assert.deepEqual(automationGate.dependsOn, ['telegram_core_operational_v1']);
  assert.match(automationAdrSource, /makosh-telegram-automation-api/);
  assert.match(automationAdrSource, /makosh-telegram-automation-core/);
  assert.match(automationAdrSource, /makosh-telegram-automation-persistence/);
  assert.match(automationAdrSource, /telegram\.automation\.query\.v1/);
  assert.match(automationAdrSource, /telegram\.automation\.command\.v1/);
  assert.match(automationAdrSource, /telegram_automation_execution_v1/);

  const callsGate = telegramSlices.get('telegram_calls_operational_v1');
  assert.deepEqual([...callsGate.dependsOn].sort(), [
    'telegram_call_history_v1',
    'telegram_call_media_v1',
    'telegram_call_signaling_v1',
  ]);
  assert.deepEqual(telegramSlices.get('telegram_call_history_v1').dependsOn, [
    'telegram_core_operational_v1',
  ]);
  assert.deepEqual(telegramSlices.get('telegram_call_signaling_v1').dependsOn, [
    'telegram_call_history_v1',
  ]);
  assert.deepEqual(telegramSlices.get('telegram_call_media_v1').dependsOn, [
    'telegram_call_signaling_v1',
  ]);
  assert.match(callsAdrSource, /makosh-telegram-calls-api/);
  assert.match(callsAdrSource, /makosh-telegram-calls-core/);
  assert.match(callsAdrSource, /makosh-telegram-calls-persistence/);
  assert.match(callsAdrSource, /makosh-telegram-call-media-contract/);
  assert.match(callsAdrSource, /makosh-telegram-call-media-tgcalls/);
  assert.match(callsAdrSource, /telegram\.calls\.query\.v1/);
  assert.match(callsAdrSource, /telegram\.calls\.command\.v1/);
  assert.match(callsAdrSource, /telegram\.calls\.realtime\.v1/);
  assert.match(callsAdrSource, /call\.id.*непостоянным/);
  assert.match(callsAdrSource, /fixture PCM[\s\S]*не закрывают production admission/);

  assert.match(realtimeAdrSource, /telegram\.realtime\.v1/);
  assert.match(realtimeAdrSource, /reset_required/);
  assert.match(realtimeAdrSource, /Состояние реализации: Реализовано/);
  assert.match(clientContractSource, /TelegramClientContractV1[\s\S]*Realtime/);
  assert.match(clientContractSource, /TELEGRAM_CLIENT_CONTRACT_REVISION: u32 = 9/);
  assert.match(runtimeAdmissionSource, /TelegramClientContractV1::Realtime/);
  assert.match(
    managedFlowSource,
    /managed_telegram_core_operational_projection_is_restart_safe/,
  );
  assert.match(managedFlowSource, /managed_telegram_realtime_route_requires_exact_grant/);
  assert.match(folderAdrSource, /Состояние реализации: Реализовано/);
  assert.match(folderAdrSource, /final provider snapshot and exact target equality/);
  assert.match(
    managedFlowSource,
    /managed_telegram_folder_reassignment_converges_after_partial_provider_failure/,
  );
  assert.match(
    tdlibSource,
    /verify-chat[\s\S]*provider_folder_ids_from_chat\(&verified_chat\)\? != target_provider_folder_ids/,
  );
  assert.match(
    telegramPersistenceSource,
    /position\.order <= 0[\s\S]*DELETE FROM makosh_data\.telegram_chat_position_projections/,
  );
  assert.match(
    telegramProjectionCacheSource,
    /position\.order <= 0[\s\S]*chat_positions\.remove/,
  );

  const beginReconfigurationBlock = clientProtoSource.match(
    /message BeginTelegramReconfigurationRequest \{[\s\S]*?\n\}/,
  )?.[0];
  assert.ok(beginReconfigurationBlock, 'missing typed Telegram Begin reconfiguration request');
  assert.match(beginReconfigurationBlock, /string reconfiguration_id = 1/);
  assert.match(beginReconfigurationBlock, /string account_id = 2/);
  assert.match(beginReconfigurationBlock, /uint64 expected_runtime_epoch = 3/);
  assert.doesNotMatch(
    beginReconfigurationBlock,
    /topology|holder|expires|now_unix|runtime_generation|grant_epoch/,
  );
  assert.match(
    clientProtoSource,
    /reserved "start_account", "stop_account", "restart_account"/,
  );
  assert.match(
    clientProtoSource,
    /service TelegramReconfigurationService[\s\S]*rpc Execute/,
  );
  assert.match(reconfigurationAdrSource, /Состояние реализации: Реализовано/);
  assert.match(reconfigurationAdrSource, /accepted\/applying crash recovery/);
  assert.match(clientContractSource, /telegram\.reconfiguration\.v1/);
  assert.match(
    clientContractSource,
    /makosh\.telegram\.v1\.TelegramReconfigurationService\/Execute/,
  );
  assert.match(runtimeAdmissionSource, /TelegramClientContractV1::Reconfiguration/);
  assert.match(
    telegramPersistenceSource,
    /telegram_runtime_reconfigurations_active_account_idx[\s\S]*state IN \('accepted', 'applying'\)/,
  );
  assert.match(
    telegramPersistenceSource,
    /complete_runtime_reconfiguration[\s\S]*transaction[\s\S]*state = 'completed'/,
  );
  assert.match(
    telegramRuntimeSource,
    /begin_runtime_reconfiguration[\s\S]*self\.runtime = None[\s\S]*TdlibAuthorizationDriver::new/,
  );
  assert.match(
    telegramRuntimeProcessSource,
    /resolve_provider_reconfiguration_parameters[\s\S]*begin_pending_runtime_reconfiguration/,
  );
  assert.match(
    telegramRuntimeProcessSource,
    /restore_account_state_durable[\s\S]*complete_pending_runtime_reconfiguration_durable/,
  );
  assert.match(tdlibSource, /pub fn create_client\(&self\)[\s\S]*self\.inner\.create/);
  assert.match(
    tdlibSource,
    /impl Drop for TdJsonClient[\s\S]*self\.library\.inner\.destroy/,
  );
  assert.match(tdjsonFixtureSource, /MAKOSH_STARTUP_RECEIVE_DELAYS/);
  assert.match(
    managedFlowSource,
    /managed_telegram_reconfiguration_route_requires_exact_grant/,
  );
  assert.match(
    managedFlowSource,
    /managed_telegram_runtime_reconfiguration_replaces_provider_session_once/,
  );
  assert.match(
    managedFlowSource,
    /managed_telegram_runtime_reconfiguration_recovers_same_epoch_after_process_crash/,
  );
});

test('WhatsApp completion admits independent read, realtime and frontend cutover units', async () => {
  const [
    inventorySource,
    whatsappAdrSource,
    generatedReadSource,
    generatedReplaySource,
    readClientSource,
    readGatewaySource,
    replayClientSource,
    replayGatewaySource,
    accountsSource,
    readControllerSource,
    replayControllerSource,
    readPanelSource,
    replayPanelSource,
    routeSource,
    appLayoutSource,
  ] = await Promise.all([
    readFile(INVENTORY_PATH, 'utf8'),
    readFile(WHATSAPP_OPERATIONAL_ADR_PATH, 'utf8'),
    readFile(WHATSAPP_FRONTEND_GENERATED_READ_PATH, 'utf8'),
    readFile(WHATSAPP_FRONTEND_GENERATED_REPLAY_PATH, 'utf8'),
    readFile(WHATSAPP_FRONTEND_READ_CLIENT_PATH, 'utf8'),
    readFile(WHATSAPP_FRONTEND_READ_GATEWAY_PATH, 'utf8'),
    readFile(WHATSAPP_FRONTEND_REPLAY_CLIENT_PATH, 'utf8'),
    readFile(WHATSAPP_FRONTEND_REPLAY_GATEWAY_PATH, 'utf8'),
    readFile(WHATSAPP_FRONTEND_ACCOUNTS_PATH, 'utf8'),
    readFile(WHATSAPP_FRONTEND_READ_CONTROLLER_PATH, 'utf8'),
    readFile(WHATSAPP_FRONTEND_REPLAY_CONTROLLER_PATH, 'utf8'),
    readFile(WHATSAPP_FRONTEND_READ_PANEL_PATH, 'utf8'),
    readFile(WHATSAPP_FRONTEND_REPLAY_PANEL_PATH, 'utf8'),
    readFile(WHATSAPP_FRONTEND_ROUTE_PATH, 'utf8'),
    readFile(FRONTEND_APP_LAYOUT_PATH, 'utf8'),
  ]);
  const inventory = JSON.parse(inventorySource);
  const whatsappSlices = new Map(
    inventory.slices
      .filter(({ owner }) => owner === 'whatsapp')
      .map((slice) => [slice.gate, slice]),
  );

  assert.deepEqual([...whatsappSlices.keys()].sort(), [
    'whatsapp_full_operational_v1',
    'whatsapp_integration_v1',
    'whatsapp_operational_read_v1',
    'whatsapp_operational_realtime_v1',
  ]);
  assert.ok([...whatsappSlices.values()].every(({ role }) => role === 'integration'));
  assert.equal(whatsappSlices.get('whatsapp_integration_v1').state, 'implemented');
  assert.equal(whatsappSlices.get('whatsapp_full_operational_v1').state, 'implemented');
  assert.equal(whatsappSlices.get('whatsapp_operational_read_v1').state, 'implemented');
  assert.equal(whatsappSlices.get('whatsapp_operational_realtime_v1').state, 'implemented');
  assert.deepEqual(whatsappSlices.get('whatsapp_operational_read_v1').dependsOn, [
    'whatsapp_integration_v1',
  ]);
  assert.deepEqual(whatsappSlices.get('whatsapp_operational_realtime_v1').dependsOn, [
    'whatsapp_operational_read_v1',
  ]);
  assert.deepEqual(
    [...whatsappSlices.get('whatsapp_full_operational_v1').dependsOn].sort(),
    [
      'client_gateway_v1',
      'nats_data_plane_v1',
      'whatsapp_operational_read_v1',
      'whatsapp_operational_realtime_v1',
    ],
  );

  assert.match(whatsappAdrSource, /whatsapp\.operational\.query\.v1/);
  assert.match(whatsappAdrSource, /whatsapp\.operational\.realtime\.v1/);
  assert.match(whatsappAdrSource, /makosh-whatsapp-api/);
  assert.match(whatsappAdrSource, /makosh-whatsapp-core/);
  assert.match(whatsappAdrSource, /makosh-whatsapp-persistence/);
  assert.match(whatsappAdrSource, /DDL-only/);
  assert.match(whatsappAdrSource, /Fake backfill запрещён/);
  assert.match(whatsappAdrSource, /Gate открыт как `implemented`/);
  assert.match(generatedReadSource, /WhatsAppOperationalQueryService/);
  assert.match(generatedReplaySource, /WhatsAppOperationalRealtimeService/);
  assert.match(readClientSource, /WhatsAppOperationalQueryService/);
  assert.match(readGatewaySource, /WhatsAppOperationalQueryV1Schema/);
  assert.match(replayClientSource, /WhatsAppOperationalRealtimeService/);
  assert.match(replayGatewaySource, /WhatsAppOperationalReplayRequestV1Schema/);
  assert.match(accountsSource, /whatsapp\.operational\.query\.v1/);
  assert.match(accountsSource, /whatsapp\.operational\.realtime\.v1/);
  assert.match(readControllerSource, /useWhatsAppOperationalRead/);
  assert.match(replayControllerSource, /useWhatsAppOperationalReplay/);
  assert.doesNotMatch(readPanelSource, /queries\/|api\/|connect\/|fetch\(/);
  assert.doesNotMatch(replayPanelSource, /queries\/|api\/|connect\/|fetch\(/);
  assert.match(routeSource, /whatsAppOperationalAccountFingerprint/);
  assert.match(appLayoutSource, /whatsapp\.operational\.query\.v1/);
  assert.match(appLayoutSource, /whatsapp\.operational\.realtime\.v1/);
});

test('Zulip completion admits lifecycle, history, read, realtime and frontend cutover units', async () => {
  const [
    inventorySource,
    zulipAdrSource,
    operationalProtoSource,
    realtimeProtoSource,
    clientContractSource,
    historyHttpSource,
    persistenceSource,
    schemaSource,
    runtimeSource,
    managedFlowSource,
    frontendGeneratedReadSource,
    frontendGeneratedReplaySource,
    frontendReadClientSource,
    frontendReadGatewaySource,
    frontendReplayClientSource,
    frontendReplayGatewaySource,
    frontendAccountsSource,
    frontendReadControllerSource,
    frontendReplayControllerSource,
    frontendReadPanelSource,
    frontendMessageRowSource,
    frontendReplayPanelSource,
    frontendRouteSource,
    appLayoutSource,
  ] = await Promise.all([
    readFile(INVENTORY_PATH, 'utf8'),
    readFile(ZULIP_OPERATIONAL_ADR_PATH, 'utf8'),
    readFile(ZULIP_OPERATIONAL_PROTO_PATH, 'utf8'),
    readFile(ZULIP_REALTIME_PROTO_PATH, 'utf8'),
    readFile(ZULIP_CLIENT_CONTRACT_PATH, 'utf8'),
    readFile(ZULIP_HISTORY_HTTP_PATH, 'utf8'),
    readFile(ZULIP_OPERATIONAL_PERSISTENCE_PATH, 'utf8'),
    readFile(ZULIP_SCHEMA_PATH, 'utf8'),
    readFile(ZULIP_RUNTIME_PATH, 'utf8'),
    readFile(ZULIP_MANAGED_FLOW_PATH, 'utf8'),
    readFile(ZULIP_FRONTEND_GENERATED_READ_PATH, 'utf8'),
    readFile(ZULIP_FRONTEND_GENERATED_REPLAY_PATH, 'utf8'),
    readFile(ZULIP_FRONTEND_READ_CLIENT_PATH, 'utf8'),
    readFile(ZULIP_FRONTEND_READ_GATEWAY_PATH, 'utf8'),
    readFile(ZULIP_FRONTEND_REPLAY_CLIENT_PATH, 'utf8'),
    readFile(ZULIP_FRONTEND_REPLAY_GATEWAY_PATH, 'utf8'),
    readFile(ZULIP_FRONTEND_ACCOUNTS_PATH, 'utf8'),
    readFile(ZULIP_FRONTEND_READ_CONTROLLER_PATH, 'utf8'),
    readFile(ZULIP_FRONTEND_REPLAY_CONTROLLER_PATH, 'utf8'),
    readFile(ZULIP_FRONTEND_READ_PANEL_PATH, 'utf8'),
    readFile(ZULIP_FRONTEND_MESSAGE_ROW_PATH, 'utf8'),
    readFile(ZULIP_FRONTEND_REPLAY_PANEL_PATH, 'utf8'),
    readFile(ZULIP_FRONTEND_ROUTE_PATH, 'utf8'),
    readFile(FRONTEND_APP_LAYOUT_PATH, 'utf8'),
  ]);
  const inventory = JSON.parse(inventorySource);
  const zulipSlices = new Map(
    inventory.slices
      .filter(({ owner }) => owner === 'zulip')
      .map((slice) => [slice.gate, slice]),
  );
  const backendGates = [
    'zulip_account_lifecycle_v1',
    'zulip_history_sync_v1',
    'zulip_operational_read_v1',
    'zulip_operational_realtime_v1',
  ];

  assert.deepEqual([...zulipSlices.keys()].sort(), [
    'zulip_account_lifecycle_v1',
    'zulip_full_operational_v1',
    'zulip_history_sync_v1',
    'zulip_integration_v1',
    'zulip_operational_read_v1',
    'zulip_operational_realtime_v1',
  ]);
  assert.ok([...zulipSlices.values()].every(({ role }) => role === 'integration'));
  assert.equal(zulipSlices.get('zulip_account_lifecycle_v1').state, 'implemented');
  assert.equal(zulipSlices.get('zulip_history_sync_v1').state, 'implemented');
  assert.equal(zulipSlices.get('zulip_operational_read_v1').state, 'implemented');
  assert.equal(zulipSlices.get('zulip_operational_realtime_v1').state, 'implemented');
  assert.equal(zulipSlices.get('zulip_integration_v1').state, 'implemented');
  assert.equal(zulipSlices.get('zulip_full_operational_v1').state, 'implemented');
  assert.deepEqual(
    [...zulipSlices.get('zulip_full_operational_v1').dependsOn].sort(),
    ['client_gateway_v1', 'nats_data_plane_v1', 'zulip_integration_v1', ...backendGates].sort(),
  );

  assert.match(zulipAdrSource, /zulip\.operational\.query\.v1/);
  assert.match(zulipAdrSource, /zulip\.operational\.realtime\.v1/);
  assert.match(zulipAdrSource, /GET \/api\/v1\/messages/);
  assert.match(zulipAdrSource, /Kernel\/Core согласуют только/);
  assert.match(zulipAdrSource, /integration, а не domain/);
  assert.match(zulipAdrSource, /generated frontend client/);
  assert.match(
    operationalProtoSource,
    /service ZulipOperationalQueryService[\s\S]*rpc Query/,
  );
  assert.match(
    realtimeProtoSource,
    /service ZulipOperationalRealtimeService[\s\S]*rpc Replay/,
  );
  assert.match(clientContractSource, /zulip\.operational\.query\.v1/);
  assert.match(clientContractSource, /zulip\.operational\.realtime\.v1/);
  assert.match(historyHttpSource, /request_for_message_history/);
  assert.match(historyHttpSource, /found_oldest/);
  assert.match(
    schemaSource,
    /ZULIP_STORAGE_BUNDLE_REVISION_V2[\s\S]*zulip_operational_message_mutations/,
  );
  assert.match(
    persistenceSource,
    /record_operational_events_and_enqueue[\s\S]*advance_cursor_in_transaction/,
  );
  assert.match(
    persistenceSource,
    /record_history_page[\s\S]*persist_history_message/,
  );
  assert.match(runtimeSource, /sync_history_page[\s\S]*fetch_message_history_page/);
  assert.match(managedFlowSource, /restart_zulip_runtime/);
  assert.match(managedFlowSource, /assert_cross_account_operational_query_is_rejected/);
  assert.match(managedFlowSource, /assert_zulip_operational_replay/);
  assert.match(zulipAdrSource, /Gate открыт как `implemented`/);
  assert.match(frontendGeneratedReadSource, /ZulipOperationalQueryService/);
  assert.match(frontendGeneratedReplaySource, /ZulipOperationalRealtimeService/);
  assert.match(frontendReadClientSource, /ZulipOperationalQueryService/);
  assert.match(frontendReadGatewaySource, /ZulipOperationalQueryV1Schema/);
  assert.match(frontendReplayClientSource, /ZulipOperationalRealtimeService/);
  assert.match(frontendReplayGatewaySource, /ZulipOperationalReplayRequestV1Schema/);
  assert.match(frontendAccountsSource, /zulip\.operational\.query\.v1/);
  assert.match(frontendAccountsSource, /zulip\.operational\.realtime\.v1/);
  assert.match(frontendReadControllerSource, /useZulipOperationalRead/);
  assert.match(frontendReplayControllerSource, /useZulipOperationalReplay/);
  assert.doesNotMatch(frontendReadPanelSource, /queries\/|api\/|connect\/|fetch\(/);
  assert.doesNotMatch(frontendMessageRowSource, /queries\/|api\/|connect\/|fetch\(/);
  assert.doesNotMatch(frontendReplayPanelSource, /queries\/|api\/|connect\/|fetch\(/);
  assert.match(frontendRouteSource, /zulipOperationalAccountFingerprint/);
  assert.match(appLayoutSource, /zulip\.operational\.query\.v1/);
  assert.match(appLayoutSource, /zulip\.operational\.realtime\.v1/);
});

test('cross-owner and AI use cases are distinct workflow units', async () => {
  const inventory = JSON.parse(await readFile(INVENTORY_PATH, 'utf8'));
  const workflowOwners = inventory.slices
    .filter(({ role }) => role === 'workflow')
    .map(({ owner }) => owner);

  assert.equal(new Set(workflowOwners).size, workflowOwners.length);
  assert.ok(workflowOwners.includes('communication_delivery_intent'));
  assert.ok(workflowOwners.includes('communication_reply_suggestion'));
  assert.ok(workflowOwners.includes('communication_translation'));
  assert.ok(workflowOwners.includes('communication_task_candidate_extraction'));
  assert.ok(workflowOwners.includes('call_transcription'));
  assert.ok(!workflowOwners.includes('communications'));
  assert.ok(!workflowOwners.includes('generic_ai'));
  assert.ok(!workflowOwners.includes('settings'));
});

test('historical presentation facades do not become admitted capabilities', async () => {
  const inventory = JSON.parse(await readFile(INVENTORY_PATH, 'utf8'));

  assert.deepEqual(inventory.historicalFacades, [
    'discord_channels',
    'google_meet_calls',
    'mattermost_channels',
    'microsoft_teams_calls',
    'phone_calls_without_admitted_provider',
    'slack_channels',
    'telemost_calls',
    'zoom_calls',
  ]);
  assert.ok(
    inventory.historicalFacades.every(
      (facade) => !inventory.slices.some(({ gate, owner }) => gate.includes(facade) || owner === facade),
    ),
  );
});
