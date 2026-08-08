import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);

test('WhatsApp host, command, status, read and replay use separate exact contracts', async () => {
  const [
    proto,
    operationalProto,
    realtimeProto,
    contracts,
    hostContract,
    publicPort,
    hostPort,
    runtimeRoot,
    runtimeMain,
    tauriHost,
  ] =
    await Promise.all([
      readFile(
        new URL(
          'src/whatsapp-api/proto/makosh/whatsapp/v1/client.proto',
          BACKEND_ROOT,
        ),
        'utf8',
      ),
      readFile(
        new URL(
          'src/whatsapp-api/proto/makosh/whatsapp/operational/v1/client.proto',
          BACKEND_ROOT,
        ),
        'utf8',
      ),
      readFile(
        new URL(
          'src/whatsapp-api/proto/makosh/whatsapp/operational/realtime/v1/client.proto',
          BACKEND_ROOT,
        ),
        'utf8',
      ),
      readFile(
        new URL(
          'src/whatsapp-api/src/client_contract.rs',
          BACKEND_ROOT,
        ),
        'utf8',
      ),
      readFile(
        new URL('src/whatsapp-api/src/host_bridge.rs', BACKEND_ROOT),
        'utf8',
      ),
      readFile(
        new URL('src/whatsapp-runtime/src/client_port.rs', BACKEND_ROOT),
        'utf8',
      ),
      readFile(
        new URL(
          'src/whatsapp-runtime/src/host_bridge_port.rs',
          BACKEND_ROOT,
        ),
        'utf8',
      ),
      readFile(
        new URL('src/whatsapp-runtime/src/lib.rs', BACKEND_ROOT),
        'utf8',
      ),
      readFile(
        new URL('src/whatsapp-runtime/src/main.rs', BACKEND_ROOT),
        'utf8',
      ),
      readFile(
        new URL('../frontend/src-tauri/src/whatsapp_companion.rs', BACKEND_ROOT),
        'utf8',
      ),
    ]);

  assert.match(proto, /service WhatsAppCommandService/);
  assert.match(proto, /rpc ExecuteCommand\s*\(/);
  assert.match(proto, /service WhatsAppQueryService/);
  assert.match(proto, /rpc GetOperationStatus\s*\(/);
  assert.match(proto, /message WhatsAppHostBridgeResponseV1/);
  assert.match(proto, /message WhatsAppHostCommandLeaseV1/);
  assert.match(proto, /message WhatsAppHostBridgeOperationV1/);
  assert.doesNotMatch(proto, /message WhatsAppClientResponseV1/);
  assert.doesNotMatch(proto, /ClaimPendingCommandsQuery/);
  assert.doesNotMatch(proto, /PendingCommandsQuery/);

  assert.match(operationalProto, /service WhatsAppOperationalQueryService/);
  assert.match(operationalProto, /rpc Query\s*\(/);
  for (const query of [
    'ListMessagesQuery',
    'SearchMessagesQuery',
    'ListDialogsQuery',
    'ListParticipantsQuery',
    'ListEventsQuery',
    'GetRuntimeStatusQuery',
  ]) {
    assert.match(operationalProto, new RegExp(`\\b${query}\\b`));
  }
  assert.doesNotMatch(
    operationalProto,
    /google\.protobuf\.Any|\bbytes\b|\bmap\s*</,
  );
  assert.match(realtimeProto, /service WhatsAppOperationalRealtimeService/);
  assert.match(realtimeProto, /rpc Replay\s*\(/);
  assert.match(realtimeProto, /reset_required/);
  assert.doesNotMatch(
    realtimeProto,
    /google\.protobuf\.Any|\bbytes\b|\bmap\s*</,
  );

  assert.match(contracts, /"whatsapp\.command\.v1"/);
  assert.match(contracts, /"whatsapp\.query\.v1"/);
  assert.match(contracts, /"whatsapp\.operational\.query\.v1"/);
  assert.match(contracts, /"whatsapp\.operational\.realtime\.v1"/);
  assert.match(
    contracts,
    /\/makosh\.whatsapp\.operational\.v1\.WhatsAppOperationalQueryService\/Query/,
  );
  assert.match(contracts, /WHATSAPP_OPERATIONAL_DESCRIPTOR_SET_V1/);
  assert.match(contracts, /WHATSAPP_OPERATIONAL_REALTIME_DESCRIPTOR_SET_V1/);
  assert.doesNotMatch(contracts, /=>\s*"whatsapp\.client"/);
  assert.match(hostContract, /HOST_BRIDGE_CONTRACT_NAME[^=]*=\s*"whatsapp\.host_bridge\.v1"/);

  assert.match(publicPort, /WhatsAppClientContractV1::Command/);
  assert.match(publicPort, /WhatsAppClientContractV1::Query/);
  assert.match(publicPort, /WhatsAppClientContractV1::OperationalQuery/);
  assert.match(publicPort, /WhatsAppClientContractV1::OperationalRealtime/);
  assert.match(publicPort, /submit_command/);
  assert.match(publicPort, /command_operation_status/);
  assert.match(publicPort, /operational_query/);
  assert.match(publicPort, /operational_replay/);
  assert.doesNotMatch(publicPort, /ClaimPendingCommands/);
  assert.doesNotMatch(publicPort, /HostObservation/);

  assert.match(hostPort, /HOST_BRIDGE_CONTRACT_NAME/);
  assert.match(hostPort, /WhatsAppHostBridgeOperationV1::ClaimCommands/);
  assert.match(hostPort, /HostObservation/);
  assert.doesNotMatch(hostPort, /"whatsapp\.client"/);
  assert.doesNotMatch(hostPort, /WhatsAppClientResponseV1/);
  assert.doesNotMatch(hostPort, /WhatsAppProviderQuery/);

  assert.match(runtimeRoot, /mod client_port;/);
  assert.match(runtimeRoot, /mod host_bridge_port;/);
  assert.match(runtimeRoot, /mod host_bridge_transport;/);
  assert.doesNotMatch(runtimeRoot, /mod client_transport;/);
  assert.match(runtimeMain, /try_handle_client_delivery/);

  assert.match(tauriHost, /HOST_BRIDGE_CONTRACT_NAME/);
  assert.match(tauriHost, /decode_host_bridge_observation_accepted/);
  assert.doesNotMatch(tauriHost, /WhatsAppClientResponseV1/);
  assert.doesNotMatch(tauriHost, /"whatsapp\.client"/);
});
