import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const REPOSITORY_ROOT = new URL('../', BACKEND_ROOT);

test('archive inspection production gate is implemented as an exact engine inventory', async () => {
  const [inventorySource, policySource, adr] = await Promise.all([
    readFile(
      new URL('architecture/communications-settings-reconstruction.json', BACKEND_ROOT),
      'utf8',
    ),
    readFile(new URL('architecture/policy.json', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL(
        'docs/adr/ADR-0359-bounded-attachment-archive-inspection-engine.md',
        REPOSITORY_ROOT,
      ),
      'utf8',
    ),
  ]);
  const inventory = JSON.parse(inventorySource);
  const policy = JSON.parse(policySource);
  const slice = inventory.slices.find(
    ({ gate }) => gate === 'attachment_archive_inspection_v1',
  );

  assert.deepEqual(slice, {
    gate: 'attachment_archive_inspection_v1',
    role: 'engine',
    owner: 'attachment_archive_inspection',
    state: 'implemented',
    dependsOn: ['blob_v1', 'attachment_security_engine_v1'],
  });
  assert.equal(
    policy.implementation.currentSlice,
    'call_transcription_managed_conformance_v1',
  );
  assert(policy.implementation.ownerInventory.engines.includes(
    'attachment_archive_inspection',
  ));
  assert(policy.implementation.productionPackages.some(
    ({ name, role, owner, surface }) =>
      name === 'makosh-attachment-archive-inspection-persistence'
      && role === 'engine'
      && owner === 'attachment_archive_inspection'
      && surface === 'persistence',
  ));
  assert.match(adr, /До выполнения всех пунктов inventory state остаётся `planned`/);
  assert.match(adr, /не распаковывает entry bytes/);
  assert.match(adr, /не изменяет safety lifecycle/);
});

test('archive inspection live evidence keeps module authority distinct from human tenancy', async () => {
  const [protocol, validation, kernelDispatch, runtime, setup, flow, delegation, ownerAdr] =
    await Promise.all([
      readFile(
        new URL(
          'src/platform/runtime_protocol/proto/makosh/runtime/v1/managed_engine_runtime.proto',
          BACKEND_ROOT,
        ),
        'utf8',
      ),
      readFile(
        new URL(
          'src/platform/runtime_protocol/src/validation/managed_engine_runtime.rs',
          BACKEND_ROOT,
        ),
        'utf8',
      ),
      readFile(
        new URL('src/kernel/src/identity/owner_control/dispatch.rs', BACKEND_ROOT),
        'utf8',
      ),
      readFile(
        new URL('src/attachment-archive-inspection-runtime/src/runtime.rs', BACKEND_ROOT),
        'utf8',
      ),
      readFile(
        new URL(
          'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/archive_inspection_managed_setup.rs',
          BACKEND_ROOT,
        ),
        'utf8',
      ),
      readFile(
        new URL(
          'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/archive_inspection_managed_flow.rs',
          BACKEND_ROOT,
        ),
        'utf8',
      ),
      readFile(
        new URL('src/attachment-security-persistence/src/delegation.rs', BACKEND_ROOT),
        'utf8',
      ),
      readFile(
        new URL(
          'docs/adr/ADR-0361-explicit-human-owner-context-for-managed-engine-runtimes.md',
          REPOSITORY_ROOT,
        ),
        'utf8',
      ),
    ]);

  assert.match(protocol, /string logical_human_owner_id = 11/);
  assert.match(validation, /valid_identifier\(&configuration\.logical_human_owner_id\)/);
  assert.match(kernelDispatch, /logical_human_owner_id: logical_human_owner\.owner_id\(\)\.to_owned\(\)/);
  assert.match(runtime, /module_owner_id: String/);
  assert.match(runtime, /logical_human_owner_id: String/);
  assert.match(setup, /logical_owner_id: ATTACHMENT_ARCHIVE_INSPECTION_OWNER_V1\.to_owned\(\)/);
  assert.match(setup, /logical_human_owner_id: ARCHIVE_INSPECTION_LOGICAL_OWNER_ID_V1\.to_owned\(\)/);
  assert.match(flow, /set_authenticated_nats_container_running\(false\)/);
  assert.match(flow, /restart_archive_inspection_runtime_v1\(/);
  assert.match(flow, /restart and replay must not transfer Blob custody or execute the parser twice/);
  assert.match(flow, /assert_private_archive_data_absent/);
  assert.match(delegation, /payload\.evidence_id == request\.safety_evidence_id/);
  assert.doesNotMatch(delegation, /envelope\.message_id == request\.safety_message_id/);
  assert.match(ownerAdr, /Состояние реализации: реализовано/);
});

test('archive inspection API is bounded and carries no Blob or provider authority', async () => {
  const [manifest, proto, source] = await Promise.all([
    readFile(
      new URL('src/attachment-archive-inspection-api/Cargo.toml', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL(
        'src/attachment-archive-inspection-api/proto/makosh/attachment_archive_inspection/v1/archive_inspection.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-archive-inspection-api/src/lib.rs', BACKEND_ROOT),
      'utf8',
    ),
  ]);

  assert.match(manifest, /role = "engine"/);
  assert.match(manifest, /owner = "attachment_archive_inspection"/);
  assert.match(manifest, /surface = "contract"/);
  assert.doesNotMatch(manifest, /makosh-(?:communications|attachment-security|blob|kernel)/);
  assert.match(proto, /bytes operation_id = 2/);
  assert.match(proto, /bytes attachment_anchor_id = 3/);
  assert.match(proto, /repeated ArchiveEntryV1 entries = 4/);
  assert.doesNotMatch(
    proto,
    /\b(?:blob_reference|provider|account_id|filesystem|source_bytes|map)\b/,
  );
  assert.match(source, /MAX_REPORT_ENTRIES_V1: usize = 1_000/);
  assert.match(source, /MAX_PATH_BYTES_V1: usize = 1_024/);
});

test('archive ingress is a target-owned event contract without engine implementation coupling', async () => {
  const [manifest, proto, source] = await Promise.all([
    readFile(
      new URL('src/attachment-archive-inspection-ingress/Cargo.toml', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL(
        'src/attachment-archive-inspection-ingress/proto/makosh/attachment_archive_inspection/ingress/v1/custody_delegation.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-archive-inspection-ingress/src/lib.rs', BACKEND_ROOT),
      'utf8',
    ),
  ]);

  assert.match(manifest, /role = "engine"/);
  assert.match(manifest, /owner = "attachment_archive_inspection"/);
  assert.match(manifest, /surface = "contract"/);
  assert.match(manifest, /makosh-events-protocol/);
  assert.match(manifest, /makosh-runtime-protocol/);
  assert.doesNotMatch(
    manifest,
    /makosh-(?:attachment-security|attachment-archive-inspection-(?:api|core|persistence|runtime|assembly)|communications|blob|kernel)/,
  );
  assert.match(proto, /message RequestArchiveInspectionCustodyDelegationV1/);
  assert.match(proto, /bytes candidate_envelope_sha256 = 5/);
  assert.match(proto, /message ArchiveInspectionCustodyDelegatedV1/);
  assert.match(proto, /bytes custody_transfer_source_proof = 9/);
  const request = proto.slice(
    proto.indexOf('message RequestArchiveInspectionCustodyDelegationV1'),
    proto.indexOf('message ArchiveInspectionCustodyDelegatedV1'),
  );
  assert.doesNotMatch(
    request,
    /\b(?:blob_reference|custody_transfer_source_proof|provider_id|account_id|filesystem_path|target_owner_id|target_module_id|target_capability_id)\b/,
  );
  assert.match(source, /ATTACHMENT_SECURITY_ARCHIVE_DELEGATION_CAPABILITY_ID_V1/);
  assert.match(source, /ATTACHMENT_ARCHIVE_INSPECTION_BLOB_TARGET_OWNER_ID_V1/);
  assert.match(source, /DurableEnvelopeKindV1::Command/);
  assert.match(source, /DurableEnvelopeKindV1::Result/);
});

test('pure archive core owns policy without transport, storage or parser dependency', async () => {
  const [manifest, source, join] = await Promise.all([
    readFile(
      new URL('src/attachment-archive-inspection-core/Cargo.toml', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-archive-inspection-core/src/lib.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-archive-inspection-core/src/join.rs', BACKEND_ROOT),
      'utf8',
    ),
  ]);

  assert.match(manifest, /makosh-attachment-archive-inspection-api/);
  assert.doesNotMatch(
    manifest,
    /makosh-(?:communications|attachment-security|blob|events|runtime|storage|kernel)|\bzip\s*=/,
  );
  assert.match(source, /ArchiveInspectionLimitsV1/);
  assert.match(source, /DuplicateEntryPath/);
  assert.match(source, /EncryptedEntry/);
  assert.match(source, /NestedArchive/);
  assert.match(source, /UnsupportedEntryType/);
  assert.match(join, /ArchiveInspectionCustodyDelegationIntentV1/);
  const intent = join.slice(
    join.indexOf('struct ArchiveInspectionCustodyDelegationIntentV1'),
    join.indexOf('enum ArchiveInspectionRejectionV1'),
  );
  assert.doesNotMatch(
    intent,
    /\b(?:blob_reference_id|declared_size|receipt_sha256|custody_transfer_source_proof)\b/,
  );
  assert.doesNotMatch(
    source,
    /TcpStream|File::|sqlx|postgres|nats|jetstream|makosh_communications|makosh_attachment_security/,
  );
});

test('archive persistence owns replay, event join and fenced jobs without foreign implementations', async () => {
  const [manifest, schema, library, observations, custody, jobs] = await Promise.all([
    readFile(
      new URL('src/attachment-archive-inspection-persistence/Cargo.toml', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL(
        'src/attachment-archive-inspection-persistence/migrations/0001_archive_inspection.sql',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-archive-inspection-persistence/src/lib.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL(
        'src/attachment-archive-inspection-persistence/src/observations.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-archive-inspection-persistence/src/custody.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-archive-inspection-persistence/src/jobs.rs', BACKEND_ROOT),
      'utf8',
    ),
  ]);

  assert.match(manifest, /role = "engine"/);
  assert.match(manifest, /owner = "attachment_archive_inspection"/);
  assert.match(manifest, /surface = "persistence"/);
  assert.match(manifest, /makosh-attachment-archive-inspection-core/);
  assert.match(manifest, /makosh-attachment-archive-inspection-ingress/);
  assert.match(manifest, /makosh-events-protocol/);
  assert.match(manifest, /makosh-storage-protocol/);
  assert.doesNotMatch(
    manifest,
    /makosh-(?:communications|attachment-security|blob|kernel|mail|telegram|whatsapp|zulip)/,
  );
  for (const table of [
    'attachment_archive_inspection_runs',
    'attachment_archive_inspection_event_inbox',
    'attachment_archive_inspection_scan_candidates',
    'attachment_archive_inspection_safety_facts',
    'attachment_archive_inspection_custody_delegation_requests',
    'attachment_archive_inspection_custody_result_inbox',
    'attachment_archive_inspection_jobs',
    'attachment_archive_inspection_reports',
    'attachment_archive_inspection_realtime',
  ]) {
    assert.match(schema, new RegExp(table));
  }
  assert.match(schema, /runtime_generation BIGINT/);
  assert.match(schema, /grant_epoch BIGINT/);
  assert.match(schema, /lease_fence BIGINT/);
  assert.doesNotMatch(
    schema,
    /\b(?:provider_id|provider_path|message_body|archive_bytes|extracted_content)\b/,
  );
  assert.match(library, /verify_storage_ready/);
  assert.match(observations, /persist_scan_candidate/);
  assert.match(observations, /persist_canonical_safety_fact/);
  assert.match(observations, /settle_anchor_runs/);
  assert.match(library, /PendingArchiveInspectionCustodyDelegationV1/);
  assert.match(custody, /pending_custody_delegation_requests/);
  assert.match(custody, /store_custody_delegation_outbox/);
  assert.match(custody, /persist_custody_delegated_result/);
  assert.match(custody, /persist_custody_delegation_rejected_result/);
  assert.match(custody, /insert_result_inbox/);
  assert.match(jobs, /claim_next_job/);
  assert.match(jobs, /recover_expired_jobs/);
  assert.match(jobs, /verify_claim/);
  assert.doesNotMatch(
    `${library}\n${observations}\n${custody}\n${jobs}`,
    /makosh_(?:communications|attachment_security|blob|kernel|mail|telegram|whatsapp|zulip)/,
  );
});

test('archive runtime is a separate managed engine with event-only custody and receipt-bound Blob reads', async () => {
  const [manifest, admission, runtime, blob, eventDecode, outbox, settings] = await Promise.all([
    readFile(
      new URL('src/attachment-archive-inspection-runtime/Cargo.toml', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-archive-inspection-runtime/src/admission.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-archive-inspection-runtime/src/runtime.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-archive-inspection-runtime/src/blob.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-archive-inspection-runtime/src/event_decode.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-archive-inspection-runtime/src/outbox.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-archive-inspection-runtime/src/settings.rs', BACKEND_ROOT),
      'utf8',
    ),
  ]);

  assert.match(manifest, /role = "engine"/);
  assert.match(manifest, /owner = "attachment_archive_inspection"/);
  assert.match(manifest, /surface = "runtime"/);
  assert.match(manifest, /makosh-attachment-security-contract/);
  assert.match(manifest, /makosh-communications-attachment-contract/);
  assert.doesNotMatch(
    manifest,
    /makosh-(?:attachment-security-(?:core|persistence|runtime)|communications-(?:core|persistence|runtime)|mail|telegram|whatsapp|zulip)/,
  );
  assert.match(admission, /ATTACHMENT_ARCHIVE_INSPECTION_BLOB_TARGET_CAPABILITY_ID_V1/);
  for (const capability of [
    'attachment_archive_inspection.candidate.observe.v1',
    'attachment_archive_inspection.custody-request.publish.v1',
    'attachment_archive_inspection.custody-result.consume.v1',
    'attachment_archive_inspection.safety-state.observe.v1',
    'attachment_archive_inspection.storage.v1',
  ]) {
    assert.match(admission, new RegExp(capability.replaceAll('.', '\\.')));
  }
  assert.match(runtime, /ManagedControlChannelV2::new/);
  assert.match(runtime, /request_managed_runtime_event_access_v2/);
  assert.match(runtime, /storage_binding\(&storage_configuration, admission\)/);
  assert.match(runtime, /record_target_blob_receipt/);
  assert.match(runtime, /inspect_zip_bytes_v1/);
  assert.match(blob, /request_managed_blob_custody_transfer_v2/);
  assert.match(blob, /read_range/);
  assert.match(blob, /delegation_result_envelope_sha256/);
  assert.match(eventDecode, /Semantics::Observation/);
  assert.match(eventDecode, /Semantics::Event/);
  assert.match(eventDecode, /Semantics::Result/);
  assert.match(outbox, /publish_exact\(permit, &record\.exact_envelope_bytes\)/);
  assert.match(settings, /ApplyModeV1::Restart/);
  assert.doesNotMatch(
    `${admission}\n${runtime}\n${blob}\n${eventDecode}\n${outbox}\n${settings}`,
    /makosh_(?:attachment_security_(?:core|persistence|runtime)|communications_(?:core|persistence|runtime)|mail|telegram|whatsapp|zulip)/,
  );
});

test('archive runtime exposes exact owner-local Start Get and shared realtime surfaces', async () => {
  const [admission, contracts, clientPort, realtime, runtime, main] = await Promise.all([
    readFile(
      new URL('src/attachment-archive-inspection-runtime/src/admission.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-archive-inspection-runtime/src/contracts.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-archive-inspection-runtime/src/client_port.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-archive-inspection-runtime/src/client_realtime.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-archive-inspection-runtime/src/runtime.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-archive-inspection-runtime/src/main.rs', BACKEND_ROOT),
      'utf8',
    ),
  ]);

  assert.match(admission, /ATTACHMENT_ARCHIVE_INSPECTION_COMMAND_CONNECT_PATH_V1/);
  assert.match(admission, /ATTACHMENT_ARCHIVE_INSPECTION_QUERY_CONNECT_PATH_V1/);
  assert.match(admission, /ProvidedSurfaceKindV1::ClientRealtime/);
  assert.match(contracts, /ATTACHMENT_ARCHIVE_INSPECTION_SCHEMA_SHA256/);
  assert.match(clientPort, /dispatch_archive_inspection_client_request_v1/);
  assert.match(clientPort, /create_run/);
  assert.match(clientPort, /load_run/);
  assert.doesNotMatch(
    clientPort,
    /\b(?:blob_reference|custody_transfer_source_proof|provider_id|account_id|filesystem_path)\b/,
  );
  assert.match(realtime, /client_realtime_window/);
  assert.match(realtime, /ManagedRuntimeClientRealtimePublishRequestV1/);
  assert.match(realtime, /attachment-archive-inspection\/\{\}/);
  assert.match(runtime, /Operation::ClientDelivery/);
  assert.match(runtime, /RejectManagedControlRequestsV2/);
  assert.match(main, /pump_control_once/);
  assert.match(main, /pump_client_realtime_once/);
});

test('archive release assembly is a separate unsigned engine unit', async () => {
  const [manifest, library, binary] = await Promise.all([
    readFile(
      new URL('src/attachment-archive-inspection-assembly/Cargo.toml', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-archive-inspection-assembly/src/lib.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-archive-inspection-assembly/src/main.rs', BACKEND_ROOT),
      'utf8',
    ),
  ]);

  assert.match(manifest, /role = "engine"/);
  assert.match(manifest, /owner = "attachment_archive_inspection"/);
  assert.match(manifest, /surface = "assembly"/);
  assert.match(manifest, /makosh-attachment-archive-inspection-runtime/);
  assert.match(manifest, /makosh-attachment-archive-inspection-persistence/);
  assert.doesNotMatch(
    manifest,
    /makosh-(?:attachment-security|communications|mail|telegram|whatsapp|zulip)/,
  );
  assert.match(library, /attachment_archive_inspection_module_descriptor_v1/);
  assert.match(library, /attachment_archive_inspection_settings_schema_v1/);
  assert.match(library, /attachment_archive_inspection_storage_bundle_v1/);
  assert.match(library, /create_new\(true\)/);
  assert.match(library, /mode\(0o600\)/);
  assert.match(library, /attachment-archive-inspection\.release-artifacts\.json/);
  assert.doesNotMatch(library, /signing_key|signature_bytes|launch_runtime|Command::new/);
  assert.match(binary, /materialize_archive_inspection_release_assembly_v1/);
  assert.doesNotMatch(binary, /ManagedControlChannel|tokio|spawn/);
});

test('ZIP adapter is exact, metadata-only and cannot extract to disk', async () => {
  const [manifest, source] = await Promise.all([
    readFile(
      new URL('src/attachment-archive-inspection-zip/Cargo.toml', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-archive-inspection-zip/src/lib.rs', BACKEND_ROOT),
      'utf8',
    ),
  ]);

  assert.match(
    manifest,
    /zip = \{ version = "=6\.0\.0", default-features = false, features = \["deflate-flate2-zlib-rs"\] \}/,
  );
  assert.match(manifest, /makosh-attachment-archive-inspection-core/);
  assert.doesNotMatch(
    manifest,
    /makosh-(?:communications|attachment-security|blob|events|runtime|storage|kernel)/,
  );
  assert.match(source, /ZipArchive::new/);
  assert.match(source, /file\.compressed_size\(\)/);
  assert.match(source, /file\.size\(\)/);
  assert.match(source, /file\.encrypted\(\)/);
  assert.match(source, /file\.unix_mode\(\)/);
  assert.doesNotMatch(
    source,
    /std::fs|File::create|create_dir|tempdir|\.extract\s*\(|enclosed_name|read_to_end|TcpStream|sqlx/,
  );
});
