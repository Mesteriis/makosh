import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const REPOSITORY_ROOT = new URL('../../../', import.meta.url);

test('Knowledge admission is exact verified-note ownership with atomic owner-local persistence', async () => {
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
    persistenceModel,
    persistenceRepository,
    persistenceSchema,
    migration,
    runtimeManifest,
    runtime,
    admission,
    blob,
    command,
    managedRuntime,
    eventOutbox,
    assemblyManifest,
    assembly,
  ] = await Promise.all([
    readFile(
      new URL('docs/adr/ADR-0370-verified-knowledge-note-owner-admission.md', REPOSITORY_ROOT),
      'utf8',
    ),
    readFile(new URL('architecture/policy.json', BACKEND_ROOT), 'utf8'),
    readFile(new URL('Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/knowledge-command-api/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/knowledge-command-api/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/knowledge-command-api/src/envelope.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL(
        'src/knowledge-command-api/proto/makosh/knowledge/command/v1/knowledge_command.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('src/knowledge-core/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/knowledge-core/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/knowledge-core/src/model.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/knowledge-core/src/creation.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/knowledge-persistence/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/knowledge-persistence/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/knowledge-persistence/src/model.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/knowledge-persistence/src/repository.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/knowledge-persistence/src/schema.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL('src/knowledge-persistence/migrations/0001_knowledge.sql', BACKEND_ROOT),
      'utf8',
    ),
    readFile(new URL('src/knowledge-runtime/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/knowledge-runtime/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/knowledge-runtime/src/admission.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/knowledge-runtime/src/blob.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/knowledge-runtime/src/command.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/knowledge-runtime/src/managed_runtime.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/knowledge-runtime/src/event_outbox.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/knowledge-assembly/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/knowledge-assembly/src/lib.rs', BACKEND_ROOT), 'utf8'),
  ]);
  const policy = JSON.parse(policySource);

  assert.equal(policy.implementation.currentSlice, 'speech_to_text_whisper_admission_v1');
  assert.equal(policy.domains.developmentAllowlist.includes('knowledge'), true);
  assert.equal(policy.domains.blocked.includes('knowledge'), false);
  assert.match(adr, /Состояние реализации: staged/);
  assert.match(adr, /Generic note CRUD, Knowledge Graph, Search, Timeline, Context, Memory/);
  assert.match(adr, /Kernel, Gateway и Event Hub остаются owner-neutral/);
  assert.match(adr, /Cross-owner path остаётся[\s\S]*event-only/);

  for (const unit of [
    'makosh-knowledge-command-api',
    'makosh-knowledge-core',
    'makosh-knowledge-persistence',
    'makosh-knowledge-runtime',
    'makosh-knowledge-assembly',
  ]) {
    assert.match(workspace, new RegExp(`"src/${unit.replace('makosh-', '')}"`));
    assert.match(adr, new RegExp(`\\b${unit}\\b`));
    assert.equal(policy.implementation.productionPackages.some(({ name }) => name === unit), true);
  }

  assert.match(apiManifest, /role = "domain"/);
  assert.match(apiManifest, /owner = "knowledge"/);
  assert.match(apiManifest, /surface = "contract"/);
  assert.match(coreManifest, /role = "domain"/);
  assert.match(coreManifest, /owner = "knowledge"/);
  assert.match(coreManifest, /surface = "implementation"/);
  assert.match(persistenceManifest, /role = "domain"/);
  assert.match(persistenceManifest, /owner = "knowledge"/);
  assert.match(persistenceManifest, /surface = "persistence"/);
  assert.match(persistenceManifest, /makosh-knowledge-core/);
  assert.match(persistenceManifest, /makosh-storage-protocol/);
  assert.doesNotMatch(
    persistenceManifest,
    /makosh-(communications|review|tasks|documents|mail|telegram|whatsapp|zulip)/,
  );

  assert.match(api, /knowledge\.reviewed-candidate\.command\.v1/);
  assert.match(api, /knowledge\.reviewed-candidate\.blob\.v1/);
  assert.match(envelope, /build_create_knowledge_note_from_reviewed_candidate_outbox_record_v1/);
  assert.match(envelope, /ResultOutcomeV1::Succeeded/);
  assert.match(envelope, /ResultOutcomeV1::Rejected/);
  assert.doesNotMatch(
    envelope.split('#[cfg(test)]')[0],
    /title|excerpt|topic_hints|provider_id|account_id/,
  );

  assert.match(protocol, /CreateKnowledgeNoteFromReviewedCandidateCommandV1/);
  assert.match(protocol, /KnowledgeNoteCreatedFromReviewedCandidateV1/);
  assert.match(protocol, /KnowledgeNoteCreationFromReviewedCandidateRejectedV1/);
  assert.match(protocol, /ReviewedKnowledgeNoteContentV1/);
  assert.match(protocol, /KNOWLEDGE_NOTE_TOPIC_HINT_DECISION_STATEMENT/);
  assert.match(protocol, /KNOWLEDGE_NOTE_TOPIC_HINT_DEADLINE_STATEMENT/);
  assert.doesNotMatch(
    protocol,
    /provider_id|account_id|project_id|task_id|decision_id|document_id|model_id|prompt|map<|ollama/,
  );

  assert.match(core, /create_verified_knowledge_note_from_reviewed_candidate_v1/);
  assert.match(model, /VerifiedKnowledgeNoteV1/);
  assert.match(model, /VerifiedKnowledgeNoteStatusV1::Verified/);
  assert.match(model, /KnowledgeNoteProvenanceV1/);
  assert.match(model, /note\.note_revision != 1/);
  assert.match(creation, /reviewed_candidate_creates_exactly_one_deterministic_verified_note/);
  assert.match(creation, /missing_human_decision_evidence_is_rejected/);
  assert.match(creation, /unordered_hints_and_invalid_confidence_are_rejected/);
  assert.doesNotMatch(
    `${core}\n${model}\n${creation}`,
    /makosh_communications|makosh_review|makosh_tasks|makosh_documents|graph|search|context|ollama|sqlx|reqwest/,
  );

  assert.match(persistence, /KnowledgePersistenceV1/);
  assert.match(persistenceModel, /note_creation_fingerprint/);
  assert.match(persistenceRepository, /reserve_command/);
  assert.match(persistenceRepository, /complete_note/);
  assert.match(persistenceRepository, /reject_note/);
  assert.match(persistenceRepository, /outbox_matches/);
  assert.match(
    persistenceRepository,
    /let mut transaction = self\.pool\.begin[\s\S]*insert_note[\s\S]*insert_outbox[\s\S]*transaction\.commit/,
  );
  assert.match(persistenceSchema, /bundle_id: "knowledge"/);
  for (const table of [
    'knowledge_reviewed_candidate_inbox',
    'knowledge_state',
    'knowledge_outbox',
  ]) {
    assert.match(migration, new RegExp(`makosh_data\\.${table}`));
  }
  assert.match(migration, /note_creation_fingerprint/);
  assert.match(migration, /materialized_blob_declared_bytes/);
  assert.match(migration, /materialized_blob_sha256/);
  assert.match(migration, /materialized_blob_custody_proof/);
  assert.match(migration, /topic_hints SMALLINT\[\]/);
  assert.match(migration, /note_revision = 1/);
  assert.doesNotMatch(
    `${persistence}\n${persistenceModel.split('#[cfg(test)]')[0]}\n${persistenceRepository}\n${persistenceSchema.split('#[cfg(test)]')[0]}\n${migration}`,
    /tasks_|review_note_candidate_|communications_|provider_id|account_id|ollama/,
  );

  assert.match(runtimeManifest, /role = "domain"/);
  assert.match(runtimeManifest, /owner = "knowledge"/);
  assert.match(runtimeManifest, /surface = "runtime"/);
  for (const dependency of [
    'makosh-knowledge-command-api',
    'makosh-knowledge-core',
    'makosh-knowledge-persistence',
    'makosh-events-jetstream',
    'makosh-blob-client',
  ]) {
    assert.match(runtimeManifest, new RegExp(dependency));
  }
  assert.doesNotMatch(
    runtimeManifest,
    /makosh-(communications|review|tasks|documents|mail|telegram|whatsapp|zulip)/,
  );
  assert.match(runtime, /KnowledgeManagedRuntimeV1/);
  assert.match(admission, /ModuleKindV1::Domain/);
  assert.match(admission, /knowledge\.reviewed-candidate\.created\.publisher\.v1/);
  assert.match(admission, /knowledge\.reviewed-candidate\.rejected\.publisher\.v1/);
  assert.match(admission, /knowledge\.storage\.v1/);
  assert.match(blob, /request_managed_blob_custody_transfer_v2/);
  assert.match(blob, /request_managed_blob_session_v2/);
  assert.match(blob, /request_managed_blob_custody_release_v2/);
  assert.match(blob, /ReviewedKnowledgeNoteContentV1/);
  assert.match(command, /consume_knowledge_note_command_once_v1/);
  assert.match(command, /create_verified_knowledge_note_from_reviewed_candidate_v1/);
  assert.match(command, /complete_note/);
  assert.match(command, /reject_note/);
  assert.match(managedRuntime, /request_managed_runtime_event_access_v2/);
  assert.match(managedRuntime, /StorageVaultLeaseAdapterV1/);
  assert.match(eventOutbox, /publish_exact/);
  assert.doesNotMatch(
    `${runtime}\n${admission}\n${blob.split('#[cfg(test)]')[0]}\n${command.split('#[cfg(test)]')[0]}\n${managedRuntime}\n${eventOutbox}`,
    /makosh_(communications|review|tasks|documents)|provider_id|account_id|ollama|reqwest|client_polling/,
  );

  assert.match(assemblyManifest, /role = "domain"/);
  assert.match(assemblyManifest, /owner = "knowledge"/);
  assert.match(assemblyManifest, /surface = "assembly"/);
  assert.match(assembly, /materialize_knowledge_release_assembly_v1/);
  assert.match(assembly, /knowledge\.runtime\.v1/);
  assert.match(assembly, /knowledge\.storage\.v1/);
  assert.match(assembly, /create_new\(true\)/);
  assert.doesNotMatch(
    `${assemblyManifest}\n${assembly.split('#[cfg(test)]')[0]}`,
    /makosh-(communications|review|tasks|documents|mail|telegram|whatsapp|zulip)|SigningKey|private_key|p256|ollama/,
  );
});
