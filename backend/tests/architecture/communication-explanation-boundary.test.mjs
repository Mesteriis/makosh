import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const REPOSITORY_ROOT = new URL('../../../', import.meta.url);

test('communication explanation agreement separates workflow domain engine and provider', async () => {
  const [
    adr,
    inventorySource,
    policySource,
    workspace,
    apiManifest,
    api,
    protocol,
    coreManifest,
    core,
    communicationsSourceProtocol,
    communicationsSourceApi,
    aiProtocol,
    aiContracts,
    aiExplanationValidation,
    aiExplanationCore,
    aiExplanationSchema,
    aiExplanationRepository,
    aiRuntimeAdmission,
    aiRuntimePorts,
    aiExplanationWorker,
    aiManagedRuntime,
    ollamaApi,
    ollamaExplanationCore,
    ollamaExplanationSchema,
    ollamaExplanationRepository,
    ollamaHttpModel,
    ollamaRuntimeAdmission,
    ollamaExplanationWorker,
    ollamaManagedRuntime,
    persistenceManifest,
    persistenceSchema,
    persistenceModel,
    persistenceRepository,
    runtimeManifest,
    runtimeAdmission,
    runtimeClientPort,
    runtimeInference,
    runtimeSourceResults,
    managedRuntime,
    assemblyManifest,
    assembly,
    release,
    communicationsExplanationSource,
    communicationsAdmission,
    communicationsEventRuntime,
    managedSetup,
    managedFlow,
    managedScript,
  ] = await Promise.all([
    readFile(
      new URL(
        'docs/adr/ADR-0364-communication-explanation-workflow-and-ai-contracts.md',
        REPOSITORY_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('architecture/communications-settings-reconstruction.json', BACKEND_ROOT)),
    readFile(new URL('architecture/policy.json', BACKEND_ROOT), 'utf8'),
    readFile(new URL('Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-explanation-api/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-explanation-api/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL(
        'src/communication-explanation-api/proto/makosh/communication_explanation/v1/explanation.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('src/communication-explanation-core/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-explanation-core/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL(
        'src/communications-ai-source-api/proto/makosh/communications/ai_source/v1/ai_source.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('src/communications-ai-source-api/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/ai-contracts/proto/makosh/ai/contracts/v1/ai.proto', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/ai-contracts/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/ai-contracts/src/explanation.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/ai-inference-core/src/explanation.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/ai-inference-persistence/migrations/0004_ai_explanation_runs.sql', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/ai-inference-persistence/src/explanation_repository.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/ai-inference-runtime/src/admission.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/ai-inference-runtime/src/managed_ports.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/ai-inference-runtime/src/explanation_worker.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/ai-inference-runtime/src/managed_runtime.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/ollama-ai-api/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/ollama-ai-core/src/explanation.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL(
        'src/ollama-ai-persistence/migrations/0004_ollama_ai_explanation_runs.sql',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('src/ollama-ai-persistence/src/explanation_repository.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(new URL('src/ollama-ai-http/src/model.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/ollama-ai-runtime/src/admission.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/ollama-ai-runtime/src/explanation_worker.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/ollama-ai-runtime/src/managed_runtime.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL('src/communication-explanation-persistence/Cargo.toml', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-explanation-persistence/migrations/0001_explanation.sql',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('src/communication-explanation-persistence/src/model.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/communication-explanation-persistence/src/repository.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(new URL('src/communication-explanation-runtime/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-explanation-runtime/src/admission.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-explanation-runtime/src/client_port.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-explanation-runtime/src/inference.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-explanation-runtime/src/source_results.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-explanation-runtime/src/managed_runtime.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-explanation-assembly/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-explanation-assembly/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('scripts/materialize-dev-release.sh', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communications-runtime/src/explanation_source.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communications-runtime/src/admission.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communications-runtime/src/event_runtime.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/communication_explanation_managed_setup.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/communication_explanation_managed_flow.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('scripts/test-authenticated-storage.mjs', BACKEND_ROOT), 'utf8'),
  ]);
  const inventory = JSON.parse(inventorySource);
  const policy = JSON.parse(policySource);
  const slice = inventory.slices.find(({ gate }) => gate === 'communication_explanation_v1');

  assert.deepEqual(slice, {
    gate: 'communication_explanation_v1',
    role: 'workflow',
    owner: 'communication_explanation',
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
    'makosh-communication-explanation-api',
    'makosh-communication-explanation-core',
    'makosh-communication-explanation-persistence',
    'makosh-communication-explanation-runtime',
    'makosh-communication-explanation-assembly',
  ]) {
    assert.match(adr, new RegExp(`\\b${unit}\\b`));
  }
  assert.match(adr, /почему один canonical[\s\S]*требовать внимания/);
  assert.match(adr, /ai\.explanation\.request\.v1/);
  assert.match(adr, /ai\.provider\.explain\.v1/);
  assert.match(adr, /Smart CC остаётся отдельным/);
  assert.match(adr, /exact reason kind\/source-basis enums/);
  assert.match(adr, /Kernel\/Gateway не компилируют[\s\S]*Explanation schema/);
  assert.match(adr, /Состояние реализации: implemented/);
  assert.doesNotMatch(adr, /generic `execute\(any\)` разрешён|Communications owns explanation/i);

  assert.equal(policy.implementation.currentSlice, 'call_transcription_managed_conformance_v1');
  assert.match(workspace, /"src\/communication-explanation-api"/);
  assert.match(workspace, /"src\/communication-explanation-core"/);
  assert.match(workspace, /"src\/communication-explanation-persistence"/);
  assert.match(workspace, /"src\/communication-explanation-runtime"/);
  assert.match(workspace, /"src\/communication-explanation-assembly"/);
  assert.match(apiManifest, /owner = "communication_explanation"/);
  assert.match(apiManifest, /surface = "contract"/);
  assert.match(coreManifest, /owner = "communication_explanation"/);
  assert.match(coreManifest, /surface = "implementation"/);
  assert.match(persistenceManifest, /owner = "communication_explanation"/);
  assert.match(persistenceManifest, /surface = "persistence"/);
  assert.match(api, /COMMUNICATION_EXPLANATION_CAPABILITY_ID_V1/);
  assert.match(protocol, /CommunicationExplanationCandidateV1/);
  assert.match(protocol, /COMMUNICATION_EXPLANATION_REASON_KIND_DEADLINE/);
  assert.match(protocol, /COMMUNICATION_EXPLANATION_SOURCE_BASIS_COMBINED/);
  assert.doesNotMatch(
    protocol,
    /provider_id|model_id|endpoint|prompt|source_body|recipient|task|note|map</,
  );
  assert.match(core, /transition_communication_explanation_v1/);
  assert.match(core, /DuplicateReasonKind/);
  assert.match(core, /allows_empty_reason_list_without_fabricating_a_reason/);
  assert.doesNotMatch(
    core,
    /communication_summary|communication_translation|makosh_ai|ollama|communications_domain/,
  );
  assert.ok(
    policy.implementation.ownerInventory.businessCapabilities.includes(
      'communication.explanation.v1',
    ),
  );
  assert.match(communicationsSourceProtocol, /PrepareCommunicationExplanationSourceCommandV1/);
  assert.match(communicationsSourceProtocol, /CommunicationExplanationSourcePreparedV1/);
  assert.match(communicationsSourceProtocol, /CommunicationExplanationSourceRejectedV1/);
  assert.match(communicationsSourceApi, /communications\.ai-explanation-source\.v1/);
  assert.match(communicationsSourceApi, /communication_explanation\.source\.blob\.v1/);
  assert.doesNotMatch(
    communicationsSourceProtocol,
    /provider_id|model_id|endpoint|prompt|recipient|task|note|map</,
  );
  assert.match(aiProtocol, /CommunicationExplanationInferenceRequestV1/);
  assert.match(aiProtocol, /AiProviderExplanationRequestV1/);
  assert.match(aiProtocol, /AI_EXPLANATION_REASON_KIND_LEGAL_OR_CONTRACTUAL/);
  assert.match(aiProtocol, /AI_EXPLANATION_SOURCE_BASIS_CANONICAL_METADATA/);
  assert.match(aiContracts, /AI_EXPLANATION_REQUEST_CAPABILITY_ID_V1/);
  assert.match(aiContracts, /AI_PROVIDER_EXPLANATION_CAPABILITY_ID_V1/);
  assert.match(aiExplanationValidation, /seal_explanation_inference_request_v1/);
  assert.match(aiExplanationValidation, /provider_result_rejects_duplicate_reason_kinds/);
  assert.doesNotMatch(
    aiExplanationValidation,
    /CommunicationSummary|CommunicationTranslation|\b(?:provider_id|model_id|endpoint|prompt)\b/,
  );
  assert.match(ollamaApi, /OLLAMA_AI_EXPLANATION_CAPABILITY_ID_V1/);
  assert.match(aiExplanationCore, /build_explanation_provider_input_v1/);
  assert.match(aiExplanationCore, /fixed-taxonomy/);
  assert.match(aiExplanationCore, /provider_result\.reasons/);
  assert.match(aiExplanationSchema, /ai_explanation_runs/);
  assert.match(aiExplanationSchema, /result_exact_bytes/);
  assert.doesNotMatch(aiExplanationSchema, /source_body|prompt_text|provider_id|model_id|endpoint/);
  assert.match(aiExplanationRepository, /accept_explanation_run/);
  assert.match(aiExplanationRepository, /decode_explanation_inference_result_v1/);
  assert.match(aiRuntimeAdmission, /AI_EXPLANATION_REQUEST_CAPABILITY_ID_V1/);
  assert.match(aiRuntimeAdmission, /AI_PROVIDER_EXPLANATION_CAPABILITY_ID_V1/);
  assert.match(aiRuntimePorts, /ai_provider_explanation_contract_reference_v1/);
  assert.match(aiExplanationWorker, /execute_explanation_payload_v1/);
  assert.match(aiExplanationWorker, /ports\.explain/);
  assert.match(aiManagedRuntime, /communication_explanation_inference_contract_reference_v1/);
  assert.doesNotMatch(
    `${aiExplanationCore}\n${aiExplanationRepository}\n${aiExplanationWorker}`,
    /communication_summary|CommunicationSummary|communication_translation|CommunicationTranslation|ollama|provider_id|model_id|endpoint/,
  );
  assert.match(ollamaExplanationCore, /OLLAMA_EXPLANATION_POLICY_V1/);
  assert.match(ollamaExplanationCore, /complete_ollama_explanation_request_v1/);
  assert.match(ollamaExplanationCore, /allows_empty_reason_list/);
  assert.match(ollamaExplanationSchema, /ollama_ai_explanation_runs/);
  assert.match(ollamaExplanationSchema, /result_exact_bytes/);
  assert.match(ollamaExplanationRepository, /encode_provider_explanation_result_v1/);
  assert.match(ollamaExplanationRepository, /decode_provider_explanation_result_v1/);
  assert.match(ollamaHttpModel, /ExplanationJsonSchemaV1/);
  assert.match(ollamaHttpModel, /additional_properties: false/);
  assert.match(ollamaHttpModel, /maximum_reason_text_bytes/);
  assert.match(ollamaRuntimeAdmission, /OLLAMA_AI_EXPLANATION_CAPABILITY_ID_V1/);
  assert.match(ollamaRuntimeAdmission, /ai_provider_explanation_contract_reference_v1/);
  assert.match(ollamaExplanationWorker, /execute_explanation_payload_v1/);
  assert.match(ollamaExplanationWorker, /port\.generate_explanation/);
  assert.match(ollamaManagedRuntime, /is_explanation/);
  assert.doesNotMatch(
    `${ollamaExplanationCore}\n${ollamaExplanationSchema}\n${ollamaExplanationRepository}\n${ollamaExplanationWorker}`,
    /makosh_communications_domain|communication_summary|CommunicationSummary|communication_translation|CommunicationTranslation|result_translated_text|source_body|prompt_text/,
  );
  for (const capability of [
    'ai.explanation.request.v1',
    'ai.provider.explain.v1',
    'communication_explanation.inference.v1',
    'communication_explanation.source.blob.v1',
    'communication_explanation.source_prepare.v1',
    'communication_explanation.source_prepared.v1',
    'communication_explanation.source_rejected.v1',
    'communication_explanation.storage.v1',
    'communications.ai-explanation-source.blob.v1',
    'communications.ai-explanation-source.v1',
  ]) {
    assert.ok(policy.implementation.ownerInventory.businessCapabilities.includes(capability));
  }
  assert.match(persistenceSchema, /communication_explanation_runs/);
  assert.match(persistenceSchema, /UNIQUE \(logical_owner_id, operation_id\)/);
  assert.match(persistenceSchema, /candidate_reasons_bytes/);
  assert.match(persistenceSchema, /communication_explanation_inbox/);
  assert.match(persistenceSchema, /communication_explanation_outbox/);
  assert.match(persistenceSchema, /communication_explanation_realtime/);
  assert.doesNotMatch(
    persistenceSchema,
    /communications_|mail_|telegram_|whatsapp_|zulip_|source_body|prompt|provider_id|model_id|endpoint/,
  );
  assert.match(persistenceModel, /encode_reasons/);
  assert.match(persistenceModel, /decode_reasons/);
  assert.match(persistenceRepository, /load_recoverable_runs/);
  assert.match(persistenceRepository, /InboxConflict/);
  assert.match(persistenceRepository, /request_fingerprint/);
  assert.doesNotMatch(
    `${persistenceManifest}\n${persistenceModel}\n${persistenceRepository}`,
    /makosh-(?:communications-domain|ai-inference|ollama|mail|telegram|whatsapp|zulip)/,
  );
  assert.match(runtimeManifest, /owner = "communication_explanation"/);
  assert.match(runtimeManifest, /surface = "runtime"/);
  assert.match(runtimeAdmission, /ModuleKindV1::Workflow/);
  assert.match(runtimeAdmission, /communication_explanation\.source_prepare\.v1/);
  assert.match(runtimeAdmission, /communication_explanation\.source_prepared\.v1/);
  assert.match(runtimeAdmission, /communication_explanation\.source_rejected\.v1/);
  assert.match(runtimeClientPort, /StartCommunicationExplanationRequestV1/);
  assert.match(runtimeClientPort, /CommunicationExplanationReasonV1/);
  assert.match(runtimeInference, /RouteModuleRequest/);
  assert.match(runtimeInference, /AiExplanationReasonKindV1/);
  assert.match(runtimeSourceResults, /seal_explanation_inference_request_v1/);
  assert.match(runtimeSourceResults, /AiUseCaseCommunicationExplanation/);
  assert.match(managedRuntime, /ManagedStorageRuntimeConfigurationV1/);
  assert.match(managedRuntime, /publish_pending/);
  assert.doesNotMatch(
    `${runtimeAdmission}\n${runtimeClientPort}\n${runtimeInference}\n${runtimeSourceResults}\n${managedRuntime}`,
    /communication_summary|CommunicationSummary|communication_translation|CommunicationTranslation|mail_|telegram_|whatsapp_|zulip_|provider_id|model_id|prompt|target_language|translated_text/,
  );
  assert.match(assemblyManifest, /owner = "communication_explanation"/);
  assert.match(assemblyManifest, /surface = "assembly"/);
  assert.match(assembly, /communication_explanation_module_descriptor_v1/);
  assert.match(assembly, /communication_explanation_storage_bundle_v1/);
  assert.match(assembly, /communication_explanation\.runtime\.v1/);
  assert.match(assembly, /communication_explanation\.storage\.v1/);
  assert.match(release, /--package makosh-communication-explanation-runtime/);
  assert.match(release, /--package makosh-communication-explanation-assembly/);
  assert.match(release, /communication_explanation\.release-artifacts\.json/);
  assert.doesNotMatch(
    assembly,
    /communication_summary|communication_translation|ollama|ai_inference|communications_domain/,
  );
  assert.match(communicationsExplanationSource, /consume_next_explanation_source_prepare_v1/);
  assert.match(communicationsExplanationSource, /ManagedBlobCustodyTargetV1/);
  assert.match(communicationsExplanationSource, /communication_explanation_source_prepared/);
  assert.doesNotMatch(
    communicationsExplanationSource,
    /makosh_communication_explanation_runtime|makosh_ai_inference|makosh_ollama|provider_id|model_id|prompt/,
  );
  assert.match(communicationsAdmission, /communications_explanation_source_capability_v1/);
  assert.match(
    communicationsAdmission,
    /communication_explanation_source_prepare_consume_request_v1/,
  );
  assert.match(communicationsEventRuntime, /explanation_source_prepare: RuntimeSubscribePermitV1/);
  assert.match(
    communicationsEventRuntime,
    /consume_next_explanation_source_prepare_v1/,
  );
  assert.match(managedSetup, /COMMUNICATION_EXPLANATION_RELEASE_ARTIFACT_ID_V1/);
  assert.match(managedSetup, /start_reserved_workflow/);
  assert.match(
    managedFlow,
    /managed_communication_explanation_reaches_ai_and_replays_through_gateway_sse/,
  );
  assert.match(
    managedFlow,
    /managed_communication_explanation_completes_real_provider_through_gateway_sse/,
  );
  assert.match(managedFlow, /required\("MAKOSH_OLLAMA_LIVE_PORT"\)/);
  assert.match(managedFlow, /read_terminal_explanation_sse_event/);
  assert.match(managedFlow, /restart_communication_explanation_runtime_v1/);
  assert.match(managedFlow, /assert_communication_explanation_runtime_fences/);
  assert.match(managedFlow, /wrong_owner/);
  assert.match(managedFlow, /stale_request/);
  assert.match(managedFlow, /revoke_owner/);
  assert.match(managedFlow, /assert_private_content_absent/);
  assert.match(
    managedScript,
    /managed_communication_explanation_reaches_ai_and_replays_through_gateway_sse/,
  );
});
