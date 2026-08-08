import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

test('managed client realtime keeps transport owner neutral and replay owner local', async () => {
  const [
    adr,
    inventorySource,
    protocol,
    validation,
    kernelRoute,
    routeStore,
    migration,
    ownerContract,
    ownerLedger,
    ownerAdapter,
    gatewayRealtime,
    kernelRealtimeConformance,
    managedRealtimeLive,
    developmentAssembly,
    materializeDevelopmentRelease,
  ] = await Promise.all([
    readFile(
      new URL(
        'docs/adr/ADR-0337-capability-routed-managed-client-realtime.md',
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
    readFile(
      new URL(
        'src/platform/runtime_protocol/proto/makosh/runtime/v1/managed_runtime_control.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/platform/runtime_protocol/src/validation/client_realtime.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('src/kernel/src/platform/client_realtime.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL(
        'src/kernel/control_store/sqlite/src/module_state/client_realtime_route.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delivery-intent-persistence/migrations/0003_client_realtime_replay.sql',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delivery-intent-api/proto/makosh/communication_delivery_intent/v1/delivery.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delivery-intent-persistence/src/client_realtime.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delivery-intent-runtime/src/client_realtime.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('src/api/gateway/runtime/src/realtime/mod.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/client_realtime_routes.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/delivery_intent_realtime_flow.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('development/assembly/src/main.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('scripts/materialize-dev-release.sh', BACKEND_ROOT),
      'utf8',
    ),
  ]);
  const inventory = JSON.parse(inventorySource);
  const platformGate = inventory.slices.find(
    ({ gate }) => gate === 'capability_routed_managed_client_realtime_v1',
  );
  const deliveryGate = inventory.slices.find(
    ({ gate }) => gate === 'communication_delivery_intent_v1',
  );
  const clientEvent = ownerContract.match(
    /message DeliveryIntentStatusChangedV1 \{[\s\S]*?\n\}/,
  )?.[0];

  assert.deepEqual(platformGate, {
    gate: 'capability_routed_managed_client_realtime_v1',
    role: 'platform',
    owner: 'kernel_capability_router',
    state: 'implemented',
    dependsOn: ['client_gateway_v1', 'module_control_plane_v1'],
  });
  assert.equal(deliveryGate?.state, 'implemented');
  assert.ok(
    deliveryGate?.dependsOn.includes(
      'capability_routed_managed_client_realtime_v1',
    ),
  );
  assert.match(adr, /bounded durable replay window/i);
  assert.match(protocol, /message ManagedRuntimeClientRealtimePublishRequestV1/);
  assert.match(protocol, /publish_client_realtime = 11/);
  assert.match(validation, /MAX_PAYLOAD_BYTES: usize = 64 \* 1024/);
  assert.match(kernelRoute, /current_managed_runtime_matches/);
  assert.match(kernelRoute, /approved_module_client_realtime_routes/);
  assert.match(kernelRoute, /initial_owner_identity/);
  assert.match(routeStore, /validate_client_realtime_routes/);
  assert.match(migration, /realtime_sequence BIGINT GENERATED ALWAYS AS IDENTITY/);
  assert.match(ownerLedger, /client_realtime_window/);
  assert.match(ownerLedger, /ORDER BY realtime_sequence ASC/);
  assert.match(ownerAdapter, /request_next_with_dispatch/);
  assert.match(ownerAdapter, /communication-delivery-intent\/\{\}/);
  assert.match(
    gatewayRealtime,
    /exact_duplicates_replay_live_delivery_and_bounded_gap_are_deterministic/,
  );
  assert.match(gatewayRealtime, /same cursor with different bytes must fail closed/);
  assert.match(
    kernelRealtimeConformance,
    /managed_realtime_publication_is_exact_owner_fenced_and_idempotent/,
  );
  assert.match(kernelRealtimeConformance, /revoked publisher must fail closed/);
  assert.match(
    managedRealtimeLive,
    /managed_delivery_intent_reaches_gateway_sse_and_replays_after_restart/,
  );
  assert.match(managedRealtimeLive, /private_body/);
  assert.match(managedRealtimeLive, /replayed\.cursor, cursor/);
  assert.match(
    developmentAssembly,
    /runtime_artifact_id: COMMUNICATION_DELIVERY_INTENT_RUNTIME_ARTIFACT,[\s\S]*?runtime_kind: ModuleRuntimeKindV1::Workflow/,
  );
  assert.match(
    materializeDevelopmentRelease,
    /--package makosh-communication-delivery-intent-runtime/,
  );
  assert.match(
    materializeDevelopmentRelease,
    /--package makosh-communication-delivery-intent-assembly/,
  );
  assert.ok(clientEvent);
  assert.doesNotMatch(
    clientEvent,
    /body|provider|account|cursor|credential|envelope/i,
  );
  assert.doesNotMatch(
    `${protocol}\n${validation}\n${kernelRoute}\n${routeStore}`,
    /makosh_communication_delivery_intent|DeliveryIntentStatusChangedV1/,
  );
});
