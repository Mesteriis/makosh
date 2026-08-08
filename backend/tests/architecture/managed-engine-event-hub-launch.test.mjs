import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const REPOSITORY_ROOT = new URL('../../../', import.meta.url);
const ADR_PATH = new URL(
  'docs/adr/ADR-0358-capability-scoped-engine-event-hub-launch-configuration.md',
  REPOSITORY_ROOT,
);

async function backendSource(path) {
  return readFile(new URL(path, BACKEND_ROOT), 'utf8');
}

test('managed Engine Event Hub configuration is capability-scoped without fake grants', async () => {
  const [adr, validator, ownerControl, aiAdmission, attachmentAdmission] = await Promise.all([
    readFile(ADR_PATH, 'utf8'),
    backendSource('src/platform/runtime_protocol/src/validation/managed_engine_runtime.rs'),
    backendSource('src/kernel/src/identity/owner_control/dispatch.rs'),
    backendSource('src/ai-inference-runtime/src/admission.rs'),
    backendSource('src/attachment-security-runtime/src/admission.rs'),
  ]);

  assert.match(adr, /approved event route absent[\s\S]*empty endpoint \+ credential_revision = 0/);
  assert.match(adr, /AI inference engine не получает пустой event adapter/);
  assert.match(validator, /valid_event_hub_configuration/);
  assert.match(
    validator,
    /\(endpoint\.is_empty\(\) && credential_revision == 0\)[\s\S]*credential_revision != 0/,
  );
  assert.match(validator, /accepts_an_exact_eventless_engine_configuration/);
  assert.match(validator, /rejects_a_partial_event_hub_configuration/);
  assert.match(ownerControl, /engine_event_hub_configuration/);
  assert.match(ownerControl, /capability_scoped_event_hub_configuration/);
  assert.match(ownerControl, /module_event_route_requests\(registration_id, capability_id\)/);
  const engineStart = ownerControl.slice(
    ownerControl.indexOf('fn start_reserved_engine_runtime'),
    ownerControl.indexOf('fn start_reserved_workflow_runtime'),
  );
  assert.match(engineStart, /engine_event_hub_configuration/);
  assert.doesNotMatch(engineStart, /platform_event_hub_topology/);
  assert.match(aiAdmission, /ProvidedSurfaceKindV1::RequestRpc/);
  assert.doesNotMatch(aiAdmission, /DurablePublisher|DurableConsumer|EventRouteDirectionV1/);
  assert.match(attachmentAdmission, /DurablePublisher/);
  assert.match(attachmentAdmission, /DurableConsumer/);
});

test('signed managed AI conformance uses the exact eventless pair and no event adapter', async () => {
  const [setup, flow, script] = await Promise.all([
    backendSource(
      'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/ai_inference_managed_setup.rs',
    ),
    backendSource(
      'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/ai_inference_managed_flow.rs',
    ),
    backendSource('scripts/test-authenticated-storage.mjs'),
  ]);

  assert.match(setup, /event_hub_endpoint: String::new\(\)/);
  assert.match(setup, /event_credential_revision: 0/);
  assert.doesNotMatch(setup, /EventHub|Nats|event_adapter|fake_event/i);
  assert.match(flow, /managed_ai_inference_routes_to_ollama_and_replays_after_restart/);
  assert.match(flow, /managed_ai_inference_completes_real_provider_generation/);
  assert.match(flow, /"owner-2"/);
  assert.match(flow, /stop\(&ollama\.registration_id\)/);
  assert.match(flow, /restart_ai_inference_runtime_v1/);
  assert.match(flow, /assert_eq!\(replayed, first\)/);
  assert.match(flow, /conflicting\.maximum_output_tokens \+= 1/);
  assert.match(script, /'makosh-ai-inference-runtime'/);
  assert.match(
    script,
    /managed_ai_inference_routes_to_ollama_and_replays_after_restart/,
  );
  assert.match(script, /MAKOSH_AI_INFERENCE_RUNTIME_BIN/);
});
