import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFile, readdir } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const REPOSITORY_ROOT = new URL('../', BACKEND_ROOT);

test('attachment preview is an implemented workflow and not a Communications facade', async () => {
  const [inventorySource, policySource, adr, rendererAdmissionAdr] = await Promise.all([
    readFile(
      new URL('architecture/communications-settings-reconstruction.json', BACKEND_ROOT),
      'utf8',
    ),
    readFile(new URL('architecture/policy.json', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL('docs/adr/ADR-0373-bounded-attachment-preview-workflow.md', REPOSITORY_ROOT),
      'utf8',
    ),
    readFile(
      new URL(
        'docs/adr/ADR-0375-static-preview-renderer-admission-and-failure-semantics.md',
        REPOSITORY_ROOT,
      ),
      'utf8',
    ),
  ]);
  const inventory = JSON.parse(inventorySource);
  const policy = JSON.parse(policySource);
  const slice = inventory.slices.find(({ gate }) => gate === 'attachment_preview_v1');

  assert.deepEqual(slice, {
    gate: 'attachment_preview_v1',
    role: 'workflow',
    owner: 'attachment_preview',
    state: 'implemented',
    dependsOn: ['blob_v1', 'attachment_security_engine_v1'],
  });
  assert.equal(policy.implementation.currentSlice, 'speech_to_text_whisper_admission_v1');
  assert(policy.implementation.ownerInventory.workflows.includes('attachment_preview'));
  assert(policy.implementation.ownerInventory.businessCapabilities.includes(
    'attachment.preview.v1',
  ));
  assert.match(adr, /Состояние реализации: managed multi-format Gateway\/client Blob\/SSE slice/);
  assert.match(adr, /Следующий managed\s+authority gate[\s\S]*source-receipt mismatch/);
  assert.match(adr, /Generated Vue browser[\s\S]*workflow adapter реализован/);
  assert.match(adr, /ADR-0376\/ADR-0377 реализуют explicit owner-authorized exact-byte/);
  assert.match(adr, /replay\s+начинается только после доказанного SSE `OPEN`/);
  assert.match(adr, /Inventory gate `attachment_preview_v1` имеет состояние\s+`implemented`/);
  assert.match(rendererAdmissionAdr, /availability является admission invariant/);
  assert.match(rendererAdmissionAdr, /environment test hook или fake outage не вводятся/);
  assert.match(adr, /Workflow не вызывает Communications или Attachment Security RPC/);
  assert.match(adr, /Legacy base64 `data:` URL не восстанавливается/);
  assert.match(adr, /exact twelve-unit package inventory/);
});

test('public Preview contract separates status ticket and private client blob bytes', async () => {
  const [manifest, controlProto, readProto, source] = await Promise.all([
    readFile(new URL('src/attachment-preview-api/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL(
        'src/attachment-preview-api/proto/makosh/attachment_preview/v1/preview.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/attachment-preview-api/proto/makosh/attachment_preview/read/v1/read.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('src/attachment-preview-api/src/lib.rs', BACKEND_ROOT), 'utf8'),
  ]);

  assert.match(manifest, /role = "workflow"/);
  assert.match(manifest, /owner = "attachment_preview"/);
  assert.match(manifest, /surface = "contract"/);
  assert.doesNotMatch(manifest, /makosh-(?:communications|attachment-security|blob|kernel)/);
  assert.match(controlProto, /rpc Start/);
  assert.match(controlProto, /rpc Get/);
  assert.match(controlProto, /rpc IssueRead/);
  const status = controlProto.slice(
    controlProto.indexOf('message GetAttachmentPreviewResponseV1'),
    controlProto.indexOf('message IssueAttachmentPreviewReadRequestV1'),
  );
  const realtime = controlProto.slice(
    controlProto.indexOf('message AttachmentPreviewStatusChangedV1'),
    controlProto.indexOf('service AttachmentPreviewCommandService'),
  );
  assert.doesNotMatch(status, /ticket|blob|bytes preview|data_url|text =/i);
  assert.doesNotMatch(realtime, /ticket|blob|data_url|text =/i);
  assert.match(readProto, /bytes opaque_read_ticket = 2/);
  assert.doesNotMatch(readProto, /blob_reference|custody|provider|filename|content_type/);
  assert.match(source, /ATTACHMENT_PREVIEW_READ_BLOB_PATH_V1/);
  assert.match(source, /ATTACHMENT_PREVIEW_READ_TICKET_BYTES_V1: usize = 32/);
  assert.match(source, /ATTACHMENT_PREVIEW_READ_TICKET_TTL_SECONDS_V1: i64 = 30/);
  assert.doesNotMatch(
    controlProto,
    /\b(?:provider|account_id|filename|filesystem|source_path|data_url|map)\b/,
  );
});

test('Preview generated browser adapter is app-composed and shares one replayable SSE stream', async () => {
  const [generator, app, route, api, controller, presentation, realtimeHub, navigation] =
    await Promise.all([
      readFile(new URL('frontend/scripts/generate-proto.mjs', REPOSITORY_ROOT), 'utf8'),
      readFile(new URL('frontend/src/app/layout/AppLayoutRoot.vue', REPOSITORY_ROOT), 'utf8'),
      readFile(
        new URL(
          'frontend/src/domains/communications/views/CanonicalCommunicationsRoute.vue',
          REPOSITORY_ROOT,
        ),
        'utf8',
      ),
      readFile(
        new URL(
          'frontend/src/workflows/attachment-preview/api/attachmentPreview.ts',
          REPOSITORY_ROOT,
        ),
        'utf8',
      ),
      readFile(
        new URL(
          'frontend/src/workflows/attachment-preview/queries/useAttachmentPreview.ts',
          REPOSITORY_ROOT,
        ),
        'utf8',
      ),
      readFile(
        new URL(
          'frontend/src/workflows/attachment-preview/presentation/AttachmentPreviewPanel.vue',
          REPOSITORY_ROOT,
        ),
        'utf8',
      ),
      readFile(
        new URL(
          'frontend/src/platform/gateway/browserGatewayRealtimeHub.ts',
          REPOSITORY_ROOT,
        ),
        'utf8',
      ),
      readFile(
        new URL(
          'frontend/src/app/queries/useClientNavigationSurface.ts',
          REPOSITORY_ROOT,
        ),
        'utf8',
      ),
    ]);

  assert.match(generator, /attachment-preview-api/);
  assert.match(generator, /attachment_preview[\s\S]*preview\.proto/);
  assert.match(app, /AttachmentPreviewWorkflow/);
  assert.match(app, /attachment_preview\.client\.v1/);
  assert.match(route, /canonicalAttachmentSelected/);
  assert.doesNotMatch(route, /workflows\/attachment-preview|attachmentPreviewClient/);
  assert.match(api, /getAttachmentPreviewCommandClient/);
  assert.match(api, /getAttachmentPreviewQueryClient/);
  assert.match(api, /getAttachmentPreviewTicketClient/);
  assert.match(api, /getBrowserGatewayRealtimeHub/);
  assert.match(api, /BrowserGatewayFetch/);
  assert.doesNotMatch(api, /domains\/communications|integrations\/(?:mail|telegram|whatsapp|zulip)/);
  assert.match(controller, /subscribeAttachmentPreviewStatus/);
  assert.doesNotMatch(controller, /setInterval\(|setTimeout\(|poll/i);
  assert.match(realtimeHub, /sharedHub/);
  assert.match(navigation, /getBrowserGatewayRealtimeHub\(\)\.subscribe/);
  assert.doesNotMatch(presentation, /fetch\(|v-html|connect\//);
});

test('Preview logs errors health and telemetry expose only fixed-shape technical state', async () => {
  const [admission, runtimeMain, diagnostics, runtime, controlProto, ownerSources, flow, formats] =
    await Promise.all([
      readFile(new URL('src/attachment-preview-runtime/src/admission.rs', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/attachment-preview-runtime/src/main.rs', BACKEND_ROOT), 'utf8'),
      readFile(
        new URL('src/attachment-preview-runtime/src/diagnostics.rs', BACKEND_ROOT),
        'utf8',
      ),
      readFile(new URL('src/attachment-preview-runtime/src/runtime.rs', BACKEND_ROOT), 'utf8'),
      readFile(
        new URL(
          'src/platform/runtime_protocol/proto/makosh/runtime/v1/managed_runtime_control.proto',
          BACKEND_ROOT,
        ),
        'utf8',
      ),
      readPreviewProductionSources(),
      readFile(
        new URL(
          'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/attachment_preview_managed_flow.rs',
          BACKEND_ROOT,
        ),
        'utf8',
      ),
      readFile(
        new URL(
          'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/attachment_preview_managed_formats.rs',
          BACKEND_ROOT,
        ),
        'utf8',
      ),
    ]);

  const descriptor = admission.slice(0, admission.indexOf('#[cfg(test)]'));
  assert.doesNotMatch(descriptor, /TelemetrySignal|telemetry_signal/);
  assert.doesNotMatch(
    ownerSources,
    /(?:["'`]\/(?:health|ready)|\bHealth(?:Check|Response|Status)\b|\bfn\s+(?:health|readiness)\s*\()/i,
  );

  const ready = controlProto.slice(
    controlProto.indexOf('message ManagedRuntimeReadyRequestV1'),
    controlProto.indexOf('message ManagedRuntimeEventCredentialRequestV1'),
  );
  assert.match(ready, /string registration_id = 1/);
  assert.match(ready, /uint64 runtime_generation = 2/);
  assert.match(ready, /uint64 grant_epoch = 3/);
  assert.doesNotMatch(
    ready,
    /(?:blob|receipt|proof|ticket|provider|account|filename|content|payload|bytes)/i,
  );

  assert.match(runtime, /pub const fn sanitized_reason_code\(self\)/);
  assert.match(diagnostics, /AttachmentPreviewDiagnosticStageV1/);
  assert.match(diagnostics, /reason=\{\}/);
  assert.doesNotMatch(`${runtimeMain}\n${diagnostics}`, /error=\{error:\?\}/);
  assert.doesNotMatch(
    diagnostics.slice(0, diagnostics.indexOf('#[cfg(test)]')),
    /(?:source|blob|receipt|proof|ticket|provider|account|filename|content|payload)/i,
  );
  assert.match(flow, /assert_private_preview_source_absent_v1/);
  assert.match(formats, /assert_private_source_absent_v1/);
});

test('pure Preview core owns evidence join lifecycle and output policy only', async () => {
  const [manifest, source, join, lifecycle, policy] = await Promise.all([
    readFile(new URL('src/attachment-preview-core/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-preview-core/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-preview-core/src/join.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-preview-core/src/lifecycle.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-preview-core/src/policy.rs', BACKEND_ROOT), 'utf8'),
  ]);

  assert.match(manifest, /makosh-attachment-preview-api/);
  assert.doesNotMatch(
    manifest,
    /makosh-(?:communications|attachment-security|blob|events|runtime|storage|kernel)|\b(?:sqlx|tokio)\s*=/,
  );
  assert.match(join, /AttachmentPreviewCustodyDelegationIntentV1/);
  const intent = join.slice(
    join.indexOf('struct AttachmentPreviewCustodyDelegationIntentV1'),
    join.indexOf('struct AttachmentPreviewEvidenceJoinV1'),
  );
  assert.doesNotMatch(
    intent,
    /\b(?:blob_reference_id|declared_size|receipt_sha256|custody_transfer_source_proof)\b/,
  );
  assert.match(lifecycle, /AttachmentPreviewStateV1/);
  assert.match(lifecycle, /AttachmentPreviewStatusV1/);
  assert.match(lifecycle, /transition_attachment_preview_status_v1/);
  assert.match(join, /source_receipt_sha256/);
  assert.match(join, /expected_state.*BlobAdmitted/s);
  assert.match(policy, /validate_preview_output_v1/);
  assert.doesNotMatch(
    `${source}\n${join}\n${lifecycle}\n${policy}`,
    /TcpStream|File::|sqlx|postgres|nats|jetstream|makosh_communications|makosh_attachment_security/,
  );
});

async function readPreviewProductionSources() {
  const roots = [
    'src/attachment-preview-api/src/',
    'src/attachment-preview-core/src/',
    'src/attachment-preview-docx/src/',
    'src/attachment-preview-image/src/',
    'src/attachment-preview-ingress/src/',
    'src/attachment-preview-media/src/',
    'src/attachment-preview-pdf/src/',
    'src/attachment-preview-persistence/src/',
    'src/attachment-preview-renderer-contract/src/',
    'src/attachment-preview-runtime/src/',
    'src/attachment-preview-text/src/',
  ];
  const sources = await Promise.all(roots.map((path) => readRustTree(new URL(path, BACKEND_ROOT))));
  return sources.flat().map(stripRustTestModule).join('\n');
}

async function readRustTree(directory) {
  const entries = await readdir(directory, { withFileTypes: true });
  const sources = await Promise.all(entries.map(async (entry) => {
    const path = new URL(entry.name, directory);
    if (entry.isDirectory()) return readRustTree(new URL(`${entry.name}/`, directory));
    if (entry.isFile() && entry.name.endsWith('.rs')) return [await readFile(path, 'utf8')];
    return [];
  }));
  return sources.flat();
}

function stripRustTestModule(source) {
  const marker = source.indexOf('#[cfg(test)]');
  return marker === -1 ? source : source.slice(0, marker);
}

test('Preview persistence owns replay jobs artifacts tickets and realtime without private content', async () => {
  const [manifest, source, model, evidence, custody, jobs, tickets, repository, schema, migration] = await Promise.all([
    readFile(new URL('src/attachment-preview-persistence/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-preview-persistence/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-preview-persistence/src/model.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-preview-persistence/src/evidence.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-preview-persistence/src/custody.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-preview-persistence/src/jobs.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-preview-persistence/src/tickets.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-preview-persistence/src/repository.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-preview-persistence/src/schema.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-preview-persistence/migrations/0001_attachment_preview.sql', BACKEND_ROOT), 'utf8'),
  ]);
  const implementation = `${source}\n${model}\n${evidence}\n${custody}\n${jobs}\n${tickets}\n${repository}\n${schema}`;
  assert.match(manifest, /role = "workflow"/);
  assert.match(manifest, /owner = "attachment_preview"/);
  assert.match(manifest, /surface = "persistence"/);
  for (const dependency of [
    'makosh-attachment-preview-api',
    'makosh-attachment-preview-core',
    'makosh-attachment-preview-ingress',
    'makosh-storage-protocol',
  ]) assert.match(manifest, new RegExp(dependency));
  assert.doesNotMatch(
    manifest,
    /makosh-(?:communications|attachment-security|blob|events|runtime|kernel|attachment-text-extraction|attachment-archive-inspection)/,
  );
  for (const table of [
    'attachment_preview_runs',
    'attachment_preview_event_inbox',
    'attachment_preview_scan_candidates',
    'attachment_preview_safety_facts',
    'attachment_preview_custody_outbox',
    'attachment_preview_custody_result_inbox',
    'attachment_preview_jobs',
    'attachment_preview_artifacts',
    'attachment_preview_read_tickets',
    'attachment_preview_realtime',
  ]) assert.match(migration, new RegExp(`makosh_data\\.${table}`));
  assert.match(evidence, /AttachmentPreviewEvidenceJoinV1/);
  assert.match(custody, /exact_envelope_bytes/);
  assert.match(jobs, /FOR UPDATE SKIP LOCKED/);
  assert.match(jobs, /runtime_generation/);
  assert.match(jobs, /grant_epoch/);
  assert.match(jobs, /renderer_identity_sha256/);
  assert.match(tickets, /ticket_sha256/);
  assert.match(tickets, /device_actor_sha256/);
  assert.match(tickets, /used_at_unix_seconds/);
  assert.match(repository, /attachment_preview_realtime/);
  assert.doesNotMatch(
    `${migration}\n${source}\n${model}\n${evidence}\n${custody}\n${jobs}\n${tickets}\n${repository}`,
    /ticket_plaintext|source_bytes|preview_bytes|text_utf8|provider_id|account_id|filename|mime_type|filesystem_path/,
  );
  assert.doesNotMatch(
    implementation,
    /TcpStream|Command::|makosh_communications|makosh_attachment_security|makosh_blob_/,
  );
  assert.doesNotMatch(evidence, /ticket_sha256|target_receipt_sha256/);
  assert.doesNotMatch(tickets, /custody_transfer_source_proof|exact_envelope_bytes/);
});

test('target-owned Preview ingress carries event custody without caller authority', async () => {
  const [manifest, proto, source, envelope] = await Promise.all([
    readFile(new URL('src/attachment-preview-ingress/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL(
        'src/attachment-preview-ingress/proto/makosh/attachment_preview/ingress/v1/custody_delegation.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('src/attachment-preview-ingress/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-preview-ingress/src/envelope.rs', BACKEND_ROOT), 'utf8'),
  ]);

  assert.match(manifest, /owner = "attachment_preview"/);
  assert.match(manifest, /surface = "contract"/);
  assert.doesNotMatch(
    manifest,
    /makosh-(?:communications|attachment-security|blob|kernel|attachment-preview-(?:api|core|runtime|persistence|assembly))/,
  );
  assert.match(proto, /message RequestAttachmentPreviewCustodyDelegationV1/);
  assert.match(proto, /message AttachmentPreviewCustodyDelegatedV1/);
  const request = proto.slice(
    proto.indexOf('message RequestAttachmentPreviewCustodyDelegationV1'),
    proto.indexOf('message AttachmentPreviewCustodyDelegatedV1'),
  );
  assert.doesNotMatch(
    request,
    /\b(?:source_reference_id|custody_transfer_source_proof|target_owner_id|target_module_id|target_capability_id|provider_id|filename|content_type)\b/,
  );
  assert.match(source, /ATTACHMENT_SECURITY_PREVIEW_DELEGATION_CAPABILITY_ID_V1/);
  assert.match(source, /ATTACHMENT_PREVIEW_BLOB_TARGET_OWNER_ID_V1/);
  assert.match(envelope, /DurableEnvelopeV1/);
  assert.match(envelope, /ResultMetadataV1/);
});

test('renderer contract is byte-only and metadata cannot select behavior', async () => {
  const [manifest, source] = await Promise.all([
    readFile(
      new URL('src/attachment-preview-renderer-contract/Cargo.toml', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-preview-renderer-contract/src/lib.rs', BACKEND_ROOT),
      'utf8',
    ),
  ]);

  assert.match(manifest, /owner = "attachment_preview"/);
  assert.match(manifest, /surface = "contract"/);
  assert.doesNotMatch(
    manifest,
    /makosh-(?:communications|attachment-security|blob|events|runtime|storage|kernel)/,
  );
  assert.match(source, /detect_attachment_preview_source_format_v1/);
  assert.match(source, /source_bytes: &'a \[u8\]/);
  assert.match(source, /DocxContainerCandidate/);
  assert.match(source, /ATTACHMENT_PREVIEW_MAX_SOURCE_BYTES_V1/);
  assert.doesNotMatch(source, /\b(?:Unavailable|TimedOut)\b/);
  assert.doesNotMatch(
    source,
    /\b(?:filename|content_type_hint|provider|account_id|filesystem|source_path|url)\b/,
  );
});

test('safe text image and media adapters are three isolated byte-only units', async () => {
  const [textManifest, text, imageManifest, image, mediaManifest, media] = await Promise.all([
    readFile(new URL('src/attachment-preview-text/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-preview-text/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-preview-image/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-preview-image/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-preview-media/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-preview-media/src/lib.rs', BACKEND_ROOT), 'utf8'),
  ]);
  for (const manifest of [textManifest, imageManifest, mediaManifest]) {
    assert.match(manifest, /makosh-attachment-preview-renderer-contract/);
    assert.doesNotMatch(
      manifest,
      /makosh-(?:communications|attachment-security|blob|events|runtime|storage|kernel)|\b(?:sqlx|tokio)\s*=/,
    );
  }
  assert.doesNotMatch(textManifest, /\bimage\s*=/);
  assert.doesNotMatch(mediaManifest, /\bimage\s*=/);
  assert.match(imageManifest, /image = \{ version = "=0\.25\.9", default-features = false/);
  assert.match(text, /normalized_visible_utf8_v1/);
  assert.match(text, /ATTACHMENT_PREVIEW_MAX_TEXT_BYTES_V1/);
  assert.match(image, /write_to\(&mut output, ImageFormat::Png\)/);
  assert.match(image, /ATTACHMENT_PREVIEW_MAX_IMAGE_PIXELS_V1/);
  assert.match(image, /has_exact_png_boundary_v1/);
  assert.match(image, /png_polyglot_with_trailing_payload_fails_closed/);
  assert.match(media, /validate_mp3_v1/);
  assert.match(media, /validate_mp4_v1/);
  assert.match(media, /allowed_mp4_brand/);
  assert.doesNotMatch(
    `${text}\n${image}\n${media}`,
    /\b(?:filename|provider|account_id|filesystem|source_path|data_url|url)\b/,
  );
});

test('PDF adapter rasterizes one bounded page without native or owner authority', async () => {
  const [manifest, source] = await Promise.all([
    readFile(new URL('src/attachment-preview-pdf/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-preview-pdf/src/lib.rs', BACKEND_ROOT), 'utf8'),
  ]);
  assert.match(manifest, /hayro = \{ version = "=0\.7\.1", default-features = true \}/);
  assert.match(manifest, /image = \{ version = "=0\.25\.9", default-features = false, features = \["png"\] \}/);
  assert.doesNotMatch(
    manifest,
    /makosh-(?:communications|attachment-security|blob|events|runtime|storage|kernel)|\b(?:sqlx|tokio)\s*=/,
  );
  assert.match(source, /render_first_page_v1/);
  assert.match(source, /MAX_RENDER_DIMENSION_V1/);
  assert.match(source, /ATTACHMENT_PREVIEW_MAX_IMAGE_PIXELS_V1/);
  assert.match(source, /FORBIDDEN_ACTIVE_MARKERS_V1/);
  assert.match(source, /oversized_source_fails_before_pdf_parsing/);
  assert.match(source, /catch_unwind/);
  assert.match(source, /AttachmentPreviewKindV1::Document/);
  assert.doesNotMatch(
    source,
    /Command::|TcpStream|File::|filesystem|source_path|provider|account_id|filename|content_type_hint|data_url|url/,
  );
});

test('DOCX adapter rebuilds a bounded fixed-font card without external resources', async () => {
  const [manifest, entrypoint, container, documentText, card, fontLicense, fontBytes] = await Promise.all([
    readFile(new URL('src/attachment-preview-docx/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-preview-docx/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-preview-docx/src/container.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-preview-docx/src/document_text.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-preview-docx/src/card.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-preview-docx/assets/DejaVu-LICENSE.txt', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-preview-docx/assets/DejaVuSans.ttf', BACKEND_ROOT)),
  ]);
  const source = `${entrypoint}\n${container}\n${documentText}\n${card}`;
  assert.match(manifest, /swash = \{ version = "=0\.2\.10", default-features = false, features = \["std", "render"\] \}/);
  assert.match(manifest, /quick-xml = \{ version = "=0\.41\.0", default-features = false \}/);
  assert.match(manifest, /zip = \{ version = "=6\.0\.0", default-features = false, features = \["deflate-flate2-zlib-rs"\] \}/);
  assert.doesNotMatch(
    manifest,
    /makosh-(?:communications|attachment-security|blob|events|runtime|storage|kernel|attachment-text-extraction)|\b(?:sqlx|tokio|reqwest)\s*=/,
  );
  assert.match(source, /include_bytes!\("\.\.\/assets\/DejaVuSans\.ttf"\)/);
  assert.match(source, /FIXED_FONT_SHA256_V1/);
  assert.equal(
    createHash('sha256').update(fontBytes).digest('hex'),
    '7da195a74c55bef988d0d48f9508bd5d849425c1770dba5d7bfc6ce9ed848954',
  );
  assert.match(source, /MAX_ZIP_ENTRIES_V1/);
  assert.match(source, /MAX_ZIP_UNCOMPRESSED_BYTES_V1/);
  assert.match(source, /MAX_DOCUMENT_XML_BYTES_V1/);
  assert.match(source, /validate_relationships_v1/);
  assert.match(source, /target_mode[\s\S]*external/);
  assert.match(source, /FORBIDDEN_ENTRY_MARKERS_V1/);
  assert.match(source, /FORBIDDEN_XML_MARKERS_V1/);
  assert.match(source, /catch_unwind/);
  assert.match(source, /AttachmentPreviewKindV1::Document/);
  assert.match(entrypoint, /mod card;/);
  assert.match(entrypoint, /mod container;/);
  assert.match(entrypoint, /mod document_text;/);
  assert.doesNotMatch(container, /\b(?:image|swash)::|render_docx_card_v1/);
  assert.doesNotMatch(card, /\b(?:ZipArchive|quick_xml)|read_bounded_docx_v1/);
  assert.doesNotMatch(documentText, /\b(?:image|swash)::|ZipArchive|render_docx_card_v1/);
  assert.match(fontLicense, /Bitstream Vera Fonts Copyright/);
  assert.doesNotMatch(
    source,
    /Command::|TcpStream|File::|filesystem|source_path|provider|account_id|filename|content_type_hint|data_url|url/,
  );
});

test('managed Preview runtime composes public contracts without owning assembly or another owner implementation', async () => {
  const [manifest, entrypoint, admission, runtime, clientPort, realtime, renderer, blob] = await Promise.all([
    readFile(new URL('src/attachment-preview-runtime/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-preview-runtime/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-preview-runtime/src/admission.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-preview-runtime/src/runtime.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-preview-runtime/src/client_port.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-preview-runtime/src/client_realtime.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-preview-runtime/src/renderer.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-preview-runtime/src/blob.rs', BACKEND_ROOT), 'utf8'),
  ]);
  const source = `${entrypoint}\n${admission}\n${runtime}\n${clientPort}\n${realtime}\n${renderer}\n${blob}`;
  assert.match(manifest, /role = "workflow"/);
  assert.match(manifest, /owner = "attachment_preview"/);
  assert.match(manifest, /surface = "runtime"/);
  assert.match(manifest, /\[\[bin\]\]/);
  assert.doesNotMatch(
    manifest,
    /makosh-(?:communications-domain|attachment-security-engine|attachment-preview-assembly|attachment-text-extraction|attachment-archive-inspection)/,
  );
  assert.match(admission, /ClientBlob/);
  assert.match(admission, /ATTACHMENT_PREVIEW_MAX_VIDEO_BYTES_V1/);
  assert.match(runtime, /try_receive_runtime_pull_delivery/);
  assert.match(runtime, /None => return Ok\(false\)/);
  assert.match(runtime, /dispatch_attachment_preview_client_request_v1/);
  assert.match(runtime, /AttachmentPreviewNestedRequestDispatcherV1/);
  assert.match(
    runtime,
    /publish_pending\([\s\S]*&mut dispatcher[\s\S]*\)\.await/,
  );
  assert.doesNotMatch(runtime, /RejectManagedControlRequestsV2/);
  assert.match(runtime, /attachment_preview_renderer_identity_v1/);
  assert.match(clientPort, /ModuleClientBlobAuthorizationV1/);
  assert.match(clientPort, /redeem_read_ticket/);
  assert.match(realtime, /PublishClientRealtime/);
  assert.match(renderer, /detect_attachment_preview_source_format_v1/);
  assert.match(blob, /makosh\.attachment-preview\.derived-blob\.v1/);
  assert.doesNotMatch(
    source,
    /sqlx::|TcpListener|Command::|data_url|ticket_plaintext|provider_id|account_id|filename|content_type_hint/,
  );
});

test('Preview release assembly is a separate unsigned workflow build unit', async () => {
  const [manifest, source, main] = await Promise.all([
    readFile(new URL('src/attachment-preview-assembly/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-preview-assembly/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-preview-assembly/src/main.rs', BACKEND_ROOT), 'utf8'),
  ]);
  assert.match(manifest, /role = "workflow"/);
  assert.match(manifest, /owner = "attachment_preview"/);
  assert.match(manifest, /surface = "assembly"/);
  assert.match(source, /attachment_preview_module_descriptor_v1/);
  assert.match(source, /attachment_preview_settings_schema_v1/);
  assert.match(source, /attachment_preview_storage_bundle_v1/);
  assert.match(source, /artifact_kind: "module_runtime"/);
  assert.match(source, /artifact_kind: "storage_bundle"/);
  assert.match(source, /create_new\(true\)/);
  assert.match(main, /materialize_attachment_preview_release_assembly_v1/);
  assert.doesNotMatch(source, /Command::new|private_key|renderer\.render|serve-inherited/);
});

test('development release builds and signs the exact Preview assembly fragment', async () => {
  const [release, developmentAssembly] = await Promise.all([
    readFile(new URL('scripts/materialize-dev-release.sh', BACKEND_ROOT), 'utf8'),
    readFile(new URL('development/assembly/src/main.rs', BACKEND_ROOT), 'utf8'),
  ]);

  assert.match(release, /--package makosh-attachment-preview-runtime/);
  assert.match(release, /--package makosh-attachment-preview-assembly/);
  assert.match(
    release,
    /makosh-attachment-preview-assembly"[\s\S]*--runtime "\$cargo_target_dir\/debug\/makosh-attachment-preview-runtime"/,
  );
  assert.match(
    release,
    /--artifact-fragment "\$attachment_preview_assembly\/attachment-preview\.release-artifacts\.json"/,
  );
  assert.doesNotMatch(
    release,
    /attachment_preview_assembly=.*(?:communications|attachment-security)/,
  );
  assert.match(developmentAssembly, /ATTACHMENT_PREVIEW_RUNTIME_ARTIFACT/);
  assert.match(developmentAssembly, /ATTACHMENT_PREVIEW_STORAGE_ARTIFACT/);
  assert.match(
    developmentAssembly,
    /runtime_artifact_id: ATTACHMENT_PREVIEW_RUNTIME_ARTIFACT[\s\S]*runtime_kind: ModuleRuntimeKindV1::Workflow/,
  );
});

test('Preview has an authenticated exact signed managed admission gate', async () => {
  const [setup, gateway, flow, formats, persistence, harness] = await Promise.all([
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/attachment_preview_managed_setup.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/attachment_preview_gateway_fixture.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/attachment_preview_managed_flow.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/attachment_preview_managed_formats.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/attachment_preview_persistence_fixture.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('scripts/test-authenticated-storage.mjs', BACKEND_ROOT), 'utf8'),
  ]);

  assert.match(setup, /SignedRuntimeArtifact::new/);
  assert.match(setup, /attachment_preview_storage_bundle_v1/);
  assert.match(setup, /issue_managed/);
  assert.match(setup, /to_managed_runtime_configuration/);
  assert.match(setup, /start_reserved_workflow/);
  assert.match(flow, /managed_attachment_preview_reaches_gateway_blob_sse_and_replays_after_restart/);
  assert.match(flow, /wait_for_ready_attachment_preview_v1/);
  assert.match(flow, /set_authenticated_nats_container_running\(false\)/);
  assert.match(flow, /set_authenticated_nats_container_running\(true\)/);
  assert.match(flow, /authenticate_secondary_gateway_router/);
  assert.match(flow, /StatusCode::NOT_FOUND/);
  assert.match(flow, /stale renderer identity/);
  assert.match(flow, /stale state revision/);
  assert.match(flow, /expired ticket/);
  assert.match(flow, /stale runtime generation/);
  assert.match(flow, /stale Attachment Preview grant epoch/);
  assert.match(flow, /stale Preview custody proof/);
  assert.match(flow, /mismatched Preview source receipt/);
  assert.match(flow, /Blob outage/);
  assert.match(flow, /Vault outage/);
  assert.match(flow, /read_terminal_attachment_preview_sse_event_after_v1/);
  assert.match(gateway, /header\("last-event-id", after_cursor\)/);
  assert.match(flow, /Some\(&first_cursor\)/);
  assert.match(flow, /assert_ne!\(continued_event\.cursor, first_cursor\)/);
  assert.match(flow, /Last-Event-ID continuation/);
  assert.match(formats, /preview-bad-pdf/);
  assert.match(formats, /preview-active-pdf/);
  assert.match(formats, /preview-bad-png/);
  assert.match(formats, /preview-unsupported/);
  assert.match(formats, /preview-polyglot/);
  assert.match(formats, /preview-oversized/);
  assert.match(formats, /assert_private_source_absent_v1/);
  assert.match(persistence, /attachment_preview_custody_outbox/);
  assert.match(persistence, /attachment_preview_artifacts/);
  assert.match(persistence, /replace_attachment_preview_job_source_receipt_v1/);
  assert.match(persistence, /expire_attachment_preview_job_lease_v1/);
  assert.match(harness, /-p'[\s\S]*makosh-attachment-preview-runtime/);
  assert.match(harness, /MAKOSH_ATTACHMENT_PREVIEW_RUNTIME_BIN/);
  assert.match(harness, /managed_attachment_preview_reaches_gateway_blob_sse_and_replays_after_restart/);
  assert.doesNotMatch(`${setup}\n${flow}\n${formats}\n${persistence}`, /makosh_communications_(?:domain|persistence)|makosh_attachment_security_(?:core|persistence|runtime)/);
});
