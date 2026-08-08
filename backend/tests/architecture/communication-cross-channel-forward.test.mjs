import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

test('cross-channel forward persistence is owner-local durable and bodyless', async () => {
  const [
    adr,
    inventorySource,
    policySource,
    apiManifest,
    coreManifest,
    api,
    core,
    contract,
    persistenceManifest,
    runtimeManifest,
    assemblyManifest,
    assemblyLib,
    runtimeAdmission,
    runtimeMain,
    managedRuntime,
    clientPort,
    clientRealtime,
    runtimeContracts,
    sourcePrepare,
    sourceResults,
    deliveryResults,
    blobTransfer,
    custodyCleanup,
    runtimeEventOutbox,
    migration,
    eventMigration,
    resultMigration,
    operations,
    workQueue,
    eventIo,
    eventOutbox,
    cleanup,
    realtime,
    schema,
    postgresConformance,
    storageRunner,
  ] = await Promise.all([
    readFile(
      new URL(
        'docs/adr/ADR-0346-cross-channel-communication-forward-workflow.md',
        PROJECT_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'architecture/communications-settings-reconstruction.json',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('architecture/policy.json', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL('src/communication-cross-channel-forward-api/Cargo.toml', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/communication-cross-channel-forward-core/Cargo.toml', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/communication-cross-channel-forward-api/src/lib.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/communication-cross-channel-forward-core/src/lib.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-cross-channel-forward-api/proto/makosh/communication_cross_channel_forward/v1/forward.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-cross-channel-forward-persistence/Cargo.toml',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('src/communication-cross-channel-forward-runtime/Cargo.toml', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/communication-cross-channel-forward-assembly/Cargo.toml', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/communication-cross-channel-forward-assembly/src/lib.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-cross-channel-forward-runtime/src/admission.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('src/communication-cross-channel-forward-runtime/src/main.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-cross-channel-forward-runtime/src/managed_runtime.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-cross-channel-forward-runtime/src/client_port.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-cross-channel-forward-runtime/src/client_realtime.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-cross-channel-forward-runtime/src/contracts.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-cross-channel-forward-runtime/src/source_prepare.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-cross-channel-forward-runtime/src/source_results.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-cross-channel-forward-runtime/src/delivery_results.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-cross-channel-forward-runtime/src/blob_transfer.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-cross-channel-forward-runtime/src/custody_cleanup.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-cross-channel-forward-runtime/src/event_outbox.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-cross-channel-forward-persistence/migrations/0001_forward_state.sql',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-cross-channel-forward-persistence/migrations/0002_event_handoff.sql',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-cross-channel-forward-persistence/migrations/0003_delivery_result_correlation.sql',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-cross-channel-forward-persistence/src/operations.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-cross-channel-forward-persistence/src/work_queue.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-cross-channel-forward-persistence/src/event_io.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-cross-channel-forward-persistence/src/event_outbox.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-cross-channel-forward-persistence/src/cleanup.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-cross-channel-forward-persistence/src/realtime.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-cross-channel-forward-persistence/src/schema.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'tests/support/communication-cross-channel-forward/tests/postgres_live.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('scripts/test-authenticated-storage.mjs', BACKEND_ROOT), 'utf8'),
  ]);
  const inventory = JSON.parse(inventorySource);
  const policy = JSON.parse(policySource);
  const gate = inventory.slices.find(
    ({ gate: name }) => name === 'communication_cross_channel_forward_v1',
  );

  assert.deepEqual(gate, {
    gate: 'communication_cross_channel_forward_v1',
    role: 'workflow',
    owner: 'communication_cross_channel_forward',
    state: 'implemented',
    dependsOn: [
      'communication_delivery_intent_v1',
      'communications_content_read_v1',
      'capability_routed_module_request_rpc_v1',
      'blob_v1',
    ],
  });
  assert.match(adr, /Caller не передаёт provider, account, provider chat locator или plaintext body/);
  assert.match(adr, /target-bound Blob delegation/);
  assert.match(
    adr,
    /не\s+импортирует provider contracts, SDK, runtime или\s+persistence/,
  );
  assert.match(adr, /Kernel[\s\S]*не декодирует source metadata, content или delivery payload/);
  assert.match(adr, /Core capability router[\s\S]*не содержит cross-channel business method/);
  assert.match(adr, /не хранит plaintext body/);
  assert.match(
    adr,
    /Live evidence закрывает exact gate `communication_cross_channel_forward_v1`/,
  );
  assert.match(
    adr,
    /общий `communications_settings_reconstruction_complete_v1` остаётся закрыт/,
  );
  assert.deepEqual(
    policy.implementation.productionPackages
      .filter(({ owner }) => owner === 'communication_cross_channel_forward')
      .map(({ name, surface }) => `${name}:${surface}`),
    [
      'makosh-communication-cross-channel-forward-api:contract',
      'makosh-communication-cross-channel-forward-core:implementation',
      'makosh-communication-cross-channel-forward-persistence:persistence',
      'makosh-communication-cross-channel-forward-runtime:runtime',
      'makosh-communication-cross-channel-forward-assembly:assembly',
    ],
  );
  assert.match(apiManifest, /role = "workflow"[\s\S]*surface = "contract"/);
  assert.match(coreManifest, /role = "workflow"[\s\S]*surface = "implementation"/);
  assert.match(
    persistenceManifest,
    /role = "workflow"[\s\S]*surface = "persistence"/,
  );
  assert.match(runtimeManifest, /role = "workflow"[\s\S]*surface = "runtime"/);
  assert.match(assemblyManifest, /role = "workflow"[\s\S]*surface = "assembly"/);
  assert.match(assemblyLib, /materialize_cross_channel_forward_release_assembly_v1/);
  assert.doesNotMatch(assemblyLib, /tokio|sqlx|async_nats|JetStreamClient/);
  assert.doesNotMatch(
    `${apiManifest}\n${coreManifest}\n${persistenceManifest}\n${runtimeManifest}\n${assemblyManifest}`,
    /makosh-(?:communications-domain|mail|telegram|whatsapp|zulip|kernel)/,
  );
  assert.match(api, /COMMUNICATION_CROSS_CHANNEL_FORWARD_CAPABILITY_ID_V1/);
  assert.match(core, /CrossChannelForwardTransitionV1/);
  assert.match(core, /RevisionExhausted/);
  assert.doesNotMatch(
    contract,
    /provider_id|account_id|body_utf8|blob_reference|\bAny\b|\bmap\s*</,
  );
  assert.match(migration, /communication_cross_channel_forward_operations/);
  assert.match(migration, /communication_cross_channel_forward_cleanup/);
  assert.match(migration, /communication_cross_channel_forward_realtime/);
  assert.match(migration, /attempt_count BETWEEN 0 AND 32/);
  assert.doesNotMatch(migration, /body_utf8|provider|mail_|telegram_|whatsapp_|zulip_/);
  assert.match(eventMigration, /communication_cross_channel_forward_event_inbox/);
  assert.match(eventMigration, /communication_cross_channel_forward_event_outbox/);
  assert.match(eventMigration, /exact_envelope_bytes/);
  assert.match(eventMigration, /source_result_message_id/);
  assert.match(eventMigration, /delivery_submit_message_id/);
  assert.match(resultMigration, /delivery_intent_command_id/);
  assert.match(resultMigration, /communication_cross_channel_forward_delivery_intent_idx/);
  assert.doesNotMatch(
    eventMigration,
    /body_utf8|provider|account_id|mail_|telegram_|whatsapp_|zulip_/,
  );
  assert.match(operations, /request_fingerprint/);
  assert.match(operations, /ON CONFLICT \(logical_owner_id, forward_id\) DO NOTHING/);
  assert.match(workQueue, /FOR UPDATE SKIP LOCKED/);
  assert.match(workQueue, /claim_epoch = operation\.claim_epoch \+ 1/);
  assert.match(workQueue, /LEAST\(attempt_count \+ 1, 32\)/);
  assert.match(eventIo, /persist_source_prepare_outbox/);
  assert.match(eventIo, /persist_source_prepared_and_delivery_submit/);
  assert.match(eventIo, /persist_source_rejected/);
  assert.match(eventIo, /ON CONFLICT DO NOTHING/);
  assert.match(eventIo, /envelope_sha256/);
  assert.match(eventIo, /insert_exact_outbox/);
  assert.match(eventIo, /insert_forward_transition/);
  assert.match(eventOutbox, /pending_event_outbox/);
  assert.match(eventOutbox, /mark_event_outbox_published/);
  assert.match(eventOutbox, /exact_envelope_bytes/);
  assert.match(eventOutbox, /ON CONFLICT DO NOTHING/);
  assert.doesNotMatch(
    `${eventIo}\n${eventOutbox}`,
    /body_utf8|provider_id|account_id|mail_|telegram_|whatsapp_|zulip_/,
  );
  assert.match(cleanup, /next_cleanup/);
  assert.match(cleanup, /reschedule_cleanup/);
  assert.match(realtime, /client_realtime_window/);
  assert.match(postgresConformance, /survives_reconnect/);
  assert.match(postgresConformance, /ClaimLost/);
  assert.match(storageRunner, /MAKOSH_COMMUNICATION_CROSS_CHANNEL_FORWARD_POSTGRES/);
  assert.match(schema, /COMMUNICATION_CROSS_CHANNEL_FORWARD_STORAGE_BUNDLE_REVISION_V3/);
  assert.match(runtimeAdmission, /ModuleKindV1::Workflow/);
  assert.match(runtimeAdmission, /max_processes: 1/);
  assert.match(runtimeAdmission, /ProvidedSurfaceKindV1::ClientRealtime/);
  assert.match(runtimeAdmission, /COMMUNICATION_CROSS_CHANNEL_FORWARD_COMMAND_CONNECT_PATH_V1/);
  assert.match(runtimeAdmission, /COMMUNICATION_CROSS_CHANNEL_FORWARD_QUERY_CONNECT_PATH_V1/);
  assert.match(runtimeAdmission, /communication_delivery_intent_submit_publish_request_v1/);
  assert.match(runtimeAdmission, /communication_delivery_intent_submitted_consume_request_v1/);
  assert.match(runtimeAdmission, /communication_delivery_intent_rejected_consume_request_v1/);
  assert.match(runtimeAdmission, /cross_channel_forward_source_prepared_consume_request_v1/);
  assert.match(runtimeMain, /Some\("serve-inherited"\)/);
  assert.match(managedRuntime, /request_managed_runtime_event_access_v2/);
  assert.match(managedRuntime, /bind_result_subscriptions/);
  assert.match(managedRuntime, /Operation::ClientDelivery/);
  assert.match(managedRuntime, /pump_client_realtime_once/);
  assert.match(managedRuntime, /signal_ready/);
  assert.match(clientPort, /start_cross_channel_forward_payload_v1/);
  assert.match(clientPort, /get_cross_channel_forward_status_payload_v1/);
  assert.match(clientRealtime, /ManagedRuntimeClientRealtimePublishRequestV1/);
  assert.match(clientRealtime, /client_realtime_window/);
  assert.match(runtimeContracts, /COMMUNICATION_CROSS_CHANNEL_FORWARD_SCHEMA_SHA256/);
  assert.match(sourcePrepare, /next_source_prepare_candidate/);
  assert.match(sourcePrepare, /persist_source_prepare_outbox/);
  assert.match(sourceResults, /decode_envelope_v1/);
  assert.match(sourceResults, /persist_source_prepared_and_delivery_submit/);
  assert.match(sourceResults, /delivery\.acknowledge\(\)/);
  assert.match(deliveryResults, /persist_delivery_submitted/);
  assert.match(deliveryResults, /persist_delivery_rejected/);
  assert.match(deliveryResults, /COMMUNICATION_DELIVERY_INTENT_BLOB_TARGET_MODULE_ID_V1/);
  assert.match(deliveryResults, /delivery\.acknowledge\(\)/);
  assert.match(blobTransfer, /BlobDataOperationReadRangeV1/);
  assert.match(blobTransfer, /BlobDataOperationWriteV1/);
  assert.match(blobTransfer, /COMMUNICATION_DELIVERY_INTENT_BLOB_TARGET_MODULE_ID_V1/);
  assert.match(custodyCleanup, /request_managed_blob_custody_release_v2/);
  assert.match(custodyCleanup, /reschedule_cleanup/);
  assert.match(runtimeEventOutbox, /publish_exact/);
  assert.match(runtimeEventOutbox, /mark_event_outbox_published/);
  assert.doesNotMatch(
    `${runtimeAdmission}\n${managedRuntime}\n${clientPort}\n${clientRealtime}\n${runtimeContracts}\n${sourcePrepare}\n${sourceResults}\n${deliveryResults}\n${blobTransfer}\n${custodyCleanup}\n${runtimeEventOutbox}`,
    /makosh-(?:mail|telegram|whatsapp|zulip)-(?:runtime|persistence|core|api)/,
  );
});

test('cross-channel source preparation is event-only and Communications-owned', async () => {
  const [
    sourceAdr,
    sourceManifest,
    sourceApi,
    sourceEnvelope,
    sourceContract,
    sourcePersistence,
    sourceRuntime,
    policySource,
  ] = await Promise.all([
    readFile(
      new URL(
        'docs/adr/ADR-0347-event-backed-cross-channel-forward-source-preparation.md',
        PROJECT_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communications-cross-channel-forward-source-api/Cargo.toml',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communications-cross-channel-forward-source-api/src/lib.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communications-cross-channel-forward-source-api/src/envelope.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communications-cross-channel-forward-source-api/proto/makosh/communications/cross_channel_forward_source/v1/cross_channel_forward_source.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communications-persistence/src/forward_source.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communications-runtime/src/cross_channel_forward_source.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('architecture/policy.json', BACKEND_ROOT), 'utf8'),
  ]);
  const policy = JSON.parse(policySource);

  assert.match(sourceAdr, /cross_channel_forward_source_prepare\.v1\s+command/);
  assert.match(sourceAdr, /cross_channel_forward_source_prepared\.v1\s+result/);
  assert.match(sourceAdr, /cross_channel_forward_source_rejected\.v1\s+result/);
  assert.match(sourceAdr, /communication_cross_channel_forward\.blob\.v1/);
  assert.match(sourceAdr, /Producer сохраняет exact `DurableEnvelopeV1` bytes/);
  assert.match(sourceAdr, /Consumer проверяет inbox message ID\/hash до mutation/);
  assert.match(sourceAdr, /не участвует в module-to-module source preparation/);
  assert.match(sourceAdr, /без direct RPC, cross-owner SQL или content leakage/);
  assert.doesNotMatch(sourceAdr, /generic provider facade|execute\(any\)/);
  assert.match(
    sourceManifest,
    /role = "domain"[\s\S]*owner = "communications"[\s\S]*surface = "contract"/,
  );
  assert.ok(
    policy.implementation.productionPackages.some(
      ({ name, owner, surface }) =>
        name === 'makosh-communications-cross-channel-forward-source-api'
        && owner === 'communications'
        && surface === 'contract',
    ),
  );
  assert.match(sourceApi, /CROSS_CHANNEL_FORWARD_SOURCE_BLOB_TARGET_OWNER_ID_V1/);
  assert.match(
    sourceApi,
    /"communication_cross_channel_forward\.blob\.v1"/,
  );
  assert.match(sourceEnvelope, /OutboxRecordV1/);
  assert.match(sourceEnvelope, /Semantics::Command/);
  assert.match(sourceEnvelope, /Semantics::Result/);
  assert.match(sourceContract, /PrepareCrossChannelForwardSourceCommandV1/);
  assert.match(sourceContract, /CrossChannelForwardBodySourceReceiptV1/);
  assert.match(sourcePersistence, /cross_channel_forward_source_snapshot/);
  assert.match(sourcePersistence, /persist_cross_channel_forward_source_result/);
  assert.match(sourcePersistence, /communications_event_inbox/);
  assert.match(sourcePersistence, /communications_domain_outbox/);
  assert.match(sourcePersistence, /ON CONFLICT \(message_id\) DO NOTHING/);
  assert.match(sourcePersistence, /1 \| 4 \| 6 => Ok\(CommunicationChannelKindV1::Mail\)/);
  assert.match(sourcePersistence, /InboxHashConflict/);
  assert.match(sourcePersistence, /StaleRevision/);
  assert.doesNotMatch(sourcePersistence, /pub (?:source|target)_provider/);
  assert.match(
    sourceRuntime,
    /consume_next_cross_channel_forward_source_prepare_v1/,
  );
  assert.match(sourceRuntime, /receive_runtime_pull_delivery/);
  assert.match(sourceRuntime, /request_managed_blob_session_v2/);
  assert.match(sourceRuntime, /BlobDataOperationReadRangeV1/);
  assert.match(sourceRuntime, /BlobDataOperationWriteV1/);
  assert.match(
    sourceRuntime,
    /CROSS_CHANNEL_FORWARD_SOURCE_BLOB_TARGET_OWNER_ID_V1/,
  );
  assert.match(
    sourceRuntime,
    /persist_cross_channel_forward_source_result[\s\S]*acknowledge\(\)/,
  );
  assert.doesNotMatch(
    `${sourceContract}\n${sourceApi}\n${sourceEnvelope}\n${sourcePersistence}\n${sourceRuntime}`,
    /provider_id|account_id|body_utf8|plaintext_body|arbitrary_target|\bAny\b|\bmap\s*</,
  );
});

test('delivery-intent workflow ingress is event-only and bodyless', async () => {
  const [
    ingressAdr,
    ingressManifest,
    ingressApi,
    ingressEnvelope,
    ingressContract,
    deliveryPersistenceManifest,
    deliveryIngressMigration,
    deliveryIngressPersistence,
    deliveryCleanupMigration,
    deliveryCleanupPersistence,
    deliveryRuntimeManifest,
    deliveryEventIngress,
    deliveryIngressResultOutbox,
    deliveryIngressCleanup,
    policySource,
  ] = await Promise.all([
    readFile(
      new URL(
        'docs/adr/ADR-0348-event-backed-delivery-intent-workflow-ingress.md',
        PROJECT_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delivery-intent-ingress-api/Cargo.toml',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delivery-intent-ingress-api/src/lib.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delivery-intent-ingress-api/src/envelope.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delivery-intent-ingress-api/proto/makosh/communication_delivery_intent/ingress/v1/delivery_intent_ingress.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delivery-intent-persistence/Cargo.toml',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delivery-intent-persistence/migrations/0004_event_ingress.sql',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delivery-intent-persistence/src/ingress_events.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delivery-intent-persistence/migrations/0005_event_ingress_cleanup.sql',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delivery-intent-persistence/src/ingress_cleanup.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delivery-intent-runtime/Cargo.toml',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delivery-intent-runtime/src/event_ingress.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delivery-intent-runtime/src/ingress_result_outbox.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delivery-intent-runtime/src/ingress_cleanup.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('architecture/policy.json', BACKEND_ROOT), 'utf8'),
  ]);
  const policy = JSON.parse(policySource);

  assert.match(ingressAdr, /makosh-communication-delivery-intent-ingress-api/);
  assert.match(ingressAdr, /communication_delivery_intent_submit\.v1\s+command/);
  assert.match(
    ingressAdr,
    /communication_delivery_intent_submitted\.v1\s+result/,
  );
  assert.match(
    ingressAdr,
    /communication_delivery_intent_rejected\.v1\s+result/,
  );
  assert.match(ingressAdr, /communication_delivery_intent\.blob\.v1/);
  assert.match(
    ingressAdr,
    /ACK разрешён только после commit\s+либо exact duplicate/,
  );
  assert.match(ingressAdr, /не участвует в module-to-module ingress/);
  assert.match(ingressAdr, /без direct RPC или cross-owner SQL/);
  assert.match(ingressAdr, /generic workflow command facade/);
  assert.doesNotMatch(ingressAdr, /execute\(any\)/);
  assert.match(
    ingressManifest,
    /role = "workflow"[\s\S]*owner = "communication_delivery_intent"[\s\S]*surface = "contract"/,
  );
  assert.equal(
    policy.implementation.currentSlice,
    'call_transcription_managed_conformance_v1',
  );
  assert.ok(
    policy.implementation.productionPackages.some(
      ({ name, owner, surface }) =>
        name === 'makosh-communication-delivery-intent-ingress-api'
        && owner === 'communication_delivery_intent'
        && surface === 'contract',
    ),
  );
  assert.match(
    ingressApi,
    /COMMUNICATION_DELIVERY_INTENT_BLOB_TARGET_OWNER_ID_V1/,
  );
  assert.match(
    deliveryPersistenceManifest,
    /owner = "communication_delivery_intent"[\s\S]*surface = "persistence"/,
  );
  assert.match(
    deliveryRuntimeManifest,
    /makosh-communication-delivery-intent-ingress-api/,
  );
  assert.match(
    deliveryIngressMigration,
    /communication_delivery_intent_ingress_inbox/,
  );
  assert.match(
    deliveryIngressMigration,
    /communication_delivery_intent_ingress_result_outbox/,
  );
  assert.match(deliveryIngressMigration, /exact_envelope_bytes/);
  assert.match(
    deliveryCleanupMigration,
    /communication_delivery_intent_ingress_cleanup/,
  );
  assert.match(deliveryCleanupMigration, /custody_source_proof/);
  assert.doesNotMatch(
    deliveryIngressMigration,
    /body_utf8|provider_id|account_id|mail_|telegram_|whatsapp_|zulip_/,
  );
  assert.match(deliveryIngressPersistence, /insert_or_fence_inbox/);
  assert.match(deliveryIngressPersistence, /create_intent_in_transaction/);
  assert.match(deliveryIngressPersistence, /insert_exact_result/);
  assert.match(deliveryIngressPersistence, /insert_cleanup/);
  assert.match(deliveryCleanupPersistence, /next_ingress_cleanup/);
  assert.match(deliveryCleanupPersistence, /reschedule_ingress_cleanup/);
  assert.match(deliveryEventIngress, /inspect_event_ingress/);
  assert.match(deliveryEventIngress, /read_delivery_intent_ingress_body_v1/);
  assert.match(deliveryEventIngress, /admit_event_ingress/);
  assert.match(
    deliveryEventIngress,
    /admit_event_ingress[\s\S]*delivery\.acknowledge\(\)/,
  );
  assert.match(deliveryIngressResultOutbox, /publish_exact/);
  assert.match(deliveryIngressResultOutbox, /mark_ingress_result_published/);
  assert.match(deliveryIngressCleanup, /request_managed_blob_custody_release_v2/);
  assert.match(deliveryIngressCleanup, /complete_ingress_cleanup/);
  assert.doesNotMatch(
    `${deliveryIngressPersistence}\n${deliveryEventIngress}\n${deliveryIngressResultOutbox}`,
    /makosh_communication_cross_channel_forward_(?:runtime|persistence|core)/,
  );
  assert.match(
    ingressApi,
    /"communication_delivery_intent\.blob\.v1"/,
  );
  assert.match(ingressEnvelope, /OutboxRecordV1/);
  assert.match(ingressEnvelope, /Semantics::Command/);
  assert.match(ingressEnvelope, /Semantics::Result/);
  assert.match(
    ingressEnvelope,
    /communication_delivery_intent_submit_message_id_v1/,
  );
  assert.match(ingressContract, /SubmitCommunicationDeliveryIntentCommandV1/);
  assert.match(ingressContract, /DeliveryIntentBodySourceReceiptV1/);
  assert.doesNotMatch(
    `${ingressContract}\n${ingressApi}\n${ingressEnvelope}`,
    /provider_id|account_id|body_utf8|plaintext_body|recipient|subject|arbitrary_target|\bAny\b|\bmap\s*</,
  );
});
