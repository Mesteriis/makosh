import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const REPOSITORY_ROOT = new URL('../../../', import.meta.url);

test('Review task-candidate is an exact domain capability, not an attention facade', async () => {
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
    repository,
    schema,
    migration,
    runtimeManifest,
    runtime,
    admission,
    managedRuntime,
    submission,
    blobMaterialization,
    clientPort,
    clientRealtime,
    eventOutbox,
    promotionResult,
    runtimeMain,
    assemblyManifest,
    assembly,
    assemblyMain,
  ] =
    await Promise.all([
      readFile(
        new URL(
          'docs/adr/ADR-0366-communication-task-candidate-extraction-and-reviewed-task-promotion.md',
          REPOSITORY_ROOT,
        ),
        'utf8',
      ),
      readFile(new URL('architecture/policy.json', BACKEND_ROOT), 'utf8'),
      readFile(new URL('Cargo.toml', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/review-task-candidate-api/Cargo.toml', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/review-task-candidate-api/src/lib.rs', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/review-task-candidate-api/src/envelope.rs', BACKEND_ROOT), 'utf8'),
      readFile(
        new URL(
          'src/review-task-candidate-api/proto/makosh/review/task_candidate/v1/task_candidate.proto',
          BACKEND_ROOT,
        ),
        'utf8',
      ),
      readFile(new URL('src/review-task-candidate-core/Cargo.toml', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/review-task-candidate-core/src/lib.rs', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/review-task-candidate-core/src/model.rs', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/review-task-candidate-core/src/lifecycle.rs', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/review-task-candidate-persistence/Cargo.toml', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/review-task-candidate-persistence/src/lib.rs', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/review-task-candidate-persistence/src/repository.rs', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/review-task-candidate-persistence/src/schema.rs', BACKEND_ROOT), 'utf8'),
      readFile(
        new URL(
          'src/review-task-candidate-persistence/migrations/0001_review_task_candidate.sql',
          BACKEND_ROOT,
        ),
        'utf8',
      ),
      readFile(new URL('src/review-task-candidate-runtime/Cargo.toml', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/review-task-candidate-runtime/src/lib.rs', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/review-task-candidate-runtime/src/admission.rs', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/review-task-candidate-runtime/src/managed_runtime.rs', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/review-task-candidate-runtime/src/submission.rs', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/review-task-candidate-runtime/src/blob_materialization.rs', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/review-task-candidate-runtime/src/client_port.rs', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/review-task-candidate-runtime/src/client_realtime.rs', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/review-task-candidate-runtime/src/event_outbox.rs', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/review-task-candidate-runtime/src/promotion_result.rs', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/review-task-candidate-runtime/src/main.rs', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/review-task-candidate-assembly/Cargo.toml', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/review-task-candidate-assembly/src/lib.rs', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/review-task-candidate-assembly/src/main.rs', BACKEND_ROOT), 'utf8'),
    ]);
  const policy = JSON.parse(policySource);

  assert.equal(policy.implementation.currentSlice, 'speech_to_text_whisper_admission_v1');
  for (const unit of [
    'makosh-review-task-candidate-api',
    'makosh-review-task-candidate-core',
    'makosh-review-task-candidate-persistence',
    'makosh-review-task-candidate-runtime',
    'makosh-review-task-candidate-assembly',
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
  assert.match(api, /review\.task-candidate\.submission\.v1/);
  assert.match(api, /review\.task-candidate\.promotion\.v1/);
  assert.match(api, /TASKS_REVIEWED_CANDIDATE_BLOB_TARGET_OWNER_ID_V1/);
  assert.match(envelope, /build_submit_review_task_candidate_outbox_record_v1/);
  assert.match(envelope, /build_review_task_candidate_approved_outbox_record_v1/);
  assert.match(envelope, /ActorKindV1::OwnerDevice/);
  assert.doesNotMatch(
    envelope.split('#[cfg(test)]')[0],
    /title|due_text_hint|assignee_label_hint/,
  );
  assert.match(protocol, /SubmitTaskCandidateForReviewCommandV1/);
  assert.match(protocol, /TaskCandidateApprovedForPromotionV1/);
  assert.match(protocol, /ReviewTargetBoundCandidateReceiptV1/);
  assert.match(protocol, /ReviewTaskCandidateStatusChangedV1/);
  assert.doesNotMatch(protocol, /provider_id|account_id|model_id|prompt|map<|google|telegram|ollama/);
  assert.match(core, /decide_review_task_candidate_v1/);
  assert.match(model, /ReviewTaskCandidatePromotionStatusV1/);
  assert.match(lifecycle, /approval_is_terminal_and_starts_separate_promotion/);
  assert.match(lifecycle, /rejection_never_requests_promotion/);
  assert.match(lifecycle, /stale_revision_and_missing_human_actor_are_rejected/);
  assert.doesNotMatch(`${core}\n${model}\n${lifecycle}`, /review_attention|makosh_communications|makosh_tasks|ollama|sqlx|reqwest/);
  assert.match(persistenceManifest, /owner = "review"/);
  assert.match(persistenceManifest, /surface = "persistence"/);
  assert.match(persistence, /ReviewTaskCandidatePersistenceV1/);
  assert.match(repository, /reserve_submission/);
  assert.match(repository, /load_recoverable_submissions/);
  assert.match(repository, /persist_materialization/);
  assert.match(repository, /complete_blob_cleanup/);
  assert.match(repository, /review_task_candidate_operations/);
  assert.match(repository, /review_task_candidate_promotion_inbox/);
  assert.match(repository, /insert_outbox/);
  assert.match(repository, /insert_realtime/);
  assert.match(schema, /review_task_candidate_storage_bundle_v1/);
  assert.match(migration, /request_sha256 BYTEA/);
  assert.match(migration, /decision_fingerprint BYTEA/);
  assert.match(migration, /review_task_candidate_outbox/);
  assert.match(migration, /review_task_candidate_realtime/);
  assert.match(migration, /materialized_blob_reference_id/);
  assert.match(migration, /cleanup_completed_at_unix_millis/);
  assert.doesNotMatch(`${persistence}\n${repository}\n${migration}`, /review_attention|communications_|tasks_|provider_id|account_id|ollama|prompt|model_id/);
  assert.match(adr, /без[\s\S]*расширения `review-attention`/);
  assert.match(runtimeManifest, /role = "domain"/);
  assert.match(runtimeManifest, /owner = "review"/);
  assert.match(runtimeManifest, /surface = "runtime"/);
  assert.match(runtime, /review_task_candidate_module_descriptor_v1/);
  assert.match(admission, /ModuleKindV1::Domain/);
  assert.match(admission, /review_task_candidate_submit_consume_request_v1/);
  assert.match(admission, /review_task_candidate_promotion_result_consume_request_v1/);
  assert.match(admission, /BlobQuotaOperationV1::Write/);
  assert.doesNotMatch(admission, /makosh_tasks|makosh_communications|ollama/);
  assert.match(managedRuntime, /request_managed_runtime_event_access_v2/);
  assert.match(managedRuntime, /logical_human_owner_id/);
  assert.match(managedRuntime, /authenticated_device_id/);
  assert.match(submission, /decode_envelope_v1/);
  assert.match(submission, /reserve_submission/);
  assert.match(submission, /transfer_review_candidate_v1/);
  assert.match(submission, /cleanup_materialization/);
  assert.match(blobMaterialization, /TASKS_REVIEWED_CANDIDATE_BLOB_TARGET_OWNER_ID_V1/);
  assert.match(clientPort, /owner_device_actor_id/);
  assert.match(clientPort, /build_review_task_candidate_approved_outbox_record_v1/);
  assert.match(clientRealtime, /ManagedRuntimeClientRealtimePublishRequestV1/);
  assert.match(eventOutbox, /publish_exact/);
  assert.match(promotionResult, /persist_promotion_result/);
  assert.match(promotionResult, /delivery\.acknowledge\(\)/);
  assert.match(promotionResult, /ReviewTaskCandidatePromotionResultV1/);
  assert.doesNotMatch(promotionResult, /makosh_tasks|reviewed_task_candidate_promotion_runtime/);
  assert.match(runtimeMain, /serve-inherited/);
  assert.match(assemblyManifest, /surface = "assembly"/);
  assert.match(assembly, /materialize_review_task_candidate_release_assembly_v1/);
  assert.match(assembly, /review_task_candidate_storage_bundle_v1/);
  assert.match(assembly, /review_task_candidate_module_descriptor_v1/);
  assert.match(assembly, /review\.task-candidate\.runtime\.v1/);
  assert.match(assembly, /review\.task-candidate\.storage\.v1/);
  assert.match(assemblyMain, /--runtime/);
  assert.doesNotMatch(assembly, /private_key|launch_managed|serve-inherited/);
  assert.doesNotMatch(
    `${managedRuntime}\n${submission}\n${blobMaterialization}\n${clientPort}`,
    /makosh_tasks::|makosh_communications::|makosh_ollama/,
  );
});
