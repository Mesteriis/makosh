import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const REPOSITORY_ROOT = new URL('../../../', import.meta.url);

test('reviewed note promotion is an event-only workflow with a fresh Knowledge custody', async () => {
  const [
    adr,
    policySource,
    workspace,
    coreManifest,
    core,
    persistenceManifest,
    persistence,
    model,
    repository,
    migration,
    runtimeManifest,
    runtime,
    admission,
    approval,
    blobHandoff,
    noteResults,
    managedRuntime,
    runtimeMain,
    assemblyManifest,
    assembly,
    assemblyMain,
    release,
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
    readFile(new URL('src/reviewed-note-candidate-promotion-core/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/reviewed-note-candidate-promotion-core/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL('src/reviewed-note-candidate-promotion-persistence/Cargo.toml', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/reviewed-note-candidate-promotion-persistence/src/lib.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/reviewed-note-candidate-promotion-persistence/src/model.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/reviewed-note-candidate-promotion-persistence/src/repository.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL(
        'src/reviewed-note-candidate-promotion-persistence/migrations/0001_reviewed_note_candidate_promotion.sql',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('src/reviewed-note-candidate-promotion-runtime/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/reviewed-note-candidate-promotion-runtime/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL('src/reviewed-note-candidate-promotion-runtime/src/admission.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/reviewed-note-candidate-promotion-runtime/src/approval.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/reviewed-note-candidate-promotion-runtime/src/blob_handoff.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/reviewed-note-candidate-promotion-runtime/src/note_results.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/reviewed-note-candidate-promotion-runtime/src/managed_runtime.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(new URL('src/reviewed-note-candidate-promotion-runtime/src/main.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL('src/reviewed-note-candidate-promotion-assembly/Cargo.toml', BACKEND_ROOT),
      'utf8',
    ),
    readFile(new URL('src/reviewed-note-candidate-promotion-assembly/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/reviewed-note-candidate-promotion-assembly/src/main.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('scripts/materialize-dev-release.sh', BACKEND_ROOT), 'utf8'),
  ]);
  const policy = JSON.parse(policySource);
  const units = [
    'makosh-reviewed-note-candidate-promotion-core',
    'makosh-reviewed-note-candidate-promotion-persistence',
    'makosh-reviewed-note-candidate-promotion-runtime',
    'makosh-reviewed-note-candidate-promotion-assembly',
  ];

  assert.equal(
    policy.implementation.currentSlice,
    'speech_to_text_whisper_admission_v1',
  );
  assert.equal(
    policy.implementation.ownerInventory.workflows.includes(
      'reviewed_note_candidate_promotion',
    ),
    true,
  );
  for (const unit of units) {
    assert.match(workspace, new RegExp(`"src/${unit.replace('makosh-', '')}"`));
    assert.match(adr, new RegExp(`\\b${unit}\\b`));
    assert.equal(
      policy.implementation.productionPackages.some(({ name }) => name === unit),
      true,
    );
  }

  assert.match(coreManifest, /role = "workflow"/);
  assert.match(coreManifest, /owner = "reviewed_note_candidate_promotion"/);
  assert.match(core, /derive_reviewed_note_candidate_command_id_v1/);
  assert.match(core, /derive_reviewed_note_candidate_result_id_v1/);
  assert.doesNotMatch(core, /makosh_review|makosh_knowledge|sqlx|reqwest|ollama/);

  assert.match(persistenceManifest, /surface = "persistence"/);
  assert.match(persistenceManifest, /conformance-test-support = \[\]/);
  assert.match(persistence, /ReviewedNoteCandidatePromotionPersistenceV1/);
  assert.match(model, /PromotionBlobReceiptV1/);
  assert.match(repository, /reserve_approval/);
  assert.match(repository, /persist_materialization/);
  assert.match(repository, /persist_approval_and_knowledge_command/);
  assert.match(repository, /persist_workflow_failure/);
  assert.match(repository, /persist_knowledge_result_and_review_result/);
  assert.match(repository, /complete_source_cleanup/);
  assert.match(migration, /reviewed_note_candidate_promotion_requests/);
  assert.match(migration, /source_blob_reference_id/);
  assert.match(migration, /materialized_blob_reference_id/);
  assert.match(migration, /workflow_failure_result_id/);
  assert.match(migration, /reviewed_note_candidate_promotion_result_inbox/);
  assert.match(migration, /reviewed_note_candidate_promotion_outbox/);
  assert.doesNotMatch(
    `${persistence}\n${repository}\n${migration}`,
    /review_note_candidate_state|knowledge_notes|provider_id|account_id|ollama/,
  );

  assert.match(runtimeManifest, /surface = "runtime"/);
  assert.match(runtimeManifest, /makosh-review-note-candidate-api/);
  assert.match(runtimeManifest, /makosh-knowledge-command-api/);
  assert.doesNotMatch(
    runtimeManifest,
    /makosh-review-note-candidate-(core|persistence|runtime|assembly)|makosh-knowledge-(core|persistence|runtime|assembly)/,
  );
  assert.match(runtime, /ReviewedNoteCandidatePromotionManagedRuntimeV1/);
  assert.match(admission, /ModuleKindV1::Workflow/);
  assert.match(admission, /descriptor\.capabilities\.len\(\), 7/);
  assert.match(admission, /BlobQuotaOperationV1::CustodyTransfer/);
  assert.match(admission, /BlobQuotaOperationV1::Write/);
  assert.match(approval, /reserve_approval/);
  assert.match(approval, /persist_materialization/);
  assert.match(approval, /build_create_knowledge_note_from_reviewed_candidate_outbox_record_v1/);
  assert.match(approval, /persist_approval_and_knowledge_command/);
  assert.match(approval, /persist_invalid_source_result/);
  assert.match(approval, /complete_source_cleanup/);
  assert.match(blobHandoff, /request_managed_blob_custody_transfer_v2/);
  assert.match(blobHandoff, /ReviewNoteCandidateContentV1::decode/);
  assert.match(blobHandoff, /ReviewedKnowledgeNoteContentV1/);
  assert.match(blobHandoff, /content\.encode_to_vec/);
  assert.match(blobHandoff, /KNOWLEDGE_REVIEWED_CANDIDATE_BLOB_CAPABILITY_ID_V1/);
  assert.match(blobHandoff, /request_managed_blob_custody_release_v2/);
  assert.match(noteResults, /consume_note_created_once_v1/);
  assert.match(noteResults, /consume_note_rejected_once_v1/);
  assert.match(noteResults, /persist_knowledge_result_and_review_result/);
  assert.match(managedRuntime, /request_managed_runtime_event_access_v2/);
  assert.match(managedRuntime, /permits\.len\(\) != 3/);
  assert.match(runtimeMain, /ManagedWorkflowRuntimeConfigurationV1/);
  assert.doesNotMatch(
    `${runtime}\n${approval}\n${blobHandoff}\n${noteResults}\n${managedRuntime}`,
    /makosh_review_note_candidate_(core|persistence|runtime|assembly)|makosh_knowledge_(core|persistence|runtime|assembly)|ClientRpc|ClientRealtime|provider_id|account_id|ollama|reqwest/,
  );

  assert.match(assemblyManifest, /surface = "assembly"/);
  assert.match(assembly, /materialize_reviewed_note_candidate_promotion_release_assembly_v1/);
  assert.match(assembly, /reviewed_note_candidate_promotion\.runtime\.v1/);
  assert.match(assembly, /reviewed_note_candidate_promotion\.storage\.v1/);
  assert.match(assemblyMain, /--runtime/);
  assert.doesNotMatch(assembly, /private_key|sign_distribution|launch_managed|serve-inherited/);
  assert.match(release, /--package makosh-reviewed-note-candidate-promotion-runtime/);
  assert.match(release, /--package makosh-reviewed-note-candidate-promotion-assembly/);
  assert.match(release, /reviewed_note_candidate_promotion\.release-artifacts\.json/);
});
