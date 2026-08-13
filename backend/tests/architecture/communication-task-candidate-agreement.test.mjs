import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const REPOSITORY_ROOT = new URL('../../../', import.meta.url);

test('task candidate agreement keeps extraction review and Tasks in separate owner units', async () => {
  const [
    adr,
    promotionAdr,
    inventorySource,
    policySource,
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
    runtimeManifest,
    runtime,
    runtimeAdmission,
    runtimeExtraction,
    runtimeReviewSubmission,
    runtimeSourceResults,
    assemblyManifest,
    assembly,
    sourceManifest,
    sourceApi,
    sourceProtocol,
    sourceEnvelope,
    communicationsRuntimeManifest,
    communicationsAdmission,
    communicationsEventRuntime,
    communicationsTaskSource,
    managedSetup,
    managedFlow,
    managedBlobNegative,
    managedPersistenceFlow,
    tasksRuntimeBlob,
    tasksRuntimeCommand,
    authenticatedStorage,
    promotionApiManifest,
    promotionApi,
    promotionProtocol,
    promotionEnvelope,
    promotionCoreManifest,
    promotionCore,
    promotionPersistenceManifest,
    promotionPersistence,
    promotionPersistenceModel,
    promotionPersistenceRepository,
    promotionPersistenceOutbox,
    promotionPersistenceSchema,
    promotionPersistenceMigration,
    promotionRuntimeManifest,
    promotionRuntime,
    promotionRuntimeAdmission,
    promotionRuntimeApproval,
    promotionRuntimeResults,
    promotionRuntimeOutbox,
    promotionManagedRuntime,
    promotionRuntimeMain,
    promotionAssemblyManifest,
    promotionAssembly,
    promotionAssemblyMain,
  ] = await Promise.all([
    readFile(
      new URL(
        'docs/adr/ADR-0366-communication-task-candidate-extraction-and-reviewed-task-promotion.md',
        REPOSITORY_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'docs/adr/ADR-0368-reviewed-task-candidate-promotion-workflow.md',
        REPOSITORY_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('architecture/communications-settings-reconstruction.json', BACKEND_ROOT)),
    readFile(new URL('architecture/policy.json', BACKEND_ROOT)),
    readFile(new URL('Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-task-candidate-api/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-task-candidate-api/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL(
        'src/communication-task-candidate-api/proto/makosh/communication_task_candidate/v1/task_candidate.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('src/communication-task-candidate-core/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-task-candidate-core/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-task-candidate-core/src/extraction.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-task-candidate-core/src/lifecycle.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-task-candidate-persistence/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-task-candidate-persistence/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-task-candidate-persistence/src/model.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-task-candidate-persistence/src/repository.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-task-candidate-persistence/src/outbox.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-task-candidate-persistence/src/schema.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-task-candidate-persistence/migrations/0001_task_candidate.sql', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-task-candidate-runtime/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-task-candidate-runtime/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-task-candidate-runtime/src/admission.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-task-candidate-runtime/src/extraction.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-task-candidate-runtime/src/review_submission.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-task-candidate-runtime/src/source_results.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-task-candidate-assembly/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-task-candidate-assembly/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communications-task-source-api/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communications-task-source-api/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL(
        'src/communications-task-source-api/proto/makosh/communications/task_source/v1/task_source.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('src/communications-task-source-api/src/envelope.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communications-runtime/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communications-runtime/src/admission.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communications-runtime/src/event_runtime.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communications-runtime/src/task_source.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/task_candidate_managed_setup.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/task_candidate_managed_flow.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/task_candidate_blob_negative.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/task_candidate_persistence_flow.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('src/tasks-runtime/src/blob.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/tasks-runtime/src/command.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('scripts/test-authenticated-storage.mjs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/review-task-candidate-promotion-api/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/review-task-candidate-promotion-api/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL(
        'src/review-task-candidate-promotion-api/proto/makosh/review/task_candidate/promotion/v1/promotion.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('src/review-task-candidate-promotion-api/src/envelope.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/reviewed-task-candidate-promotion-core/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/reviewed-task-candidate-promotion-core/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/reviewed-task-candidate-promotion-persistence/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/reviewed-task-candidate-promotion-persistence/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/reviewed-task-candidate-promotion-persistence/src/model.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/reviewed-task-candidate-promotion-persistence/src/repository.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/reviewed-task-candidate-promotion-persistence/src/outbox.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/reviewed-task-candidate-promotion-persistence/src/schema.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL(
        'src/reviewed-task-candidate-promotion-persistence/migrations/0001_reviewed_task_candidate_promotion.sql',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('src/reviewed-task-candidate-promotion-runtime/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/reviewed-task-candidate-promotion-runtime/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/reviewed-task-candidate-promotion-runtime/src/admission.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/reviewed-task-candidate-promotion-runtime/src/approval.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/reviewed-task-candidate-promotion-runtime/src/task_results.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/reviewed-task-candidate-promotion-runtime/src/event_outbox.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/reviewed-task-candidate-promotion-runtime/src/managed_runtime.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/reviewed-task-candidate-promotion-runtime/src/main.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/reviewed-task-candidate-promotion-assembly/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/reviewed-task-candidate-promotion-assembly/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/reviewed-task-candidate-promotion-assembly/src/main.rs', BACKEND_ROOT), 'utf8'),
  ]);
  const inventory = JSON.parse(inventorySource);
  const policy = JSON.parse(policySource);
  const slice = inventory.slices.find(
    ({ gate }) => gate === 'communication_task_candidate_extraction_v1',
  );

  assert.deepEqual(slice, {
    gate: 'communication_task_candidate_extraction_v1',
    role: 'workflow',
    owner: 'communication_task_candidate_extraction',
    state: 'implemented',
    dependsOn: ['communications_content_read_v1'],
  });
  assert.equal(policy.domains.registered.includes('tasks'), true);
  assert.equal(policy.domains.developmentAllowlist.includes('tasks'), true);
  assert.equal(policy.domains.blocked.includes('tasks'), false);
  assert.equal(policy.implementation.currentSlice, 'speech_to_text_whisper_admission_v1');
  assert.equal(
    policy.implementation.ownerInventory.businessCapabilities.includes(
      'review.task-candidate.promotion-result.v1',
    ),
    true,
  );
  assert.equal(
    policy.implementation.ownerInventory.businessCapabilities.includes(
      'review.task-candidate.promotion-result.consumer.v1',
    ),
    true,
  );
  assert.equal(
    policy.implementation.ownerInventory.businessCapabilities.includes(
      'reviewed_task_candidate_promotion.storage.v1',
    ),
    true,
  );
  assert.match(adr, /Состояние реализации: implemented/);
  assert.match(adr, /Communications остаётся canonical evidence\/source owner/);
  assert.match(adr, /Extraction остаётся workflow/);
  assert.match(adr, /Review владеет human decision/);
  assert.match(adr, /Tasks — durable Task truth/);
  assert.match(adr, /typed durable commands\/results\/events/);
  assert.match(adr, /target-bound Blob custody/);
  assert.match(adr, /общий[\s\S]*replayable SSE/);
  assert.match(adr, /Periodic polling не вводится/);
  assert.match(adr, /AI Engine и Ollama не используются/);
  assert.match(adr, /Kernel, Gateway и Event Hub остаются owner-neutral/);
  assert.match(adr, /CreateTaskFromReviewedCandidateCommandV1/);
  assert.match(adr, /не создаёт Task до approve/);
  assert.match(adr, /reject[\s\S]*никогда не создаёт Task/);
  assert.match(adr, /approve[\s\S]*ровно один source-backed Task/);
  assert.match(adr, /reviewed_task_candidate_promotion workflow/);
  assert.match(promotionAdr, /owner `reviewed_task_candidate_promotion`/);
  assert.match(promotionAdr, /Workflow не читает candidate Blob/);
  assert.match(promotionAdr, /Review напрямую публикует Tasks command/);
  assert.match(promotionAdr, /Kernel или Gateway преобразует payload/);
  assert.match(promotionAdr, /accepted command/i);
  assert.doesNotMatch(promotionAdr, /generic business facade разрешён/);
  assert.doesNotMatch(adr, /generic `create\(entity_kind, payload\)` разрешён/);
  assert.doesNotMatch(adr, /Communications владеет Task|Tasks читает Communications storage/);

  for (const unit of [
    'makosh-communication-task-candidate-api',
    'makosh-communication-task-candidate-core',
    'makosh-communication-task-candidate-persistence',
    'makosh-communication-task-candidate-runtime',
    'makosh-communication-task-candidate-assembly',
    'makosh-communications-task-source-api',
  ]) {
    assert.match(workspace, new RegExp(`"src/${unit.replace('makosh-', '')}"`));
    assert.match(adr, new RegExp(`\\b${unit}\\b`));
  }
  assert.match(apiManifest, /owner = "communication_task_candidate_extraction"/);
  assert.match(apiManifest, /surface = "contract"/);
  assert.match(api, /communication\.task-candidate-extraction\.v1/);
  assert.match(protocol, /candidate_digest/);
  assert.match(protocol, /COMMUNICATION_TASK_SIGNAL_KIND_EXPLICIT_ACTION/);
  assert.match(protocol, /COMMUNICATION_TASK_SIGNAL_KIND_DIRECT_REQUEST/);
  assert.match(protocol, /COMMUNICATION_TASK_SIGNAL_KIND_FOLLOW_UP/);
  assert.doesNotMatch(protocol, /project_id|contact_id|persona_id|provider_id|account_id|model_id|prompt|map</);
  assert.match(coreManifest, /role = "workflow"/);
  assert.match(coreManifest, /owner = "communication_task_candidate_extraction"/);
  assert.match(core, /extract_communication_task_candidates_v1/);
  assert.match(extraction, /empty_source_does_not_fabricate_a_task_candidate/);
  assert.match(extraction, /duplicate_title_across_subject_and_body_becomes_one_combined_candidate/);
  assert.match(lifecycle, /SourceIdentityMismatch/);
  assert.doesNotMatch(`${core}\n${extraction}\n${lifecycle}`, /makosh_communications|makosh_review|makosh_tasks|ollama|reqwest|sqlx/);
  assert.match(persistenceManifest, /owner = "communication_task_candidate_extraction"/);
  assert.match(persistenceManifest, /surface = "persistence"/);
  assert.match(persistence, /CommunicationTaskCandidatePersistenceV1/);
  assert.match(persistenceModel, /candidate_codec_preserves_all_typed_fields_and_empty_result/);
  assert.match(persistenceSchema, /communication_task_candidate_extraction_storage_bundle_v1/);
  assert.match(migration, /communication_task_candidate_extraction_runs/);
  assert.match(migration, /communication_task_candidate_extraction_inbox/);
  assert.match(migration, /communication_task_candidate_extraction_outbox/);
  assert.match(migration, /communication_task_candidate_extraction_realtime/);
  assert.match(persistenceRepository, /persist_extraction_transition/);
  assert.match(persistenceRepository, /review_submissions/);
  assert.match(
    persistenceRepository,
    /communication_task_candidate_extraction_outbox[\s\S]*insert_realtime_transition[\s\S]*transaction\.commit/,
  );
  assert.match(persistenceOutbox, /unpublished_events/);
  assert.match(persistenceOutbox, /mark_event_published/);
  assert.doesNotMatch(`${persistence}\n${persistenceModel}\n${persistenceRepository}\n${migration}`, /communication_recipient_suggestion|makosh_communications|makosh_review|makosh_tasks|ollama|prompt|provider_id/);
  assert.match(runtimeManifest, /owner = "communication_task_candidate_extraction"/);
  assert.match(runtimeManifest, /surface = "runtime"/);
  assert.match(runtime, /CommunicationTaskCandidateManagedRuntimeV1/);
  assert.match(runtimeAdmission, /communication_task_candidate_extraction\.source\.blob\.v1/);
  assert.match(runtimeAdmission, /communication_task_candidate_extraction\.review_submission\.v1/);
  assert.match(runtimeAdmission, /review_task_candidate_submit_contract_reference_v1/);
  assert.match(runtimeAdmission, /review_task_candidate_submit_publish_request_v1/);
  assert.match(runtimeAdmission, /BlobQuotaOperationV1::Write/);
  assert.match(runtimeAdmission, /ProvidedSurfaceKindV1::ClientRealtime/);
  assert.match(runtimeExtraction, /extract_communication_task_candidates_v1/);
  assert.match(runtimeExtraction, /CommunicationTaskSourceContentV1/);
  assert.match(runtimeExtraction, /prepare_review_submissions_v1/);
  assert.match(runtimeReviewSubmission, /build_submit_review_task_candidate_outbox_record_v1/);
  assert.match(runtimeReviewSubmission, /write_review_candidate_v1/);
  assert.match(runtimeReviewSubmission, /ReviewTaskCandidateEnvelopeContextV1/);
  assert.match(runtimeReviewSubmission, /SubmitTaskCandidateForReviewCommandV1/);
  assert.match(runtimeSourceResults, /source_read_receipt_bytes/);
  assert.match(runtimeSourceResults, /materialize_task_source_v1/);
  assert.match(runtimeManifest, /makosh-review-task-candidate-api/);
  assert.doesNotMatch(runtimeManifest, /makosh-review-task-candidate-(core|persistence|runtime|assembly)/);
  assert.doesNotMatch(
    `${runtime}\n${runtimeAdmission}\n${runtimeExtraction}\n${runtimeReviewSubmission}\n${runtimeSourceResults}`,
    /recipient_suggestion|makosh_review_task_candidate_(core|persistence|runtime|assembly)|makosh_tasks|ollama|reqwest|prompt|provider_id/,
  );
  assert.match(assemblyManifest, /owner = "communication_task_candidate_extraction"/);
  assert.match(assemblyManifest, /surface = "assembly"/);
  assert.match(assembly, /communication_task_candidate_extraction_storage_bundle_v1/);
  assert.match(assembly, /communication_task_candidate_extraction\.runtime\.v1/);
  assert.match(assembly, /communication_task_candidate_extraction\.storage\.v1/);
  assert.doesNotMatch(assembly, /recipient_suggestion|makosh_communications|makosh_review|makosh_tasks|ollama|provider_id/);
  assert.match(sourceManifest, /owner = "communications"/);
  assert.match(sourceManifest, /surface = "contract"/);
  assert.match(sourceApi, /communications\.task-source\.v1/);
  assert.match(sourceApi, /communication_task_candidate_extraction\.source\.blob\.v1/);
  assert.match(sourceProtocol, /PrepareCommunicationTaskSourceCommandV1/);
  assert.match(sourceProtocol, /CommunicationTaskSourceContentV1/);
  assert.match(sourceProtocol, /CommunicationTaskSourcePreparedV1/);
  assert.match(sourceProtocol, /CommunicationTaskSourceRejectedV1/);
  assert.doesNotMatch(sourceProtocol, /provider_id|account_id|model_id|prompt|map</);
  assert.match(sourceEnvelope, /target_capability: COMMUNICATIONS_TASK_SOURCE_CAPABILITY_ID_V1/);
  assert.match(communicationsRuntimeManifest, /makosh-communications-task-source-api/);
  assert.match(communicationsAdmission, /communications_task_source_capability_v1/);
  assert.match(communicationsAdmission, /communications\.task-source\.blob\.v1/);
  assert.match(communicationsEventRuntime, /consume_next_task_source_prepare_v1/);
  assert.match(communicationsTaskSource, /CommunicationTaskSourceContentV1/);
  assert.match(communicationsTaskSource, /subject_utf8: snapshot\.subject_utf8\.clone\(\)/);
  assert.match(communicationsTaskSource, /write_target_bound_source/);
  assert.match(communicationsTaskSource, /persist_source_result/);
  assert.doesNotMatch(communicationsTaskSource, /provider_id|account_id|model_id|prompt|ollama|reqwest/);
  assert.match(managedSetup, /installed_task_candidate_ensemble_release_v1/);
  assert.match(managedSetup, /communication_task_candidate_extraction\.runtime\.v1/);
  assert.match(managedSetup, /review\.task-candidate\.runtime\.v1/);
  assert.match(managedSetup, /reviewed_task_candidate_promotion\.runtime\.v1/);
  assert.match(managedSetup, /tasks\.runtime\.v1/);
  assert.match(managedSetup, /ManagedWorkflowRuntimeConfigurationV1/);
  assert.match(managedSetup, /ManagedDomainRuntimeConfigurationV1/);
  assert.match(
    managedFlow,
    /managed_task_candidate_approve_reject_reaches_gateway_sse_and_replays_after_restart/,
  );
  assert.match(managedFlow, /configure_communications_jetstream/);
  assert.match(managedFlow, /start_communications_domain/);
  assert.match(
    managedFlow,
    /assert_communications_transferred_body_projection_with_plaintext/,
  );
  assert.match(managedFlow, /started\.len\(\), 4/);
  assert.match(managedFlow, /start_task_candidate_ensemble_v1/);
  assert.match(managedFlow, /start_task_candidate_extraction_v1/);
  assert.match(managedFlow, /wait_for_ready_task_candidate_extraction_v1/);
  assert.match(managedFlow, /wait_for_rejected_task_candidate_extraction_v1/);
  assert.match(managedFlow, /wait_for_extracted_task_candidate_reviews_v1/);
  assert.match(managedFlow, /assert_no_task_materialization_v1/);
  assert.match(managedFlow, /decide_task_candidate_v1/);
  assert.match(managedFlow, /ReviewTaskCandidateErrorCodeOperationConflict/);
  assert.match(managedFlow, /ReviewTaskCandidateErrorCodeRevisionConflict/);
  assert.match(managedFlow, /assert_exact_task_materialization_v1/);
  assert.match(managedFlow, /read_task_candidate_extraction_terminal_event_v1/);
  assert.match(managedFlow, /read_task_candidate_terminal_events_v1/);
  assert.match(managedFlow, /assert_task_candidate_runtime_fences_v1/);
  assert.match(managedFlow, /route_task_candidate_start_as_v1/);
  assert.match(managedFlow, /revoke_owner/);
  assert.match(managedFlow, /restart_task_candidate_runtime_v1/);
  assert.match(managedFlow, /replayed_extraction\.cursor, extraction_cursor/);
  assert.match(managedFlow, /replayed\.approved\.cursor, approved_cursor/);
  assert.match(managedFlow, /replayed\.rejected\.cursor, rejected_cursor/);
  assert.match(managedFlow, /assert_reviewed_task_candidate_persistence_negatives_v1/);
  assert.match(managedFlow, /assert_tasks_reject_stale_blob_receipt_v1/);
  assert.match(managedBlobNegative, /TaskCreationRejectCodeBlobMismatch/);
  assert.match(managedBlobNegative, /stale Blob receipt must not create Task/);
  assert.match(tasksRuntimeBlob, /BlobClientError::Rejected\(_\)[\s\S]*TasksBlobErrorV1::InvalidReceipt/);
  assert.match(tasksRuntimeBlob, /BlobClientError::Unavailable[\s\S]*TasksBlobErrorV1::Unavailable/);
  assert.match(
    tasksRuntimeCommand,
    /TasksBlobErrorV1::InvalidReceipt[\s\S]*TaskCreationRejectCodeV1::TaskCreationRejectCodeBlobMismatch/,
  );
  assert.match(managedPersistenceFlow, /PersistPromotionApprovalOutcomeV1::Duplicate/);
  assert.match(managedPersistenceFlow, /ApprovalConflict/);
  assert.match(managedPersistenceFlow, /PersistPromotionResultOutcomeV1::Duplicate/);
  assert.match(managedPersistenceFlow, /ReviewedTaskCandidatePromotionPersistenceErrorV1::NotFound/);
  assert.match(managedPersistenceFlow, /ResultConflict/);
  assert.match(managedPersistenceFlow, /OutboxConflict/);
  assert.match(
    authenticatedStorage,
    /MAKOSH_STORAGE_MANAGED_TEST_FILTER[\s\S]*managed_task_candidate_approve_reject_reaches_gateway_sse_and_replays_after_restart/,
  );
  assert.match(
    authenticatedStorage,
    /MAKOSH_REVIEWED_TASK_CANDIDATE_PROMOTION_RUNTIME_BIN/,
  );
  assert.match(promotionApiManifest, /role = "domain"/);
  assert.match(promotionApiManifest, /owner = "review"/);
  assert.match(promotionApiManifest, /surface = "contract"/);
  assert.match(promotionApi, /review\.task-candidate\.promotion-result\.v1/);
  assert.match(promotionApi, /DurableEnvelopeKindV1::Event/);
  assert.match(promotionProtocol, /ReviewTaskCandidatePromotionResultV1/);
  assert.match(promotionProtocol, /expected_review_revision/);
  assert.doesNotMatch(
    promotionProtocol,
    /title|due_text|assignee_label|source_body|provider_id|account_id|map</,
  );
  assert.match(promotionEnvelope, /causation_message_id/);
  assert.match(promotionEnvelope, /ActorKindV1::Module/);
  assert.match(promotionCoreManifest, /role = "workflow"/);
  assert.match(promotionCoreManifest, /owner = "reviewed_task_candidate_promotion"/);
  assert.match(promotionCore, /derive_reviewed_task_candidate_command_id_v1/);
  assert.match(promotionCore, /derive_reviewed_task_candidate_result_id_v1/);
  assert.doesNotMatch(
    `${promotionCoreManifest}\n${promotionCore}`,
    /makosh-review-task-candidate-(core|persistence|runtime)|makosh-tasks-(core|persistence|runtime)|sqlx|async-nats|reqwest|ollama/,
  );
  assert.match(workspace, /"src\/reviewed-task-candidate-promotion-persistence"/);
  assert.match(promotionAdr, /makosh-reviewed-task-candidate-promotion-persistence/);
  assert.match(promotionPersistenceManifest, /role = "workflow"/);
  assert.match(promotionPersistenceManifest, /owner = "reviewed_task_candidate_promotion"/);
  assert.match(promotionPersistenceManifest, /surface = "persistence"/);
  assert.match(promotionPersistenceManifest, /conformance-test-support = \[\]/);
  assert.match(promotionPersistence, /ReviewedTaskCandidatePromotionPersistenceV1/);
  assert.match(promotionPersistenceRepository, /persist_approval_and_tasks_command/);
  assert.match(promotionPersistenceRepository, /persist_tasks_result_and_review_result/);
  assert.match(
    promotionPersistenceRepository,
    /verify_result_inbox[\s\S]*verify_exact_outbox/,
  );
  assert.match(promotionPersistenceRepository, /derive_reviewed_task_candidate_command_id_v1/);
  assert.match(promotionPersistenceRepository, /derive_reviewed_task_candidate_result_id_v1/);
  assert.match(promotionPersistenceOutbox, /unpublished_events/);
  assert.match(promotionPersistenceOutbox, /mark_event_published/);
  assert.match(promotionPersistenceSchema, /reviewed_task_candidate_promotion_storage_bundle_v1/);
  assert.match(promotionPersistenceMigration, /reviewed_task_candidate_promotion_requests/);
  assert.match(promotionPersistenceMigration, /reviewed_task_candidate_promotion_result_inbox/);
  assert.match(promotionPersistenceMigration, /reviewed_task_candidate_promotion_outbox/);
  assert.doesNotMatch(
    `${promotionPersistence}\n${promotionPersistenceModel}\n${promotionPersistenceRepository}\n${promotionPersistenceOutbox}\n${promotionPersistenceMigration}`,
    /candidate_content|source_body|custody_proof|provider_id|account_id|title|due_text|assignee_label|makosh_review_task_candidate_(core|persistence|runtime)|makosh_tasks_(core|persistence|runtime)/,
  );
  assert.match(workspace, /"src\/reviewed-task-candidate-promotion-runtime"/);
  assert.match(promotionRuntimeManifest, /role = "workflow"/);
  assert.match(promotionRuntimeManifest, /owner = "reviewed_task_candidate_promotion"/);
  assert.match(promotionRuntimeManifest, /surface = "runtime"/);
  assert.match(promotionRuntime, /ReviewedTaskCandidatePromotionManagedRuntimeV1/);
  assert.match(promotionRuntimeAdmission, /capabilities: vec!\[/);
  assert.match(promotionRuntimeAdmission, /descriptor\.capabilities\.len\(\), 6/);
  assert.match(promotionRuntimeAdmission, /ProvidedSurfaceKindV1::DurableConsumer/);
  assert.match(promotionRuntimeAdmission, /ProvidedSurfaceKindV1::DurablePublisher/);
  assert.doesNotMatch(promotionRuntimeAdmission, /BlobQuotaRequestV1|ClientRpc|ClientRealtime/);
  assert.match(promotionRuntimeApproval, /consume_approval_once_v1/);
  assert.match(promotionRuntimeApproval, /build_create_task_from_reviewed_candidate_outbox_record_v1/);
  assert.match(promotionRuntimeApproval, /persist_approval_and_tasks_command/);
  assert.match(promotionRuntimeResults, /consume_task_created_once_v1/);
  assert.match(promotionRuntimeResults, /consume_task_rejected_once_v1/);
  assert.match(promotionRuntimeResults, /build_review_task_candidate_promotion_result_outbox_record_v1/);
  assert.match(promotionRuntimeResults, /persist_tasks_result_and_review_result/);
  assert.match(promotionRuntimeOutbox, /publish_exact/);
  assert.match(promotionManagedRuntime, /permits\.len\(\) != 3/);
  assert.match(promotionRuntimeMain, /ManagedWorkflowRuntimeConfigurationV1/);
  assert.doesNotMatch(
    `${promotionRuntimeManifest}\n${promotionRuntime}\n${promotionRuntimeAdmission}\n${promotionRuntimeApproval}\n${promotionRuntimeResults}\n${promotionRuntimeOutbox}\n${promotionManagedRuntime}`,
    /makosh-review-task-candidate-(core|persistence|runtime|assembly)|makosh-tasks-(core|persistence|runtime|assembly)|makosh-blob|ClientRpc|ClientRealtime|provider_id|account_id|ollama|reqwest/,
  );
  assert.match(workspace, /"src\/reviewed-task-candidate-promotion-assembly"/);
  assert.match(promotionAssemblyManifest, /role = "workflow"/);
  assert.match(promotionAssemblyManifest, /owner = "reviewed_task_candidate_promotion"/);
  assert.match(promotionAssemblyManifest, /surface = "assembly"/);
  assert.match(
    promotionAssembly,
    /materialize_reviewed_task_candidate_promotion_release_assembly_v1/,
  );
  assert.match(promotionAssembly, /reviewed_task_candidate_promotion\.runtime\.v1/);
  assert.match(promotionAssembly, /reviewed_task_candidate_promotion\.storage\.v1/);
  assert.match(promotionAssemblyMain, /--runtime/);
  assert.doesNotMatch(promotionAssembly, /private_key|launch_managed|serve-inherited/);
});
