import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

test('client system status uses the shared typed Gateway realtime boundary', async () => {
  const [
    adr,
    inventorySource,
    payloadContract,
    gatewayRealtime,
    kernelReconciler,
    navigationQuery,
    frontendDecoder,
  ] = await Promise.all([
    readFile(
      new URL(
        'docs/adr/ADR-0338-client-system-status-over-shared-realtime.md',
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
        'src/api/gateway/contracts/proto/makosh/gateway/v1/client_system_status_realtime.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('src/api/gateway/runtime/src/realtime/mod.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/kernel/src/platform/system_status_realtime.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('frontend/src/app/queries/useClientNavigationSurface.ts', PROJECT_ROOT),
      'utf8',
    ),
    readFile(
      new URL(
        'frontend/src/platform/gateway/browserGatewaySystemStatus.ts',
        PROJECT_ROOT,
      ),
      'utf8',
    ),
  ]);
  const inventory = JSON.parse(inventorySource);
  const gate = inventory.slices.find(
    ({ gate: candidate }) => candidate === 'client_system_status_realtime_v1',
  );

  assert.deepEqual(gate, {
    gate: 'client_system_status_realtime_v1',
    role: 'platform',
    owner: 'kernel_system_status',
    state: 'implemented',
    dependsOn: ['client_gateway_v1'],
  });
  assert.match(adr, /GET \/api\/realtime\/v1\/events/);
  assert.match(adr, /Состояние реализации: Реализовано/);
  assert.match(adr, /Периодический client polling всего bootstrap запрещён/);
  assert.match(payloadContract, /message ClientSystemStatusChangedV1/);
  assert.match(payloadContract, /repeated ClientSystemComponentStatusV1 statuses = 2/);
  const payloadMessage = payloadContract.match(
    /message ClientSystemStatusChangedV1 \{[\s\S]*?\n\}/,
  )?.[0] ?? '';
  assert.doesNotMatch(
    payloadMessage,
    /owner_id|device_id|provider|credential|secret|message_body|content/,
  );
  assert.match(gatewayRealtime, /system_status_canonical/);
  assert.match(gatewayRealtime, /system_status_is_typed_change_only_and_replayable/);
  assert.match(kernelReconciler, /client_system_status\(store, supervisor, true\)/);
  assert.match(frontendDecoder, /makosh\.gateway\.system-status/);
  assert.match(frontendDecoder, /platform\.system_status\.changed/);
  assert.match(navigationQuery, /getBrowserGatewayRealtimeHub\(\)\.subscribe/);
  assert.doesNotMatch(navigationQuery, /BOOTSTRAP_REFRESH_MS|setInterval/);
  assert.doesNotMatch(
    `${gatewayRealtime}\n${kernelReconciler}`,
    /makosh-(mail|telegram|whatsapp|zulip)|communications-domain/,
  );
});
