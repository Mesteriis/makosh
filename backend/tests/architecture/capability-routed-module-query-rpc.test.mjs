import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

test('managed module query RPC foundation is typed bounded and owner neutral', async () => {
  const [
    adr,
    inventorySource,
    protocol,
    validation,
    control,
    supervisor,
    queryRouter,
    queryStore,
    migration,
  ] =
    await Promise.all([
      readFile(
        new URL(
          'docs/adr/ADR-0336-capability-routed-module-query-rpc.md',
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
          'src/platform/runtime_protocol/src/validation/module_query.rs',
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
          'src/kernel/src/modules/capability/module_query.rs',
          BACKEND_ROOT,
        ),
        'utf8',
      ),
      readFile(
        new URL(
          'src/kernel/control_store/sqlite/src/module_state/module_query_route.rs',
          BACKEND_ROOT,
        ),
        'utf8',
      ),
      readFile(
        new URL(
          'src/kernel/control_store/sqlite/src/schema/v44_to_v45.rs',
          BACKEND_ROOT,
        ),
        'utf8',
      ),
    ]);
  const inventory = JSON.parse(inventorySource);
  const platformGate = inventory.slices.find(
    ({ gate }) => gate === 'capability_routed_module_query_rpc_v1',
  );
  const deliveryGate = inventory.slices.find(
    ({ gate }) => gate === 'communication_delivery_intent_v1',
  );

  assert.deepEqual(platformGate, {
    gate: 'capability_routed_module_query_rpc_v1',
    role: 'platform',
    owner: 'kernel_capability_router',
    state: 'implemented',
    dependsOn: ['module_control_plane_v1'],
  });
  assert.equal(deliveryGate?.state, 'implemented');
  assert.ok(
    deliveryGate?.dependsOn.includes('capability_routed_module_query_rpc_v1'),
  );
  assert.match(adr, /caller не передаёт target registration/i);
  assert.match(protocol, /message ManagedRuntimeModuleQueryRequestV1/);
  assert.match(protocol, /message ManagedRuntimeModuleQueryDeliveryV1/);
  assert.match(protocol, /message ManagedRuntimeModuleQueryResponseV1/);
  assert.match(protocol, /route_module_query = 9/);
  assert.match(protocol, /deliver_module_query = 10/);
  assert.match(validation, /MODULE_QUERY_MAX_PAYLOAD_BYTES_V1: usize = 256 \* 1024/);
  assert.match(validation, /MODULE_QUERY_MAX_DEADLINE_MILLIS_V1: u32 = 10_000/);
  assert.match(validation, /response\.request_id/);
  assert.match(control, /trait ManagedRuntimeModuleQueryHandler/);
  assert.match(supervisor, /configure_module_query_handler/);
  assert.match(queryRouter, /module_contract_dependencies/);
  assert.match(queryRouter, /approved_module_query_rpc_routes/);
  assert.match(queryRouter, /current_managed_runtime_matches/);
  assert.match(queryRouter, /initial_owner_identity/);
  assert.match(queryRouter, /provider is ambiguous/);
  assert.match(queryStore, /validate_module_query_contracts/);
  assert.match(migration, /makosh_kernel_module_query_rpc_route_request/);
  assert.match(migration, /makosh_kernel_module_contract_dependency/);
  assert.doesNotMatch(
    `${protocol}\n${validation}\n${control}\n${supervisor}\n${queryRouter}\n${queryStore}\n${migration}`,
    /makosh_(?:communications|mail|telegram|whatsapp|zulip)|Communications|Mail|Telegram|WhatsApp|Zulip/,
  );
});
