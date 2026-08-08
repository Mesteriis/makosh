import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

test('bulk delivery managed runtime uses request RPC and safe replay without domain or integration coupling', async () => {
  const [
    adr,
    inventorySource,
    policySource,
    apiManifest,
    coreManifest,
    persistenceManifest,
    runtimeManifest,
    assemblyManifest,
    api,
    core,
    contract,
    persistence,
    migration,
    realtimeMigration,
    runtimeWorker,
    runtimeClient,
    managedRuntime,
    managedDeliveryPort,
    clientRealtime,
    admission,
    assembly,
    deliveryIntentSetup,
    managedSetup,
    managedFlow,
    conformanceRunner,
    devRelease,
    developmentAssembly,
  ] =
    await Promise.all([
    readFile(
      new URL(
        'docs/adr/ADR-0340-bounded-communication-bulk-delivery-workflow.md',
        PROJECT_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'architecture/communications-settings-reconstruction.json',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
      readFile(new URL('architecture/policy.json', BACKEND_ROOT), 'utf8'),
      readFile(
        new URL('src/communication-bulk-action-api/Cargo.toml', BACKEND_ROOT),
        'utf8',
      ),
      readFile(
        new URL('src/communication-bulk-action-core/Cargo.toml', BACKEND_ROOT),
        'utf8',
      ),
      readFile(
        new URL(
          'src/communication-bulk-action-persistence/Cargo.toml',
          BACKEND_ROOT,
        ),
        'utf8',
      ),
      readFile(
        new URL('src/communication-bulk-action-runtime/Cargo.toml', BACKEND_ROOT),
        'utf8',
      ),
      readFile(
        new URL(
          'src/communication-bulk-action-assembly/Cargo.toml',
          BACKEND_ROOT,
        ),
        'utf8',
      ),
      readFile(
        new URL('src/communication-bulk-action-api/src/lib.rs', BACKEND_ROOT),
        'utf8',
      ),
      readFile(
        new URL('src/communication-bulk-action-core/src/lib.rs', BACKEND_ROOT),
        'utf8',
      ),
      readFile(
        new URL(
          'src/communication-bulk-action-api/proto/makosh/communication_bulk_action/v1/bulk_action.proto',
          BACKEND_ROOT,
        ),
        'utf8',
      ),
      readFile(
        new URL(
          'src/communication-bulk-action-persistence/src/execution.rs',
          BACKEND_ROOT,
        ),
        'utf8',
      ),
      readFile(
        new URL(
          'src/communication-bulk-action-persistence/migrations/0001_bulk_delivery_state.sql',
          BACKEND_ROOT,
        ),
        'utf8',
      ),
      readFile(
        new URL(
          'src/communication-bulk-action-persistence/migrations/0002_client_realtime_replay.sql',
          BACKEND_ROOT,
        ),
        'utf8',
      ),
      readFile(
        new URL(
          'src/communication-bulk-action-runtime/src/worker.rs',
          BACKEND_ROOT,
        ),
        'utf8',
      ),
      readFile(
        new URL(
          'src/communication-bulk-action-runtime/src/client_port.rs',
          BACKEND_ROOT,
        ),
        'utf8',
      ),
      readFile(
        new URL(
          'src/communication-bulk-action-runtime/src/managed_runtime.rs',
          BACKEND_ROOT,
        ),
        'utf8',
      ),
      readFile(
        new URL(
          'src/communication-bulk-action-runtime/src/managed_delivery_port.rs',
          BACKEND_ROOT,
        ),
        'utf8',
      ),
      readFile(
        new URL(
          'src/communication-bulk-action-runtime/src/client_realtime.rs',
          BACKEND_ROOT,
        ),
        'utf8',
      ),
      readFile(
        new URL(
          'src/communication-bulk-action-runtime/src/admission.rs',
          BACKEND_ROOT,
        ),
        'utf8',
      ),
      readFile(
        new URL(
          'src/communication-bulk-action-assembly/src/lib.rs',
          BACKEND_ROOT,
        ),
        'utf8',
      ),
      readFile(
        new URL(
          'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/delivery_intent_managed_setup.rs',
          BACKEND_ROOT,
        ),
        'utf8',
      ),
      readFile(
        new URL(
          'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/bulk_action_managed_setup.rs',
          BACKEND_ROOT,
        ),
        'utf8',
      ),
      readFile(
        new URL(
          'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/bulk_action_managed_flow.rs',
          BACKEND_ROOT,
        ),
        'utf8',
      ),
      readFile(
        new URL('scripts/test-authenticated-storage.mjs', BACKEND_ROOT),
        'utf8',
      ),
      readFile(
        new URL('scripts/materialize-dev-release.sh', BACKEND_ROOT),
        'utf8',
      ),
      readFile(
        new URL('development/assembly/src/main.rs', BACKEND_ROOT),
        'utf8',
      ),
    ]);
  const inventory = JSON.parse(inventorySource);
  const policy = JSON.parse(policySource);
  const gate = inventory.slices.find(
    ({ gate: name }) => name === 'communication_bulk_action_v1',
  );

  assert.deepEqual(gate, {
    gate: 'communication_bulk_action_v1',
    role: 'workflow',
    owner: 'communication_bulk_action',
    state: 'implemented',
    dependsOn: [
      'communication_delivery_intent_v1',
      'capability_routed_module_request_rpc_v1',
    ],
  });
  assert.match(adr, /`1\.\.=100` targets/);
  assert.match(adr, /64 KiB module `request_rpc`/);
  assert.match(adr, /Kernel не retry-ит mutation/);
  assert.match(adr, /одну bounded lease/);
  assert.match(adr, /Private body[\s\S]*не попадает в logs\/events\/errors\/status/);
  assert.match(adr, /Принятый ADR сам по себе gate не открывает/);
  assert.equal(
    policy.implementation.currentSlice,
    'call_transcription_managed_conformance_v1',
  );
  assert.deepEqual(
    policy.implementation.productionPackages
      .filter(({ owner }) => owner === 'communication_bulk_action')
      .map(({ name, surface }) => `${name}:${surface}`),
    [
      'makosh-communication-bulk-action-api:contract',
      'makosh-communication-bulk-action-core:implementation',
      'makosh-communication-bulk-action-persistence:persistence',
      'makosh-communication-bulk-action-runtime:runtime',
      'makosh-communication-bulk-action-assembly:assembly',
    ],
  );
  assert.match(apiManifest, /role = "workflow"[\s\S]*surface = "contract"/);
  assert.match(coreManifest, /role = "workflow"[\s\S]*surface = "implementation"/);
  assert.match(
    persistenceManifest,
    /role = "workflow"[\s\S]*surface = "persistence"/,
  );
  assert.match(
    runtimeManifest,
    /role = "workflow"[\s\S]*surface = "runtime"/,
  );
  assert.match(assemblyManifest, /role = "workflow"[\s\S]*surface = "assembly"/);
  assert.doesNotMatch(
    `${apiManifest}\n${coreManifest}\n${persistenceManifest}\n${runtimeManifest}`,
    /makosh-(?:communications-domain|mail|telegram|whatsapp|zulip|kernel)/,
  );
  assert.match(api, /COMMUNICATION_BULK_ACTION_MAX_TARGETS_V1: usize = 100/);
  assert.match(core, /MAX_TARGET_BODY_BYTES_V1: usize = 64 \* 1024/);
  assert.match(core, /DuplicateTargetId/);
  assert.doesNotMatch(contract, /provider_id|account_id|\bAny\b|\bmap\s*</);
  assert.match(persistence, /MAX_TARGET_ATTEMPTS_V1: u16 = 3/);
  assert.match(persistence, /FOR UPDATE SKIP LOCKED/);
  assert.match(persistence, /claim_epoch/);
  assert.match(migration, /body_utf8 BYTEA/);
  assert.match(migration, /target_count BETWEEN 1 AND 100/);
  assert.match(realtimeMigration, /communication_bulk_action_realtime/);
  assert.match(realtimeMigration, /realtime_sequence/);
  assert.doesNotMatch(
    `${persistence}\n${migration}`,
    /communications_(?:messages|conversations)|mail_|telegram_|provider_/,
  );
  assert.match(runtimeWorker, /DeliveryIntentRequestPortV1/);
  assert.match(runtimeWorker, /mark_target_retryable/);
  assert.match(runtimeClient, /start_bulk_delivery_payload_v1/);
  assert.match(runtimeClient, /get_status_payload_v1/);
  assert.match(managedRuntime, /ManagedControlChannelV2/);
  assert.match(managedRuntime, /process_next_target_v1/);
  assert.match(managedDeliveryPort, /Operation::RouteModuleRequest/);
  assert.match(managedDeliveryPort, /delivery_intent_command_contract_v1/);
  assert.match(clientRealtime, /PublishClientRealtime/);
  assert.match(clientRealtime, /BulkDeliveryStatusChangedV1/);
  assert.match(admission, /dependencies: vec!\[delivery_intent_command_contract_v1\(\)\]/);
  assert.match(assembly, /communication_bulk_action_module_descriptor_v1/);
  assert.match(assembly, /communication_bulk_action_storage_bundle_v1/);
  assert.match(assembly, /communication_bulk_action\.runtime\.v1/);
  assert.match(
    managedSetup,
    /communication_delivery_intent_module_descriptor_v1/,
  );
  assert.match(
    deliveryIntentSetup,
    /configure_delivery_intent_runtime_routes[\s\S]*configure_module_request_handler/,
  );
  assert.doesNotMatch(managedSetup, /configure_bulk_action_request_route/);
  assert.match(managedFlow, /managed_bulk_action_reaches_gateway_sse_and_replays_after_restart/);
  assert.match(managedFlow, /admit_delivery_intent_runtime/);
  assert.match(managedFlow, /COMMUNICATION_BULK_ACTION_QUERY_CONNECT_PATH_V1/);
  assert.match(managedFlow, /last-event-id/);
  assert.match(
    managedFlow,
    /BulkDeliveryBatchStateCompleted[\s\S]*restart_bulk_action_runtime/,
  );
  assert.match(
    conformanceRunner,
    /MAKOSH_COMMUNICATION_BULK_ACTION_RUNTIME_BIN/,
  );
  assert.match(
    conformanceRunner,
    /managed_bulk_action_reaches_gateway_sse_and_replays_after_restart/,
  );
  assert.match(devRelease, /makosh-communication-bulk-action-assembly/);
  assert.match(
    developmentAssembly,
    /COMMUNICATION_BULK_ACTION_RUNTIME_ARTIFACT/,
  );
  assert.match(
    developmentAssembly,
    /runtime_kind: ModuleRuntimeKindV1::Workflow/,
  );
  assert.doesNotMatch(
    `${runtimeWorker}\n${runtimeClient}`,
    /body_utf8.*(?:log|event|status)|makosh-(?:mail|telegram|whatsapp|zulip)/,
  );
});
