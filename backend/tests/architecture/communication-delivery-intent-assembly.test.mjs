import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

const paths = {
  policy: new URL('architecture/policy.json', BACKEND_ROOT),
  reconstruction: new URL(
    'architecture/communications-settings-reconstruction.json',
    BACKEND_ROOT,
  ),
  apiManifest: new URL('src/communication-delivery-intent-api/Cargo.toml', BACKEND_ROOT),
  coreManifest: new URL('src/communication-delivery-intent-core/Cargo.toml', BACKEND_ROOT),
  persistenceManifest: new URL(
    'src/communication-delivery-intent-persistence/Cargo.toml',
    BACKEND_ROOT,
  ),
  runtimeManifest: new URL(
    'src/communication-delivery-intent-runtime/Cargo.toml',
    BACKEND_ROOT,
  ),
  assemblyManifest: new URL(
    'src/communication-delivery-intent-assembly/Cargo.toml',
    BACKEND_ROOT,
  ),
  assembly: new URL(
    'src/communication-delivery-intent-assembly/src/lib.rs',
    BACKEND_ROOT,
  ),
  runtimeAdmission: new URL(
    'src/communication-delivery-intent-runtime/src/admission.rs',
    BACKEND_ROOT,
  ),
  providerEventAdmission: new URL(
    'src/communication-delivery-intent-runtime/src/provider_event_admission.rs',
    BACKEND_ROOT,
  ),
  eventRuntime: new URL(
    'src/communication-delivery-intent-runtime/src/event_runtime.rs',
    BACKEND_ROOT,
  ),
  runtimeCoordinator: new URL(
    'src/communication-delivery-intent-runtime/src/coordinator.rs',
    BACKEND_ROOT,
  ),
  clientPort: new URL(
    'src/communication-delivery-intent-runtime/src/client_port.rs',
    BACKEND_ROOT,
  ),
  submitPort: new URL(
    'src/communication-delivery-intent-runtime/src/submit_port.rs',
    BACKEND_ROOT,
  ),
  communicationsQueryClient: new URL(
    'src/communication-delivery-intent-runtime/src/communications_query_client.rs',
    BACKEND_ROOT,
  ),
  runtimeProcess: new URL(
    'src/communication-delivery-intent-runtime/src/runtime.rs',
    BACKEND_ROOT,
  ),
  persistence: new URL(
    'src/communication-delivery-intent-persistence/src/intents.rs',
    BACKEND_ROOT,
  ),
  migration: new URL(
    'src/communication-delivery-intent-persistence/migrations/0001_delivery_intent_state.sql',
    BACKEND_ROOT,
  ),
  contract: new URL(
    'src/communication-delivery-intent-api/proto/makosh/communication_delivery_intent/v1/delivery.proto',
    BACKEND_ROOT,
  ),
  adr: new URL(
    'docs/adr/ADR-0330-provider-neutral-communication-delivery-intent-workflow.md',
    PROJECT_ROOT,
  ),
};

test('delivery intent assembly is an exact managed event workflow slice', async () => {
  const [
    policySource,
    reconstructionSource,
    apiManifest,
    coreManifest,
    persistenceManifest,
    runtimeManifest,
    assemblyManifest,
    assembly,
    runtimeAdmission,
    providerEventAdmission,
    eventRuntime,
    runtimeCoordinator,
    clientPort,
    submitPort,
    communicationsQueryClient,
    runtimeProcess,
    persistence,
    migration,
    contract,
    adr,
  ] =
    await Promise.all([
      readFile(paths.policy, 'utf8'),
      readFile(paths.reconstruction, 'utf8'),
      readFile(paths.apiManifest, 'utf8'),
      readFile(paths.coreManifest, 'utf8'),
      readFile(paths.persistenceManifest, 'utf8'),
      readFile(paths.runtimeManifest, 'utf8'),
      readFile(paths.assemblyManifest, 'utf8'),
      readFile(paths.assembly, 'utf8'),
      readFile(paths.runtimeAdmission, 'utf8'),
      readFile(paths.providerEventAdmission, 'utf8'),
      readFile(paths.eventRuntime, 'utf8'),
      readFile(paths.runtimeCoordinator, 'utf8'),
      readFile(paths.clientPort, 'utf8'),
      readFile(paths.submitPort, 'utf8'),
      readFile(paths.communicationsQueryClient, 'utf8'),
      readFile(paths.runtimeProcess, 'utf8'),
      readFile(paths.persistence, 'utf8'),
      readFile(paths.migration, 'utf8'),
      readFile(paths.contract, 'utf8'),
      readFile(paths.adr, 'utf8'),
    ]);
  const policy = JSON.parse(policySource);
  const reconstruction = JSON.parse(reconstructionSource);
  const deliverySlice = reconstruction.slices.find(
    ({ gate }) => gate === 'communication_delivery_intent_v1',
  );
  const exportSlice = reconstruction.slices.find(
    ({ gate }) => gate === 'communications_export_v1',
  );

  assert.equal(
    policy.implementation.currentSlice,
    'speech_to_text_whisper_admission_v1',
  );
  assert.deepEqual(policy.implementation.ownerInventory.workflows, [
    'attachment_preview',
    'attachment_preview_evidence_replay',
    'attachment_text_extraction',
    'attachment_translation',
    'call_transcription',
    'communication_bulk_action',
    'communication_cross_channel_forward',
    'communication_delayed_delivery',
    'communication_delivery_intent',
    'communication_explanation',
    'communication_note_candidate_extraction',
    'communication_recipient_suggestion',
    'communication_reply_suggestion',
    'communication_summary',
    'communication_task_candidate_extraction',
    'communication_translation',
    'communications_export',
    'mail_persons_sync',
    'reviewed_note_candidate_promotion',
    'reviewed_person_match_candidate_promotion',
    'reviewed_task_candidate_promotion',
  ]);
  assert.deepEqual(
    policy.implementation.productionPackages
      .filter(({ owner }) => owner === 'communication_delivery_intent')
      .map(({ name, surface }) => `${name}:${surface}`),
    [
      'makosh-communication-delivery-intent-api:contract',
      'makosh-communication-delivery-intent-core:implementation',
      'makosh-communication-delivery-intent-persistence:persistence',
      'makosh-communication-delivery-intent-runtime:runtime',
      'makosh-communication-delivery-intent-assembly:assembly',
      'makosh-communication-delivery-intent-event-adapters:implementation',
      'makosh-communication-delivery-intent-ingress-api:contract',
    ],
  );
  assert.equal(deliverySlice?.state, 'implemented');
  assert.equal(exportSlice?.state, 'implemented');
  assert.match(apiManifest, /role = "workflow"[\s\S]*surface = "contract"/);
  assert.match(coreManifest, /role = "workflow"[\s\S]*surface = "implementation"/);
  assert.match(
    persistenceManifest,
    /role = "workflow"[\s\S]*surface = "persistence"/,
  );
  assert.match(runtimeManifest, /role = "workflow"[\s\S]*surface = "runtime"/);
  assert.match(assemblyManifest, /role = "workflow"[\s\S]*surface = "assembly"/);
  assert.match(coreManifest, /makosh-communications-api/);
  assert.doesNotMatch(
    `${apiManifest}\n${coreManifest}\n${persistenceManifest}\n${assemblyManifest}`,
    /makosh-(?:mail|telegram|whatsapp|zulip|communications-domain|communications-persistence)/,
  );
  assert.doesNotMatch(
    runtimeManifest,
    /makosh-(?:mail|telegram|whatsapp|zulip)-(?:runtime|persistence)/,
  );
  assert.match(assembly, /validate_descriptor_v1/);
  assert.match(assembly, /validate_storage_bundle/);
  assert.match(assembly, /create_new\(true\)/);
  assert.doesNotMatch(assembly, /ClientRpcRouteV1|async_nats|sqlx|provider command/);
  assert.match(runtimeAdmission, /StorageNamespaceRequestV1/);
  assert.match(runtimeAdmission, /BlobQuotaRequestV1/);
  assert.match(runtimeAdmission, /BlobQuotaOperationV1::Write/);
  assert.match(runtimeAdmission, /ClientRpcRouteV1/);
  assert.match(
    runtimeAdmission,
    /COMMUNICATION_DELIVERY_INTENT_COMMAND_CONNECT_PATH_V1/,
  );
  assert.match(
    runtimeAdmission,
    /COMMUNICATION_DELIVERY_INTENT_QUERY_CONNECT_PATH_V1/,
  );
  assert.match(providerEventAdmission, /EventRouteDirectionV1::Publish/);
  assert.match(providerEventAdmission, /EventRouteDirectionV1::Consume/);
  assert.match(eventRuntime, /publish_exact/);
  assert.match(eventRuntime, /acknowledge\(\)/);
  assert.match(runtimeCoordinator, /DeliveryIntentBodyMaterializerV1/);
  assert.match(runtimeCoordinator, /DeliveryIntentBodyBlobReceiptV1/);
  assert.match(clientPort, /submit_delivery_intent_payload_v1/);
  assert.match(submitPort, /SubmitDeliveryIntentRequestV1/);
  assert.match(clientPort, /GetDeliveryIntentStatusRequestV1/);
  assert.doesNotMatch(`${clientPort}\n${submitPort}`, /provider_id|account_id/);
  assert.match(communicationsQueryClient, /RouteModuleQuery/);
  assert.match(communicationsQueryClient, /COMMUNICATIONS_QUERY_SCHEMA_SHA256/);
  assert.match(runtimeProcess, /describe_managed_runtime/);
  assert.match(runtimeProcess, /signal_ready/);
  assert.match(runtimeProcess, /dispatch_delivery_intent_client_request_v1/);
  assert.match(runtimeProcess, /StorageVaultLeaseAdapterV1/);
  assert.doesNotMatch(persistence, /PlannedDeliveryIntentV1|pub body_utf8/);
  assert.match(persistence, /DeliveryIntentBodyBlobReceiptV1/);
  assert.match(persistence, /ON CONFLICT \(logical_owner_id, intent_id\)/);
  assert.match(
    persistence,
    /jobs\.logical_owner_id = candidate\.logical_owner_id/,
  );
  assert.match(migration, /PRIMARY KEY \(logical_owner_id, intent_id\)/);
  assert.match(migration, /body_reference_id/);
  assert.match(migration, /body_custody_source_proof/);
  assert.doesNotMatch(migration, /body_ciphertext|body_nonce|body_key_epoch/);
  assert.doesNotMatch(migration, /body_utf8|communications_messages|mail_|telegram_/);
  assert.match(contract, /bytes conversation_id/);
  assert.match(contract, /optional bytes reply_to_message_id/);
  assert.doesNotMatch(contract, /\b(?:map|Any|provider_id|account_id)\b/);
  assert.match(adr, /Kernel[\s\S]*не декодирует request body/);
  assert.match(adr, /Persistence unit не принимает `PlannedDeliveryIntentV1`/);
  assert.match(adr, /command и status ClientRpc closure реализованы/);
  assert.match(adr, /Live disposable managed contour/);
  assert.match(adr, /`communication_delivery_intent_v1` реализован/);
});
