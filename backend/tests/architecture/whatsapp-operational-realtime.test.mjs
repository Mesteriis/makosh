import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);

test('WhatsApp operational replay remains a distinct integration capability', async () => {
  const [
    contract,
    api,
    wire,
    persistence,
    runtimePort,
    managedRuntime,
  ] = await Promise.all([
    readFile(
      new URL(
        'src/whatsapp-api/proto/makosh/whatsapp/operational/realtime/v1/client.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('src/whatsapp-api/src/realtime.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL('src/whatsapp-api/src/realtime_wire.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/whatsapp-persistence/src/operational.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/whatsapp-runtime/src/client_port.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(new URL('src/whatsapp-runtime/src/managed.rs', BACKEND_ROOT), 'utf8'),
  ]);

  assert.match(contract, /service WhatsAppOperationalRealtimeService/);
  assert.match(contract, /rpc Replay\s*\(/);
  assert.match(contract, /earliest_available_sequence/);
  assert.match(contract, /latest_available_sequence/);
  assert.match(contract, /reset_required/);
  assert.match(contract, /string account_id/);
  assert.doesNotMatch(contract, /google\.protobuf\.Any|\bbytes\b|\bmap\s*</);

  assert.match(api, /MAX_OPERATIONAL_REPLAY_LIMIT/);
  assert.match(api, /validate_operational_replay_request/);
  assert.match(api, /validate_operational_replay_response/);
  assert.match(wire, /encode_operational_replay_response/);
  assert.match(wire, /decode_operational_replay_response/);

  assert.match(persistence, /replay_operational_events/);
  assert.match(persistence, /MIN\(sequence\)/);
  assert.match(persistence, /MAX\(sequence\)/);
  assert.match(persistence, /ORDER BY sequence ASC/);
  assert.match(persistence, /cursor_exists/);
  assert.match(persistence, /event_from_row/);
  assert.doesNotMatch(
    persistence,
    /makosh_data\.communications_|makosh_(?:kernel|gateway)/,
  );

  assert.match(runtimePort, /WhatsAppClientContractV1::OperationalRealtime/);
  assert.match(runtimePort, /OperationalReplay/);
  assert.match(managedRuntime, /request\.account_id != self\.account_id/);
  assert.match(managedRuntime, /replay_operational_events\(request\)/);
  assert.doesNotMatch(managedRuntime, /SELECT |INSERT |UPDATE |DELETE /);
});
