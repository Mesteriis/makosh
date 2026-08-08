import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const REPOSITORY_ROOT = new URL('../../../', import.meta.url);

async function backendSource(path) {
  return readFile(new URL(path, BACKEND_ROOT), 'utf8');
}

test('communication summary agreement keeps workflow domain engine and integration separate', async () => {
  const [adr, inventorySource] = await Promise.all([
    readFile(
      new URL(
        'docs/adr/ADR-0362-communication-summary-workflow-and-ai-contracts.md',
        REPOSITORY_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('architecture/communications-settings-reconstruction.json', BACKEND_ROOT)),
  ]);
  const inventory = JSON.parse(inventorySource);
  const slice = inventory.slices.find(({ gate }) => gate === 'communication_summary_v1');

  assert.deepEqual(slice, {
    gate: 'communication_summary_v1',
    role: 'workflow',
    owner: 'communication_summary',
    state: 'implemented',
    dependsOn: [
      'communications_ai_context_source_v1',
      'ai_inference_v1',
      'ollama_ai_provider_v1',
      'capability_routed_module_request_rpc_v1',
      'blob_v1',
    ],
  });
  for (const unit of [
    'makosh-communication-summary-api',
    'makosh-communication-summary-core',
    'makosh-communication-summary-persistence',
    'makosh-communication-summary-runtime',
    'makosh-communication-summary-assembly',
  ]) {
    assert.match(adr, new RegExp(`\\b${unit}\\b`));
  }
  assert.match(adr, /ai\.summary\.request\.v1/);
  assert.match(adr, /ai\.provider\.summarize\.v1/);
  assert.match(adr, /existing managed workflow admission/);
  assert.match(adr, /Kernel\/Gateway не компилируют summary schema/);
  assert.match(adr, /Task\/note\/deadline extraction не смешивается/);
  assert.match(adr, /Gate[\s\S]*`communication_summary_v1`[\s\S]*закрыт/);
  assert.doesNotMatch(adr, /generic `execute\(any\)` разрешён|Communications owns summary/i);
});

test('Communications summary source is a distinct event and target-bound Blob handoff', async () => {
  const [runtime, admission, eventRuntime, sourceApi, replyRuntime] = await Promise.all([
    readFile(new URL('src/communications-runtime/src/summary_source.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communications-runtime/src/admission.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communications-runtime/src/event_runtime.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communications-ai-source-api/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communications-runtime/src/ai_source.rs', BACKEND_ROOT), 'utf8'),
  ]);

  assert.match(runtime, /PrepareCommunicationSummarySourceCommandV1/);
  assert.match(runtime, /build_communication_summary_source_prepared_outbox_record_v1/);
  assert.match(runtime, /build_communication_summary_source_rejected_outbox_record_v1/);
  assert.match(runtime, /COMMUNICATION_SUMMARY_SOURCE_BLOB_TARGET_OWNER_ID_V1/);
  assert.match(runtime, /COMMUNICATION_SUMMARY_SOURCE_BLOB_TARGET_MODULE_ID_V1/);
  assert.match(runtime, /COMMUNICATION_SUMMARY_SOURCE_BLOB_TARGET_CAPABILITY_ID_V1/);
  assert.match(runtime, /communications-ai-summary-source-copy-v1/);
  assert.match(admission, /communications\.ai-summary-source\.v1/);
  assert.match(admission, /communications\.ai-summary-source\.blob\.v1/);
  assert.match(eventRuntime, /communication_summary_source_prepare_contract_reference_v1/);
  assert.match(eventRuntime, /CommunicationsConsumerV1::SummarySourcePrepare/);
  assert.match(sourceApi, /"communication_summary"/);
  assert.match(sourceApi, /"makosh-communication-summary-runtime"/);
  assert.doesNotMatch(runtime, /makosh_ollama|ollama|provider_sdk|provider identity/i);
  assert.doesNotMatch(replyRuntime, /PrepareCommunicationSummarySourceCommandV1/);
});

test('summary API and core are isolated concrete workflow units', async () => {
  const [apiManifest, api, proto, coreManifest, core] = await Promise.all([
    backendSource('src/communication-summary-api/Cargo.toml'),
    backendSource('src/communication-summary-api/src/lib.rs'),
    backendSource(
      'src/communication-summary-api/proto/makosh/communication_summary/v1/summary.proto',
    ),
    backendSource('src/communication-summary-core/Cargo.toml'),
    backendSource('src/communication-summary-core/src/lib.rs'),
  ]);

  for (const manifest of [apiManifest, coreManifest]) {
    assert.match(manifest, /role = "workflow"/);
    assert.match(manifest, /owner = "communication_summary"/);
    assert.doesNotMatch(
      manifest,
      /makosh-communications-|makosh-ai-inference|makosh-ollama|makosh-mail|makosh-telegram/,
    );
  }
  assert.match(apiManifest, /surface = "contract"/);
  assert.match(coreManifest, /surface = "implementation"/);
  assert.match(api, /communication\.summary\.v1/);
  assert.match(proto, /message StartCommunicationSummaryRequestV1/);
  assert.match(proto, /message CommunicationSummaryCandidateV1/);
  assert.match(proto, /COMMUNICATION_SUMMARY_LANGUAGE_SPANISH/);
  assert.match(proto, /COMMUNICATION_SUMMARY_LENGTH_DETAILED/);
  assert.match(core, /transition_communication_summary_v1/);
  assert.match(core, /CommunicationSummaryStateV1::PreparingSource/);
  assert.match(core, /CommunicationSummaryStateV1::AwaitingInference/);
  assert.match(core, /CommunicationSummaryStateV1::Ready/);
  assert.doesNotMatch(
    proto,
    /provider_id|model_id|endpoint|prompt|source_body|action_items|deadlines|map</,
  );
  assert.doesNotMatch(
    `${coreManifest}\n${core}`,
    /communications|ai-inference|ollama|provider|model|prompt|sqlx|kernel|gateway|reqwest/,
  );
});

test('summary persistence owns atomic workflow state without foreign storage or private content', async () => {
  const [manifest, repository, outbox, realtime, schema, migration] = await Promise.all([
    backendSource('src/communication-summary-persistence/Cargo.toml'),
    backendSource('src/communication-summary-persistence/src/repository.rs'),
    backendSource('src/communication-summary-persistence/src/outbox.rs'),
    backendSource('src/communication-summary-persistence/src/realtime.rs'),
    backendSource('src/communication-summary-persistence/src/schema.rs'),
    backendSource('src/communication-summary-persistence/migrations/0001_summary.sql'),
  ]);

  assert.match(manifest, /role = "workflow"/);
  assert.match(manifest, /owner = "communication_summary"/);
  assert.match(manifest, /surface = "persistence"/);
  assert.match(repository, /create_run/);
  assert.match(repository, /persist_source_result/);
  assert.match(repository, /persist_inference_transition/);
  assert.match(repository, /load_recoverable_runs/);
  assert.match(repository, /transaction\.commit\(\)/);
  assert.match(outbox, /unpublished_source_prepare_events/);
  assert.match(realtime, /client_realtime_window/);
  assert.match(schema, /owner_id: "communication_summary"/);
  assert.match(migration, /CREATE TABLE makosh_data\.communication_summary_runs/);
  assert.match(migration, /UNIQUE \(logical_owner_id, operation_id\)/);
  assert.match(migration, /CREATE TABLE makosh_data\.communication_summary_inbox/);
  assert.match(migration, /CREATE TABLE makosh_data\.communication_summary_outbox/);
  assert.match(migration, /CREATE TABLE makosh_data\.communication_summary_realtime/);
  assert.doesNotMatch(
    `${manifest}\n${migration}`,
    /communications_|mail_|telegram_|whatsapp_|zulip_|source_body|prompt|provider_id|model_id|endpoint/,
  );
});

test('summary runtime and assembly expose only exact event request and release boundaries', async () => {
  const [
    policySource,
    manifest,
    admission,
    sourceResults,
    inference,
    realtime,
    managed,
    assemblyManifest,
    assembly,
    release,
  ] = await Promise.all([
      backendSource('architecture/policy.json'),
      backendSource('src/communication-summary-runtime/Cargo.toml'),
      backendSource('src/communication-summary-runtime/src/admission.rs'),
      backendSource('src/communication-summary-runtime/src/source_results.rs'),
      backendSource('src/communication-summary-runtime/src/inference.rs'),
      backendSource('src/communication-summary-runtime/src/client_realtime.rs'),
      backendSource('src/communication-summary-runtime/src/managed_runtime.rs'),
      backendSource('src/communication-summary-assembly/Cargo.toml'),
      backendSource('src/communication-summary-assembly/src/lib.rs'),
      backendSource('scripts/materialize-dev-release.sh'),
    ]);
  const policy = JSON.parse(policySource);

  assert.equal(
    policy.implementation.currentSlice,
    'call_transcription_managed_conformance_v1',
  );
  assert.deepEqual(
    policy.implementation.productionPackages
      .filter(({ owner }) => owner === 'communication_summary')
      .map(({ name, surface }) => [name, surface]),
    [
      ['makosh-communication-summary-api', 'contract'],
      ['makosh-communication-summary-core', 'implementation'],
      ['makosh-communication-summary-persistence', 'persistence'],
      ['makosh-communication-summary-runtime', 'runtime'],
      ['makosh-communication-summary-assembly', 'assembly'],
    ],
  );
  assert.match(manifest, /role = "workflow"/);
  assert.match(manifest, /owner = "communication_summary"/);
  assert.match(manifest, /surface = "runtime"/);
  assert.match(admission, /ModuleKindV1::Workflow/);
  assert.match(admission, /ProvidedSurfaceKindV1::ClientRpc/);
  assert.match(admission, /ProvidedSurfaceKindV1::ClientRealtime/);
  assert.match(admission, /ProvidedSurfaceKindV1::DurablePublisher/);
  assert.match(admission, /ProvidedSurfaceKindV1::DurableConsumer/);
  assert.match(admission, /communication_summary_inference_contract_reference_v1/);
  assert.match(admission, /BlobQuotaOperationV1::CustodyTransfer/);
  assert.match(sourceResults, /receive_runtime_pull_delivery/);
  assert.match(sourceResults, /materialize_summary_source_for_ai_v1/);
  assert.match(sourceResults, /delivery\.acknowledge\(\)/);
  assert.match(inference, /Operation::RouteModuleRequest/);
  assert.match(inference, /persist_inference_transition/);
  assert.match(realtime, /Operation::PublishClientRealtime/);
  assert.match(managed, /recover_accepted_communication_summary_once_v1/);
  assert.match(assemblyManifest, /surface = "assembly"/);
  assert.match(assembly, /communication_summary\.release-artifacts\.json/);
  assert.match(assembly, /communication_summary_storage_bundle_v1/);
  assert.match(release, /--package makosh-communication-summary-runtime/);
  assert.match(release, /--package makosh-communication-summary-assembly/);
  assert.match(release, /communication_summary\.release-artifacts\.json/);
  assert.doesNotMatch(
    `${manifest}\n${assemblyManifest}`,
    /makosh-communications-runtime|makosh-ai-inference-(core|runtime|persistence)|makosh-ollama/,
  );
});

test('signed Summary conformance crosses Gateway SSE event and request boundaries', async () => {
  const [setup, flow, managedScript] = await Promise.all([
    backendSource(
      'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/communication_summary_managed_setup.rs',
    ),
    backendSource(
      'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/communication_summary_managed_flow.rs',
    ),
    backendSource('scripts/test-authenticated-storage.mjs'),
  ]);

  assert.match(setup, /communication_summary_release_artifact_v1/);
  assert.match(setup, /communication_summary_storage_bundle_v1/);
  assert.match(setup, /start_communication_summary_runtime_v1/);
  assert.match(setup, /restart_communication_summary_runtime_v1/);
  assert.match(flow, /managed_communication_summary_reaches_ai_and_replays_through_gateway_sse/);
  assert.match(flow, /managed_communication_summary_completes_real_provider_through_gateway_sse/);
  assert.match(flow, /authenticate_gateway_router/);
  assert.match(flow, /COMMUNICATION_SUMMARY_COMMAND_CONNECT_PATH_V1/);
  assert.match(flow, /COMMUNICATION_SUMMARY_QUERY_CONNECT_PATH_V1/);
  assert.match(flow, /read_terminal_summary_sse_event/);
  assert.match(flow, /route_communication_summary_as/);
  assert.match(flow, /assert_communication_summary_runtime_fences/);
  assert.match(flow, /stale Communication Summary runtime generation/);
  assert.match(flow, /stale Communication Summary grant epoch/);
  assert.match(flow, /conflicting_request/);
  assert.match(flow, /CommunicationSummaryErrorCodeInvalidRequest/);
  assert.match(flow, /CommunicationSummaryErrorCodeSourceRejected/);
  assert.match(flow, /transition_registration/);
  assert.match(flow, /assert_private_content_absent/);
  assert.match(managedScript, /makosh-communication-summary-runtime/);
  assert.match(
    managedScript,
    /managed_communication_summary_reaches_ai_and_replays_through_gateway_sse/,
  );
  assert.doesNotMatch(
    `${setup}\n${flow}`,
    /makosh_communication_reply_suggestion|makosh_mail_runtime|makosh_telegram_runtime|makosh_whatsapp_runtime|makosh_zulip_runtime/,
  );
});
