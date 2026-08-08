import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const REPOSITORY_ROOT = new URL('../../../', import.meta.url);

const ADR_PATH = new URL(
  'docs/adr/ADR-0353-communication-reply-suggestion-and-ai-inference-boundary.md',
  REPOSITORY_ROOT,
);
const RENEWABLE_AUTHORITY_ADR_PATH = new URL(
  'docs/adr/ADR-0356-renewable-blob-authority-for-durable-ai-workflows.md',
  REPOSITORY_ROOT,
);
const POLICY_PATH = new URL('architecture/policy.json', BACKEND_ROOT);
const INVENTORY_PATH = new URL(
  'architecture/communications-settings-reconstruction.json',
  BACKEND_ROOT,
);

async function backendSource(path) {
  return readFile(new URL(path, BACKEND_ROOT), 'utf8');
}

test('reply suggestion agreement keeps domain workflow engine and integration separate', async () => {
  const [adr, policySource, inventorySource] = await Promise.all([
    readFile(ADR_PATH, 'utf8'),
    readFile(POLICY_PATH, 'utf8'),
    readFile(INVENTORY_PATH, 'utf8'),
  ]);
  const policy = JSON.parse(policySource);
  const inventory = JSON.parse(inventorySource);
  const slices = new Map(inventory.slices.map((slice) => [slice.gate, slice]));

  assert.equal(policy.aiContext.firstConcreteUseCase, 'communication_reply_suggestion_v1');
  assert.equal(policy.aiContext.firstConcreteUseCaseAdr, 'ADR-0353');
  assert.equal(
    policy.aiContext.communicationsPrivateContentHandoff,
    'event_backed_target_bound_blob_custody_v1',
  );
  assert.equal(policy.aiContext.clientContentTicketReuseForWorkflowEnabled, false);
  assert.equal(policy.aiContext.inferenceOwnerRole, 'engine');
  assert.equal(policy.aiContext.firstProviderIntegration, 'ollama_ai_provider_v1');
  assert.equal(policy.aiContext.firstProviderEgressPolicy, 'local_loopback_only');
  assert.equal(policy.aiContext.callerSelectedProviderOrModelEnabled, false);
  assert.equal(policy.aiContext.providerImplementationInsideInferenceOwnerEnabled, false);

  assert.deepEqual(slices.get('communications_ai_context_source_v1'), {
    gate: 'communications_ai_context_source_v1',
    role: 'domain',
    owner: 'communications',
    state: 'implemented',
    dependsOn: ['communications_content_read_v1', 'nats_data_plane_v1', 'blob_v1'],
  });
  assert.deepEqual(slices.get('ai_inference_v1'), {
    gate: 'ai_inference_v1',
    role: 'engine',
    owner: 'ai',
    state: 'implemented',
    dependsOn: [
      'capability_routed_module_request_rpc_v1',
      'blob_v1',
      'ollama_ai_provider_v1',
    ],
  });
  assert.deepEqual(slices.get('ollama_ai_provider_v1'), {
    gate: 'ollama_ai_provider_v1',
    role: 'integration',
    owner: 'ollama',
    state: 'implemented',
    dependsOn: [
      'capability_routed_module_request_rpc_v1',
      'managed_integration_settings_apply_v1',
    ],
  });
  assert.deepEqual(slices.get('communication_reply_suggestion_v1'), {
    gate: 'communication_reply_suggestion_v1',
    role: 'workflow',
    owner: 'communication_reply_suggestion',
    state: 'implemented',
    dependsOn: [
      'communications_ai_context_source_v1',
      'ai_inference_v1',
      'capability_routed_module_request_rpc_v1',
      'blob_v1',
    ],
  });

  assert.match(adr, /makosh-communications-ai-source-api/);
  assert.match(adr, /makosh-ai-contracts/);
  assert.match(adr, /makosh-communication-reply-suggestion-api/);
  assert.match(adr, /makosh-ollama-ai-api/);
  assert.match(adr, /makosh-ollama-ai-persistence/);
  assert.match(adr, /Ollama `\/api\/chat` не предоставляет доказанного idempotency key/);
  assert.match(adr, /Client content ticket из ADR-0315 не используется/);
  assert.match(adr, /Mock or canned response не\s+является production\s+evidence/);
  assert.doesNotMatch(
    adr,
    /Gateway (?:fetches|reads) (?:the )?message body|generic ai context workflow/i,
  );
});

test('Communications AI source is one provider-neutral event contract unit', async () => {
  const [manifest, api, envelope, proto] = await Promise.all([
    backendSource('src/communications-ai-source-api/Cargo.toml'),
    backendSource('src/communications-ai-source-api/src/lib.rs'),
    backendSource('src/communications-ai-source-api/src/envelope.rs'),
    backendSource(
      'src/communications-ai-source-api/proto/makosh/communications/ai_source/v1/ai_source.proto',
    ),
  ]);

  assert.match(manifest, /role = "domain"/);
  assert.match(manifest, /owner = "communications"/);
  assert.match(manifest, /surface = "contract"/);
  assert.doesNotMatch(
    manifest,
    /communication-reply-suggestion|ollama|ai-inference|sqlx|kernel|gateway/,
  );
  assert.match(api, /communication_reply_source_prepare/);
  assert.match(api, /communication_reply_source_prepared/);
  assert.match(api, /communication_reply_source_rejected/);
  assert.match(api, /communication_reply_suggestion\.source\.blob\.v1/);
  assert.match(envelope, /DurableEnvelopeV1/);
  assert.match(envelope, /target_capability: COMMUNICATIONS_AI_SOURCE_CAPABILITY_ID_V1/);
  assert.match(envelope, /validate_envelope_v1/);
  assert.match(proto, /uint64 expected_source_revision = 3/);
  assert.match(proto, /message CommunicationReplySourceContentV1/);
  assert.match(proto, /bytes sender_utf8 = 1/);
  assert.match(proto, /bytes subject_utf8 = 2/);
  assert.match(proto, /bytes body_utf8 = 3/);
  assert.match(proto, /CommunicationReplySourceContentReceiptV1 source_content = 5/);
  assert.match(proto, /bytes custody_transfer_source_proof = 4/);
  assert.doesNotMatch(proto, /CommunicationReplyBodySourceReceiptV1|body_source/);
  assert.doesNotMatch(
    `${api}\n${envelope}\n${proto}`,
    /provider_id|provider_account|provider_locator|model_id|model_key|prompt|string target_owner|string target_module|string target_capability|message_body|body_text/,
  );
});

test('canonical Mail subject and sender reach reply source content without a provider facade', async () => {
  const [
    ingressProto,
    ingress,
    mailCore,
    mailRuntime,
    canonicalApi,
    migration,
    persistence,
    sourceRuntime,
    workflowRuntime,
  ] = await Promise.all([
    backendSource(
      'src/communications-ingress/proto/makosh/communications/ingress/v1/observation.proto',
    ),
    backendSource('src/communications-ingress/src/lib.rs'),
    backendSource('src/mail-core/src/lib.rs'),
    backendSource('src/mail-runtime/src/managed.rs'),
    backendSource('src/communications-api/src/lib.rs'),
    backendSource(
      'src/communications-persistence/migrations/0015_communications_message_subject.sql',
    ),
    backendSource('src/communications-persistence/src/source_snapshot.rs'),
    backendSource('src/communications-runtime/src/ai_source.rs'),
    backendSource('src/communication-reply-suggestion-runtime/src/blob_materialization.rs'),
  ]);

  assert.match(ingressProto, /optional string message_subject = 15/);
  assert.match(ingress, /MAX_MESSAGE_SUBJECT_BYTES/);
  assert.match(ingress, /with_message_subject/);
  assert.match(mailCore, /draft_ingress_observation_with_sender_subject_body/);
  assert.match(mailCore, /with_message_subject/);
  assert.match(mailRuntime, /message\.subject\.clone\(\)/);
  assert.match(mailRuntime, /preview\.subject\.clone\(\)/);
  assert.match(canonicalApi, /pub message_subject: Option<String>/);
  assert.match(migration, /ADD COLUMN message_subject TEXT/);
  assert.match(persistence, /evidence\.message_subject/);
  assert.match(persistence, /sender_utf8/);
  assert.match(persistence, /subject_utf8/);
  assert.match(sourceRuntime, /CommunicationReplySourceContentV1/);
  assert.match(sourceRuntime, /encode_communication_reply_source_content_v1/);
  assert.match(workflowRuntime, /decode_communication_reply_source_content_v1/);
  assert.match(workflowRuntime, /sender_utf8: source_content\.sender_utf8/);
  assert.match(workflowRuntime, /subject_utf8: source_content\.subject_utf8/);
  assert.doesNotMatch(
    `${mailCore}\n${sourceRuntime}\n${workflowRuntime}`,
    /provider\s*==|match\s+provider|MailImap\s*=>|MailGmail\s*=>/,
  );
});

test('Communications AI source runtime commits an owner-bound event handoff before ack', async () => {
  const [manifest, persistence, runtime, admission, eventRuntime, managedFlow, managedScript] =
    await Promise.all([
    backendSource('src/communications-runtime/Cargo.toml'),
    backendSource('src/communications-persistence/src/source_snapshot.rs'),
    backendSource('src/communications-runtime/src/ai_source.rs'),
    backendSource('src/communications-runtime/src/admission.rs'),
    backendSource('src/communications-runtime/src/event_runtime.rs'),
    backendSource(
      'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/communications_ai_source_managed_flow.rs',
    ),
    backendSource('scripts/test-authenticated-storage.mjs'),
  ]);

  assert.match(manifest, /makosh-communications-ai-source-api/);
  assert.match(persistence, /communications_event_inbox/);
  assert.match(persistence, /communications_domain_outbox/);
  assert.match(persistence, /canonical_revision/);
  assert.match(persistence, /last_evidence_id/);
  assert.match(persistence, /body_blob_reference_id/);
  assert.match(persistence, /body_blob_declared_bytes/);
  assert.match(persistence, /body_blob_sha256/);
  assert.match(persistence, /transaction\s*\.commit\(\)/);
  assert.match(runtime, /payload\.logical_owner_id != logical_human_owner_id/);
  assert.match(runtime, /COMMUNICATION_REPLY_SOURCE_BLOB_TARGET_OWNER_ID_V1/);
  assert.match(runtime, /COMMUNICATION_REPLY_SOURCE_BLOB_TARGET_MODULE_ID_V1/);
  assert.match(runtime, /COMMUNICATION_REPLY_SOURCE_BLOB_TARGET_CAPABILITY_ID_V1/);
  assert.match(runtime, /COMMUNICATIONS_AI_SOURCE_BLOB_CAPABILITY_ID/);
  assert.match(admission, /communications_ai_source_blob_capability_v1/);
  assert.match(admission, /communications_ai_source_capability_v1/);
  assert.match(eventRuntime, /communication_reply_source_prepare_contract_reference_v1/);
  assert.match(eventRuntime, /CommunicationsConsumerV1::AiSourcePrepare/);
  assert.match(
    managedFlow,
    /managed_communications_ai_source_is_event_only_and_revision_fenced/,
  );
  assert.match(managedFlow, /CommunicationReplySourceRejectCodeStaleRevision/);
  assert.match(managedFlow, /CommunicationReplySourceRejectCodeSourceMissingOrInactive/);
  assert.match(managedFlow, /assert_private_content_absent/);
  assert.match(managedScript, /managed_communications_ai_source_is_event_only_and_revision_fenced/);

  const persisted = runtime.indexOf('.persist_source_result(');
  const acknowledged = runtime.indexOf('delivery.acknowledge()', persisted);
  assert.ok(persisted >= 0);
  assert.ok(acknowledged > persisted);
  assert.doesNotMatch(
    `${persistence}\n${runtime}`,
    /provider_id|provider_account|provider_locator|model_id|model_key|prompt|ollama/,
  );
});

test('reply suggestion client API is one concrete provider-neutral workflow contract', async () => {
  const [manifest, api, proto] = await Promise.all([
    backendSource('src/communication-reply-suggestion-api/Cargo.toml'),
    backendSource('src/communication-reply-suggestion-api/src/lib.rs'),
    backendSource(
      'src/communication-reply-suggestion-api/proto/makosh/communication_reply_suggestion/v1/reply_suggestion.proto',
    ),
  ]);

  assert.match(manifest, /role = "workflow"/);
  assert.match(manifest, /owner = "communication_reply_suggestion"/);
  assert.match(manifest, /surface = "contract"/);
  assert.match(api, /communication\.reply_suggestion\.v1/);
  assert.match(api, /CommunicationReplySuggestionCommandService\/Start/);
  assert.match(api, /CommunicationReplySuggestionQueryService\/Get/);
  assert.match(proto, /message StartReplySuggestionRequestV1/);
  assert.match(proto, /uint64 expected_source_revision = 4/);
  assert.match(proto, /message ReplySuggestionCandidateV1/);
  assert.match(proto, /REPLY_SUGGESTION_TONE_PROFESSIONAL/);
  assert.match(proto, /REPLY_SUGGESTION_LANGUAGE_SPANISH/);
  assert.doesNotMatch(manifest, /communications-|ai-inference|ollama/);
  assert.doesNotMatch(api, /communications-|ai-inference|ollama/);
  assert.doesNotMatch(
    proto,
    /provider_id|model_id|endpoint|prompt|source_body|google\.protobuf\.Any|map</,
  );
});

test('reply suggestion core owns only the bounded workflow state machine', async () => {
  const [manifest, core] = await Promise.all([
    backendSource('src/communication-reply-suggestion-core/Cargo.toml'),
    backendSource('src/communication-reply-suggestion-core/src/lib.rs'),
  ]);

  assert.match(manifest, /role = "workflow"/);
  assert.match(manifest, /owner = "communication_reply_suggestion"/);
  assert.match(manifest, /surface = "implementation"/);
  assert.match(core, /validate_reply_suggestion_draft_v1/);
  assert.match(core, /transition_reply_suggestion_v1/);
  assert.match(core, /ReplySuggestionStateV1::PreparingSource/);
  assert.match(core, /ReplySuggestionStateV1::AwaitingInference/);
  assert.match(core, /ReplySuggestionStateV1::Ready/);
  assert.match(core, /current\.inference_request_digest != Some\(candidate\.request_digest\)/);
  assert.match(core, /current\.source_sha256 != Some\(candidate\.source_sha256\)/);
  assert.doesNotMatch(
    `${manifest}\n${core}`,
    /communications|ai-inference|ollama|provider|model|endpoint|prompt|sqlx|kernel|gateway|reqwest|async.nats/,
  );
});

test('reply suggestion persistence is owner-local atomic replay state without private source', async () => {
  const [manifest, api, model, repository, outbox, realtime, schema, migration] =
    await Promise.all([
      backendSource('src/communication-reply-suggestion-persistence/Cargo.toml'),
      backendSource('src/communication-reply-suggestion-persistence/src/lib.rs'),
      backendSource('src/communication-reply-suggestion-persistence/src/model.rs'),
      backendSource('src/communication-reply-suggestion-persistence/src/repository.rs'),
      backendSource('src/communication-reply-suggestion-persistence/src/outbox.rs'),
      backendSource('src/communication-reply-suggestion-persistence/src/realtime.rs'),
      backendSource('src/communication-reply-suggestion-persistence/src/schema.rs'),
      backendSource(
        'src/communication-reply-suggestion-persistence/migrations/0001_reply_suggestion.sql',
      ),
    ]);

  assert.match(manifest, /role = "workflow"/);
  assert.match(manifest, /owner = "communication_reply_suggestion"/);
  assert.match(manifest, /surface = "persistence"/);
  assert.match(api, /CommunicationReplySuggestionPersistenceV1/);
  assert.match(model, /request_fingerprint/);
  assert.match(repository, /create_run/);
  assert.match(repository, /persist_source_result/);
  assert.match(repository, /persist_inference_transition/);
  assert.match(repository, /load_recoverable_runs/);
  assert.match(repository, /transition_reply_suggestion_v1/);
  assert.match(repository, /transaction\.commit\(\)/);
  assert.match(outbox, /unpublished_source_prepare_events/);
  assert.match(outbox, /mark_source_prepare_published/);
  assert.match(realtime, /client_realtime_window/);
  assert.match(schema, /owner_id: "communication_reply_suggestion"/);
  assert.match(migration, /CREATE TABLE makosh_data\.communication_reply_suggestion_runs/);
  assert.match(migration, /UNIQUE \(logical_owner_id, operation_id\)/);
  assert.match(migration, /CREATE TABLE makosh_data\.communication_reply_suggestion_inbox/);
  assert.match(migration, /CREATE TABLE makosh_data\.communication_reply_suggestion_outbox/);
  assert.match(migration, /CREATE TABLE makosh_data\.communication_reply_suggestion_realtime/);
  assert.doesNotMatch(
    `${manifest}\n${model}\n${repository}\n${outbox}\n${realtime}\n${migration}`,
    /communications_|mail_|telegram_|whatsapp_|zulip_|source_body|prompt|provider_id|model_id|endpoint|serde_json|google\.protobuf\.Any|map</,
  );
});

test('reply suggestion runtime coordinates event source, AI request, Blob custody, and client SSE', async () => {
  const [adr, manifest, admission, sourceResults, materialization, inference, realtime, runtime, processRoot] =
    await Promise.all([
      readFile(RENEWABLE_AUTHORITY_ADR_PATH, 'utf8'),
      backendSource('src/communication-reply-suggestion-runtime/Cargo.toml'),
      backendSource('src/communication-reply-suggestion-runtime/src/admission.rs'),
      backendSource('src/communication-reply-suggestion-runtime/src/source_results.rs'),
      backendSource('src/communication-reply-suggestion-runtime/src/blob_materialization.rs'),
      backendSource('src/communication-reply-suggestion-runtime/src/inference.rs'),
      backendSource('src/communication-reply-suggestion-runtime/src/client_realtime.rs'),
      backendSource('src/communication-reply-suggestion-runtime/src/managed_runtime.rs'),
      backendSource('src/communication-reply-suggestion-runtime/src/main.rs'),
    ]);

  assert.match(adr, /Digest не включает `custody_transfer_source_proof`/);
  assert.match(adr, /только затем подтверждает source event/);
  assert.match(manifest, /role = "workflow"/);
  assert.match(manifest, /owner = "communication_reply_suggestion"/);
  assert.match(manifest, /surface = "runtime"/);
  assert.match(admission, /ModuleKindV1::Workflow/);
  assert.match(admission, /ProvidedSurfaceKindV1::ClientRpc/);
  assert.match(admission, /ProvidedSurfaceKindV1::DurablePublisher/);
  assert.match(admission, /ProvidedSurfaceKindV1::DurableConsumer/);
  assert.match(admission, /communication_reply_inference_contract_reference_v1/);
  assert.match(admission, /BlobQuotaOperationV1::CustodyTransfer/);
  assert.match(sourceResults, /receive_runtime_pull_delivery/);
  assert.match(sourceResults, /materialize_reply_source_for_ai_v1/);
  assert.match(sourceResults, /complete_blob_cleanup/);
  assert.match(sourceResults, /delivery\.acknowledge\(\)/);
  assert.match(materialization, /request_managed_blob_custody_transfer_v2/);
  assert.match(materialization, /AI_INFERENCE_BLOB_CAPABILITY_ID_V1/);
  assert.match(materialization, /release_reply_source_blobs_v1/);
  assert.match(inference, /Operation::RouteModuleRequest/);
  assert.match(inference, /persist_inference_transition/);
  assert.match(realtime, /client_realtime_window/);
  assert.match(realtime, /Operation::PublishClientRealtime/);
  assert.match(runtime, /publish_pending/);
  assert.match(runtime, /request\.logical_owner_id == admission\.logical_owner_id/);
  assert.match(processRoot, /ManagedWorkflowRuntimeConfigurationV1/);
  assert.doesNotMatch(
    `${manifest}\n${admission}\n${sourceResults}\n${materialization}\n${inference}\n${runtime}`,
    /makosh-mail|makosh-telegram|makosh-whatsapp|makosh-zulip|makosh-ollama|\bprovider_id\b|\bmodel_id\b|\bprompt_text\b|reqwest|hyper/,
  );
});

test('signed Reply Suggestion conformance crosses only event and request RPC boundaries', async () => {
  const [setup, flow, managedScript, gateway] = await Promise.all([
    backendSource(
      'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/reply_suggestion_managed_setup.rs',
    ),
    backendSource(
      'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/reply_suggestion_managed_flow.rs',
    ),
    backendSource('scripts/test-authenticated-storage.mjs'),
    backendSource('src/kernel/src/platform/gateway.rs'),
  ]);

  assert.match(setup, /communications_release_artifacts/);
  assert.match(setup, /ollama_ai_release_artifact_v1/);
  assert.match(setup, /ai_inference_release_artifact_v1/);
  assert.match(setup, /reply_suggestion_release_artifact_v1/);
  assert.match(setup, /ManagedWorkflowRuntimeConfigurationV1/);
  assert.match(flow, /managed_reply_suggestion_reaches_ai_and_replays_through_gateway_sse/);
  assert.match(
    flow,
    /managed_reply_suggestion_completes_real_provider_through_gateway_sse/,
  );
  assert.match(flow, /required\("MAKOSH_OLLAMA_LIVE_PORT"\)/);
  assert.match(flow, /start_communications_domain/);
  assert.match(flow, /start_reply_suggestion_runtime_v1/);
  assert.match(flow, /start_ai_inference_runtime_v1/);
  assert.match(flow, /start_ollama_ai_runtime_v1/);
  assert.match(flow, /COMMUNICATION_REPLY_SUGGESTION_COMMAND_CONNECT_PATH_V1/);
  assert.match(flow, /\/api\/realtime\/v1\/events/);
  assert.match(flow, /restart_reply_suggestion_runtime_v1/);
  assert.match(flow, /UnavailableOllamaProbeV1/);
  assert.match(flow, /route_reply_suggestion_as/);
  assert.match(flow, /"owner-2"/);
  assert.match(flow, /transition_registration/);
  assert.match(flow, /ModuleRegistrationState::Revoked/);
  assert.match(flow, /PlatformStorageBindingStateV1::Revoking/);
  assert.match(flow, /ReplySuggestionStateReady/);
  assert.match(flow, /stop\(&ollama\.registration_id\)/);
  assert.match(flow, /ReplySuggestionErrorCodeSourceRejected/);
  assert.match(flow, /assert_private_content_absent/);
  assert.match(managedScript, /makosh-communication-reply-suggestion-runtime/);
  assert.match(
    managedScript,
    /managed_reply_suggestion_reaches_ai_and_replays_through_gateway_sse/,
  );
  assert.match(gateway, /"RUNTIME_UNAVAILABLE" => ClientRpcRouteErrorV1::Unavailable/);
  assert.doesNotMatch(
    `${setup}\n${flow}`,
    /makosh_mail_runtime|makosh_telegram_runtime|makosh_whatsapp_runtime|makosh_zulip_runtime/,
  );
});

test('reply suggestion assembly emits only unsigned workflow runtime and storage inputs', async () => {
  const [manifest, assembly, cli, release] = await Promise.all([
    backendSource('src/communication-reply-suggestion-assembly/Cargo.toml'),
    backendSource('src/communication-reply-suggestion-assembly/src/lib.rs'),
    backendSource('src/communication-reply-suggestion-assembly/src/main.rs'),
    backendSource('scripts/materialize-dev-release.sh'),
  ]);

  assert.match(manifest, /role = "workflow"/);
  assert.match(manifest, /owner = "communication_reply_suggestion"/);
  assert.match(manifest, /surface = "assembly"/);
  assert.match(assembly, /communication_reply_suggestion_module_descriptor_v1/);
  assert.match(assembly, /communication_reply_suggestion_settings_schema_v1/);
  assert.match(assembly, /communication_reply_suggestion_storage_bundle_v1/);
  assert.match(assembly, /artifact_kind: "module_runtime"/);
  assert.match(assembly, /artifact_kind: "storage_bundle"/);
  assert.match(assembly, /create_new\(true\)/);
  assert.match(assembly, /mode\(0o600\)/);
  assert.match(cli, /--runtime/);
  assert.match(release, /--package makosh-communication-reply-suggestion-runtime/);
  assert.match(release, /--package makosh-communication-reply-suggestion-assembly/);
  assert.match(release, /communication_reply_suggestion\.release-artifacts\.json/);
  assert.doesNotMatch(
    `${manifest}\n${assembly}\n${cli}`,
    /signing|private_key|launch|makosh-communications|makosh-ollama|\bprovider_id\b|\bmodel_id\b|prompt_text/,
  );
});

test('AI public contracts are one concrete provider-neutral engine unit', async () => {
  const [manifest, api, validation, proto] = await Promise.all([
    backendSource('src/ai-contracts/Cargo.toml'),
    backendSource('src/ai-contracts/src/lib.rs'),
    backendSource('src/ai-contracts/src/validation.rs'),
    backendSource('src/ai-contracts/proto/makosh/ai/contracts/v1/ai.proto'),
  ]);

  assert.match(manifest, /role = "engine"/);
  assert.match(manifest, /owner = "ai"/);
  assert.match(manifest, /surface = "contract"/);
  assert.match(api, /communication_reply_suggestion_inference/);
  assert.match(api, /ai_provider_reply_generation/);
  assert.match(api, /AI_INFERENCE_REQUEST_CAPABILITY_ID_V1/);
  assert.match(api, /AI_PROVIDER_GENERATION_CAPABILITY_ID_V1/);
  assert.match(validation, /compute_reply_inference_request_digest_v1/);
  assert.match(validation, /content\.encoded_len\(\) > AI_MAX_PRIVATE_SOURCE_BYTES_V1/);
  assert.match(validation, /AI_CONTRACTS_SCHEMA_SHA256/);
  assert.match(validation, /AiEgressPolicyLocalOnly/);
  assert.match(proto, /message AiContextReceiptV1/);
  assert.match(proto, /message CommunicationReplySuggestionInferenceRequestV1/);
  assert.match(proto, /message CommunicationReplySuggestionInferenceResultV1/);
  assert.match(proto, /message AiProviderReplyGenerationRequestV1/);
  assert.match(proto, /message AiProviderReplyGenerationResultV1/);
  assert.match(proto, /message AiReplySourceContentV1/);
  assert.match(proto, /bytes sender_utf8 = 1/);
  assert.match(proto, /bytes subject_utf8 = 2/);
  assert.match(proto, /bytes body_utf8 = 3/);
  assert.match(proto, /uint32 maximum_output_bytes/);
  assert.match(proto, /uint32 maximum_output_tokens/);
  assert.match(proto, /AiInferenceCompletenessV1 completeness = 10/);
  assert.match(proto, /uint32 confidence_basis_points = 11/);
  assert.match(proto, /uint64 provider_settings_revision = 12/);
  assert.doesNotMatch(
    `${api}\n${validation}\n${proto}`,
    /(?:string|bytes)\s+(?:provider_id|provider_name|model_id|model_name|endpoint|prompt_text)\b|google\.protobuf\.Any|map<|string target_owner|string target_module|string target_capability/,
  );
  assert.doesNotMatch(manifest, /communications|reply-suggestion|ollama|sqlx|gateway|kernel/);
});

test('AI inference core owns lifecycle and fixed policy without provider implementation', async () => {
  const [manifest, core] = await Promise.all([
    backendSource('src/ai-inference-core/Cargo.toml'),
    backendSource('src/ai-inference-core/src/lib.rs'),
  ]);

  assert.match(manifest, /role = "engine"/);
  assert.match(manifest, /owner = "ai"/);
  assert.match(manifest, /surface = "implementation"/);
  assert.match(manifest, /makosh-ai-contracts/);
  assert.match(core, /AiInferenceRunStateV1/);
  assert.match(core, /accept_reply_inference_v1/);
  assert.match(core, /begin_reply_inference_v1/);
  assert.match(core, /complete_reply_inference_v1/);
  assert.match(core, /reject_reply_inference_v1/);
  assert.match(core, /AI_INFERENCE_PROVIDER_POLICY_REVISION_V1/);
  assert.match(core, /prompt_policy_sha256_v1/);
  assert.match(core, /build_reply_provider_input_v1/);
  assert.match(core, /AI_REPLY_SOURCE_BODY_EXCERPT_BYTES_V1/);
  assert.doesNotMatch(
    `${manifest}\n${core}`,
    /communications|reply-suggestion|ollama|reqwest|hyper|sqlx|gateway|kernel|settings_registry|provider_id|model_id|endpoint/,
  );
});

test('AI inference persistence is typed owner-local and stores no private source body', async () => {
  const [manifest, api, model, repository, schema, migration] = await Promise.all([
    backendSource('src/ai-inference-persistence/Cargo.toml'),
    backendSource('src/ai-inference-persistence/src/lib.rs'),
    backendSource('src/ai-inference-persistence/src/model.rs'),
    backendSource('src/ai-inference-persistence/src/repository.rs'),
    backendSource('src/ai-inference-persistence/src/schema.rs'),
    backendSource('src/ai-inference-persistence/migrations/0001_ai_inference_runs.sql'),
  ]);

  assert.match(manifest, /role = "engine"/);
  assert.match(manifest, /owner = "ai"/);
  assert.match(manifest, /surface = "persistence"/);
  assert.match(api, /AiInferencePersistenceV1/);
  assert.match(model, /validate_transition/);
  assert.match(model, /provider_settings_revision/);
  assert.match(repository, /accept_run/);
  assert.match(repository, /persist_transition/);
  assert.match(repository, /load_recoverable_runs/);
  assert.match(repository, /selected_provider_settings_revision/);
  assert.match(schema, /owner_id: "ai"/);
  assert.match(migration, /CREATE TABLE makosh_data\.ai_inference_runs/);
  assert.match(migration, /request_digest BYTEA/);
  assert.match(migration, /source_reference_id BYTEA/);
  assert.match(migration, /result_body_utf8 BYTEA/);
  assert.doesNotMatch(
    `${manifest}\n${api}\n${model}\n${repository}\n${migration}`,
    /communications_|mail_|telegram_|whatsapp_|zulip_|message_body|provider_id|model_id|endpoint|prompt_text|serde_json|google\.protobuf\.Any|map</,
  );
});

test('AI inference runtime owns exact managed execution without provider implementation', async () => {
  const [manifest, admission, ports, worker, runtime, processRoot, persistence, managedFlow] = await Promise.all([
    backendSource('src/ai-inference-runtime/Cargo.toml'),
    backendSource('src/ai-inference-runtime/src/admission.rs'),
    backendSource('src/ai-inference-runtime/src/managed_ports.rs'),
    backendSource('src/ai-inference-runtime/src/worker.rs'),
    backendSource('src/ai-inference-runtime/src/managed_runtime.rs'),
    backendSource('src/ai-inference-runtime/src/main.rs'),
    backendSource('src/ai-inference-persistence/src/repository.rs'),
    backendSource(
      'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/ai_inference_managed_flow.rs',
    ),
  ]);

  assert.match(manifest, /role = "engine"/);
  assert.match(manifest, /owner = "ai"/);
  assert.match(manifest, /surface = "runtime"/);
  assert.match(admission, /ModuleKindV1::Engine/);
  assert.match(admission, /ProvidedSurfaceKindV1::RequestRpc/);
  assert.match(admission, /ai_provider_reply_generation_contract_reference_v1/);
  assert.match(admission, /BlobQuotaOperationV1::CustodyTransfer/);
  assert.match(admission, /StorageNamespaceRequestV1/);
  assert.match(ports, /request_managed_blob_custody_transfer_v2/);
  assert.match(ports, /BlobDataOperationReadRangeV1/);
  assert.match(ports, /Operation::RouteModuleRequest/);
  assert.match(worker, /accept_reply_inference_v1/);
  assert.match(worker, /persist_transition/);
  assert.match(worker, /complete_reply_inference_v1/);
  assert.match(worker, /reject_reply_inference_v1/);
  assert.match(worker, /load_recoverable_runs/);
  assert.match(runtime, /Operation::DeliverModuleRequest/);
  assert.match(runtime, /recover_pending_v1/);
  assert.match(runtime, /delivery\.logical_owner_id != self\.logical_human_owner_id/);
  assert.match(runtime, /&self\.logical_human_owner_id/);
  assert.match(processRoot, /ManagedEngineRuntimeConfigurationV1/);
  assert.match(processRoot, /logical_human_owner_id: configuration\.logical_human_owner_id/);
  assert.match(processRoot, /validate_settings_snapshot_against_schema_v1/);
  assert.match(persistence, /WHERE logical_owner_id = \$1 AND run_state IN \(1, 2\)/);
  assert.match(
    managedFlow,
    /fn managed_ai_inference_completes_real_provider_generation\(\)/,
  );
  assert.match(managedFlow, /required\("MAKOSH_OLLAMA_LIVE_PORT"\)/);
  assert.match(managedFlow, /AiInferenceTerminalStatusReady/);
  assert.match(managedFlow, /AiInferenceCompletenessComplete/);
  assert.match(managedFlow, /"owner-2"/);
  assert.match(managedFlow, /stop\(&ollama\.registration_id\)/);
  assert.match(managedFlow, /assert_eq!\(replayed, first\)/);
  assert.doesNotMatch(managedFlow, /canned|OllamaAiHttpFixture/i);
  assert.doesNotMatch(
    `${manifest}\n${admission}\n${ports}\n${worker}\n${runtime}`,
    /makosh-communications|communication-reply-suggestion|makosh-ollama|reqwest|hyper|nats|\bprovider_id\b|\bmodel_id\b|endpoint|prompt_text/,
  );
});

test('AI inference assembly emits only unsigned engine runtime and storage inputs', async () => {
  const [manifest, assembly] = await Promise.all([
    backendSource('src/ai-inference-assembly/Cargo.toml'),
    backendSource('src/ai-inference-assembly/src/lib.rs'),
  ]);

  assert.match(manifest, /role = "engine"/);
  assert.match(manifest, /owner = "ai"/);
  assert.match(manifest, /surface = "assembly"/);
  assert.match(assembly, /ai_inference_module_descriptor_v1/);
  assert.match(assembly, /ai_inference_settings_schema_v1/);
  assert.match(assembly, /ai_inference_storage_bundle_v1/);
  assert.match(assembly, /module_runtime/);
  assert.match(assembly, /storage_bundle/);
  assert.match(assembly, /create_new\(true\)/);
  assert.doesNotMatch(
    `${manifest}\n${assembly}`,
    /communications|reply-suggestion|ollama|signing|private_key|provider_id|model_id|endpoint|prompt_text/,
  );
});

test('Ollama API and core are separate integration units with fixed local policy', async () => {
  const [apiManifest, api, settings, coreManifest, core] = await Promise.all([
    backendSource('src/ollama-ai-api/Cargo.toml'),
    backendSource('src/ollama-ai-api/src/lib.rs'),
    backendSource('src/ollama-ai-api/src/settings.rs'),
    backendSource('src/ollama-ai-core/Cargo.toml'),
    backendSource('src/ollama-ai-core/src/lib.rs'),
  ]);

  for (const manifest of [apiManifest, coreManifest]) {
    assert.match(manifest, /role = "integration"/);
    assert.match(manifest, /owner = "ollama"/);
    assert.doesNotMatch(manifest, /communications|reply-suggestion|ai-inference/);
  }
  assert.match(api, /OLLAMA_AI_LOOPBACK_HOST_V1: &str = "127\.0\.0\.1"/);
  assert.match(api, /OLLAMA_AI_MAX_TIMEOUT_MILLIS_V1: u64 = 30_000/);
  assert.match(settings, /SettingTargetScopeV1::ConfigurationInstance/);
  assert.match(settings, /SettingApplyModeV1::RestartModule/);
  assert.match(core, /compute_provider_reply_generation_request_digest_v1/);
  assert.match(core, /OllamaAiRunStateV1::Uncertain/);
  assert.match(core, /No markdown/);
  assert.doesNotMatch(
    `${api}\n${settings}\n${core}`,
    /https?:\/\/(?!127\.0\.0\.1)|provider_id|caller.*model|automatic.*download/i,
  );
});

test('Ollama HTTP owns one bounded loopback dialect without redirects or model substitution', async () => {
  const [manifest, client, model, wire] = await Promise.all([
    backendSource('src/ollama-ai-http/Cargo.toml'),
    backendSource('src/ollama-ai-http/src/lib.rs'),
    backendSource('src/ollama-ai-http/src/model.rs'),
    backendSource('src/ollama-ai-http/src/wire.rs'),
  ]);

  assert.match(manifest, /role = "integration"/);
  assert.match(manifest, /owner = "ollama"/);
  assert.match(manifest, /makosh-ollama-ai-core/);
  assert.match(client, /OLLAMA_AI_LOOPBACK_HOST_V1/);
  assert.match(client, /"GET",\s*"\/api\/tags"/);
  assert.match(client, /"POST",\s*"\/api\/chat"/);
  assert.match(model, /stream: false/);
  assert.match(model, /think: false/);
  assert.match(model, /format: reply_json_schema_v1\(\)/);
  assert.match(model, /required: \["subject", "body", "language"\]/);
  assert.match(model, /additional_properties: false/);
  assert.match(model, /allowed: \["english", "spanish", "russian"\]/);
  assert.match(model, /response\.model != plan\.model/);
  assert.match(wire, /const LOOPBACK_HOST: &str = "127\.0\.0\.1"/);
  assert.match(wire, /Accept-Encoding: identity/);
  assert.match(wire, /\(300\.\.400\)\.contains\(&status\)/);
  assert.match(wire, /MAX_RESPONSE_BYTES/);
  assert.doesNotMatch(`${client}\n${model}\n${wire}`, /reqwest|ureq|automatic.*download/i);
});

test('Ollama persistence fences replay without storing private provider input', async () => {
  const [manifest, model, repository, schema, migration] = await Promise.all([
    backendSource('src/ollama-ai-persistence/Cargo.toml'),
    backendSource('src/ollama-ai-persistence/src/model.rs'),
    backendSource('src/ollama-ai-persistence/src/repository.rs'),
    backendSource('src/ollama-ai-persistence/src/schema.rs'),
    backendSource('src/ollama-ai-persistence/migrations/0001_ollama_ai_runs.sql'),
  ]);

  assert.match(manifest, /role = "integration"/);
  assert.match(manifest, /owner = "ollama"/);
  assert.match(manifest, /surface = "persistence"/);
  assert.match(model, /OllamaAiRunStateV1::Uncertain/);
  assert.match(model, /current\.run\.request_digest != transition\.next_run\.request_digest/);
  assert.match(repository, /ON CONFLICT \(logical_owner_id, request_id\) DO NOTHING/);
  assert.match(repository, /SELECT_RUN_FOR_UPDATE/);
  assert.match(schema, /owner_id: "ollama"/);
  assert.match(migration, /selected_model_revision_sha256/);
  assert.match(migration, /result_provider_settings_revision = settings_revision/);
  assert.doesNotMatch(`${repository}\n${migration}`, /prompt_utf8|input_utf8|http_body|communications_/i);
  assert.doesNotMatch(migration, /password|credentials?|provider_request/i);
});

test('Ollama managed runtime owns provider execution and crash ambiguity fencing', async () => {
  const [manifest, admission, runtime, worker, processRoot, managedFlow, managedSetup] =
    await Promise.all([
    backendSource('src/ollama-ai-runtime/Cargo.toml'),
    backendSource('src/ollama-ai-runtime/src/admission.rs'),
    backendSource('src/ollama-ai-runtime/src/managed_runtime.rs'),
    backendSource('src/ollama-ai-runtime/src/worker.rs'),
    backendSource('src/ollama-ai-runtime/src/main.rs'),
    backendSource(
      'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/ollama_ai_managed_flow.rs',
    ),
    backendSource(
      'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/ollama_ai_managed_setup.rs',
    ),
  ]);

  assert.match(manifest, /surface = "runtime"/);
  assert.match(manifest, /makosh-ollama-ai-http/);
  assert.match(admission, /ModuleKindV1::Integration/);
  assert.match(admission, /ai_provider_reply_generation_contract_reference_v1/);
  assert.match(runtime, /Operation::DeliverModuleRequest/);
  assert.match(runtime, /delivery\.logical_owner_id != self\.logical_human_owner_id/);
  assert.match(runtime, /&self\.logical_human_owner_id/);
  assert.match(worker, /persist_transition_v1\(persistence, &persisted, executing\)\.await/);
  assert.match(worker, /mark_uncertain_v1\(persistence, persisted\)\.await/);
  assert.match(worker, /confirmed\.digest != plan\.model_digest/);
  assert.match(processRoot, /ManagedIntegrationRuntimeConfigurationV1/);
  assert.match(processRoot, /logical_human_owner_id: configuration\.logical_human_owner_id/);
  assert.match(processRoot, /serve-inherited/);
  assert.match(
    managedFlow,
    /fn managed_ollama_ai_runtime_completes_real_provider_generation\(\)/,
  );
  assert.match(managedFlow, /required\("MAKOSH_OLLAMA_LIVE_PORT"\)/);
  assert.match(managedFlow, /AiInferenceTerminalStatusReady/);
  assert.match(managedFlow, /"owner-2"/);
  assert.match(managedSetup, /"makosh-conformance:latest"/);
  assert.match(managedSetup, /Value::UnsignedIntegerValue\(30_000\)/);
  assert.doesNotMatch(managedFlow, /canned|OllamaAiHttpFixture/i);
  assert.doesNotMatch(
    `${admission}\n${runtime}\n${worker}\n${processRoot}`,
    /makosh_communications|communications_|automatic.*download/i,
  );
});

test('Ollama assembly emits only unsigned runtime contracts and owner storage input', async () => {
  const [manifest, assembly, cli, release] = await Promise.all([
    backendSource('src/ollama-ai-assembly/Cargo.toml'),
    backendSource('src/ollama-ai-assembly/src/lib.rs'),
    backendSource('src/ollama-ai-assembly/src/main.rs'),
    backendSource('scripts/materialize-dev-release.sh'),
  ]);

  assert.match(manifest, /surface = "assembly"/);
  assert.match(manifest, /makosh-ollama-ai-runtime/);
  assert.match(assembly, /ollama_ai_module_descriptor_v1/);
  assert.match(assembly, /ollama_ai_settings_schema_v1/);
  assert.match(assembly, /ollama_ai_storage_bundle_v1/);
  assert.match(assembly, /create_new\(true\)/);
  assert.match(assembly, /mode\(0o600\)/);
  assert.match(cli, /--runtime/);
  assert.match(release, /--package makosh-ollama-ai-runtime/);
  assert.match(release, /--package makosh-ollama-ai-assembly/);
  assert.match(release, /ollama-ai\.release-artifacts\.json/);
  assert.doesNotMatch(`${assembly}\n${cli}`, /signing|launch|\/api\/chat|communications_/i);
});
