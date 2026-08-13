import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const REPOSITORY_ROOT = new URL('../../../', import.meta.url);

test('communication translation agreement isolates workflow domain engine and provider', async () => {
  const [
    adr,
    inventorySource,
    policySource,
    workspace,
    apiManifest,
    api,
    protocol,
    core,
    persistenceManifest,
    persistenceSchema,
    persistenceRepository,
    runtimeManifest,
    runtimeAdmission,
    runtimeInference,
    runtimeSourceResults,
    managedRuntime,
    communicationsTranslationSource,
    communicationsAdmission,
    communicationsEventRuntime,
    communicationsSourceProtocol,
    aiProtocol,
    aiContracts,
    aiTranslationValidation,
    aiTranslationCore,
    aiTranslationSchema,
    aiTranslationRepository,
    aiTranslationWorker,
    aiRuntimeAdmission,
    aiManagedPorts,
    aiManagedRuntime,
    ollamaApi,
    ollamaCore,
    ollamaHttp,
    ollamaTranslationSchema,
    ollamaTranslationRepository,
    ollamaTranslationWorker,
    ollamaRuntimeAdmission,
    ollamaManagedRuntime,
    assemblyManifest,
    assembly,
    release,
    managedSetup,
    managedFlow,
    managedScript,
  ] = await Promise.all([
    readFile(
      new URL(
        'docs/adr/ADR-0363-communication-translation-workflow-and-ai-contracts.md',
        REPOSITORY_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('architecture/communications-settings-reconstruction.json', BACKEND_ROOT)),
    readFile(new URL('architecture/policy.json', BACKEND_ROOT), 'utf8'),
    readFile(new URL('Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-translation-api/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-translation-api/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL(
        'src/communication-translation-api/proto/makosh/communication_translation/v1/translation.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('src/communication-translation-core/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL('src/communication-translation-persistence/Cargo.toml', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-translation-persistence/migrations/0001_translation.sql',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('src/communication-translation-persistence/src/repository.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(new URL('src/communication-translation-runtime/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL('src/communication-translation-runtime/src/admission.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/communication-translation-runtime/src/inference.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/communication-translation-runtime/src/source_results.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/communication-translation-runtime/src/managed_runtime.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(new URL('src/communications-runtime/src/translation_source.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communications-runtime/src/admission.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communications-runtime/src/event_runtime.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL(
        'src/communications-ai-source-api/proto/makosh/communications/ai_source/v1/ai_source.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('src/ai-contracts/proto/makosh/ai/contracts/v1/ai.proto', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/ai-contracts/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/ai-contracts/src/translation.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/ai-inference-core/src/translation.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL(
        'src/ai-inference-persistence/migrations/0003_ai_translation_runs.sql',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('src/ai-inference-persistence/src/translation_repository.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/ai-inference-runtime/src/translation_worker.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(new URL('src/ai-inference-runtime/src/admission.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/ai-inference-runtime/src/managed_ports.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/ai-inference-runtime/src/managed_runtime.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/ollama-ai-api/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/ollama-ai-core/src/translation.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/ollama-ai-http/src/model.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL(
        'src/ollama-ai-persistence/migrations/0003_ollama_ai_translation_runs.sql',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('src/ollama-ai-persistence/src/translation_repository.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(new URL('src/ollama-ai-runtime/src/translation_worker.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/ollama-ai-runtime/src/admission.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/ollama-ai-runtime/src/managed_runtime.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-translation-assembly/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-translation-assembly/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('scripts/materialize-dev-release.sh', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/communication_translation_managed_setup.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/communication_translation_managed_flow.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('scripts/test-authenticated-storage.mjs', BACKEND_ROOT), 'utf8'),
  ]);
  const inventory = JSON.parse(inventorySource);
  const policy = JSON.parse(policySource);
  const slice = inventory.slices.find(({ gate }) => gate === 'communication_translation_v1');

  assert.deepEqual(slice, {
    gate: 'communication_translation_v1',
    role: 'workflow',
    owner: 'communication_translation',
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
    'makosh-communication-translation-api',
    'makosh-communication-translation-core',
    'makosh-communication-translation-persistence',
    'makosh-communication-translation-runtime',
    'makosh-communication-translation-assembly',
  ]) {
    assert.match(adr, new RegExp(`\\b${unit}\\b`));
  }
  assert.match(adr, /ai\.translation\.request\.v1/);
  assert.match(adr, /ai\.provider\.translate\.v1/);
  assert.match(adr, /перевод одного canonical communication evidence item/);
  assert.match(adr, /Attachment translation[\s\S]*отдельным/);
  assert.match(adr, /Thread translation[\s\S]*не\s+является неявным batch mode/);
  assert.match(adr, /Kernel\/Gateway не компилируют Translation\s+schema/);
  assert.match(adr, /Gate[\s\S]*`communication_translation_v1`[\s\S]*закрыт/);
  assert.doesNotMatch(adr, /generic `execute\(any\)` разрешён|Communications owns translation/i);

  assert.equal(
    policy.implementation.currentSlice,
    'speech_to_text_whisper_admission_v1',
  );
  assert.match(workspace, /"src\/communication-translation-api"/);
  assert.match(workspace, /"src\/communication-translation-core"/);
  assert.match(workspace, /"src\/communication-translation-runtime"/);
  assert.match(workspace, /"src\/communication-translation-assembly"/);
  assert.match(apiManifest, /owner = "communication_translation"/);
  assert.match(apiManifest, /surface = "contract"/);
  assert.match(api, /COMMUNICATION_TRANSLATION_CAPABILITY_ID_V1/);
  assert.match(protocol, /CommunicationTranslationCandidateV1/);
  assert.match(protocol, /COMMUNICATION_TRANSLATION_LANGUAGE_ENGLISH/);
  assert.match(protocol, /COMMUNICATION_TRANSLATION_LANGUAGE_RUSSIAN/);
  assert.match(protocol, /COMMUNICATION_TRANSLATION_LANGUAGE_SPANISH/);
  assert.doesNotMatch(protocol, /provider_id|model_id|endpoint|prompt|source_body|thread_id|attachment_id|map</);
  assert.match(core, /transition_communication_translation_v1/);
  assert.match(core, /DigestMismatch/);
  assert.doesNotMatch(core, /communication_summary|makosh_ai|ollama|communications_domain/);
  for (const capability of [
    'ai.provider.translate.v1',
    'ai.translation.request.v1',
    'communication.translation.v1',
    'communication_translation.source.blob.v1',
    'communication_translation.storage.v1',
    'communications.ai-translation-source.v1',
  ]) {
    assert.ok(policy.implementation.ownerInventory.businessCapabilities.includes(capability));
  }
  assert.match(communicationsSourceProtocol, /PrepareCommunicationTranslationSourceCommandV1/);
  assert.match(communicationsSourceProtocol, /CommunicationTranslationSourcePreparedV1/);
  assert.match(communicationsSourceProtocol, /CommunicationTranslationSourceRejectedV1/);
  assert.match(communicationsTranslationSource, /consume_next_translation_source_prepare_v1/);
  assert.match(communicationsTranslationSource, /write_target_bound_source/);
  assert.match(communicationsTranslationSource, /delivery\.acknowledge\(\)/);
  assert.doesNotMatch(
    communicationsTranslationSource,
    /makosh_communication_translation_runtime|makosh_ai_inference|makosh_ollama|provider_id|model_id|prompt/,
  );
  assert.match(communicationsAdmission, /communications_translation_source_capability_v1/);
  assert.match(communicationsAdmission, /communications_translation_source_blob_capability_v1/);
  assert.match(communicationsAdmission, /communication_translation_source_prepare_consume_request_v1/);
  assert.match(communicationsEventRuntime, /translation_source_prepare: RuntimeSubscribePermitV1/);
  assert.match(communicationsEventRuntime, /consume_next_translation_source_prepare_v1/);
  assert.match(aiProtocol, /CommunicationTranslationInferenceRequestV1/);
  assert.match(aiProtocol, /AiProviderTranslationRequestV1/);
  assert.match(aiContracts, /AI_TRANSLATION_REQUEST_CAPABILITY_ID_V1/);
  assert.match(aiContracts, /AI_PROVIDER_TRANSLATION_CAPABILITY_ID_V1/);
  assert.match(aiTranslationValidation, /seal_translation_inference_request_v1/);
  assert.doesNotMatch(aiTranslationValidation, /CommunicationSummary|CommunicationReply/);
  assert.match(aiTranslationCore, /build_translation_provider_input_v1/);
  assert.match(aiTranslationCore, /body_utf8/);
  assert.match(aiTranslationCore, /provider_result\.target_language != run\.request\.target_language/);
  assert.match(aiTranslationSchema, /makosh_data\.ai_translation_runs/);
  assert.match(aiTranslationSchema, /result_translated_text_utf8/);
  assert.match(aiTranslationSchema, /result_detected_source_language/);
  assert.match(aiTranslationRepository, /ON CONFLICT \(logical_owner_id, run_id\) DO NOTHING/);
  assert.match(aiTranslationRepository, /load_recoverable_translation_runs/);
  assert.match(aiTranslationWorker, /execute_translation_payload_v1/);
  assert.match(aiTranslationWorker, /materialize_translation_source/);
  assert.match(aiTranslationWorker, /ports\.translate/);
  assert.doesNotMatch(
    `${aiTranslationCore}\n${aiTranslationSchema}\n${aiTranslationRepository}\n${aiTranslationWorker}`,
    /CommunicationSummary|CommunicationReply|communication_summary|communication_reply|ollama|provider_id|model_id|endpoint/,
  );
  assert.match(aiRuntimeAdmission, /AI_TRANSLATION_REQUEST_CAPABILITY_ID_V1/);
  assert.match(aiRuntimeAdmission, /AI_PROVIDER_TRANSLATION_CAPABILITY_ID_V1/);
  assert.match(aiManagedPorts, /ai_provider_translation_contract_reference_v1/);
  assert.match(aiManagedRuntime, /communication_translation_inference_contract_reference_v1/);
  assert.match(ollamaApi, /OLLAMA_AI_TRANSLATION_CAPABILITY_ID_V1/);
  assert.match(ollamaCore, /OLLAMA_TRANSLATION_POLICY_V1/);
  assert.match(ollamaCore, /OllamaTranslationPlanV1/);
  assert.match(ollamaCore, /target_language: request\.target_language/);
  assert.match(ollamaHttp, /translation_json_schema_v1/);
  assert.match(ollamaHttp, /detected_source_language/);
  assert.match(ollamaTranslationSchema, /makosh_data\.ollama_ai_translation_runs/);
  assert.match(ollamaTranslationSchema, /result_translated_text_utf8/);
  assert.match(ollamaTranslationRepository, /ON CONFLICT \(logical_owner_id, request_id\) DO NOTHING/);
  assert.match(ollamaTranslationWorker, /execute_translation_payload_v1/);
  assert.match(ollamaTranslationWorker, /generate_translation/);
  assert.match(ollamaRuntimeAdmission, /ai_provider_translation_contract_reference_v1/);
  assert.match(ollamaManagedRuntime, /execute_translation_payload_v1/);
  assert.doesNotMatch(
    `${ollamaCore}\n${ollamaHttp}\n${ollamaTranslationSchema}\n${ollamaTranslationRepository}\n${ollamaTranslationWorker}\n${ollamaRuntimeAdmission}\n${ollamaManagedRuntime}`,
    /makosh_communications|makosh_ai_inference|communication_translation_runtime|communications_domain|provider_id|endpoint_url/,
  );
  assert.doesNotMatch(aiProtocol, /provider_id|model_id|map</);
  assert.match(persistenceManifest, /owner = "communication_translation"/);
  assert.match(persistenceManifest, /surface = "persistence"/);
  assert.match(persistenceSchema, /communication_translation_runs/);
  assert.match(persistenceSchema, /request_fingerprint/);
  assert.match(persistenceSchema, /communication_translation_inbox/);
  assert.match(persistenceSchema, /communication_translation_outbox/);
  assert.match(persistenceSchema, /communication_translation_realtime/);
  assert.match(persistenceRepository, /ON CONFLICT \(logical_owner_id, operation_id\)/);
  assert.doesNotMatch(
    `${persistenceSchema}\n${persistenceRepository}`,
    /communication_summary|communications_|mail_|telegram_|whatsapp_|zulip_|source_body|provider_id|model_id|endpoint/,
  );
  assert.match(runtimeManifest, /owner = "communication_translation"/);
  assert.match(runtimeManifest, /surface = "runtime"/);
  assert.match(runtimeAdmission, /ModuleKindV1::Workflow/);
  assert.match(runtimeAdmission, /communication_translation\.source_prepare\.v1/);
  assert.match(runtimeAdmission, /communication_translation\.source_prepared\.v1/);
  assert.match(runtimeAdmission, /communication_translation\.source_rejected\.v1/);
  assert.match(runtimeInference, /RouteModuleRequest/);
  assert.match(runtimeInference, /translated_text_utf8/);
  assert.match(runtimeInference, /detected_source_language/);
  assert.match(runtimeSourceResults, /seal_translation_inference_request_v1/);
  assert.match(runtimeSourceResults, /AiUseCaseCommunicationTranslation/);
  assert.match(managedRuntime, /ManagedStorageRuntimeConfigurationV1/);
  assert.doesNotMatch(
    `${runtimeAdmission}\n${runtimeInference}\n${runtimeSourceResults}\n${managedRuntime}`,
    /communication_summary|CommunicationSummary|mail_|telegram_|whatsapp_|zulip_|provider_id|model_id|prompt/,
  );
  assert.match(assemblyManifest, /owner = "communication_translation"/);
  assert.match(assemblyManifest, /surface = "assembly"/);
  assert.match(assembly, /communication_translation_module_descriptor_v1/);
  assert.match(assembly, /communication_translation_storage_bundle_v1/);
  assert.match(assembly, /communication_translation\.runtime\.v1/);
  assert.match(assembly, /communication_translation\.storage\.v1/);
  assert.match(release, /--package makosh-communication-translation-runtime/);
  assert.match(release, /--package makosh-communication-translation-assembly/);
  assert.match(release, /communication_translation\.release-artifacts\.json/);
  assert.doesNotMatch(assembly, /communication_summary|ollama|ai_inference|communications_domain/);
  assert.match(managedSetup, /InstalledSignedBundle::install/);
  assert.match(managedSetup, /start_reserved_workflow/);
  assert.match(managedFlow, /managed_communication_translation_reaches_ai_and_replays_through_gateway_sse/);
  assert.match(managedFlow, /COMMUNICATION_TRANSLATION_COMMAND_CONNECT_PATH_V1/);
  assert.match(managedFlow, /COMMUNICATION_TRANSLATION_QUERY_CONNECT_PATH_V1/);
  assert.match(managedFlow, /read_terminal_translation_sse_event/);
  assert.match(managedFlow, /restart_communication_translation_runtime_v1/);
  assert.match(managedFlow, /assert_communication_translation_runtime_fences/);
  assert.match(managedFlow, /stale Communication Translation runtime generation/);
  assert.match(managedFlow, /stale Communication Translation grant epoch/);
  assert.match(managedFlow, /CommunicationTranslationErrorCodeSourceRejected/);
  assert.match(managedFlow, /managed_communication_translation_completes_real_provider_through_gateway_sse/);
  assert.match(managedFlow, /MAKOSH_OLLAMA_LIVE_PORT/);
  assert.match(managedScript, /MAKOSH_COMMUNICATION_TRANSLATION_RUNTIME_BIN/);
  assert.match(managedScript, /managed_communication_translation_reaches_ai_and_replays_through_gateway_sse/);
});
