import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const REPOSITORY_ROOT = new URL('../../../', import.meta.url);

test('Review note-candidate is a distinct domain capability without Task or Knowledge implementation coupling', async () => {
  const [
    adr,
    policySource,
    workspace,
    apiManifest,
    api,
    envelope,
    protocol,
    coreManifest,
    core,
    model,
    lifecycle,
    persistenceManifest,
    persistence,
    persistenceModel,
    repository,
    migration,
    promotionManifest,
    promotionProtocol,
    promotionEnvelope,
    runtimeManifest,
    admission,
    submission,
    blobMaterialization,
    promotionResult,
    clientRealtime,
    managedRuntime,
    assemblyManifest,
    assembly,
  ] = await Promise.all([
    readFile(
      new URL(
        'docs/adr/ADR-0369-communication-note-candidate-extraction-and-reviewed-knowledge-promotion.md',
        REPOSITORY_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('architecture/policy.json', BACKEND_ROOT), 'utf8'),
    readFile(new URL('Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/review-note-candidate-api/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/review-note-candidate-api/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/review-note-candidate-api/src/envelope.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL(
        'src/review-note-candidate-api/proto/makosh/review/note_candidate/v1/note_candidate.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('src/review-note-candidate-core/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/review-note-candidate-core/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/review-note-candidate-core/src/model.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/review-note-candidate-core/src/lifecycle.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/review-note-candidate-persistence/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/review-note-candidate-persistence/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/review-note-candidate-persistence/src/model.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/review-note-candidate-persistence/src/repository.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL(
        'src/review-note-candidate-persistence/migrations/0001_review_note_candidate.sql',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('src/review-note-candidate-promotion-api/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL(
        'src/review-note-candidate-promotion-api/proto/makosh/review/note_candidate/promotion/v1/promotion.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('src/review-note-candidate-promotion-api/src/envelope.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/review-note-candidate-runtime/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/review-note-candidate-runtime/src/admission.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/review-note-candidate-runtime/src/submission.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/review-note-candidate-runtime/src/blob_materialization.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/review-note-candidate-runtime/src/promotion_result.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/review-note-candidate-runtime/src/client_realtime.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/review-note-candidate-runtime/src/managed_runtime.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/review-note-candidate-assembly/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/review-note-candidate-assembly/src/lib.rs', BACKEND_ROOT), 'utf8'),
  ]);
  const policy = JSON.parse(policySource);

  assert.equal(policy.implementation.currentSlice, 'speech_to_text_whisper_admission_v1');
  for (const unit of [
    'makosh-review-note-candidate-api',
    'makosh-review-note-candidate-core',
    'makosh-review-note-candidate-persistence',
    'makosh-review-note-candidate-promotion-api',
    'makosh-review-note-candidate-runtime',
    'makosh-review-note-candidate-assembly',
  ]) {
    assert.match(workspace, new RegExp(`"src/${unit.replace('makosh-', '')}"`));
    assert.match(adr, new RegExp(`\\b${unit}\\b`));
    assert.equal(policy.implementation.productionPackages.some(({ name }) => name === unit), true);
  }

  assert.match(apiManifest, /role = "domain"/);
  assert.match(apiManifest, /owner = "review"/);
  assert.match(apiManifest, /surface = "contract"/);
  assert.match(coreManifest, /role = "domain"/);
  assert.match(coreManifest, /owner = "review"/);
  assert.match(coreManifest, /surface = "implementation"/);
  assert.match(persistenceManifest, /role = "domain"/);
  assert.match(persistenceManifest, /owner = "review"/);
  assert.match(persistenceManifest, /surface = "persistence"/);
  assert.match(promotionManifest, /role = "domain"/);
  assert.match(promotionManifest, /owner = "review"/);
  assert.match(promotionManifest, /surface = "contract"/);
  assert.match(runtimeManifest, /role = "domain"/);
  assert.match(runtimeManifest, /owner = "review"/);
  assert.match(runtimeManifest, /surface = "runtime"/);
  assert.match(assemblyManifest, /role = "domain"/);
  assert.match(assemblyManifest, /owner = "review"/);
  assert.match(assemblyManifest, /surface = "assembly"/);

  assert.match(api, /review\.note-candidate\.submission\.v1/);
  assert.match(api, /review\.note-candidate\.promotion\.v1/);
  assert.match(api, /REVIEWED_NOTE_CANDIDATE_PROMOTION_BLOB_TARGET_OWNER_ID_V1/);
  assert.match(api, /"reviewed_note_candidate_promotion"/);
  assert.match(envelope, /build_submit_review_note_candidate_outbox_record_v1/);
  assert.match(envelope, /build_review_note_candidate_approved_outbox_record_v1/);
  assert.match(envelope, /ActorKindV1::OwnerDevice/);
  assert.doesNotMatch(
    envelope.split('#[cfg(test)]')[0],
    /title|excerpt|topic_hints|provider_id|account_id/,
  );

  assert.match(protocol, /SubmitNoteCandidateForReviewCommandV1/);
  assert.match(protocol, /NoteCandidateApprovedForPromotionV1/);
  assert.match(protocol, /ReviewNoteCandidateContentV1/);
  assert.match(protocol, /string excerpt/);
  assert.match(protocol, /REVIEW_NOTE_TOPIC_HINT_FINANCIAL/);
  assert.match(protocol, /REVIEW_NOTE_TOPIC_HINT_LEGAL/);
  assert.match(protocol, /REVIEW_NOTE_TOPIC_HINT_DECISION_STATEMENT/);
  assert.match(protocol, /REVIEW_NOTE_TOPIC_HINT_DEADLINE_STATEMENT/);
  assert.doesNotMatch(
    protocol,
    /due_text_hint|assignee_label_hint|task_id|knowledge_note_id|provider_id|account_id|model_id|prompt|map<|ollama/,
  );

  assert.match(core, /decide_review_note_candidate_v1/);
  assert.match(model, /ReviewNoteCandidatePromotionStatusV1/);
  assert.match(model, /ReviewNoteTopicHintV1/);
  assert.match(model, /promoted_note_id/);
  assert.match(lifecycle, /approval_is_terminal_and_starts_separate_promotion/);
  assert.match(lifecycle, /rejection_never_requests_promotion/);
  assert.match(lifecycle, /stale_revision_and_missing_human_actor_are_rejected/);
  assert.doesNotMatch(
    `${core}\n${model}\n${lifecycle}`,
    /review_attention|makosh_communications|makosh_tasks|makosh_knowledge|ollama|sqlx|reqwest/,
  );

  assert.match(persistence, /ReviewNoteCandidatePersistenceV1/);
  assert.match(persistenceModel, /PersistReviewNoteCandidatePromotionResultV1/);
  assert.match(repository, /reserve_submission/);
  assert.match(repository, /complete_submission/);
  assert.match(repository, /pub async fn decide\(/);
  assert.match(repository, /persist_promotion_result/);
  for (const table of [
    'review_note_candidate_submissions',
    'review_note_candidate_state',
    'review_note_candidate_operations',
    'review_note_candidate_promotion_inbox',
    'review_note_candidate_outbox',
    'review_note_candidate_realtime',
  ]) {
    assert.match(migration, new RegExp(`makosh_data\\.${table}`));
  }
  assert.match(migration, /topic_hints SMALLINT\[\] NOT NULL/);
  assert.match(migration, /confidence_basis_points INTEGER NOT NULL/);
  assert.doesNotMatch(
    `${persistence}\n${persistenceModel}\n${repository}\n${migration}`,
    /makosh_(communications|tasks|knowledge)|makosh-(communications|tasks|knowledge)|provider_id|account_id|ollama/,
  );

  assert.match(promotionProtocol, /ReviewNoteCandidatePromotionResultV1/);
  assert.match(promotionProtocol, /optional bytes note_id/);
  assert.doesNotMatch(
    promotionProtocol,
    /title|excerpt|topic_hints|source_body|provider_id|account_id|map</,
  );
  assert.match(promotionEnvelope, /DurableEnvelopeV1/);
  assert.match(promotionEnvelope, /ActorKindV1::Module/);

  assert.match(admission, /review_note_candidate_module_descriptor_v1/);
  assert.match(admission, /ModuleKindV1::Domain/);
  assert.match(admission, /review_note_candidate_promotion_result_consume_request_v1/);
  assert.match(submission, /consume_review_note_candidate_submission_once_v1/);
  assert.match(submission, /ReviewNoteSourceBasisV1/);
  assert.match(submission, /ReviewNoteTopicHintV1/);
  assert.match(blobMaterialization, /REVIEWED_NOTE_CANDIDATE_PROMOTION_BLOB_TARGET_OWNER_ID_V1/);
  assert.match(blobMaterialization, /write_promotion_candidate_v1/);
  assert.match(promotionResult, /consume_review_note_candidate_promotion_result_once_v1/);
  assert.match(clientRealtime, /ManagedRuntimeClientRealtimePublishRequestV1/);
  assert.match(clientRealtime, /review-note-candidate\/\{\}/);
  assert.match(managedRuntime, /RuntimeSubscribePermitV1/);
  assert.doesNotMatch(
    `${runtimeManifest}\n${admission}\n${submission}\n${blobMaterialization}\n${promotionResult}\n${clientRealtime}\n${managedRuntime}`,
    /makosh_(communications|tasks|knowledge)|makosh-(communications|tasks|knowledge)|ollama|provider_id|account_id/,
  );
  assert.match(assembly, /materialize_review_note_candidate_release_assembly_v1/);
  assert.match(assembly, /REVIEW_NOTE_CANDIDATE_RUNTIME_ARTIFACT_ID_V1/);
  assert.match(assembly, /REVIEW_NOTE_CANDIDATE_STORAGE_ARTIFACT_ID_V1/);
  assert.match(assembly, /create_new\(true\)/);
  assert.doesNotMatch(
    assembly,
    /makosh_(communications|tasks|knowledge)|makosh-(communications|tasks|knowledge)|signing[_-]?key|private[_-]?key|ollama/,
  );
});
