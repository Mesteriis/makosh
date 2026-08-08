import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const REPOSITORY_ROOT = new URL('../', BACKEND_ROOT);

test('attachment translation agreement keeps workflow source engine and provider separate', async () => {
  const [inventorySource, policySource, workspace, apiManifest, api, apiProto, readProto, ingressManifest, ingress, coreManifest, core, persistenceManifest, persistence, schema, ticketSchema, tickets, aiProto, aiContract, aiCore, aiSchema, aiRepository, aiWorker, aiAdmission, runtimeManifest, runtimeAdmission, runtimeSource, runtimeClient, assemblyManifest, assembly, adr] = await Promise.all([
    readFile(
      new URL('architecture/communications-settings-reconstruction.json', BACKEND_ROOT),
      'utf8',
    ),
    readFile(new URL('architecture/policy.json', BACKEND_ROOT), 'utf8'),
    readFile(new URL('Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-translation-api/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-translation-api/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL(
        'src/attachment-translation-api/proto/makosh/attachment_translation/v1/translation.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/attachment-translation-api/proto/makosh/attachment_translation/read/v1/read.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('src/attachment-translation-ingress/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-translation-ingress/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-translation-core/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-translation-core/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-translation-persistence/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-translation-persistence/src/repository.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-translation-persistence/migrations/0001_translation.sql', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-translation-persistence/migrations/0002_translation_read_tickets.sql', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-translation-persistence/src/tickets.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/ai-contracts/proto/makosh/ai/contracts/v1/ai.proto', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/ai-contracts/src/attachment_translation.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/ai-inference-core/src/attachment_translation.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/ai-inference-persistence/migrations/0005_ai_attachment_translation_runs.sql', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/ai-inference-persistence/src/attachment_translation_repository.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/ai-inference-runtime/src/attachment_translation_worker.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/ai-inference-runtime/src/admission.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-translation-runtime/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-translation-runtime/src/admission.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-translation-runtime/src/source_results.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-translation-runtime/src/client_port.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-translation-assembly/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-translation-assembly/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL('docs/adr/ADR-0378-bounded-attachment-translation-workflow.md', REPOSITORY_ROOT),
      'utf8',
    ),
  ]);
  const inventory = JSON.parse(inventorySource);
  const policy = JSON.parse(policySource);
  const slice = inventory.slices.find(({ gate }) => gate === 'attachment_translation_v1');

  assert.deepEqual(slice, {
    gate: 'attachment_translation_v1',
    role: 'workflow',
    owner: 'attachment_translation',
    state: 'implemented',
    dependsOn: [
      'attachment_text_extraction_v1',
      'ai_inference_v1',
      'ollama_ai_provider_v1',
      'capability_routed_module_request_rpc_v1',
      'blob_v1',
    ],
  });
  for (const unit of [
    'makosh-attachment-translation-api',
    'makosh-attachment-translation-ingress',
    'makosh-attachment-translation-core',
    'makosh-attachment-translation-persistence',
    'makosh-attachment-translation-persistence',
    'makosh-attachment-translation-runtime',
    'makosh-attachment-translation-assembly',
  ]) {
    assert.match(adr, new RegExp(`\\b${unit}\\b`));
  }
  assert.match(adr, /`attachment_translation` является workflow, не domain и не integration/);
  assert.match(adr, /Workflow не вызывает Attachment Text Extraction RPC/);
  assert.match(adr, /ai\.attachment-translation\.request\.v1/);
  assert.match(adr, /distinct capability/);
  assert.match(adr, /Source text и translated[\s\S]*не попадают в SQL workflow owner/);
  assert.match(adr, /`attachment_translation_v1` переведён в `implemented`/);
  assert.equal(policy.implementation.currentSlice, 'call_transcription_managed_conformance_v1');
  assert(policy.implementation.ownerInventory.workflows.includes('attachment_translation'));
  for (const packageName of [
    'makosh-attachment-translation-api',
    'makosh-attachment-translation-ingress',
    'makosh-attachment-translation-core',
  ]) {
    assert(policy.implementation.productionPackages.some(({ name }) => name === packageName));
  }
  assert.match(workspace, /"src\/attachment-translation-api"/);
  assert.match(workspace, /"src\/attachment-translation-ingress"/);
  assert.match(workspace, /"src\/attachment-translation-core"/);
  assert.match(workspace, /"src\/attachment-translation-persistence"/);
  assert.match(workspace, /"src\/attachment-translation-runtime"/);
  assert.match(workspace, /"src\/attachment-translation-assembly"/);
  assert.match(apiManifest, /owner = "attachment_translation"/);
  assert.match(api, /ATTACHMENT_TRANSLATION_TICKET_CONNECT_PATH_V1/);
  assert.doesNotMatch(apiProto, /translated_text_utf8|provider_id|model_id|prompt/);
  assert.match(readProto, /opaque_read_ticket/);
  assert.doesNotMatch(readProto, /reference_id|sha256|translated_text/);
  assert.match(ingressManifest, /surface = "contract"/);
  assert.match(ingress, /attachment_translation_source_requested_contract_reference_v1/);
  assert.match(ingress, /ATTACHMENT_TEXT_EXTRACTION_TRANSLATION_SOURCE_CAPABILITY_ID_V1/);
  assert.match(coreManifest, /surface = "implementation"/);
  assert.match(core, /AttachmentTranslationStateV1/);
  assert.match(core, /MaterializingResult/);
  assert.doesNotMatch(core, /translated_text_utf8|communications|ollama|ai_inference/);
  assert.match(persistenceManifest, /surface = "persistence"/);
  assert.match(persistence, /persist_source_result/);
  assert.match(persistence, /persist_inference_result/);
  assert.match(persistence, /persist_materialization_result/);
  assert.match(schema, /source_extraction_run_id/);
  assert.match(schema, /pending_translated_sha256/);
  assert.match(schema, /artifact_translated_sha256/);
  assert.doesNotMatch(schema, /translated_text|source_text|provider_id|model_id|prompt/);
  assert.match(ticketSchema, /attachment_translation_read_tickets/);
  assert.match(ticketSchema, /artifact_runtime_generation/);
  assert.match(tickets, /device_actor_sha256/);
  assert.match(tickets, /TicketUsed/);
  assert.match(aiProto, /AI_USE_CASE_ATTACHMENT_TRANSLATION = 5/);
  assert.match(aiProto, /message AttachmentTranslationInferenceRequestV1/);
  assert.match(aiProto, /message AttachmentTranslationInferenceResultV1/);
  assert.doesNotMatch(
    aiProto.match(/message AttachmentTranslationInferenceRequestV1 \{[\s\S]*?\n\}/)?.[0] ?? '',
    /provider_id|model_id|endpoint|prompt/,
  );
  assert.match(aiContract, /seal_attachment_translation_inference_request_v1/);
  assert.match(aiContract, /AiUseCaseAttachmentTranslation/);
  assert.match(aiCore, /build_attachment_translation_provider_input_v1/);
  assert.match(aiCore, /validate_attachment_translation_source_text_v1/);
  assert.match(aiSchema, /makosh_data\.ai_attachment_translation_runs/);
  assert.match(aiRepository, /load_recoverable_attachment_translation_runs/);
  assert.match(aiWorker, /execute_attachment_translation_payload_v1/);
  assert.match(aiWorker, /materialize_attachment_translation_source/);
  assert.match(aiAdmission, /AI_ATTACHMENT_TRANSLATION_REQUEST_CAPABILITY_ID_V1/);
  assert.match(runtimeManifest, /owner = "attachment_translation"/);
  assert.doesNotMatch(
    runtimeManifest,
    /attachment-text-extraction-runtime|ai-inference-runtime|ollama-ai-runtime/,
  );
  assert.match(runtimeAdmission, /ProvidedSurfaceKindV1::ClientBlob/);
  assert.match(runtimeAdmission, /attachment_translation_source_requested_publish_request_v1/);
  assert.match(runtimeSource, /consume_translation_source_prepared_once_v1/);
  assert.match(runtimeSource, /seal_attachment_translation_inference_request_v1/);
  assert.doesNotMatch(runtimeSource, /communications|provider_id|model_id|prompt/);
  assert.match(runtimeClient, /ModuleClientBlobAuthorizationV1/);
  assert.match(runtimeClient, /dispatch_attachment_translation_client_request_v1/);
  assert.match(assemblyManifest, /surface = "assembly"/);
  assert.match(assembly, /attachment_translation_storage_bundle_v1/);
  assert.match(assembly, /attachment_translation_module_descriptor_v1/);
  assert(policy.implementation.ownerInventory.businessCapabilities.includes(
    'ai.attachment-translation.request.v1',
  ));
  assert.doesNotMatch(
    adr,
    /Communications owns attachment translation|legacy REST facade открывает gate|caller выбирает provider/,
  );
});

test('text extraction produces translation source only through durable target-owned events', async () => {
  const [runtimeManifest, admission, runtime, source, persistence, migration, release, developmentAssembly, policySource] = await Promise.all([
    readFile(
      new URL('src/attachment-text-extraction-runtime/Cargo.toml', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-text-extraction-runtime/src/admission.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-text-extraction-runtime/src/runtime.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-text-extraction-runtime/src/translation_source.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-text-extraction-persistence/src/translation_source.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL(
        'src/attachment-text-extraction-persistence/migrations/0002_translation_source.sql',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('scripts/materialize-dev-release.sh', BACKEND_ROOT), 'utf8'),
    readFile(new URL('development/assembly/src/main.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('architecture/policy.json', BACKEND_ROOT), 'utf8'),
  ]);
  const policy = JSON.parse(policySource);

  assert.match(runtimeManifest, /makosh-attachment-translation-ingress/);
  assert.doesNotMatch(
    runtimeManifest,
    /makosh-attachment-translation-(?:core|persistence|runtime|assembly)/,
  );
  assert.match(admission, /ATTACHMENT_TEXT_EXTRACTION_TRANSLATION_SOURCE_CAPABILITY_ID_V1/);
  assert.match(admission, /attachment_translation_source_requested_consume_request_v1/);
  assert.match(admission, /attachment_translation_source_prepared_publish_request_v1/);
  assert.match(admission, /attachment_translation_source_rejected_publish_request_v1/);
  assert.match(runtime, /ConsumerV1::TranslationSource/);
  assert.match(runtime, /delivery\s*\.acknowledge\(\)/);
  assert.match(source, /translation_source_request_already_processed/);
  assert.match(source, /read_artifact_v1/);
  assert.match(source, /write_translation_source_v1/);
  assert.match(source, /persist_translation_source_result/);
  assert.doesNotMatch(source, /query_rpc|client_rpc|postgres|sqlx/);
  assert.match(persistence, /request_envelope_sha256/);
  assert.match(persistence, /exact_result_envelope_bytes/);
  assert.match(migration, /translation_source_inbox/);
  assert.match(migration, /translation_source_outbox/);
  assert.doesNotMatch(migration, /source_text|translated_text|provider_id|model_id|prompt/);
  assert.match(release, /makosh-attachment-translation-assembly/);
  assert.match(release, /attachment_translation\.release-artifacts\.json/);
  assert.match(developmentAssembly, /ATTACHMENT_TRANSLATION_RUNTIME_ARTIFACT/);
  assert.match(developmentAssembly, /PRE_ATTACHMENT_TRANSLATION_MODULE_PLAN_RUNTIME_ARTIFACTS_V3/);
  assert.equal(policy.implementation.currentSlice, 'call_transcription_managed_conformance_v1');
  assert(policy.implementation.ownerInventory.businessCapabilities.includes(
    'attachment_text_extraction.translation-source.v1',
  ));
});

test('attachment translation has an exact signed managed lifecycle and restart gate', async () => {
  const [setup, flow, gateway, harness, runner, recovery, testkitManifest] = await Promise.all([
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/attachment_translation_managed_setup.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/attachment_translation_managed_flow.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/attachment_translation_gateway_fixture.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('tests/support/kernel-recovery/src/tests/managed_storage_vault_docker.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(new URL('scripts/test-authenticated-storage.mjs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-translation-runtime/src/recovery.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('tests/support/kernel-recovery/Cargo.toml', BACKEND_ROOT), 'utf8'),
  ]);

  assert.match(setup, /installed_attachment_translation_ensemble_release_v1/);
  assert.match(setup, /attachment_text_extraction_release_artifact_v1/);
  assert.match(setup, /ollama_ai_release_artifact_v1/);
  assert.match(setup, /ai_inference_release_artifact_v1/);
  assert.match(setup, /attachment_translation_release_artifact_v1/);
  assert.match(setup, /restart_attachment_translation_runtime_v1/);
  assert.match(setup, /storage_successor::reserve/);
  assert.match(flow, /managed_attachment_translation_reaches_source_ai_and_gateway_sse/);
  assert.match(flow, /managed_attachment_translation_completes_and_reads_real_provider_result/);
  assert.match(flow, /required\("MAKOSH_OLLAMA_LIVE_PORT"\)/);
  assert.match(flow, /Text Extraction workflow/);
  assert.match(flow, /AttachmentTranslationErrorCodeInferenceRejected/);
  assert.match(flow, /AttachmentTranslationErrorCodeSourceRejected/);
  assert.match(flow, /transition_registration/);
  assert.match(flow, /assert_attachment_translation_runtime_fences_v1/);
  assert.match(flow, /stale Attachment Translation runtime generation/);
  assert.match(flow, /stale Attachment Translation grant epoch/);
  assert.match(flow, /authenticate_secondary_gateway_router/);
  assert.match(flow, /assert_attachment_translation_read_fenced_after_restart_v1/);
  assert.match(flow, /Attachment Translation restart must not restart/);
  assert.match(gateway, /ATTACHMENT_TRANSLATION_READ_BLOB_PATH_V1/);
  assert.match(gateway, /open_attachment_translation_sse_v1/);
  assert.match(gateway, /read_terminal_attachment_translation_sse_response_v1/);
  assert.doesNotMatch(gateway, /wait_for_terminal_attachment_translation_v1/);
  assert.match(gateway, /Last-Event-ID|replayable SSE fixture/);
  assert.match(harness, /mod attachment_translation_managed_setup/);
  assert.match(harness, /mod attachment_translation_gateway_fixture/);
  assert.match(harness, /mod attachment_translation_managed_flow/);
  assert.match(runner, /makosh-attachment-translation-runtime/);
  assert.match(runner, /MAKOSH_ATTACHMENT_TRANSLATION_RUNTIME_BIN/);
  assert.match(
    runner,
    /managed_attachment_translation_reaches_source_ai_and_gateway_sse/,
  );
  assert.match(recovery, /materialize_ai_source_from_authority_v1/);
  assert.match(recovery, /refresh_runtime_bound_source/);
  assert.match(recovery, /complete_source_cleanup/);
  assert.match(testkitManifest, /makosh-attachment-translation-api/);
  assert.match(testkitManifest, /makosh-attachment-translation-persistence/);
  assert.match(testkitManifest, /makosh-attachment-translation-runtime/);
});
