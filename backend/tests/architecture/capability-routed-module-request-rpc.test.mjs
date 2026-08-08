import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

test('managed module request RPC routing is typed bounded and separate from query RPC', async () => {
  const [
    adr,
    inventorySource,
    protocol,
    validation,
    control,
    supervisor,
    managedSupervisor,
    descriptor,
    routeStore,
    router,
    migration,
    providerAdmission,
    providerPort,
    providerRuntime,
    liveConformance,
  ] =
    await Promise.all([
      readFile(
        new URL(
          'docs/adr/ADR-0339-capability-routed-module-request-rpc.md',
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
          'src/platform/runtime_protocol/src/validation/module_request.rs',
          BACKEND_ROOT,
        ),
        'utf8',
      ),
      readFile(
        new URL('src/kernel/src/runtime/lifecycle/control.rs', BACKEND_ROOT),
        'utf8',
      ),
      readFile(
        new URL(
          'src/kernel/src/runtime/lifecycle/supervisor.rs',
          BACKEND_ROOT,
        ),
        'utf8',
      ),
      readFile(
        new URL(
          'src/kernel/src/runtime/managed/supervisor.rs',
          BACKEND_ROOT,
        ),
        'utf8',
      ),
      readFile(
        new URL(
          'src/kernel/src/modules/registration/descriptor/mod.rs',
          BACKEND_ROOT,
        ),
        'utf8',
      ),
      readFile(
        new URL(
          'src/kernel/control_store/sqlite/src/module_state/module_request_route.rs',
          BACKEND_ROOT,
        ),
        'utf8',
      ),
      readFile(
        new URL(
          'src/kernel/src/modules/capability/module_request.rs',
          BACKEND_ROOT,
        ),
        'utf8',
      ),
      readFile(
        new URL(
          'src/kernel/control_store/sqlite/src/schema/v46_to_v47.rs',
          BACKEND_ROOT,
        ),
        'utf8',
      ),
      readFile(
        new URL(
          'src/communication-delivery-intent-runtime/src/admission.rs',
          BACKEND_ROOT,
        ),
        'utf8',
      ),
      readFile(
        new URL(
          'src/communication-delivery-intent-runtime/src/module_request_port.rs',
          BACKEND_ROOT,
        ),
        'utf8',
      ),
      readFile(
        new URL(
          'src/communication-delivery-intent-runtime/src/runtime.rs',
          BACKEND_ROOT,
        ),
        'utf8',
      ),
      readFile(
        new URL(
          'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/delivery_intent_module_request_flow.rs',
          BACKEND_ROOT,
        ),
        'utf8',
      ),
    ]);
  const inventory = JSON.parse(inventorySource);
  const platformGate = inventory.slices.find(
    ({ gate }) => gate === 'capability_routed_module_request_rpc_v1',
  );
  const bulkGate = inventory.slices.find(
    ({ gate }) => gate === 'communication_bulk_action_v1',
  );

  assert.deepEqual(platformGate, {
    gate: 'capability_routed_module_request_rpc_v1',
    role: 'platform',
    owner: 'kernel_capability_router',
    state: 'implemented',
    dependsOn: ['module_control_plane_v1'],
  });
  assert.ok(
    bulkGate?.dependsOn.includes('capability_routed_module_request_rpc_v1'),
  );
  assert.match(adr, /Kernel не повторяет request автоматически/);
  assert.match(protocol, /message ManagedRuntimeModuleRequestRequestV1/);
  assert.match(protocol, /message ManagedRuntimeModuleRequestDeliveryV1/);
  assert.match(protocol, /message ManagedRuntimeModuleRequestResponseV1/);
  assert.match(protocol, /route_module_request = 12/);
  assert.match(protocol, /deliver_module_request = 13/);
  assert.match(
    validation,
    /MODULE_REQUEST_MAX_PAYLOAD_BYTES_V1: usize = 64 \* 1024/,
  );
  assert.match(
    validation,
    /MODULE_REQUEST_MAX_DEADLINE_MILLIS_V1: u32 = 30_000/,
  );
  assert.match(control, /trait ManagedRuntimeModuleRequestHandler/);
  assert.match(supervisor, /configure_module_request_handler/);
  assert.match(managedSupervisor, /Operation::DeliverModuleRequest/);
  assert.match(managedSupervisor, /ControlResult::ModuleRequestDelivery/);
  assert.match(descriptor, /ProvidedSurfaceKindV1::RequestRpc/);
  assert.match(descriptor, /bind_module_request_contracts/);
  assert.match(
    routeStore,
    /approved_module_request_rpc_routes/,
  );
  assert.match(
    migration,
    /CREATE TABLE makosh_kernel_module_request_rpc_route_request/,
  );
  assert.match(router, /module_contract_dependencies/);
  assert.match(router, /approved_module_request_rpc_routes/);
  assert.match(router, /current_managed_runtime_matches/);
  assert.match(router, /ensure_caller_fence\(&self\.store, expectation\)\?/);
  assert.match(router, /ensure_provider_fence\(&self\.store, &provider, &provider_launch\)\?/);
  assert.doesNotMatch(router, /retry|DeliverModuleQuery/);
  assert.match(providerAdmission, /ProvidedSurfaceKindV1::RequestRpc/);
  assert.match(providerPort, /validate_module_request_delivery_v1/);
  assert.match(providerPort, /submit_delivery_intent_payload_v1/);
  assert.match(providerRuntime, /Operation::DeliverModuleRequest/);
  assert.match(providerRuntime, /ControlResult::ModuleRequestDelivery/);
  assert.match(liveConformance, /ModuleRequestRouteHandlerV1/);
  assert.match(liveConformance, /DeliveryIntentStatusAccepted/);
  assert.doesNotMatch(
    `${protocol}\n${validation}\n${control}\n${supervisor}\n${descriptor}\n${routeStore}\n${router}\n${migration}`,
    /makosh_(?:communications|mail|telegram|whatsapp|zulip)|Communications|Mail|Telegram|WhatsApp|Zulip/,
  );
});
