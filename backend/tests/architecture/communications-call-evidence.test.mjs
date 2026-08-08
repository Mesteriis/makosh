import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const REPOSITORY_ROOT = new URL('../../../', import.meta.url);

async function backendSource(path) {
  return readFile(new URL(path, BACKEND_ROOT), 'utf8');
}

test('call evidence contracts core and persistence are separate Communications domain units', async () => {
  const [apiManifest, ingressManifest, coreManifest, persistenceManifest, policySource] = await Promise.all([
    backendSource('src/communications-call-evidence-api/Cargo.toml'),
    backendSource('src/communications-call-evidence-ingress/Cargo.toml'),
    backendSource('src/communications-call-evidence-core/Cargo.toml'),
    backendSource('src/communications-call-evidence-persistence/Cargo.toml'),
    backendSource('architecture/policy.json'),
  ]);
  const policy = JSON.parse(policySource);

  for (const manifest of [apiManifest, ingressManifest, coreManifest]) {
    assert.match(manifest, /role = "domain"/);
    assert.match(manifest, /owner = "communications"/);
    assert.doesNotMatch(
      manifest,
      /telegram-(?:runtime|tdlib|calls)|whatsapp-(?:runtime|host)|zoom|sqlx|kernel|gateway/,
    );
  }
  assert.match(persistenceManifest, /role = "domain"/);
  assert.match(persistenceManifest, /owner = "communications"/);
  assert.match(ingressManifest, /surface = "contract"/);
  assert.match(coreManifest, /surface = "implementation"/);
  assert.match(persistenceManifest, /surface = "persistence"/);
  assert.match(apiManifest, /surface = "contract"/);
  assert.match(coreManifest, /makosh-communications-call-evidence-ingress/);
  assert.match(persistenceManifest, /makosh-communications-call-evidence-core/);
  assert.match(persistenceManifest, /makosh-storage-protocol/);
  assert.doesNotMatch(ingressManifest, /communications-call-evidence-core/);
  assert.doesNotMatch(persistenceManifest, /telegram|whatsapp|zulip|mail-/);

  assert.deepEqual(
    policy.implementation.productionPackages
      .filter(({ name }) => name.startsWith('makosh-communications-call-evidence-'))
      .map(({ name, role, owner, surface }) => `${name}:${role}:${owner}:${surface}`),
    [
      'makosh-communications-call-evidence-ingress:domain:communications:contract',
      'makosh-communications-call-evidence-core:domain:communications:implementation',
      'makosh-communications-call-evidence-persistence:domain:communications:persistence',
      'makosh-communications-call-evidence-api:domain:communications:contract',
    ],
  );
  assert.ok(
    policy.dependencies.integrationDomainContractPackages.includes(
      'makosh-communications-call-evidence-ingress',
    ),
  );
  assert.ok(
    !policy.dependencies.integrationDomainContractPackages.includes(
      'makosh-communications-call-evidence-core',
    ),
  );
  assert.ok(
    !policy.dependencies.integrationDomainContractPackages.includes(
      'makosh-communications-call-evidence-persistence',
    ),
  );
});

test('call evidence generated query and shared realtime are typed and client safe', async () => {
  const [proto, api, persistence, queryPort, realtime, admission, runtimeMain] =
    await Promise.all([
      backendSource(
        'src/communications-call-evidence-api/proto/makosh/communications/call_evidence/client/v1/client.proto',
      ),
      backendSource('src/communications-call-evidence-api/src/lib.rs'),
      backendSource('src/communications-call-evidence-persistence/src/repository.rs'),
      backendSource('src/communications-runtime/src/call_evidence_query_port.rs'),
      backendSource('src/communications-runtime/src/call_evidence_realtime.rs'),
      backendSource('src/communications-runtime/src/admission.rs'),
      backendSource('src/communications-runtime/src/main.rs'),
    ]);

  assert.match(proto, /service CallEvidenceQueryService/);
  assert.match(proto, /GetCallEvidenceRequestV1 get/);
  assert.match(proto, /ListCallEvidenceRequestV1 list/);
  assert.match(proto, /optional CallProviderV1 provider/);
  assert.match(proto, /message CallEvidenceChangedV1/);
  const protoWithoutComments = proto.replaceAll(/\/\/.*$/gm, '');
  assert.doesNotMatch(
    protoWithoutComments,
    /\b(?:source_call_cursor|account_cursor|conversation_cursor|participant_cursor|provider_call_id|phone_number|transcript|audio_bytes|credential|session_store)\b|map\s*<|google\.protobuf\.Any/,
  );
  assert.match(api, /CALL_EVIDENCE_QUERY_CONNECT_PATH_V1/);
  assert.match(api, /CALL_EVIDENCE_REALTIME_CONTRACT_NAME_V1/);
  assert.match(persistence, /pub async fn list\(/);
  assert.match(persistence, /ORDER BY canonical_revision DESC, call_evidence_id DESC/);
  assert.match(queryPort, /handle_call_evidence_query_v1/);
  assert.match(queryPort, /provider_filter/);
  assert.match(queryPort, /state_filter/);
  assert.match(realtime, /ManagedRuntimeClientRealtimePublishRequestV1/);
  assert.match(realtime, /communications-call-evidence\/\{\}/);
  assert.match(realtime, /last_sequence: Option<u64>/);
  assert.match(admission, /CALL_EVIDENCE_CLIENT_CAPABILITY_ID_V1/);
  assert.match(admission, /ProvidedSurfaceKindV1::ClientRealtime/);
  assert.match(runtimeMain, /publish_call_evidence_realtime\(\)/);
  assert.doesNotMatch(realtime, /source_call_cursor|account_cursor|provider_call_id|transcript/);
});

test('call evidence durable observation is exact typed and locator negative', async () => {
  const [proto, ingress, envelope] = await Promise.all([
    backendSource(
      'src/communications-call-evidence-ingress/proto/makosh/communications/call_evidence/v1/call_evidence.proto',
    ),
    backendSource('src/communications-call-evidence-ingress/src/lib.rs'),
    backendSource('src/communications-call-evidence-ingress/src/envelope.rs'),
  ]);

  assert.match(proto, /message CallEvidenceObservedV1/);
  assert.match(proto, /bytes call_evidence_id = 1/);
  assert.match(proto, /bytes source_call_cursor_sha256 = 2/);
  assert.match(proto, /uint64 source_revision = 11/);
  assert.match(proto, /CallTerminalDispositionV1 terminal_disposition/);
  const protoWithoutComments = proto.replaceAll(/\/\/.*$/gm, '');
  assert.doesNotMatch(
    protoWithoutComments,
    /\b(?:account_id|call_id|chat_id|provider_user_id|username|phone_number|encryption_key|signaling|pcm|audio_bytes|transcript|credential|session|raw_json|debug_log)\b/,
  );
  assert.doesNotMatch(protoWithoutComments, /\bgoogle\.protobuf\.Any\b|\bmap\s*</);

  assert.match(ingress, /CALL_EVIDENCE_OBSERVED_CONTRACT_NAME_V1.*call_evidence_observed/s);
  assert.match(ingress, /DurableEnvelopeKindV1::Observation/);
  assert.match(ingress, /EventSubscriptionRequirementV1::Required/);
  assert.match(envelope, /partition_key: call_evidence_id\.to_vec\(\)/);
  assert.match(envelope, /source_sequence: Some\(draft\.source_revision\)/);
  assert.match(envelope, /source_call_cursor_sha256/);
  assert.doesNotMatch(envelope, /payload[\s\S]{0,600}external_(?:account|call|conversation|participant)_id/);
});

test('call evidence core is monotonic terminal and provider behavior free', async () => {
  const core = await backendSource('src/communications-call-evidence-core/src/lib.rs');

  assert.match(core, /CallEvidenceApplyOutcomeV1::Duplicate/);
  assert.match(core, /CallEvidenceApplyOutcomeV1::Stale/);
  assert.match(core, /CallEvidenceCoreErrorV1::RevisionConflict/);
  assert.match(core, /CallEvidenceCoreErrorV1::TerminalConflict/);
  assert.match(core, /CallEvidenceCoreErrorV1::StateRegression/);
  assert.doesNotMatch(
    core,
    /createCall|acceptCall|discardCall|tgcalls|TDLib|WhatsAppHost|ZoomClient|provider command/,
  );
});

test('call evidence persistence is owner local atomic and private-content negative', async () => {
  const [manifest, repository, migration] = await Promise.all([
    backendSource('src/communications-call-evidence-persistence/Cargo.toml'),
    backendSource('src/communications-call-evidence-persistence/src/repository.rs'),
    backendSource(
      'src/communications-call-evidence-persistence/migrations/0001_call_evidence.sql',
    ),
  ]);

  assert.match(manifest, /makosh-communications-call-evidence-core/);
  assert.match(manifest, /makosh-storage-protocol/);
  assert.match(repository, /existing_inbox_outcome/);
  assert.match(repository, /InboxHashConflict/);
  assert.match(repository, /FOR UPDATE/);
  assert.match(repository, /transaction\.commit\(\)/);
  assert.match(repository, /next_realtime_sequence/);
  assert.match(migration, /communications_call_evidence_inbox/);
  assert.match(migration, /communications_call_evidence_projection/);
  assert.match(migration, /communications_call_evidence_history/);
  assert.match(migration, /communications_call_evidence_realtime_frames/);
  assert.doesNotMatch(
    repository,
    /\b(?:phone_number|raw_provider|provider_call_id|provider_account_id|pcm|audio_bytes|transcript|cookie|session_store|debug_log)\b/,
  );
  assert.doesNotMatch(
    migration,
    /\b(?:phone_number|username|raw_provider|provider_call_id|provider_account_id|pcm|audio_bytes|transcript|credential|cookie|session_store|debug_log)\b/,
  );
});

test('managed Communications consumer is exact fenced and acknowledges after persistence', async () => {
  const [runtimeManifest, admission, consumer, eventRuntime, runtimeBundle, assemblyManifest] =
    await Promise.all([
      backendSource('src/communications-runtime/Cargo.toml'),
      backendSource('src/communications-runtime/src/admission.rs'),
      backendSource('src/communications-runtime/src/call_evidence_consumer.rs'),
      backendSource('src/communications-runtime/src/event_runtime.rs'),
      backendSource('src/communications-runtime/src/storage_bundle.rs'),
      backendSource('src/communications-assembly/Cargo.toml'),
    ]);

  for (const dependency of [
    'makosh-communications-call-evidence-core',
    'makosh-communications-call-evidence-ingress',
    'makosh-communications-call-evidence-persistence',
  ]) {
    assert.match(runtimeManifest, new RegExp(dependency));
  }
  assert.match(admission, /communications\.call-evidence\.observe\.v1/);
  assert.match(admission, /call_evidence_observed_consume_request_v1/);
  assert.match(consumer, /FenceKindV1::RuntimeLease/);
  assert.match(consumer, /source_fence\.epoch != source\.runtime_generation/);
  assert.match(consumer, /metadata\.source_sequence != Some\(payload\.source_revision\)/);
  assert.match(consumer, /persistence[\s\S]*\.consume\(/);
  assert.match(
    consumer,
    /\.consume\([\s\S]*\.await[\s\S]*delivery\.acknowledge\(\)\.await/,
  );
  assert.match(eventRuntime, /CommunicationsConsumerV1::CallEvidence/);
  assert.match(eventRuntime, /call_evidence_persistence[\s\S]{0,80}\.verify_storage_ready/);
  assert.match(runtimeBundle, /append_communications_call_evidence_storage_v1/);
  assert.doesNotMatch(assemblyManifest, /makosh-communications-persistence/);
  const productionConsumer = consumer.replace(/#\[cfg\(test\)\][\s\S]*$/u, '');
  assert.doesNotMatch(
    productionConsumer,
    /telegram-(?:runtime|tdlib|calls)|whatsapp-(?:runtime|host)|provider_call_id|provider_account_id/,
  );
});

test('Telegram owns the call evidence producer and relays exact outbox bytes', async () => {
  const [
    persistenceManifest,
    runtimeManifest,
    mapper,
    repository,
    outbox,
    migration,
    admission,
    relay,
    process,
    assembly,
  ] = await Promise.all([
    backendSource('src/telegram-calls-persistence/Cargo.toml'),
    backendSource('src/telegram-runtime/Cargo.toml'),
    backendSource('src/telegram-calls-persistence/src/call_evidence.rs'),
    backendSource('src/telegram-calls-persistence/src/repository.rs'),
    backendSource('src/telegram-calls-persistence/src/call_evidence_outbox.rs'),
    backendSource('src/telegram-calls-persistence/src/schema.rs'),
    backendSource('src/telegram-runtime/src/admission.rs'),
    backendSource('src/telegram-runtime/src/call_evidence_outbox.rs'),
    backendSource('src/telegram-runtime/src/process.rs'),
    backendSource('src/telegram-assembly/src/lib.rs'),
  ]);

  for (const manifest of [persistenceManifest, runtimeManifest]) {
    assert.match(manifest, /makosh-communications-call-evidence-ingress/);
    assert.doesNotMatch(
      manifest,
      /makosh-communications-(?:call-evidence-(?:core|persistence)|runtime|assembly)/,
    );
  }

  const productionMapper = mapper.replace(/#\[cfg\(test\)\][\s\S]*$/u, '');
  assert.match(productionMapper, /build_call_evidence_observed_outbox_record_v1/);
  assert.match(productionMapper, /external_account_id: session\.account_id\.clone\(\)/);
  assert.match(productionMapper, /external_call_id: session\.call_session_id\.clone\(\)/);
  assert.doesNotMatch(
    productionMapper,
    /(?:encode_to_vec|payload:|DurableEnvelopeV1)/,
  );

  assert.match(repository, /ingest_provider_update_with_call_evidence/);
  assert.match(
    repository,
    /persist_history[\s\S]*insert_call_evidence_outbox[\s\S]*transaction\s*\.commit\(\)/,
  );
  assert.match(outbox, /ON CONFLICT \(message_id\) DO NOTHING/);
  assert.match(outbox, /existing_hash[\s\S]*existing_bytes/);
  assert.match(outbox, /TelegramCallsPersistenceError::IdempotencyConflict/);
  assert.match(migration, /telegram_call_evidence_outbox/);
  assert.match(migration, /exact_envelope_bytes BYTEA NOT NULL/);
  assert.match(assembly, /telegram_storage_bundle_with_call_evidence_v9/);

  assert.match(admission, /telegram\.call-evidence\.publish\.v1/);
  assert.match(admission, /call_evidence_observed_publish_request_v1/);
  assert.match(relay, /RuntimeOutboxPublisherV1/);
  assert.match(relay, /relay_once\(&mut store, &publisher\)/);
  assert.match(
    relay,
    /Err\(_\) => return Err\(TelegramCallEvidenceOutboxRelayErrorV1::Unavailable\)/,
  );
  assert.match(process, /relay_call_evidence_outbox_once_v1/);
  assert.match(
    process,
    /Ok\(_\) \| Err\(TelegramCallEvidenceOutboxRelayErrorV1::Unavailable\) => \{\}/,
  );
});

test('call evidence gate has live managed outage, SSE and restart evidence', async () => {
  const [adr, ownerAdr, managedFlow, integrationProtocol, domainProtocol, inventorySource] = await Promise.all([
    readFile(
      new URL(
        'docs/adr/ADR-0349-event-backed-communications-call-evidence.md',
        REPOSITORY_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'docs/adr/ADR-0350-explicit-human-owner-context-for-managed-domain-and-integration-runtimes.md',
        REPOSITORY_ROOT,
      ),
      'utf8',
    ),
    backendSource(
      'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/call_evidence_managed_flow.rs',
    ),
    backendSource(
      'src/platform/runtime_protocol/proto/makosh/runtime/v1/managed_integration_runtime.proto',
    ),
    backendSource(
      'src/platform/runtime_protocol/proto/makosh/runtime/v1/managed_domain_runtime.proto',
    ),
    backendSource('architecture/communications-settings-reconstruction.json'),
  ]);
  const inventory = JSON.parse(inventorySource);
  const gate = inventory.slices.find(
    ({ gate: name }) => name === 'communications_call_evidence_v1',
  );

  assert.deepEqual(gate, {
    gate: 'communications_call_evidence_v1',
    role: 'domain',
    owner: 'communications',
    state: 'implemented',
    dependsOn: ['communications_canonical_read_v2'],
  });
  assert.match(adr, /Integration не импортирует Communications implementation или persistence/);
  assert.match(adr, /Communications не импортирует integration API, runtime, SDK или storage/);
  assert.match(adr, /ADR и static package presence сами по\s+себе gate не открывают/);
  assert.match(adr, /live managed proof from integration outbox through NATS/);
  assert.match(ownerAdr, /module owner никогда не используется как fallback human owner/);
  assert.match(integrationProtocol, /string logical_human_owner_id = 14/);
  assert.match(domainProtocol, /string logical_human_owner_id = 10/);
  assert.match(managedFlow, /set_authenticated_nats_container_running\(false\)/);
  assert.match(managedFlow, /wait_for_pending_call_evidence/);
  assert.match(managedFlow, /CALL_EVIDENCE_QUERY_CONNECT_PATH_V1/);
  assert.match(managedFlow, /\/api\/realtime\/v1\/events/);
  assert.match(managedFlow, /restart_communications_domain/);
  assert.match(managedFlow, /assert_call_evidence_event_is_client_safe/);
});
