import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);

test('WhatsApp managed admission is wired as an integration-owned conformance slice', async () => {
  const [manifest, harness, runner] = await Promise.all([
    readFile(
      new URL('tests/support/kernel-recovery/Cargo.toml', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('scripts/test-authenticated-storage.mjs', BACKEND_ROOT),
      'utf8',
    ),
  ]);

  for (const packageName of [
    'makosh-whatsapp-api',
    'makosh-whatsapp-persistence',
    'makosh-whatsapp-runtime',
  ]) {
    assert.match(manifest, new RegExp(`^${packageName} = `, 'm'));
  }

  for (const supportModule of [
    'whatsapp_managed_setup',
    'whatsapp_managed_fixture',
    'whatsapp_host_fixture',
    'whatsapp_managed_flow',
    'whatsapp_event_flow',
  ]) {
    assert.match(harness, new RegExp(`mod ${supportModule};`));
  }

  assert.match(runner, /'-p',\s*'makosh-whatsapp-runtime'/);
  assert.match(
    runner,
    /managed_whatsapp_runtime_uses_signed_kernel_admission_and_host_route_fencing/,
  );
  assert.match(
    runner,
    /managed_whatsapp_runtime_delivers_live_command_and_event_only_communications_handoff/,
  );
  assert.match(runner, /MAKOSH_WHATSAPP_RUNTIME_BIN:/);
});

test('WhatsApp managed read conformance covers projection, cursors and access fences', async () => {
  const [eventFlow, managedFlow] = await Promise.all([
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/whatsapp_event_flow.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/whatsapp_managed_flow.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
  ]);

  for (const evidence of [
    'OperationalMessage',
    'OperationalDialog',
    'OperationalParticipant',
    'OperationalParticipantRemoved',
    'OperationalResyncState',
    'SearchMessages',
    'ListEvents',
    'projection_ready',
    'exact duplicate host delivery is idempotent',
    'older provider observation must not overwrite',
    'stale body must not resurrect',
    'delivery_state',
    'assert_cross_account_operational_query_is_rejected',
  ]) {
    assert.ok(eventFlow.includes(evidence), `missing managed WhatsApp read evidence: ${evidence}`);
  }
  assert.match(managedFlow, /assert_ungranted_whatsapp_operational_query_is_rejected/);
  assert.match(managedFlow, /assert_stale_whatsapp_query_generation_is_rejected/);
});

test('WhatsApp managed replay conformance covers restart, reset, grants and privacy', async () => {
  const [eventFlow, managedFlow, setup] = await Promise.all([
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/whatsapp_event_flow.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/whatsapp_managed_flow.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/whatsapp_managed_setup.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
  ]);

  for (const evidence of [
    'assert_whatsapp_operational_replay',
    'assert_whatsapp_operational_replay_after_restart',
    'strictly ascending',
    'reset_required',
    'assert_cross_account_operational_replay_is_rejected',
    'provider command payload must not leak',
  ]) {
    assert.ok(eventFlow.includes(evidence), `missing managed WhatsApp replay evidence: ${evidence}`);
  }
  assert.match(managedFlow, /assert_ungranted_whatsapp_operational_replay_is_rejected/);
  assert.match(setup, /WhatsAppClientContractV1::OperationalRealtime/);
});

test('WhatsApp managed launch receives an exact Kernel-fenced private host route', async () => {
  const [setup, managedRuntime, persistence] = await Promise.all([
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/whatsapp_managed_setup.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('src/whatsapp-runtime/src/managed.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/whatsapp-persistence/src/durable.rs', BACKEND_ROOT),
      'utf8',
    ),
  ]);

  assert.match(setup, /ManagedIntegrationHostBridgeConfigurationV1/);
  assert.match(
    setup,
    /managed_launch::start_staged_with_host_bridge_configuration/,
  );
  assert.match(setup, /reservation\.runtime_generation\(\)/);
  assert.match(setup, /reservation\.grant_epoch\(\)/);
  assert.match(setup, /route_binding_sha256/);
  assert.doesNotMatch(setup, /makosh_communications_(?:runtime|persistence)/);
  assert.doesNotMatch(
    managedRuntime,
    /durable\s*\.\s*initialize\s*\(/,
    'Storage Control applies the admitted bundle; WhatsApp runtime cannot run DDL',
  );
  assert.match(persistence, /\.database\(binding\.access\(\)\.pool_alias\(\)\)/);
  assert.match(
    persistence,
    /max_connections\(u32::from\(\s*binding\.access\(\)\.effective_budgets\(\)\.max_connections\(\)/,
  );
});
