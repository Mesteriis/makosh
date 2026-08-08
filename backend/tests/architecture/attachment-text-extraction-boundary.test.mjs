import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const REPOSITORY_ROOT = new URL('../', BACKEND_ROOT);

test('text extraction is an implemented workflow and not a Communications facade', async () => {
  const [inventorySource, policySource, adr] = await Promise.all([
    readFile(
      new URL('architecture/communications-settings-reconstruction.json', BACKEND_ROOT),
      'utf8',
    ),
    readFile(new URL('architecture/policy.json', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL(
        'docs/adr/ADR-0371-bounded-attachment-text-extraction-workflow.md',
        REPOSITORY_ROOT,
      ),
      'utf8',
    ),
  ]);
  const inventory = JSON.parse(inventorySource);
  const policy = JSON.parse(policySource);
  const slice = inventory.slices.find(
    ({ gate }) => gate === 'attachment_text_extraction_v1',
  );

  assert.deepEqual(slice, {
    gate: 'attachment_text_extraction_v1',
    role: 'workflow',
    owner: 'attachment_text_extraction',
    state: 'implemented',
    dependsOn: ['blob_v1', 'attachment_security_engine_v1'],
  });
  assert.equal(
    policy.implementation.currentSlice,
    'call_transcription_managed_conformance_v1',
  );
  assert(policy.implementation.ownerInventory.workflows.includes(
    'attachment_text_extraction',
  ));
  assert.match(adr, /Состояние реализации: реализовано/);
  assert.match(adr, /managed production contour/i);
  assert.match(adr, /stale custody proof/);
  assert.match(adr, /Vault outage/);
  assert.match(adr, /Workflow не вызывает Communications или Attachment Security RPC/);
  assert.match(adr, /Get.*realtime.*не содержат text или\nBlob authority/s);
});

test('client contract keeps status separate from bounded private content', async () => {
  const [manifest, proto, source] = await Promise.all([
    readFile(
      new URL('src/attachment-text-extraction-api/Cargo.toml', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL(
        'src/attachment-text-extraction-api/proto/makosh/attachment_text_extraction/v1/text_extraction.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-text-extraction-api/src/lib.rs', BACKEND_ROOT),
      'utf8',
    ),
  ]);

  assert.match(manifest, /role = "workflow"/);
  assert.match(manifest, /owner = "attachment_text_extraction"/);
  assert.match(manifest, /surface = "contract"/);
  assert.doesNotMatch(
    manifest,
    /makosh-(?:communications|attachment-security|blob|kernel)/,
  );
  assert.match(proto, /rpc Start/);
  assert.match(proto, /rpc Get/);
  assert.match(proto, /rpc ReadText/);
  assert.match(proto, /bytes text_utf8 = 2/);
  const status = proto.slice(
    proto.indexOf('message GetAttachmentTextExtractionResponseV1'),
    proto.indexOf('message ReadAttachmentTextRequestV1'),
  );
  const realtime = proto.slice(
    proto.indexOf('message AttachmentTextExtractionStatusChangedV1'),
    proto.indexOf('service AttachmentTextExtractionCommandService'),
  );
  assert.doesNotMatch(status, /text_utf8|blob_reference/);
  assert.doesNotMatch(realtime, /text_utf8|blob_reference/);
  assert.doesNotMatch(
    proto,
    /\b(?:provider|account_id|filename|content_type|filesystem|source_path|map)\b/,
  );
  assert.match(source, /MAX_DERIVED_BYTES_V1: usize = 1024 \* 1024/);
  assert.match(source, /MAX_VISIBLE_BYTES_V1: usize = 64 \* 1024/);
});

test('target-owned ingress carries event custody without caller-selected authority', async () => {
  const [manifest, proto, source, envelope] = await Promise.all([
    readFile(
      new URL('src/attachment-text-extraction-ingress/Cargo.toml', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL(
        'src/attachment-text-extraction-ingress/proto/makosh/attachment_text_extraction/ingress/v1/custody_delegation.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-text-extraction-ingress/src/lib.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-text-extraction-ingress/src/envelope.rs', BACKEND_ROOT),
      'utf8',
    ),
  ]);

  assert.match(manifest, /role = "workflow"/);
  assert.match(manifest, /surface = "contract"/);
  assert.doesNotMatch(
    manifest,
    /makosh-(?:communications|attachment-security|blob|kernel|attachment-text-extraction-(?:api|core|runtime|persistence|assembly))/,
  );
  assert.match(proto, /message RequestAttachmentTextCustodyDelegationV1/);
  assert.match(proto, /message AttachmentTextCustodyDelegatedV1/);
  const request = proto.slice(
    proto.indexOf('message RequestAttachmentTextCustodyDelegationV1'),
    proto.indexOf('message AttachmentTextCustodyDelegatedV1'),
  );
  assert.doesNotMatch(
    request,
    /\b(?:source_reference_id|custody_transfer_source_proof|target_owner_id|target_module_id|target_capability_id|provider_id|filename|content_type)\b/,
  );
  assert.match(source, /ATTACHMENT_SECURITY_TEXT_EXTRACTION_DELEGATION_CAPABILITY_ID_V1/);
  assert.match(source, /ATTACHMENT_TEXT_EXTRACTION_BLOB_TARGET_OWNER_ID_V1/);
  assert.match(envelope, /DurableEnvelopeV1/);
  assert.match(source, /DurableEnvelopeKindV1/);
});

test('pure core owns join and lifecycle without transport storage or parsers', async () => {
  const [manifest, source, join, lifecycle, content] = await Promise.all([
    readFile(
      new URL('src/attachment-text-extraction-core/Cargo.toml', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-text-extraction-core/src/lib.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-text-extraction-core/src/join.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-text-extraction-core/src/lifecycle.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-text-extraction-core/src/content.rs', BACKEND_ROOT),
      'utf8',
    ),
  ]);

  assert.match(manifest, /makosh-attachment-text-extraction-api/);
  assert.doesNotMatch(
    manifest,
    /makosh-(?:communications|attachment-security|blob|events|runtime|storage|kernel)|\b(?:pdf|docx|ocr|sqlx|tokio)\s*=/,
  );
  assert.match(join, /AttachmentTextCustodyDelegationIntentV1/);
  const intent = join.slice(
    join.indexOf('struct AttachmentTextCustodyDelegationIntentV1'),
    join.indexOf('enum AttachmentTextExtractionRejectionV1'),
  );
  assert.doesNotMatch(
    intent,
    /\b(?:blob_reference_id|declared_size|receipt_sha256|custody_transfer_source_proof)\b/,
  );
  assert.match(lifecycle, /AttachmentTextExtractionStateV1/);
  assert.match(content, /visible_attachment_text_v1/);
  assert.doesNotMatch(
    `${source}\n${join}\n${lifecycle}\n${content}`,
    /TcpStream|File::|sqlx|postgres|nats|jetstream|makosh_communications|makosh_attachment_security/,
  );
});

test('parser contract and adapters are five isolated byte-only units', async () => {
  const [contractManifest, contract, plainManifest, pdfManifest, docxManifest, ocrManifest, ocr] =
    await Promise.all([
      readFile(
        new URL('src/attachment-text-extraction-parser-contract/Cargo.toml', BACKEND_ROOT),
        'utf8',
      ),
      readFile(
        new URL('src/attachment-text-extraction-parser-contract/src/lib.rs', BACKEND_ROOT),
        'utf8',
      ),
      readFile(
        new URL('src/attachment-text-extraction-plain/Cargo.toml', BACKEND_ROOT),
        'utf8',
      ),
      readFile(
        new URL('src/attachment-text-extraction-pdf/Cargo.toml', BACKEND_ROOT),
        'utf8',
      ),
      readFile(
        new URL('src/attachment-text-extraction-docx/Cargo.toml', BACKEND_ROOT),
        'utf8',
      ),
      readFile(
        new URL('src/attachment-text-extraction-ocr/Cargo.toml', BACKEND_ROOT),
        'utf8',
      ),
      readFile(
        new URL('src/attachment-text-extraction-ocr/src/lib.rs', BACKEND_ROOT),
        'utf8',
      ),
    ]);
  const manifests = [contractManifest, plainManifest, pdfManifest, docxManifest, ocrManifest];

  for (const manifest of manifests) {
    assert.match(manifest, /role = "workflow"/);
    assert.match(manifest, /owner = "attachment_text_extraction"/);
    assert.doesNotMatch(
      manifest,
      /makosh-(?:communications|attachment-security|blob|events|runtime|storage|kernel|attachment-text-extraction-(?:api|core|persistence|runtime|assembly))/,
    );
  }
  assert.match(contract, /detect_attachment_text_parser_v1/);
  assert.match(contract, /source\.starts_with\(b"%PDF-"\)/);
  assert.match(contract, /source\.starts_with\(b"PK\\x03\\x04"\)/);
  assert.match(pdfManifest, /pdf-text-extract = \{ version = "=0\.2\.0"/);
  assert.match(docxManifest, /quick-xml = \{ version = "=0\.41\.0"/);
  assert.match(docxManifest, /zip = \{ version = "=6\.0\.0"/);
  assert.match(ocr, /ATTACHMENT_TEXT_OCR_LANGUAGES_V1: &str = "eng\+rus"/);
  assert.match(ocr, /executable_sha256/);
  assert.match(ocr, /english_model_sha256/);
  assert.match(ocr, /russian_model_sha256/);
  assert.match(ocr, /\.env_clear\(\)/);
  assert.doesNotMatch(ocr, /\b(?:sh|bash|zsh)\b|Command::new\("tesseract"\)/);
});

test('text extraction persistence owns exact joins and fenced jobs without transport ownership or plaintext', async () => {
  const [manifest, schema, repository, observations, custody, jobs] = await Promise.all([
    readFile(
      new URL('src/attachment-text-extraction-persistence/Cargo.toml', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL(
        'src/attachment-text-extraction-persistence/migrations/0001_text_extraction.sql',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-text-extraction-persistence/src/repository.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-text-extraction-persistence/src/observations.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-text-extraction-persistence/src/custody.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-text-extraction-persistence/src/jobs.rs', BACKEND_ROOT),
      'utf8',
    ),
  ]);

  assert.match(manifest, /role = "workflow"/);
  assert.match(manifest, /owner = "attachment_text_extraction"/);
  assert.match(manifest, /surface = "persistence"/);
  assert.match(manifest, /makosh-attachment-text-extraction-core/);
  assert.match(manifest, /makosh-storage-protocol/);
  assert.doesNotMatch(
    manifest,
    /makosh-(?:communications|attachment-security|blob|events|runtime|kernel)|attachment-text-extraction-(?:plain|pdf|docx|ocr|runtime|assembly)/,
  );
  for (const table of [
    'attachment_text_extraction_runs',
    'attachment_text_extraction_event_inbox',
    'attachment_text_extraction_scan_candidates',
    'attachment_text_extraction_safety_facts',
    'attachment_text_extraction_custody_outbox',
    'attachment_text_extraction_custody_result_inbox',
    'attachment_text_extraction_jobs',
    'attachment_text_extraction_artifacts',
    'attachment_text_extraction_realtime',
  ]) {
    assert.match(schema, new RegExp(`makosh_data\\.${table}`));
  }
  assert.doesNotMatch(
    schema,
    /text_utf8|extracted_content|source_bytes|provider_id|filename|mime_type/,
  );
  assert.match(repository, /ON CONFLICT \(logical_owner_id, operation_id\) DO NOTHING/);
  assert.match(repository, /state_revision = \$10/);
  assert.match(repository, /find_artifact/);
  assert.match(repository, /realtime_after/);
  assert.match(repository, /append_realtime/);
  assert.match(observations, /pg_advisory_xact_lock/);
  assert.match(observations, /decide_attachment_text_join_v1/);
  assert.match(observations, /reject_anchor_runs/);
  assert.match(custody, /exact_envelope_bytes/);
  assert.match(custody, /insert_result_inbox/);
  assert.match(custody, /candidate_declared_size != payload\.declared_size/);
  assert.match(custody, /candidate_receipt_sha256 != id32_input\(&payload\.receipt_sha256\)/);
  assert.match(custody, /attachment_text_extraction_scan_candidates c/);
  assert.doesNotMatch(custody, /DurableEnvelopeV1|makosh_events_protocol|prost::Message/);
  assert.match(jobs, /FOR UPDATE SKIP LOCKED/);
  assert.match(jobs, /runtime_generation/);
  assert.match(jobs, /grant_epoch/);
  assert.match(jobs, /lease_fence/);
  assert.match(jobs, /recover_expired_jobs/);
  assert.match(jobs, /complete_job/);
  assert.doesNotMatch(
    [repository, observations, custody, jobs].join('\n'),
    /text_utf8|extracted_content|source_bytes|provider_id|filename|mime_type/,
  );
});

test('managed runtime composes exact Event Blob parser client SSE and OCR resource ports without domain implementations', async () => {
  const [manifest, admission, runtime, eventDecode, blob, clientPort, realtime, parser, ocrResources, main] =
    await Promise.all([
      readFile(new URL('src/attachment-text-extraction-runtime/Cargo.toml', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/attachment-text-extraction-runtime/src/admission.rs', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/attachment-text-extraction-runtime/src/runtime.rs', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/attachment-text-extraction-runtime/src/event_decode.rs', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/attachment-text-extraction-runtime/src/blob.rs', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/attachment-text-extraction-runtime/src/client_port.rs', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/attachment-text-extraction-runtime/src/client_realtime.rs', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/attachment-text-extraction-runtime/src/parser.rs', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/attachment-text-extraction-runtime/src/ocr_resources.rs', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/attachment-text-extraction-runtime/src/main.rs', BACKEND_ROOT), 'utf8'),
    ]);
  const ocrProduction = ocrResources.split('#[cfg(test)]')[0];

  assert.match(manifest, /role = "workflow"/);
  assert.match(manifest, /surface = "runtime"/);
  assert.match(manifest, /makosh-events-jetstream/);
  assert.match(manifest, /makosh-blob-client/);
  assert.match(manifest, /makosh-attachment-text-extraction-persistence/);
  assert.doesNotMatch(
    manifest,
    /makosh-(?:communications-runtime|attachment-security-engine|kernel)/,
  );
  assert.match(admission, /ModuleKindV1::Workflow/);
  assert.match(admission, /ATTACHMENT_TEXT_EXTRACTION_CONTENT_CONNECT_PATH_V1/);
  assert.match(admission, /BlobQuotaOperationV1::Write/);
  assert.match(admission, /ATTACHMENT_TEXT_EXTRACTION_OCR_CAPABILITY_ID_V1/);
  assert.match(ocrResources, /attachment_text_extraction\.ocr_runtime\.v1/);
  assert.match(admission, /RuntimeArtifactUseV1::NativeExecutable/);
  assert.match(admission, /RuntimeArtifactUseV1::ReadOnlyData/);
  assert.match(runtime, /try_receive_runtime_pull_delivery/);
  assert.match(runtime, /None => return Ok\(false\)/);
  assert.match(runtime, /materialize_pending_custody_requests/);
  assert.match(runtime, /process_next_job/);
  assert.match(runtime, /pump_client_realtime_once/);
  assert.match(eventDecode, /OutboxRecordV1::accept/);
  assert.match(eventDecode, /payload_sha256/);
  assert.match(blob, /request_managed_blob_custody_transfer_v2/);
  assert.match(blob, /BlobDataOperationWriteV1/);
  assert.match(clientPort, /ReadText/);
  assert.match(clientPort, /request\.logical_owner_id != logical_owner_id/);
  assert.match(realtime, /PublishClientRealtime/);
  assert.match(parser, /detect_attachment_text_parser_v1/);
  assert.match(parser, /matches_artifact_identity_v1/);
  assert.match(parser, /configuration\.executable_sha256/);
  assert.match(parser, /configuration\.english_model_sha256/);
  assert.match(parser, /configuration\.russian_model_sha256/);
  assert.match(runtime, /matches_artifact_identity_v1/);
  assert.match(ocrResources, /attachment_text_extraction\.ocr\.eng\.v1/);
  assert.match(ocrResources, /attachment_text_extraction\.ocr\.runner\.v1/);
  assert.match(ocrResources, /attachment_text_extraction\.ocr\.rus\.v1/);
  assert.match(ocrResources, /eng\.traineddata/);
  assert.match(ocrResources, /rus\.traineddata/);
  assert.match(ocrResources, /create_new\(true\)/);
  assert.match(ocrResources, /from_mode\(0o400\)/);
  assert.doesNotMatch(ocrProduction, /std::env|Command::new|tesseract"|settings|provider/);
  assert.match(main, /serve-inherited/);
  assert.match(main, /ManagedWorkflowRuntimeConfigurationV1/);
  assert.match(main, /validate_managed_workflow_runtime_configuration/);
  assert.match(main, /configuration\.runtime_artifacts/);
  assert.match(main, /ocr_resources\.configuration\(\)\.clone\(\)/);
  assert.doesNotMatch(main, /AttachmentTextExtractionParserRuntimeV1::new\(None\)/);
  assert.doesNotMatch(main, /ManagedEngineRuntimeConfigurationV1|settings-snapshot-path/);
  assert.doesNotMatch(
    [runtime, eventDecode, blob, clientPort, realtime, parser, ocrResources, main].join('\n'),
    /provider_id|account_id|filename|mime_type|source_path/,
  );
  assert.doesNotMatch(`${eventDecode}\n${realtime}`, /text_utf8/);
});

test('release assembly is a separate unsigned build unit and never launches runtime', async () => {
  const [manifest, source, main] = await Promise.all([
    readFile(new URL('src/attachment-text-extraction-assembly/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-text-extraction-assembly/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-text-extraction-assembly/src/main.rs', BACKEND_ROOT), 'utf8'),
  ]);

  assert.match(manifest, /role = "workflow"/);
  assert.match(manifest, /surface = "assembly"/);
  assert.match(manifest, /makosh-attachment-text-extraction-runtime/);
  assert.match(source, /attachment_text_extraction_module_descriptor_v1/);
  assert.match(source, /attachment_text_extraction_storage_bundle_v1/);
  assert.match(source, /artifact_kind: "module_runtime"/);
  assert.match(source, /artifact_kind: "module_runtime_native_executable"/);
  assert.match(source, /artifact_kind: "module_runtime_read_only_data"/);
  assert.match(source, /bound_module_id: ATTACHMENT_TEXT_EXTRACTION_MODULE_ID_V1/);
  assert.match(source, /ATTACHMENT_TEXT_EXTRACTION_OCR_ENGLISH_ARTIFACT_ID_V1/);
  assert.match(source, /ATTACHMENT_TEXT_EXTRACTION_OCR_RUNNER_ARTIFACT_ID_V1/);
  assert.match(source, /ATTACHMENT_TEXT_EXTRACTION_OCR_RUSSIAN_ARTIFACT_ID_V1/);
  assert.match(source, /artifact_kind: "storage_bundle"/);
  assert.match(source, /create_new\(true\)/);
  assert.doesNotMatch(source, /Command::new|signing|private_key/);
  assert.match(main, /materialize_attachment_text_extraction_release_assembly_v1/);
});

test('OCR native release build is pinned static reproducible and system-fallback negative', async () => {
  const build = await readFile(
    new URL('scripts/build-attachment-text-extraction-ocr-macos.sh', BACKEND_ROOT),
    'utf8',
  );

  assert.match(build, /6e1d56a847e697de07b38619356550e5cf4e8633/);
  assert.match(build, /63aef18d98432b8582a1565e241f7bd2ee9cc8d9/);
  assert.match(build, /51b7f2abdade71cd9bb0e7a373ef2610ec6f9daf/);
  assert.match(build, /2b978915d82377df13fcbb1fb56660195ded868a/);
  assert.match(build, /87416418657359cb625c412a48b6e1d6d41c29bd/);
  assert.match(build, /800fc86838e913fff969b499886c80baeb4ccfd00f0e39906b34aa334f39ab6c/);
  assert.match(build, /7d4322bd2a7749724879683fc3912cb542f19906c83bcc1a52132556427170b2/);
  assert.match(build, /e16e5e036cce1d9ec2b00063cf8b54472625b9e14d893a169e2b0dedeb4df225/);
  assert.match(build, /readonly XCODE_VERSION="26\.6"/);
  assert.match(build, /export PATH="\/usr\/bin:\/bin:\/usr\/sbin:\/sbin"/);
  assert.match(build, /-DCMAKE_FIND_USE_SYSTEM_ENVIRONMENT_PATH=OFF/);
  assert.match(build, /-DCMAKE_IGNORE_PREFIX_PATH="\/opt\/homebrew;\/usr\/local"/);
  assert.match(build, /-DBUILD_SHARED_LIBS=OFF/);
  assert.match(build, /-DDISABLE_ARCHIVE=ON/);
  assert.match(build, /-DDISABLE_CURL=ON/);
  assert.match(build, /otool -L/);
  assert.match(build, /echo "\$\{RUNNER_NAME\}:"/);
  assert.match(build, /-ffile-prefix-map=\$\{isolated_build_root\}=\/usr\/src\/makosh-ocr\/build/);
  assert.match(build, /grep -Ev '\^\(\/usr\/lib\/\|\/System\/Library\/\)'/);
  assert.match(build, /--verify-reproducibility/);
  assert.match(build, /cmp -s/);
  assert.match(build, /"release_eligible": \$\{reproducibility_verified\}/);
  assert.doesNotMatch(
    build,
    /(?:^|\s)brew(?:\s|$)|Command::new|submodule update --remote|--branch\s/m,
  );
});

test('managed conformance stages exact OCR resources through the workflow release boundary', async () => {
  const [setup, flow, signedBundle, harness] = await Promise.all([
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/attachment_text_extraction_managed_setup.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/attachment_text_extraction_managed_flow.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/platform/managed/signed_bundle/manifest.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('scripts/test-authenticated-storage.mjs', BACKEND_ROOT),
      'utf8',
    ),
  ]);

  assert.match(setup, /install_with_runtime_resources/);
  assert.match(setup, /SignedRuntimeResource::native_executable/);
  assert.match(setup, /SignedRuntimeResource::read_only_data/);
  assert.match(setup, /start_reserved_workflow/);
  assert.match(setup, /start_reserved_workflow[\s\S]*&capability_ids/);
  assert.doesNotMatch(setup, /start_reserved_(?:engine|integration|domain)/);
  assert.match(flow, /restart_attachment_text_extraction_runtime_v1/);
  assert.match(flow, /text\.runtime_generation, previous_generation \+ 1/);
  assert.match(flow, /attachment_text_extraction\.ocr_runtime\.v1/);
  assert.match(flow, /text-extraction-malformed-pdf/);
  assert.match(flow, /text-extraction-unsupported/);
  assert.match(flow, /text-extraction-parser-unavailable/);
  assert.match(flow, /replace_attachment_text_parser_identity_v1/);
  assert.match(flow, /stale_parser_identity/);
  assert.match(flow, /text-extraction-stale-custody-proof/);
  assert.match(flow, /wait_for_attachment_text_stale_proof_failure_v1/);
  assert.match(flow, /proof_source_grant_epoch/);
  assert.match(flow, /ModuleRegistrationState::Suspended/);
  assert.match(flow, /approve_module_registration/);
  assert.match(flow, /stop\(blob_binding::BLOB_PROCESS_ID\)/);
  assert.match(flow, /stop\("vault"\)/);
  assert.match(flow, /vault_unavailable_read/);
  assert.match(flow, /AttachmentTextExtractionErrorCodeV1::Unavailable/);
  assert.match(flow, /remove_staged_attachment_text_extraction_ocr_runner_v1/);
  assert.match(
    signedBundle,
    /ModuleRuntimeNativeExecutable[\s\S]*ModuleRuntimeReadOnlyData/,
  );
  assert.match(
    harness,
    /managed_attachment_text_extraction_completes_through_gateway_and_replays_after_restart/,
  );
  assert.match(harness, /MAKOSH_ATTACHMENT_TEXT_EXTRACTION_OCR_RUNNER/);
});
