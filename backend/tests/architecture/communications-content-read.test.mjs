import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

const paths = {
  inventory: new URL('architecture/communications-settings-reconstruction.json', BACKEND_ROOT),
  manifest: new URL('src/communications-content-api/Cargo.toml', BACKEND_ROOT),
  ticketProto: new URL(
    'src/communications-content-api/proto/makosh/communications/content/ticket/v1/ticket.proto',
    BACKEND_ROOT,
  ),
  readProto: new URL(
    'src/communications-content-api/proto/makosh/communications/content/read/v1/read.proto',
    BACKEND_ROOT,
  ),
  admission: new URL('src/communications-runtime/src/admission.rs', BACKEND_ROOT),
  persistence: new URL('src/communications-persistence/src/content_read.rs', BACKEND_ROOT),
  tickets: new URL('src/communications-runtime/src/content_ticket_store.rs', BACKEND_ROOT),
  blobPort: new URL('src/communications-runtime/src/content_blob_client_port.rs', BACKEND_ROOT),
  managed: new URL(
    'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker.rs',
    BACKEND_ROOT,
  ),
  adr: new URL(
    'docs/adr/ADR-0315-communications-message-body-content-read.md',
    PROJECT_ROOT,
  ),
  generator: new URL('frontend/scripts/generate-proto.mjs', PROJECT_ROOT),
  frontendAdapter: new URL(
    'frontend/src/domains/communications/queries/canonicalCommunicationsContent.ts',
    PROJECT_ROOT,
  ),
  frontendController: new URL(
    'frontend/src/domains/communications/queries/useCanonicalCommunicationContent.ts',
    PROJECT_ROOT,
  ),
  frontendModel: new URL(
    'frontend/src/domains/communications/presentation/canonicalCommunicationContentModel.ts',
    PROJECT_ROOT,
  ),
  frontendPresentation: new URL(
    'frontend/src/domains/communications/presentation/CanonicalCommunicationContent.vue',
    PROJECT_ROOT,
  ),
};

test('Communications content is an exact separately admitted owner capability', async () => {
  const [inventorySource, manifest, ticketProto, readProto, admission, adr] = await Promise.all([
    readFile(paths.inventory, 'utf8'),
    readFile(paths.manifest, 'utf8'),
    readFile(paths.ticketProto, 'utf8'),
    readFile(paths.readProto, 'utf8'),
    readFile(paths.admission, 'utf8'),
    readFile(paths.adr, 'utf8'),
  ]);
  const inventory = JSON.parse(inventorySource);
  const gate = inventory.slices.find((slice) => slice.gate === 'communications_content_read_v1');

  assert.deepEqual(gate, {
    gate: 'communications_content_read_v1',
    role: 'domain',
    owner: 'communications',
    state: 'implemented',
    dependsOn: ['communications_canonical_read_v2', 'communications.blob.v1'],
  });
  assert.match(adr, /Состояние реализации: implemented/);
  assert.match(manifest, /name = "makosh-communications-content-api"/);
  assert.match(manifest, /surface = "contract"/);
  assert.match(ticketProto, /IssueMessageBodyRead\(IssueMessageBodyReadRequestV1\)/);
  assert.match(readProto, /bytes opaque_read_capability = 2/);
  for (const proto of [ticketProto, readProto]) {
    assert.doesNotMatch(
      proto.replaceAll(/\/\/.*$/gm, ''),
      /blob_ref|provider|digest|locator|credential/i,
    );
  }
  assert.match(admission, /COMMUNICATIONS_CONTENT_CAPABILITY_ID.*"communications\.content\.v1"/s);
  assert.match(admission, /ProvidedSurfaceKindV1::ClientRpc/);
  assert.match(admission, /ProvidedSurfaceKindV1::ClientBlob/);
  assert.match(
    admission,
    /communications_content_capability_v1[\s\S]*allowed_operations:\s*vec!\[BlobQuotaOperationV1::ReadRange as i32\]/,
  );
});

test('Communications content tickets stay one-use owner-local and current-receipt bound', async () => {
  const [persistence, tickets, blobPort, managed] = await Promise.all([
    readFile(paths.persistence, 'utf8'),
    readFile(paths.tickets, 'utf8'),
    readFile(paths.blobPort, 'utf8'),
    readFile(paths.managed, 'utf8'),
  ]);

  assert.match(persistence, /message\.lifecycle_state = 1/);
  assert.match(persistence, /message\.canonical_body_state = 4/);
  assert.match(persistence, /evidence\.body_state = 4/);
  assert.match(tickets, /getrandom::fill/);
  assert.match(tickets, /const TICKET_TTL_SECONDS: i64 = 30/);
  assert.match(tickets, /tickets\.remove\(position\)/);
  assert.doesNotMatch(tickets, /HashMap|BTreeMap|sqlx|ControlStore/);
  assert.match(blobPort, /current_receipt_if_unchanged/);
  assert.match(blobPort, /edit_delete_or_replaced_receipt_invalidates_the_ticket/);
  assert.match(managed, /CONTENT_TICKET_CONNECT_PATH_V1/);
  assert.match(managed, /CONTENT_READ_BLOB_PATH_V1/);
  assert.match(managed, /assert_eq!\(read\(\)\.status\(\), hyper::StatusCode::NOT_FOUND\)/);
  assert.match(managed, /headers\(\)\.get\("digest"\)\.is_none/);
});

test('Communications frontend reads generated canonical content with no provider fallback', async () => {
  const [generator, adapter, controller, model, presentation] = await Promise.all([
    readFile(paths.generator, 'utf8'),
    readFile(paths.frontendAdapter, 'utf8'),
    readFile(paths.frontendController, 'utf8'),
    readFile(paths.frontendModel, 'utf8'),
    readFile(paths.frontendPresentation, 'utf8'),
  ]);

  assert.match(generator, /communications-content-api/);
  assert.match(adapter, /getCommunicationsContentTicketConnectClient/);
  assert.match(adapter, /BrowserGatewayFetch/);
  assert.match(adapter, /application\/octet-stream/);
  assert.match(controller, /AbortController/);
  assert.match(controller, /requestGeneration !== generation/);
  assert.match(model, /TextDecoder\('utf-8', \{ fatal: true \}\)/);
  assert.match(presentation, /<pre v-if="model\.status === 'ready'">/);
  assert.doesNotMatch(presentation, /v-html|fetch\(|connect\//);
  for (const source of [adapter, controller, model, presentation]) {
    assert.doesNotMatch(source, /integrations\/(mail|telegram|whatsapp|zulip)/);
    assert.doesNotMatch(source, /blobRef|providerLocator|x-blob-reference/);
  }
});
