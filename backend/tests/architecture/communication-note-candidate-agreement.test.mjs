import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const REPOSITORY_ROOT = new URL('../../../', import.meta.url);

test('note candidate agreement separates Communications workflow Review and Knowledge owners', async () => {
  const [
    adr,
    inventorySource,
    workspace,
    apiManifest,
    api,
    protocol,
    coreManifest,
    core,
    extraction,
    lifecycle,
    persistenceManifest,
    persistence,
    persistenceModel,
    persistenceRepository,
    persistenceOutbox,
    persistenceSchema,
    migration,
    sourceManifest,
    sourceApi,
    sourceProtocol,
    sourceEnvelope,
    runtimeManifest,
    runtime,
    runtimeAdmission,
    runtimeExtraction,
    runtimeReviewSubmission,
    runtimeSourceResults,
    assemblyManifest,
    assembly,
    communicationsRuntimeManifest,
    communicationsAdmission,
    communicationsEventRuntime,
    communicationsNoteSource,
    managedSetup,
    managedFlow,
    managedGatewayFlow,
    managedBlobNegative,
    managedPersistenceFlow,
    authenticatedStorageRunner,
    policySource,
  ] = await Promise.all([
    readFile(
      new URL(
        'docs/adr/ADR-0369-communication-note-candidate-extraction-and-reviewed-knowledge-promotion.md',
        REPOSITORY_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('architecture/communications-settings-reconstruction.json', BACKEND_ROOT)),
    readFile(new URL('Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-note-candidate-api/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-note-candidate-api/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL(
        'src/communication-note-candidate-api/proto/makosh/communication_note_candidate/v1/note_candidate.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('src/communication-note-candidate-core/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-note-candidate-core/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-note-candidate-core/src/extraction.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-note-candidate-core/src/lifecycle.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-note-candidate-persistence/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-note-candidate-persistence/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-note-candidate-persistence/src/model.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-note-candidate-persistence/src/repository.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-note-candidate-persistence/src/outbox.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-note-candidate-persistence/src/schema.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL(
        'src/communication-note-candidate-persistence/migrations/0001_note_candidate.sql',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('src/communications-note-source-api/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communications-note-source-api/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL(
        'src/communications-note-source-api/proto/makosh/communications/note_source/v1/note_source.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('src/communications-note-source-api/src/envelope.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-note-candidate-runtime/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-note-candidate-runtime/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-note-candidate-runtime/src/admission.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-note-candidate-runtime/src/extraction.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-note-candidate-runtime/src/review_submission.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-note-candidate-runtime/src/source_results.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-note-candidate-assembly/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-note-candidate-assembly/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communications-runtime/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communications-runtime/src/admission.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communications-runtime/src/event_runtime.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communications-runtime/src/note_source.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/note_candidate_managed_setup.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/note_candidate_managed_flow.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/note_candidate_gateway_flow.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/note_candidate_blob_negative.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/note_candidate_persistence_flow.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('scripts/test-authenticated-storage.mjs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('architecture/policy.json', BACKEND_ROOT), 'utf8'),
  ]);
  const inventory = JSON.parse(inventorySource);
  const policy = JSON.parse(policySource);
  const slice = inventory.slices.find(
    ({ gate }) => gate === 'communication_note_candidate_extraction_v1',
  );

  assert.deepEqual(slice, {
    gate: 'communication_note_candidate_extraction_v1',
    role: 'workflow',
    owner: 'communication_note_candidate_extraction',
    state: 'implemented',
    dependsOn: ['communications_content_read_v1'],
  });
  assert.equal(policy.implementation.currentSlice, 'speech_to_text_whisper_admission_v1');
  assert.match(adr, /Состояние реализации: implemented/);
  assert.match(adr, /Communications остаётся canonical evidence\/source owner/);
  assert.match(adr, /Extraction остаётся workflow/);
  assert.match(adr, /Review владеет human decision/);
  assert.match(adr, /Knowledge —[\s\S]*durable verified note truth/);
  assert.match(adr, /typed command\/event/);
  assert.match(adr, /target-bound Blob custody/);
  assert.match(adr, /общий replayable SSE/);
  assert.match(adr, /Periodic polling/);
  assert.match(adr, /не использует AI Engine/);
  assert.match(adr, /Ollama остаётся concrete integration/);
  assert.match(adr, /reject никогда не создаёт Knowledge note/i);
  assert.match(adr, /approve —[\s\S]*ровно одну note/i);
  assert.doesNotMatch(adr, /Communications владеет Knowledge|Knowledge читает Communications storage/);
  assert.match(managedFlow, /managed_note_candidate_approve_reject_reaches_gateway_sse_and_replays_after_restart/);
  assert.match(managedFlow, /assert_ne!\(approved_source_message_id, rejected_source_message_id\)/);
  assert.match(managedFlow, /assert_no_note_materialization_v1/);
  assert.match(managedFlow, /assert_exact_note_materialization_v1/);
  assert.match(managedFlow, /restart_note_candidate_runtime_v1/);
  assert.match(managedFlow, /replayed_approved_extraction\.cursor/);
  assert.match(managedFlow, /replayed_rejected_extraction\.cursor/);
  assert.match(managedFlow, /assert_knowledge_reject_stale_blob_receipt_v1/);
  assert.match(managedSetup, /\[NoteCandidateManagedUnitV1; 4\]/);
  assert.match(managedSetup, /COMMUNICATION_NOTE_CANDIDATE_MODULE_ID_V1/);
  assert.match(managedSetup, /REVIEW_NOTE_CANDIDATE_MODULE_ID_V1/);
  assert.match(managedSetup, /REVIEWED_NOTE_CANDIDATE_PROMOTION_MODULE_ID_V1/);
  assert.match(managedSetup, /KNOWLEDGE_MODULE_ID_V1/);
  assert.match(managedGatewayFlow, /knowledge_state/);
  assert.match(managedBlobNegative, /KnowledgeNoteCreationRejectCodeBlobMismatch/);
  assert.match(managedPersistenceFlow, /reserve_approval/);
  assert.match(managedPersistenceFlow, /persist_materialization/);
  assert.match(authenticatedStorageRunner, /managed_note_candidate_approve_reject_reaches_gateway_sse_and_replays_after_restart/);

  for (const unit of [
    'makosh-communication-note-candidate-api',
    'makosh-communication-note-candidate-core',
    'makosh-communication-note-candidate-persistence',
    'makosh-communication-note-candidate-runtime',
    'makosh-communication-note-candidate-assembly',
    'makosh-communications-note-source-api',
  ]) {
    assert.match(workspace, new RegExp(`"src/${unit.replace('makosh-', '')}"`));
    assert.match(adr, new RegExp(`\\b${unit}\\b`));
  }

  assert.match(apiManifest, /role = "workflow"/);
  assert.match(apiManifest, /owner = "communication_note_candidate_extraction"/);
  assert.match(apiManifest, /surface = "contract"/);
  assert.match(api, /communication\.note-candidate-extraction\.v1/);
  assert.match(protocol, /candidate_digest/);
  assert.match(protocol, /string excerpt/);
  assert.match(protocol, /COMMUNICATION_NOTE_TOPIC_HINT_FINANCIAL/);
  assert.match(protocol, /COMMUNICATION_NOTE_TOPIC_HINT_LEGAL/);
  assert.match(protocol, /COMMUNICATION_NOTE_TOPIC_HINT_DECISION_STATEMENT/);
  assert.match(protocol, /COMMUNICATION_NOTE_TOPIC_HINT_DEADLINE_STATEMENT/);
  assert.doesNotMatch(
    protocol,
    /knowledge_note_id|decision_id|document_id|provider_id|account_id|model_id|prompt|map</,
  );

  assert.match(coreManifest, /role = "workflow"/);
  assert.match(coreManifest, /owner = "communication_note_candidate_extraction"/);
  assert.match(core, /extract_communication_note_candidates_v1/);
  assert.match(extraction, /empty_source_does_not_fabricate_a_note_candidate/);
  assert.match(extraction, /legacy_markers_produce_one_bounded_review_candidate/);
  assert.match(extraction, /take\(5\)/);
  assert.match(lifecycle, /SourceIdentityMismatch/);
  assert.doesNotMatch(
    `${core}\n${extraction}\n${lifecycle}`,
    /makosh_communications|makosh_review|makosh_knowledge|ollama|reqwest|sqlx|prompt/,
  );

  assert.match(persistenceManifest, /role = "workflow"/);
  assert.match(persistenceManifest, /owner = "communication_note_candidate_extraction"/);
  assert.match(persistenceManifest, /surface = "persistence"/);
  assert.match(persistence, /CommunicationNoteCandidatePersistenceV1/);
  assert.match(persistenceModel, /candidate_codec_preserves_all_typed_fields_and_empty_result/);
  assert.match(persistenceModel, /CommunicationNoteTopicHintV1/);
  assert.match(persistenceSchema, /communication_note_candidate_extraction_storage_bundle_v1/);
  assert.match(migration, /communication_note_candidate_extraction_runs/);
  assert.match(migration, /communication_note_candidate_extraction_inbox/);
  assert.match(migration, /communication_note_candidate_extraction_outbox/);
  assert.match(migration, /communication_note_candidate_extraction_realtime/);
  assert.match(persistenceRepository, /persist_extraction_transition/);
  assert.match(persistenceRepository, /review_submissions/);
  assert.match(persistenceOutbox, /unpublished_events/);
  assert.match(persistenceOutbox, /mark_event_published/);
  assert.doesNotMatch(
    `${persistence}\n${persistenceModel}\n${persistenceRepository}\n${migration}`,
    /makosh_communications|makosh_review|makosh_knowledge|ollama|prompt|provider_id/,
  );

  assert.match(sourceManifest, /role = "domain"/);
  assert.match(sourceManifest, /owner = "communications"/);
  assert.match(sourceManifest, /surface = "contract"/);
  assert.match(sourceApi, /communications\.note-source\.v1/);
  assert.match(sourceApi, /communication_note_candidate_extraction\.source\.blob\.v1/);
  assert.match(sourceProtocol, /PrepareCommunicationNoteSourceCommandV1/);
  assert.match(sourceProtocol, /CommunicationNoteSourceContentReceiptV1/);
  assert.match(sourceProtocol, /subject_utf8/);
  assert.match(sourceProtocol, /body_utf8/);
  assert.doesNotMatch(sourceProtocol, /provider_id|account_id|model_id|prompt|map</);
  assert.match(sourceEnvelope, /build_communication_note_source_prepare_outbox_record_v1/);
  assert.match(sourceEnvelope, /build_communication_note_source_prepared_outbox_record_v1/);
  assert.match(sourceEnvelope, /build_communication_note_source_rejected_outbox_record_v1/);
  assert.doesNotMatch(sourceEnvelope, /source_content\.subject_utf8|source_content\.body_utf8/);

  assert.match(runtimeManifest, /role = "workflow"/);
  assert.match(runtimeManifest, /owner = "communication_note_candidate_extraction"/);
  assert.match(runtimeManifest, /surface = "runtime"/);
  assert.match(runtime, /CommunicationNoteCandidateManagedRuntimeV1/);
  assert.match(runtimeAdmission, /communication_note_candidate_extraction\.source\.blob\.v1/);
  assert.match(runtimeAdmission, /communication_note_candidate_extraction\.review_submission\.v1/);
  assert.match(runtimeAdmission, /review_note_candidate_submit_publish_request_v1/);
  assert.match(runtimeAdmission, /ProvidedSurfaceKindV1::ClientRealtime/);
  assert.match(runtimeExtraction, /extract_communication_note_candidates_v1/);
  assert.match(runtimeExtraction, /CommunicationNoteSourceContentV1/);
  assert.match(runtimeExtraction, /prepare_review_submissions_v1/);
  assert.match(runtimeReviewSubmission, /build_submit_review_note_candidate_outbox_record_v1/);
  assert.match(runtimeReviewSubmission, /write_review_candidate_v1/);
  assert.match(runtimeReviewSubmission, /SubmitNoteCandidateForReviewCommandV1/);
  assert.match(runtimeSourceResults, /materialize_note_source_v1/);
  assert.match(runtimeManifest, /makosh-review-note-candidate-api/);
  assert.doesNotMatch(runtimeManifest, /makosh-review-note-candidate-(core|persistence|runtime|assembly)/);
  assert.doesNotMatch(
    `${runtime}\n${runtimeAdmission}\n${runtimeExtraction}\n${runtimeReviewSubmission}\n${runtimeSourceResults}`,
    /makosh_review_note_candidate_(core|persistence|runtime|assembly)|makosh_knowledge|ollama|reqwest|prompt|provider_id/,
  );

  assert.match(assemblyManifest, /role = "workflow"/);
  assert.match(assemblyManifest, /owner = "communication_note_candidate_extraction"/);
  assert.match(assemblyManifest, /surface = "assembly"/);
  assert.match(assembly, /communication_note_candidate_extraction_storage_bundle_v1/);
  assert.match(assembly, /communication_note_candidate_extraction\.runtime\.v1/);
  assert.match(assembly, /communication_note_candidate_extraction\.storage\.v1/);
  assert.doesNotMatch(assembly, /makosh_communications|makosh_review|makosh_knowledge|ollama|provider_id/);

  assert.match(communicationsRuntimeManifest, /makosh-communications-note-source-api/);
  assert.match(communicationsAdmission, /communications_note_source_capability_v1/);
  assert.match(communicationsAdmission, /communications\.note-source\.blob\.v1/);
  assert.match(communicationsEventRuntime, /consume_next_note_source_prepare_v1/);
  assert.match(communicationsNoteSource, /CommunicationNoteSourceContentV1/);
  assert.match(communicationsNoteSource, /subject_utf8: snapshot\.subject_utf8\.clone\(\)/);
  assert.match(communicationsNoteSource, /write_target_bound_source/);
  assert.match(communicationsNoteSource, /persist_source_result/);
  assert.doesNotMatch(
    communicationsNoteSource,
    /provider_id|account_id|model_id|prompt|ollama|reqwest/,
  );
});
