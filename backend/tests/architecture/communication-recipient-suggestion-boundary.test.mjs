import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const REPOSITORY_ROOT = new URL('../../../', import.meta.url);

test('recipient suggestion agreement separates source ownership from workflow decisions', async () => {
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
    sourceManifest,
    sourceApi,
    sourceProtocol,
    sourceEnvelope,
    persistenceManifest,
    persistenceSchema,
    persistenceModel,
    persistenceRepository,
    persistenceRealtime,
    runtimeManifest,
    runtimeAdmission,
    runtimeEvaluation,
    runtimeSourceResults,
    runtimeManaged,
    communicationsRuntimeManifest,
    communicationsAdmission,
    communicationsEventRuntime,
    communicationsRecipientSource,
    communicationsSourceSnapshot,
    assemblyManifest,
    assembly,
    releaseScript,
    managedSetup,
    managedFlow,
    managedScript,
  ] = await Promise.all([
    readFile(
      new URL(
        'docs/adr/ADR-0365-communication-recipient-suggestion-workflow-and-source-boundary.md',
        REPOSITORY_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('architecture/communications-settings-reconstruction.json', BACKEND_ROOT),
      'utf8',
    ),
    readFile(new URL('architecture/policy.json', BACKEND_ROOT), 'utf8'),
    readFile(new URL('Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-recipient-suggestion-api/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-recipient-suggestion-api/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL(
        'src/communication-recipient-suggestion-api/proto/makosh/communication_recipient_suggestion/v1/recipient_suggestion.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('src/communication-recipient-suggestion-core/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-recipient-suggestion-core/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communications-recipient-source-api/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communications-recipient-source-api/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL(
        'src/communications-recipient-source-api/proto/makosh/communications/recipient_source/v1/recipient_source.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('src/communications-recipient-source-api/src/envelope.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL('src/communication-recipient-suggestion-persistence/Cargo.toml', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-recipient-suggestion-persistence/migrations/0001_recipient_suggestion.sql',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('src/communication-recipient-suggestion-persistence/src/model.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/communication-recipient-suggestion-persistence/src/repository.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/communication-recipient-suggestion-persistence/src/realtime.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(new URL('src/communication-recipient-suggestion-runtime/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-recipient-suggestion-runtime/src/admission.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-recipient-suggestion-runtime/src/evaluation.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-recipient-suggestion-runtime/src/source_results.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-recipient-suggestion-runtime/src/managed_runtime.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communications-runtime/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communications-runtime/src/admission.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communications-runtime/src/event_runtime.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communications-runtime/src/recipient_source.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communications-persistence/src/source_snapshot.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-recipient-suggestion-assembly/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-recipient-suggestion-assembly/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('scripts/materialize-dev-release.sh', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/communication_recipient_suggestion_managed_setup.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/communication_recipient_suggestion_managed_flow.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('scripts/test-authenticated-storage.mjs', BACKEND_ROOT), 'utf8'),
  ]);
  const inventory = JSON.parse(inventorySource);
  const policy = JSON.parse(policySource);
  const source = inventory.slices.find(({ gate }) => gate === 'communications_recipient_source_v1');
  const workflow = inventory.slices.find(
    ({ gate }) => gate === 'communication_recipient_suggestion_v1',
  );

  assert.deepEqual(source, {
    gate: 'communications_recipient_source_v1',
    role: 'domain',
    owner: 'communications',
    state: 'implemented',
    dependsOn: ['communications_canonical_read_v2', 'blob_v1', 'nats_data_plane_v1'],
  });
  assert.deepEqual(workflow, {
    gate: 'communication_recipient_suggestion_v1',
    role: 'workflow',
    owner: 'communication_recipient_suggestion',
    state: 'implemented',
    dependsOn: ['communications_recipient_source_v1', 'client_gateway_v1', 'blob_v1'],
  });
  for (const unit of [
    'makosh-communications-recipient-source-api',
    'makosh-communication-recipient-suggestion-api',
    'makosh-communication-recipient-suggestion-core',
    'makosh-communication-recipient-suggestion-persistence',
    'makosh-communication-recipient-suggestion-runtime',
    'makosh-communication-recipient-suggestion-assembly',
  ]) {
    assert.match(adr, new RegExp(`\\b${unit}\\b`));
  }
  assert.match(adr, /какую bounded[\s\S]*организационную роль/);
  assert.match(adr, /accounting_or_bookkeeping/);
  assert.match(adr, /legal_counsel/);
  assert.match(adr, /project_stakeholder/);
  assert.match(adr, /target-bound Blob/);
  assert.match(adr, /общий replayable SSE/);
  assert.match(adr, /Kernel\/Gateway не компилируют/);
  assert.match(adr, /Состояние реализации: implemented/);
  assert.doesNotMatch(adr, /Communications (?:owns|владеет) recipient decision|generic `execute\(any\)`/i);

  assert.equal(
    policy.implementation.currentSlice,
    'call_transcription_managed_conformance_v1',
  );
  assert.match(workspace, /"src\/communication-recipient-suggestion-api"/);
  assert.match(workspace, /"src\/communication-recipient-suggestion-core"/);
  assert.match(workspace, /"src\/communication-recipient-suggestion-persistence"/);
  assert.match(workspace, /"src\/communication-recipient-suggestion-runtime"/);
  assert.match(workspace, /"src\/communications-recipient-source-api"/);
  assert.match(apiManifest, /owner = "communication_recipient_suggestion"/);
  assert.match(apiManifest, /surface = "contract"/);
  assert.match(coreManifest, /owner = "communication_recipient_suggestion"/);
  assert.match(coreManifest, /surface = "implementation"/);
  assert.match(api, /COMMUNICATION_RECIPIENT_SUGGESTION_CAPABILITY_ID_V1/);
  assert.match(protocol, /COMMUNICATION_RECIPIENT_ROLE_ACCOUNTING_OR_BOOKKEEPING/);
  assert.match(protocol, /COMMUNICATION_RECIPIENT_ROLE_LEGAL_COUNSEL/);
  assert.match(protocol, /COMMUNICATION_RECIPIENT_ROLE_PROJECT_STAKEHOLDER/);
  assert.doesNotMatch(
    protocol,
    /email_address|contact_id|person_id|organization_id|provider_id|account_id|model_id|prompt|source_body|map</,
  );
  assert.match(core, /evaluate_communication_recipient_candidates_v1/);
  assert.match(core, /allows_empty_candidate_list_without_fabricating_a_recipient/);
  assert.match(core, /evaluates_accounting_signal_without_fabricating_other_roles/);
  assert.match(core, /evaluates_legal_signal_without_fabricating_other_roles/);
  assert.match(core, /evaluates_project_signal_without_fabricating_other_roles/);
  assert.match(core, /SourceDigestMismatch/);
  assert.doesNotMatch(
    core,
    /makosh_ai|ollama|communications_domain|communication_explanation|communication_reply_suggestion/,
  );
  assert.match(sourceManifest, /owner = "communications"/);
  assert.match(sourceManifest, /surface = "contract"/);
  assert.match(sourceApi, /communications\.recipient-source\.v1/);
  assert.match(sourceApi, /communication_recipient_suggestion\.source\.blob\.v1/);
  assert.match(sourceProtocol, /PrepareCommunicationRecipientSourceCommandV1/);
  assert.match(sourceProtocol, /CommunicationRecipientSourcePreparedV1/);
  assert.match(sourceProtocol, /CommunicationRecipientSourceRejectedV1/);
  assert.match(sourceEnvelope, /target_capability: COMMUNICATIONS_RECIPIENT_SOURCE_CAPABILITY_ID_V1/);
  assert.doesNotMatch(
    sourceProtocol,
    /email_address|contact_id|person_id|organization_id|provider_id|account_id|model_id|prompt|source_body|map</,
  );
  assert.match(persistenceManifest, /owner = "communication_recipient_suggestion"/);
  assert.match(persistenceManifest, /surface = "persistence"/);
  assert.match(persistenceSchema, /communication_recipient_suggestion_realtime/);
  assert.match(persistenceSchema, /evaluation_receipt_bytes/);
  assert.match(persistenceSchema, /candidate_bytes/);
  assert.match(persistenceModel, /encode_candidates/);
  assert.match(persistenceRepository, /persist_evaluation_transition/);
  assert.match(persistenceRealtime, /client_realtime_window/);
  for (const privateOrForeign of [
    'source_body',
    'email_address',
    'provider_id',
    'account_id',
    'model_id',
    'prompt',
    'communications_',
  ]) {
    assert.doesNotMatch(persistenceSchema, new RegExp(privateOrForeign));
  }
  assert.match(runtimeManifest, /owner = "communication_recipient_suggestion"/);
  assert.match(runtimeManifest, /surface = "runtime"/);
  assert.doesNotMatch(runtimeManifest, /makosh-ai-contracts|ollama|communication-explanation/);
  assert.match(runtimeAdmission, /ModuleKindV1::Workflow/);
  assert.match(runtimeAdmission, /ProvidedSurfaceKindV1::ClientRealtime/);
  assert.match(runtimeAdmission, /communication_recipient_source_prepared_consume_request_v1/);
  assert.doesNotMatch(runtimeAdmission, /inference|makosh_ai|ollama/i);
  assert.match(runtimeEvaluation, /evaluate_communication_recipient_candidates_v1/);
  assert.match(runtimeSourceResults, /receive_runtime_pull_delivery/);
  assert.match(runtimeSourceResults, /materialize_recipient_source_v1/);
  assert.match(runtimeSourceResults, /release_recipient_source_v1/);
  assert.match(runtimeManaged, /publish_pending/);
  assert.doesNotMatch(
    [runtimeAdmission, runtimeEvaluation, runtimeSourceResults, runtimeManaged].join('\n'),
    /communications-domain|makosh-communications-(?:core|persistence)|makosh_ai|ollama/i,
  );
  assert.match(communicationsRuntimeManifest, /makosh-communications-recipient-source-api/);
  assert.match(
    communicationsAdmission,
    /COMMUNICATIONS_RECIPIENT_SOURCE_BLOB_CAPABILITY_ID/,
  );
  assert.match(
    communicationsAdmission,
    /communication_recipient_source_prepare_consume_request_v1/,
  );
  assert.match(
    communicationsEventRuntime,
    /consume_next_recipient_source_prepare_v1/,
  );
  assert.match(communicationsRecipientSource, /write_target_bound_source/);
  assert.match(communicationsRecipientSource, /CommunicationRecipientBodySourceReceiptV1/);
  assert.match(communicationsRecipientSource, /persist_source_result/);
  assert.doesNotMatch(
    communicationsRecipientSource,
    /sender_utf8|subject_utf8|provider_id|account_id|email_address|makosh_ai|ollama/,
  );
  assert.match(communicationsSourceSnapshot, /CommunicationsSourceSnapshotV1/);
  assert.doesNotMatch(communicationsSourceSnapshot, /CommunicationsAiSource/);
  assert.match(workspace, /"src\/communication-recipient-suggestion-assembly"/);
  assert.match(assemblyManifest, /owner = "communication_recipient_suggestion"/);
  assert.match(assemblyManifest, /surface = "assembly"/);
  assert.match(assembly, /communication_recipient_suggestion_module_descriptor_v1/);
  assert.match(assembly, /communication_recipient_suggestion_storage_bundle_v1/);
  assert.match(assembly, /communication_recipient_suggestion\.release-artifacts\.json/);
  assert.doesNotMatch(assembly, /signing_key|private_key|makosh_provider|ollama/i);
  assert.match(releaseScript, /--package makosh-communication-recipient-suggestion-assembly/);
  assert.match(
    releaseScript,
    /communication_recipient_suggestion\.release-artifacts\.json/,
  );
  assert.match(managedSetup, /COMMUNICATION_RECIPIENT_SUGGESTION_RELEASE_ARTIFACT_ID_V1/);
  assert.match(managedSetup, /start_reserved_workflow/);
  assert.match(
    managedFlow,
    /managed_recipient_suggestion_reaches_gateway_sse_and_replays_after_restart/,
  );
  assert.match(managedFlow, /read_terminal_sse_event/);
  assert.match(managedFlow, /restart_communication_recipient_suggestion_runtime_v1/);
  assert.match(managedFlow, /assert_runtime_fences/);
  assert.match(managedFlow, /wrong_owner/);
  assert.match(managedFlow, /conflicting/);
  assert.match(managedFlow, /stale/);
  assert.match(managedFlow, /revoke_owner/);
  assert.match(managedFlow, /assert_private_content_absent/);
  assert.doesNotMatch(`${managedSetup}\n${managedFlow}`, /ollama|makosh_ai|communication_explanation/i);
  assert.match(
    managedScript,
    /managed_recipient_suggestion_reaches_gateway_sse_and_replays_after_restart/,
  );
  assert.ok(
    policy.implementation.ownerInventory.workflows.includes(
      'communication_recipient_suggestion',
    ),
  );
  assert.ok(
    policy.implementation.ownerInventory.businessCapabilities.includes(
      'communication.recipient-suggestion.v1',
    ),
  );
  for (const capability of [
    'communications.recipient-source.v1',
    'communications.recipient-source.blob.v1',
    'communication_recipient_suggestion.source.blob.v1',
    'communication_recipient_suggestion.source_prepare.v1',
    'communication_recipient_suggestion.source_prepared.v1',
    'communication_recipient_suggestion.source_rejected.v1',
    'communication_recipient_suggestion.storage.v1',
  ]) {
    assert.ok(policy.implementation.ownerInventory.businessCapabilities.includes(capability));
  }
});
