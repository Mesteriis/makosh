import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const POLICY_PATH = new URL('architecture/policy.json', BACKEND_ROOT);

test('Attachment Security candidate contract is provider-neutral and payload-bounded', async () => {
  const [proto, admission, candidate, manifest] = await Promise.all([
    readFile(
      new URL(
        'src/attachment-security-contract/proto/makosh/attachment_security/v1/scan_candidate.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-security-contract/src/admission.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-security-contract/src/candidate.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-security-contract/Cargo.toml', BACKEND_ROOT),
      'utf8',
    ),
  ]);
  const schema = proto.replaceAll(/\/\/.*$/gm, '');
  const fields = [...schema.matchAll(
    /^\s*(bytes|uint64|int64)\s+([a-z0-9_]+)\s*=\s*(\d+);$/gm,
  )].map(([, type, name, number]) => `${type} ${name} ${number}`);

  assert.deepEqual(fields, [
    'bytes attachment_anchor_id 1',
    'bytes blob_reference_id 2',
    'uint64 declared_size 3',
    'bytes blob_receipt_sha256 4',
    'int64 observed_at_unix_seconds 5',
    'bytes custody_transfer_source_proof 6',
  ]);
  assert.doesNotMatch(
    schema,
    /\b(?:provider|locator|filename|media_type|path|scanner|setting|content|payload|map)\b/i,
  );
  assert.match(admission, /DurableEnvelopeKindV1::Observation/);
  assert.match(admission, /EventRouteDirectionV1::Publish/);
  assert.doesNotMatch(admission, /EventRouteDirectionV1::Subscribe/);
  assert.match(
    candidate,
    /ATTACHMENT_SECURITY_MAX_CUSTODY_SOURCE_PROOF_BYTES_V1: usize = 2_048/,
  );
  assert.match(manifest, /role = "engine"/);
  assert.match(manifest, /owner = "attachment_security"/);
  assert.match(manifest, /surface = "contract"/);
});

test('Attachment Security core and ClamAV adapter remain separate engine units', async () => {
  const [coreManifest, core, clamavManifest, endpoint, instream] = await Promise.all([
    readFile(new URL('src/attachment-security-core/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-security-core/src/join.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-security-clamav/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-security-clamav/src/endpoint.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-security-clamav/src/instream.rs', BACKEND_ROOT), 'utf8'),
  ]);

  assert.match(coreManifest, /makosh-attachment-security-contract/);
  assert.doesNotMatch(
    coreManifest,
    /makosh-(?:communications|blob|attachment-security-clamav|runtime-protocol|storage-protocol)/,
  );
  assert.doesNotMatch(
    core,
    /makosh_communications|TcpStream|std::io|postgres|sqlx|nats|jetstream/i,
  );

  assert.match(clamavManifest, /makosh-attachment-security-contract/);
  assert.match(clamavManifest, /makosh-attachment-security-core/);
  assert.doesNotMatch(
    clamavManifest,
    /makosh-(?:communications|blob|runtime-protocol|storage-protocol)/,
  );
  assert.match(endpoint, /Ipv4Addr::LOCALHOST/);
  assert.match(instream, /const INSTREAM_COMMAND: &\[u8\] = b"zINSTREAM\\0"/);
  assert.match(instream, /response == b"stream: OK"/);
  assert.doesNotMatch(instream, /enum ClamAvScanErrorV1[\s\S]*?\bString\b/);
  assert.doesNotMatch(`${endpoint}\n${instream}`, /makosh_communications|blob_store|postgres|sqlx/i);
});

test('Attachment Security persistence owns the durable join, bounded jobs and exact outbox', async () => {
  const [
    manifest,
    schema,
    custodySchema,
    retryPolicySchema,
    retryPolicyIndexSchema,
    scannerRetryPolicyIndexSchema,
    archiveDelegationSchema,
    observation,
    jobs,
    delegation,
    textDelegationSchema,
    textDelegation,
    previewDelegationSchema,
    previewDelegation,
    recovery,
    runtime,
  ] = await Promise.all([
    readFile(new URL('src/attachment-security-persistence/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL(
        'src/attachment-security-persistence/migrations/0001_attachment_security_state.sql',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/attachment-security-persistence/migrations/0002_attachment_security_blob_custody.sql',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/attachment-security-persistence/migrations/0003_attachment_security_custody_successor_retry_policy.sql',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/attachment-security-persistence/migrations/0004_attachment_security_retry_policy_recovery_index.sql',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/attachment-security-persistence/migrations/0005_attachment_security_scanner_retry_policy_recovery_index.sql',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/attachment-security-persistence/migrations/0006_attachment_security_archive_delegation.sql',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-security-persistence/src/observation.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(new URL('src/attachment-security-persistence/src/jobs.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL('src/attachment-security-persistence/src/delegation.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL(
        'src/attachment-security-persistence/migrations/0007_attachment_security_text_extraction_delegation.sql',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-security-persistence/src/text_delegation.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL(
        'src/attachment-security-persistence/migrations/0008_attachment_security_preview_delegation.sql',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-security-persistence/src/preview_delegation.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-security-persistence/src/recovery.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(new URL('src/attachment-security-runtime/src/runtime.rs', BACKEND_ROOT), 'utf8'),
  ]);

  assert.match(manifest, /surface = "persistence"/);
  assert.match(manifest, /makosh-attachment-security-core/);
  assert.match(manifest, /makosh-communications-attachment-contract/);
  assert.doesNotMatch(manifest, /\[features\]/);
  assert.doesNotMatch(
    manifest,
    /makosh-(?:communications-(?!attachment-contract)|blob|kernel|attachment-security-clamav)/,
  );
  assert.equal(schema.match(/CREATE TABLE makosh_data\./g)?.length, 7);
  assert.doesNotMatch(schema, /makosh_data\.(?:communications|mail|telegram|zulip|whatsapp)_/);
  assert.match(schema, /attachment_security_event_inbox/);
  assert.match(schema, /envelope_sha256/);
  assert.match(schema, /max_attempts INTEGER NOT NULL CHECK \(max_attempts BETWEEN 1 AND 32\)/);
  assert.match(custodySchema, /custody_transfer_source_proof BYTEA NOT NULL/);
  assert.match(custodySchema, /octet_length\(custody_transfer_source_proof\) BETWEEN 1 AND 2048/);
  assert.match(custodySchema, /target_blob_reference_id BYTEA/);
  assert.match(custodySchema, /attachment_security_target_blob_receipt_complete/);
  assert.match(archiveDelegationSchema, /attachment_security_archive_delegation_inbox/);
  assert.match(archiveDelegationSchema, /attachment_security_archive_delegation_jobs/);
  assert.match(archiveDelegationSchema, /attachment_security_archive_delegation_outbox/);
  assert.match(textDelegationSchema, /attachment_security_text_extraction_delegation_inbox/);
  assert.match(textDelegationSchema, /attachment_security_text_extraction_delegation_jobs/);
  assert.match(textDelegationSchema, /attachment_security_text_extraction_delegation_outbox/);
  assert.match(previewDelegationSchema, /attachment_security_preview_delegation_inbox/);
  assert.match(previewDelegationSchema, /attachment_security_preview_delegation_jobs/);
  assert.match(previewDelegationSchema, /attachment_security_preview_delegation_outbox/);
  assert.match(delegation, /candidate_inbox\.envelope_sha256 = \$3/);
  assert.match(delegation, /payload\.verdict == AttachmentSafetyVerdictV1::SafeForDelivery/);
  assert.match(delegation, /insert_result_outbox/);
  assert.match(textDelegation, /candidate_inbox\.envelope_sha256 = \$3/);
  assert.match(textDelegation, /payload\.verdict == AttachmentSafetyVerdictV1::SafeForDelivery/);
  assert.match(textDelegation, /insert_result_outbox/);
  assert.match(previewDelegation, /candidate_inbox\.envelope_sha256 = \$3/);
  assert.match(previewDelegation, /payload\.verdict == AttachmentSafetyVerdictV1::SafeForDelivery/);
  assert.match(previewDelegation, /insert_result_outbox/);
  assert.match(retryPolicySchema, /retry_policy_revision SMALLINT NOT NULL DEFAULT 1/);
  assert.doesNotMatch(retryPolicySchema, /\bUPDATE\b|ALTER COLUMN/);
  assert.match(
    retryPolicyIndexSchema,
    /attachment_security_scan_jobs_retry_policy_recovery_idx/,
  );
  assert.match(retryPolicyIndexSchema, /retry_policy_revision/);
  assert.match(retryPolicyIndexSchema, /WHERE state = 3/);
  assert.match(
    scannerRetryPolicyIndexSchema,
    /attachment_security_scan_jobs_scanner_retry_policy_recovery_idx/,
  );
  assert.match(scannerRetryPolicyIndexSchema, /target_blob_reference_id IS NOT NULL/);
  assert.match(scannerRetryPolicyIndexSchema, /target_blob_receipt_sha256 IS NOT NULL/);
  assert.match(scannerRetryPolicyIndexSchema, /outbox_message_id IS NULL/);
  assert.match(recovery, /WHERE state = 3/);
  assert.match(recovery, /target_blob_reference_id IS NULL/);
  assert.match(recovery, /target_blob_receipt_sha256 IS NULL/);
  assert.match(recovery, /outbox_message_id IS NULL/);
  assert.match(recovery, /attempt_count = 0/);
  assert.match(recovery, /retry_policy_revision = \$1/);
  assert.match(recovery, /retry_policy_revision = 1/);
  assert.match(recovery, /target_blob_reference_id IS NOT NULL/);
  assert.match(recovery, /target_blob_receipt_sha256 IS NOT NULL/);
  assert.match(recovery, /retry_policy_revision = 2/);
  assert.match(recovery, /ATTACHMENT_SECURITY_RETRY_POLICY_REVISION_V3/);
  assert.match(jobs, /retry_policy_revision\) VALUES \([\s\S]*\$12\)/);
  assert.match(jobs, /ATTACHMENT_SECURITY_RETRY_POLICY_REVISION_V3/);
  assert.match(runtime, /reconcile_retry_policies_v3\(\)/);
  assert.match(observation, /attachment_security_join_locks/);
  assert.match(observation, /FOR UPDATE/);
  assert.match(observation, /decide_scan_join_v1/);
  assert.match(jobs, /FOR UPDATE SKIP LOCKED/);
  assert.match(jobs, /attempt_count >= max_attempts/);
  assert.match(jobs, /attempt_count = \$4 FOR UPDATE/);
  assert.match(jobs, /AttachmentSafetyVerdictOutboxRecordV1/);
  assert.match(jobs, /AttachmentSafetyExpectedStateV1::BlobAdmitted/);
  assert.match(jobs, /exact_envelope_bytes/);
  assert.match(jobs, /OutboxHashConflict/);
  assert.doesNotMatch(
    `${observation}\n${jobs}`,
    /makosh_communications_(?!attachment_contract)|provider_(?:id|locator|sdk)|scanner_signature/i,
  );
});

test('Mail publishes scan candidates through one exact contract and a separate durable outbox', async () => {
  const [manifest, admission, managed, durable, relay, main] = await Promise.all([
    readFile(new URL('src/mail-runtime/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/mail-runtime/src/admission.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/mail-runtime/src/managed.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/mail-persistence/src/durable.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL('src/mail-runtime/src/attachment_security_outbox.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(new URL('src/mail-runtime/src/main.rs', BACKEND_ROOT), 'utf8'),
  ]);

  assert.match(manifest, /^makosh-attachment-security-contract =/m);
  assert.doesNotMatch(
    manifest,
    /makosh-attachment-security-(?:core|clamav|persistence|runtime|assembly)/,
  );
  assert.match(
    admission,
    /MAIL_ATTACHMENT_SCAN_CANDIDATE_PUBLISH_CAPABILITY_ID: &str =\s*"mail\.attachment\.scan-candidate\.publish\.v1"/,
  );
  assert.match(
    admission,
    /attachment_security_scan_candidate_observed_publish_request_v1\(\)/,
  );
  assert.match(managed, /build_attachment_security_scan_candidate_outbox_record_v1/);
  assert.match(managed, /blob_reference_id: write\.reference_id/);
  assert.match(managed, /blob_receipt_sha256: write\.receipt_sha256/);
  assert.match(
    managed,
    /complete_attachment_blob_admission\([\s\S]*attachment_security_record\.as_ref\(\)/,
  );
  assert.match(durable, /mail_attachment_security_outbox/);
  assert.match(durable, /insert_attachment_security_outbox\(/);
  assert.match(relay, /pending_attachment_security_outbox/);
  assert.match(relay, /publish_exact\(permit, record\.exact_bytes\(\)\)/);
  assert.match(relay, /mark_attachment_security_outbox_published/);
  assert.match(main, /relay_attachment_security_outbox\(now\)/);
  assert.doesNotMatch(
    `${managed}\n${durable}\n${relay}`,
    /makosh_attachment_security_(?:core|clamav|persistence|runtime|assembly)/,
  );
});

test('Attachment Security runtime is a managed engine with event-only business boundaries', async () => {
  const [manifest, admission, runtime, scanner, delegation, textDelegation, previewDelegation, decoder, outbox] = await Promise.all([
    readFile(new URL('src/attachment-security-runtime/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-security-runtime/src/admission.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-security-runtime/src/runtime.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-security-runtime/src/scan.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-security-runtime/src/delegation.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-security-runtime/src/text_delegation.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-security-runtime/src/preview_delegation.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-security-runtime/src/event_decode.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-security-runtime/src/outbox.rs', BACKEND_ROOT), 'utf8'),
  ]);

  assert.match(manifest, /role = "engine"/);
  assert.match(manifest, /owner = "attachment_security"/);
  assert.match(manifest, /surface = "runtime"/);
  for (const dependency of [
    'makosh-attachment-security-contract',
    'makosh-attachment-security-core',
    'makosh-attachment-security-clamav',
    'makosh-attachment-security-persistence',
    'makosh-attachment-archive-inspection-ingress',
    'makosh-attachment-preview-ingress',
    'makosh-attachment-text-extraction-ingress',
    'makosh-communications-attachment-contract',
    'makosh-blob-client',
    'makosh-events-jetstream',
  ]) {
    assert.match(manifest, new RegExp(`^${dependency} =`, 'm'));
  }
  assert.doesNotMatch(
    manifest,
    /makosh-(?:communications-(?!attachment-contract)|mail|telegram|whatsapp|zulip|kernel)/,
  );
  assert.deepEqual(
    [...admission.matchAll(
      /pub const ATTACHMENT_SECURITY_[A-Z_]+_CAPABILITY_ID: &str =\s*"([^"]+)";/g,
    )].map(([, capability]) => capability).sort(),
    [
      'attachment_security.archive-delegation-result.publish.v1',
      'attachment_security.candidate.observe.v1',
      'attachment_security.communications-state.observe.v1',
      'attachment_security.preview-delegation-result.publish.v1',
      'attachment_security.storage.v1',
      'attachment_security.text-extraction-delegation-result.publish.v1',
      'attachment_security.verdict.publish.v1',
    ],
  );
  assert.match(
    admission,
    /ATTACHMENT_SECURITY_BLOB_CAPABILITY_ID: &str =\s*ATTACHMENT_SECURITY_BLOB_CUSTODY_TARGET_CAPABILITY_ID/,
  );
  assert.match(admission, /ModuleKindV1::Engine/);
  assert.match(runtime, /ManagedControlChannelV2/);
  assert.match(runtime, /request_managed_runtime_event_access_v2/);
  assert.match(scanner, /receipt_sha256: Some\(&target_blob\.receipt_sha256\)/);
  assert.match(scanner, /receipt_sha256: &claimed\.job\.blob_receipt_sha256/);
  assert.match(scanner, /request_managed_blob_custody_transfer_v2/);
  assert.match(delegation, /request_managed_blob_custody_delegation_v2/);
  assert.match(delegation, /ATTACHMENT_ARCHIVE_INSPECTION_BLOB_TARGET_OWNER_ID_V1/);
  assert.match(delegation, /predecessor_evidence_envelope_sha256/);
  assert.match(textDelegation, /request_managed_blob_custody_delegation_v2/);
  assert.match(textDelegation, /ATTACHMENT_TEXT_EXTRACTION_BLOB_TARGET_OWNER_ID_V1/);
  assert.match(textDelegation, /predecessor_evidence_envelope_sha256/);
  assert.match(previewDelegation, /request_managed_blob_custody_delegation_v2/);
  assert.match(previewDelegation, /ATTACHMENT_PREVIEW_BLOB_TARGET_OWNER_ID_V1/);
  assert.match(previewDelegation, /predecessor_evidence_envelope_sha256/);
  assert.match(runtime, /persist_archive_delegation_request/);
  assert.match(runtime, /complete_archive_delegation_with_outbox/);
  assert.match(runtime, /persist_text_delegation_request/);
  assert.match(runtime, /complete_text_delegation_with_outbox/);
  assert.match(runtime, /persist_preview_delegation_request/);
  assert.match(runtime, /complete_preview_delegation_with_outbox/);
  assert.match(scanner, /scan_clamav_loopback_v1/);
  assert.match(runtime, /retry_scan_job/);
  assert.match(runtime, /complete_scan_job_with_outbox/);
  assert.match(decoder, /Semantics::Observation/);
  assert.match(decoder, /Semantics::Event/);
  assert.match(decoder, /BlobPending/);
  assert.match(decoder, /BlobAdmitted/);
  assert.match(outbox, /publish_exact\(permit, record\.exact_bytes\(\)\)/);
  assert.doesNotMatch(
    `${runtime}\n${scanner}\n${delegation}\n${decoder}\n${outbox}`,
    /makosh_(?:communications_(?:domain|runtime|persistence|api)|mail|telegram|whatsapp|zulip|kernel)/,
  );
});

test('Attachment Security Blob reads are one-use and receipt-bound below the engine', async () => {
  const [protocol, client, kernelSession, serviceSession, service] = await Promise.all([
    readFile(
      new URL(
        'src/platform/runtime_protocol/proto/makosh/runtime/v1/blob_runtime.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('src/platform/blob/client/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/kernel/src/platform/blob/session.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL('src/platform/blob/service/src/control/data/session.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/platform/blob/service/src/control/data/service.rs', BACKEND_ROOT),
      'utf8',
    ),
  ]);

  assert.match(protocol, /bytes expected_plaintext_sha256 = 21;/);
  assert.match(client, /BlobDataOperationV1::BlobDataOperationReadRangeV1/);
  assert.match(client, /exact_receipt_binding\(&grant\.expected_plaintext_sha256/);
  assert.match(kernelSession, /expected_plaintext_sha256: request\.receipt_sha256\.clone\(\)/);
  assert.match(serviceSession, /expected_plaintext_sha256: Option<\[u8; 32\]>/);
  assert.match(service, /exact_read_range_binding/);
  assert.match(service, /Sha256::digest\(plaintext\)/);
  assert.doesNotMatch(
    `${kernelSession}\n${service}`,
    /makosh_(?:communications|attachment_security)|clamav/i,
  );
});

test('Cross-owner Blob custody binds a public module audience and current runtime fences', async () => {
  const [
    controlProtocol,
    blobProtocol,
    attachmentContract,
    communicationsContract,
    attachmentAdmission,
    communicationsAdmission,
    mailRuntime,
    kernelSession,
    blobSession,
  ] = await Promise.all([
    readFile(
      new URL(
        'src/platform/runtime_protocol/proto/makosh/runtime/v1/managed_runtime_control.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/platform/runtime_protocol/proto/makosh/runtime/v1/blob_runtime.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-security-contract/src/admission.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(new URL('src/communications-ingress/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-security-runtime/src/admission.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communications-runtime/src/admission.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/mail-runtime/src/managed.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/kernel/src/platform/blob/session.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL('src/platform/blob/service/src/control/data/session.rs', BACKEND_ROOT),
      'utf8',
    ),
  ]);

  assert.match(controlProtocol, /string custody_target_module_id = 14;/);
  assert.match(controlProtocol, /ManagedRuntimeBlobCustodyDelegationRequestV1/);
  assert.match(controlProtocol, /ManagedRuntimeBlobCustodyDelegationDeliveryV1/);
  assert.doesNotMatch(controlProtocol, /custody_target_registration_id/);
  assert.match(blobProtocol, /string target_module_id = 19;/);
  assert.match(blobProtocol, /BlobCustodySourceProofKindV1 proof_kind = 22;/);
  assert.match(blobProtocol, /bytes delegation_id = 23;/);
  assert.match(blobProtocol, /bytes predecessor_proof_sha256 = 24;/);
  assert.doesNotMatch(blobProtocol, /string target_registration_id = 19;/);
  assert.match(
    attachmentContract,
    /ATTACHMENT_SECURITY_BLOB_CUSTODY_TARGET_MODULE_ID: &str =\s*"makosh-attachment-security-runtime"/,
  );
  assert.match(
    communicationsContract,
    /COMMUNICATIONS_BLOB_CUSTODY_TARGET_MODULE_ID: &str = "makosh-communications-runtime"/,
  );
  assert.match(
    attachmentAdmission,
    /ATTACHMENT_SECURITY_MODULE_ID: &str = ATTACHMENT_SECURITY_BLOB_CUSTODY_TARGET_MODULE_ID/,
  );
  assert.match(
    communicationsAdmission,
    /COMMUNICATIONS_MODULE_ID: &str = COMMUNICATIONS_BLOB_CUSTODY_TARGET_MODULE_ID/,
  );
  assert.match(mailRuntime, /module_id: ATTACHMENT_SECURITY_BLOB_CUSTODY_TARGET_MODULE_ID/);
  assert.match(mailRuntime, /module_id: COMMUNICATIONS_BLOB_CUSTODY_TARGET_MODULE_ID/);
  assert.doesNotMatch(mailRuntime, /makosh-attachment-security-runtime|makosh-communications-runtime/);
  assert.match(kernelSession, /expectation\.module_id\(\)/);
  assert.match(kernelSession, /target_registration_id: expectation\.registration_id\(\)\.to_owned\(\)/);
  const transferStart = kernelSession.indexOf('fn issue_custody_transfer(');
  const transferEnd = kernelSession.indexOf('\npub(crate) fn valid_request(', transferStart);
  const transfer = kernelSession.slice(transferStart, transferEnd);
  assert.match(transfer, /catalog::resolve\(&\*self\.store\)/);
  assert.match(transfer, /entry\.grant_epoch\(\) == source\.grant_epoch/);
  assert.match(transfer, /required_source_operation\(&source\)/);
  assert.match(
    kernelSession,
    /BlobCustodySourceProofKindOriginalWriteV1[\s\S]*Some\(ModuleBlobOperationV1::Write\)/,
  );
  assert.match(
    kernelSession,
    /BlobCustodySourceProofKindCurrentCustodianRedelegationV1[\s\S]*Some\(ModuleBlobOperationV1::CustodyTransfer\)/,
  );
  assert.doesNotMatch(transfer, /current_managed_runtime_matches/);
  assert.doesNotMatch(blobSession, /source\.owner_id != target_reference\.owner_id\(\)/);
  assert.match(blobSession, /valid_source_proof_lineage\(source\)/);
  assert.match(blobSession, /kernel_signed_transfer_keeps_distinct_cross_owner_fences/);
});

test('Attachment Security release assembly is a separate unsigned engine unit', async () => {
  const [manifest, assembly, command] = await Promise.all([
    readFile(new URL('src/attachment-security-assembly/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-security-assembly/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-security-assembly/src/main.rs', BACKEND_ROOT), 'utf8'),
  ]);
  const dependencySection = manifest.split('[dependencies]\n')[1] ?? '';
  const dependencies = [...dependencySection.matchAll(/^([a-z0-9_-]+)\s*=/gm)]
    .map(([, dependency]) => dependency)
    .sort();

  assert.match(manifest, /role = "engine"/);
  assert.match(manifest, /owner = "attachment_security"/);
  assert.match(manifest, /surface = "assembly"/);
  assert.deepEqual(dependencies, [
    'makosh-attachment-security-persistence',
    'makosh-attachment-security-runtime',
    'makosh-runtime-protocol',
    'makosh-storage-protocol',
    'prost',
    'serde',
    'serde_json',
  ]);
  for (const file of [
    'attachment-security.runtime.descriptor.pb',
    'attachment-security.runtime.settings.pb',
    'attachment-security.storage.bundle.pb',
    'attachment-security.release-artifacts.json',
  ]) {
    assert.ok(assembly.includes(file), `assembly must materialize ${file}`);
  }
  assert.match(assembly, /validate_descriptor_v1/);
  assert.match(assembly, /validate_settings_schema_v1/);
  assert.match(assembly, /validate_storage_bundle/);
  assert.match(assembly, /create_new\(true\)/);
  assert.match(command, /--build-id/);
  assert.match(command, /--output-dir/);
  assert.match(command, /--runtime/);
  assert.doesNotMatch(
    `${manifest}\n${assembly}\n${command}`,
    /makosh-(?:communications|mail|telegram|whatsapp|zulip|kernel|blob|events)|SigningKey|sign_manifest|ed25519|p256/,
  );
});

test('Attachment Security managed conformance launches the signed Engine through typed settings', async () => {
  const [
    manifest,
    script,
    setup,
    flow,
    eventFlow,
    clamav,
    blobFixture,
    persistenceFixture,
  ] = await Promise.all([
    readFile(
      new URL('tests/support/kernel-recovery/Cargo.toml', BACKEND_ROOT),
      'utf8',
    ),
    readFile(new URL('scripts/test-authenticated-storage.mjs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/'
          + 'attachment_security_managed_setup.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/'
          + 'attachment_security_managed_flow.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/'
          + 'attachment_security_event_flow.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/'
          + 'attachment_security_clamav_fixture.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/'
          + 'attachment_security_blob_fixture.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/'
          + 'attachment_security_persistence_fixture.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
  ]);

  assert.match(manifest, /^makosh-attachment-security-runtime =/m);
  assert.match(manifest, /^sqlx =/m);
  assert.match(script, /'-p',\s*'makosh-attachment-security-runtime'/);
  assert.match(script, /MAKOSH_ATTACHMENT_SECURITY_RUNTIME_BIN:/);
  assert.match(
    script,
    /managed_attachment_security_engine_starts_with_exact_signed_contracts/,
  );
  assert.match(setup, /SignedRuntimeArtifact::new\(/);
  assert.match(setup, /attachment_security_module_descriptor_v1\(/);
  assert.match(setup, /start_reserved_engine\(/);
  assert.match(setup, /storage::successor::reserve\(/);
  assert.match(setup, /restart_attachment_security_runtime\(/);
  assert.doesNotMatch(setup, /start_reserved_(?:domain|integration)\(/);
  assert.match(setup, /ManagedEngineRuntimeConfigurationV1/);
  assert.match(setup, /attachment_security_settings_snapshot\(/);
  assert.match(flow, /supervisor\s*\.is_active\(&attachment_security\.registration_id\)/);
  assert.match(flow, /assert_threat_attachment_security_verdict_flow\(/);
  assert.match(flow, /assert_attachment_security_scanner_failure_is_fail_closed\(/);
  assert.match(
    flow,
    /assert_attachment_security_outbox_replays_after_nats_outage_and_restart\(/,
  );
  assert.match(flow, /stop\(COMMUNICATIONS_REGISTRATION\)/);
  assert.match(flow, /restart_communications_domain\(/);
  assert.match(flow, /runtime_generation,\s*2/);
  assert.match(flow, /runtime_generation,\s*3/);
  assert.match(flow, /blob_source\.advance_runtime_generation\(/);
  assert.match(flow, /blob_source\.revoke\(/);
  assert.match(
    flow,
    /assert_attachment_security_custody_failure_is_fail_closed\(/,
  );
  assert.match(flow, /supervisor\s*\.stop\("vault"\)/);
  assert.match(flow, /supervisor\s*\.stop\("blob"\)/);
  assert.match(
    flow,
    /transition_module_registration\([\s\S]*ModuleRegistrationState::Revoked/,
  );
  assert.match(blobFixture, /admit_authority_source\(/);
  assert.match(blobFixture, /advance_runtime_generation\(/);
  assert.match(blobFixture, /transition_module_registration\(/);
  assert.match(clamav, /Ipv4Addr::LOCALHOST/);
  assert.match(clamav, /b"zINSTREAM\\0"/);
  for (const outcome of ['Threat', 'Malformed', 'Disconnect', 'Timeout']) {
    assert.match(clamav, new RegExp(`${outcome} = \\d`));
    assert.match(flow, new RegExp(`ClamAvFixtureOutcomeV1::${outcome}`));
  }
  for (const outcome of [
    'CustodyProbe',
    'VaultOutageProbe',
    'BlobOutageProbe',
    'TargetRevokedProbe',
  ]) {
    assert.match(clamav, new RegExp(`${outcome} = \\d`));
    assert.match(flow, new RegExp(`ClamAvFixtureOutcomeV1::${outcome}`));
  }
  assert.match(clamav, /Fixture-Signature FOUND/);
  assert.match(clamav, /HeldClean = \d/);
  assert.match(eventFlow, /AttachmentSafetyVerdictV1::Quarantined/);
  assert.match(eventFlow, /scanner failure must not create a verdict/);
  assert.match(eventFlow, /after\.outbox, before\.outbox/);
  assert.match(eventFlow, /set_authenticated_nats_container_running\(false\)/);
  assert.match(eventFlow, /restarted relay must publish the exact persisted verdict bytes/);
  assert.match(eventFlow, /stale Attachment Security verdict must not mutate Communications state/);
  assert.match(eventFlow, /custody failure must not create a verdict/);
  assert.match(eventFlow, /!first_job\.target_blob_receipt_present/);
  assert.match(eventFlow, /!first_job\.outbox_message_id_present/);
  assert.match(persistenceFixture, /WHERE attachment_anchor_id = \$1/);
  assert.doesNotMatch(
    `${setup}\n${flow}`,
    /makosh_communications_(?:domain|persistence|runtime)/,
  );
});

test('Attachment Security remains one exact engine after Mail integration admission', async () => {
  const policy = JSON.parse(await readFile(POLICY_PATH, 'utf8'));
  const productionPackages = policy.implementation.productionPackages;

  assert.equal(
    policy.implementation.currentSlice,
    'call_transcription_managed_conformance_v1',
  );
  assert.deepEqual(policy.implementation.ownerInventory.engines, [
    'ai',
    'attachment_archive_inspection',
    'attachment_security',
    'speech_to_text',
  ]);
  assert.deepEqual(
    productionPackages
      .filter(({ name }) => name.startsWith('makosh-attachment-security-'))
      .map(({ name }) => name),
    [
      'makosh-attachment-security-contract',
      'makosh-attachment-security-core',
      'makosh-attachment-security-clamav',
      'makosh-attachment-security-persistence',
      'makosh-attachment-security-runtime',
      'makosh-attachment-security-assembly',
    ],
  );
  assert.deepEqual(
    policy.implementation.ownerInventory.businessCapabilities.filter(
      (capability) => capability.startsWith('attachment_security.'),
    ),
    [
      'attachment_security.archive-delegation-result.publish.v1',
      'attachment_security.archive-inspection-delegation.v1',
      'attachment_security.blob.v1',
      'attachment_security.candidate.observe.v1',
      'attachment_security.communications-state.observe.v1',
      'attachment_security.storage.v1',
      'attachment_security.text-extraction-delegation-result.publish.v1',
      'attachment_security.text-extraction-delegation.v1',
      'attachment_security.verdict.publish.v1',
    ],
  );
  assert.deepEqual(
    policy.phaseGates.requires.attachment_security_engine_v1,
    [
      'managed_launch_trust_v1',
      'vault_v1',
      'storage_control_v1',
      'nats_data_plane_v1',
      'blob_v1',
    ],
  );
  assert.equal(
    policy.phaseGates.notAuthorized.includes('attachment_security_engine_v1'),
    false,
  );
});
