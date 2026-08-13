import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const REPOSITORY_ROOT = new URL('../../../', import.meta.url);

test('Tasks reviewed-candidate command and core are distinct target-owned units', async () => {
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
    creation,
    persistenceManifest,
    persistence,
    repository,
    schema,
    migration,
    runtimeManifest,
    runtime,
    admission,
    command,
    blob,
    eventOutbox,
    runtimeMain,
    assemblyManifest,
    assembly,
    assemblyMain,
    release,
  ] = await Promise.all([
    readFile(
      new URL(
        'docs/adr/ADR-0366-communication-task-candidate-extraction-and-reviewed-task-promotion.md',
        REPOSITORY_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('architecture/policy.json', BACKEND_ROOT), 'utf8'),
    readFile(new URL('Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/tasks-command-api/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/tasks-command-api/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/tasks-command-api/src/envelope.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL('src/tasks-command-api/proto/makosh/tasks/command/v1/tasks_command.proto', BACKEND_ROOT),
      'utf8',
    ),
    readFile(new URL('src/tasks-core/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/tasks-core/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/tasks-core/src/model.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/tasks-core/src/creation.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/tasks-persistence/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/tasks-persistence/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/tasks-persistence/src/repository.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/tasks-persistence/src/schema.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/tasks-persistence/migrations/0001_tasks.sql', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/tasks-runtime/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/tasks-runtime/src/managed_runtime.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/tasks-runtime/src/admission.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/tasks-runtime/src/command.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/tasks-runtime/src/blob.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/tasks-runtime/src/event_outbox.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/tasks-runtime/src/main.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/tasks-assembly/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/tasks-assembly/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/tasks-assembly/src/main.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('scripts/materialize-dev-release.sh', BACKEND_ROOT), 'utf8'),
  ]);
  const policy = JSON.parse(policySource);

  assert.equal(policy.implementation.currentSlice, 'speech_to_text_whisper_admission_v1');
  for (const unit of [
    'makosh-tasks-command-api',
    'makosh-tasks-core',
    'makosh-tasks-persistence',
    'makosh-tasks-runtime',
    'makosh-tasks-assembly',
  ]) {
    assert.match(workspace, new RegExp(`"src/${unit.replace('makosh-', '')}"`));
    assert.match(adr, new RegExp(`\\b${unit}\\b`));
    assert.equal(policy.implementation.productionPackages.some(({ name }) => name === unit), true);
  }
  assert.match(apiManifest, /role = "domain"/);
  assert.match(apiManifest, /owner = "tasks"/);
  assert.match(apiManifest, /surface = "contract"/);
  assert.match(coreManifest, /role = "domain"/);
  assert.match(coreManifest, /owner = "tasks"/);
  assert.match(coreManifest, /surface = "implementation"/);
  assert.match(protocol, /CreateTaskFromReviewedCandidateCommandV1/);
  assert.match(protocol, /TaskCreatedFromReviewedCandidateV1/);
  assert.match(protocol, /TaskCreationFromReviewedCandidateRejectedV1/);
  assert.match(protocol, /TasksTargetBoundCandidateReceiptV1/);
  assert.doesNotMatch(protocol, /provider_id|account_id|project_id|calendar_event_id|map<|ollama/);
  assert.match(api, /tasks\.reviewed-candidate\.command\.v1/);
  assert.match(api, /create_task_from_reviewed_candidate_consume_request_v1/);
  assert.match(envelope, /build_create_task_from_reviewed_candidate_outbox_record_v1/);
  assert.match(envelope, /ResultOutcomeV1::Succeeded/);
  assert.match(envelope, /ResultOutcomeV1::Rejected/);
  assert.match(core, /create_task_from_reviewed_candidate_v1/);
  assert.match(model, /TaskProvenanceV1/);
  assert.match(model, /derive_task_id_v1/);
  assert.match(model, /task_creation_fingerprint_v1/);
  assert.match(creation, /reviewed_candidate_creates_exactly_one_deterministic_open_task/);
  assert.match(creation, /hints_do_not_materialize_foreign_domain_identity/);
  assert.doesNotMatch(
    `${core}\n${model}\n${creation}`,
    /makosh_review|makosh_communications|makosh_calendar|makosh_contacts|makosh_projects|sqlx|reqwest/,
  );
  assert.match(persistenceManifest, /owner = "tasks"/);
  assert.match(persistenceManifest, /surface = "persistence"/);
  assert.match(persistence, /TasksPersistenceV1/);
  assert.match(repository, /reserve_command/);
  assert.match(repository, /complete_task/);
  assert.match(repository, /reject_task/);
  assert.match(repository, /load_recoverable_commands/);
  assert.match(repository, /claim_next_pending_outbox/);
  assert.match(repository, /complete_blob_cleanup/);
  assert.match(schema, /tasks_storage_bundle_v1/);
  assert.match(migration, /tasks_reviewed_candidate_inbox/);
  assert.match(migration, /command_envelope_sha256/);
  assert.match(migration, /command_fingerprint/);
  assert.match(migration, /tasks_state/);
  assert.match(migration, /tasks_outbox/);
  assert.doesNotMatch(
    `${persistence}\n${repository}\n${migration}`,
    /review_task_candidate_|communications_|calendar_|contacts_|projects_|obligations_|provider_id|account_id/,
  );
  assert.match(runtimeManifest, /role = "domain"/);
  assert.match(runtimeManifest, /owner = "tasks"/);
  assert.match(runtimeManifest, /surface = "runtime"/);
  assert.match(runtimeMain, /serve-inherited/);
  assert.match(runtimeMain, /recover_command_once/);
  assert.match(admission, /ModuleKindV1::Domain/);
  assert.match(admission, /BlobQuotaOperationV1::ReadRange/);
  assert.match(admission, /BlobQuotaOperationV1::CustodyTransfer/);
  assert.match(admission, /BlobQuotaOperationV1::ReleaseCustody/);
  assert.doesNotMatch(admission, /BlobQuotaOperationV1::Write as i32/);
  assert.match(runtime, /request_managed_runtime_event_access_v2/);
  assert.match(runtime, /exact_subscription/);
  assert.match(command, /reserve_command/);
  assert.match(command, /recover_task_command_once_v1/);
  assert.match(command, /complete_task/);
  assert.match(command, /reject_task/);
  assert.match(command, /delivery\.acknowledge\(\)/);
  assert.match(blob, /request_managed_blob_custody_transfer_v2/);
  assert.match(blob, /request_managed_blob_custody_release_v2/);
  assert.match(command, /command_envelope_sha256/);
  assert.match(command, /persist_materialization/);
  assert.match(eventOutbox, /publish_exact/);
  assert.doesNotMatch(
    `${runtimeManifest}\n${runtime}\n${admission}\n${command}\n${blob}\n${eventOutbox}\n${runtimeMain}`,
    /makosh-review|makosh-communications|makosh-calendar|makosh-contacts|makosh-projects|makosh-ollama|reqwest/,
  );
  assert.match(assemblyManifest, /role = "domain"/);
  assert.match(assemblyManifest, /owner = "tasks"/);
  assert.match(assemblyManifest, /surface = "assembly"/);
  assert.match(assembly, /materialize_tasks_release_assembly_v1/);
  assert.match(assembly, /tasks_module_descriptor_v1/);
  assert.match(assembly, /tasks_storage_bundle_v1/);
  assert.match(assembly, /create_new\(true\)/);
  assert.match(assemblyMain, /--runtime/);
  assert.doesNotMatch(
    `${assemblyManifest}\n${assembly}\n${assemblyMain}`,
    /makosh-review|makosh-communications|makosh-calendar|makosh-ollama|sign_distribution|SigningKey|launch_runtime/,
  );
  for (const unit of [
    'makosh-communication-task-candidate-runtime',
    'makosh-communication-task-candidate-assembly',
    'makosh-review-task-candidate-runtime',
    'makosh-review-task-candidate-assembly',
    'makosh-reviewed-task-candidate-promotion-runtime',
    'makosh-reviewed-task-candidate-promotion-assembly',
    'makosh-tasks-runtime',
    'makosh-tasks-assembly',
  ]) {
    assert.match(release, new RegExp(`--package ${unit}\\b`));
  }
  for (const fragment of [
    'communication_task_candidate.release-artifacts.json',
    'review-task-candidate.release-artifacts.json',
    'reviewed_task_candidate_promotion.release-artifacts.json',
    'tasks.release-artifacts.json',
  ]) {
    assert.match(release, new RegExp(`--artifact-fragment .*${fragment.replaceAll('.', '\\.')}`));
  }
});
