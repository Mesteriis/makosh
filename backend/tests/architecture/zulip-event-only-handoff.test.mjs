import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);

test('Zulip keeps provider delivery event-only and retries a durable outbox after NATS outage', async () => {
  const [runtime, runtimeLib, liveFlow, managedSuite, harness] = await Promise.all([
    readFile(new URL('src/zulip-runtime/src/managed.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/zulip-runtime/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/zulip_event_flow.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('scripts/test-authenticated-storage.mjs', BACKEND_ROOT), 'utf8'),
  ]);

  assert.match(
    runtime,
    /Err\(ZulipCommunicationsOutboxRelayError::Unavailable\) => 0/,
  );
  assert.match(
    runtime,
    /Err\(error @ ZulipCommunicationsOutboxRelayError::Persistence\(_\)\)/,
  );
  assert.match(runtimeLib, /module_id: ZULIP_MODULE_ID\.to_owned\(\)/);
  assert.doesNotMatch(runtimeLib, /module_id: "zulip-runtime"/);

  assert.match(liveFlow, /set_authenticated_nats_container_running\(false\)/);
  assert.match(liveFlow, /set_authenticated_nats_container_running\(true\)/);
  assert.match(liveFlow, /while relay\.is_ready\(&contour\.zulip\.registration_id\) != Ok\(true\)/);
  assert.match(liveFlow, /duplicate Zulip observation must not create a second Communications event/);
  assert.match(liveFlow, /assert_communications_query_delivery/);
  assert.doesNotMatch(liveFlow, /makosh_communications_(domain|persistence|runtime)/);

  assert.match(managedSuite, /mod zulip_event_flow;/);
  assert.match(
    harness,
    /managed_zulip_runtime_delivers_live_command_and_event_only_communications_handoff/,
  );
});
