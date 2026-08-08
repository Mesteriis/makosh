import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

const paths = {
  inventory: new URL('architecture/communications-settings-reconstruction.json', BACKEND_ROOT),
  proto: new URL(
    'src/communications-api/proto/makosh/communications/query/v1/query.proto',
    BACKEND_ROOT,
  ),
  admission: new URL('src/communications-runtime/src/admission.rs', BACKEND_ROOT),
  moduleQuery: new URL(
    'src/communications-runtime/src/query_module_port.rs',
    BACKEND_ROOT,
  ),
  cursor: new URL('src/communications-runtime/src/canonical_read_cursor.rs', BACKEND_ROOT),
  persistence: new URL('src/communications-persistence/src/canonical_read.rs', BACKEND_ROOT),
  migration: new URL(
    'src/communications-persistence/migrations/0011_communications_canonical_read_v2_indexes.sql',
    BACKEND_ROOT,
  ),
  storageBundle: new URL('src/communications-persistence/src/schema/bundle.rs', BACKEND_ROOT),
  managed: new URL(
    'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/communications_setup.rs',
    BACKEND_ROOT,
  ),
  adr: new URL(
    'docs/adr/ADR-0313-communications-canonical-read-v2-detail-and-pagination.md',
    PROJECT_ROOT,
  ),
  frontendRead: new URL(
    'frontend/src/domains/communications/queries/canonicalCommunicationsRead.ts',
    PROJECT_ROOT,
  ),
  frontendDetail: new URL(
    'frontend/src/domains/communications/queries/canonicalCommunicationsDetail.ts',
    PROJECT_ROOT,
  ),
  frontendController: new URL(
    'frontend/src/domains/communications/queries/useCanonicalCommunicationDetail.ts',
    PROJECT_ROOT,
  ),
  frontendPresentation: new URL(
    'frontend/src/domains/communications/presentation/CanonicalCommunicationDetail.vue',
    PROJECT_ROOT,
  ),
};

test('Communications canonical read v2 is one exact owner contract and admitted revision', async () => {
  const [inventorySource, proto, admission, moduleQuery, adr] = await Promise.all([
    readFile(paths.inventory, 'utf8'),
    readFile(paths.proto, 'utf8'),
    readFile(paths.admission, 'utf8'),
    readFile(paths.moduleQuery, 'utf8'),
    readFile(paths.adr, 'utf8'),
  ]);
  const inventory = JSON.parse(inventorySource);
  const gate = inventory.slices.find((slice) => slice.gate === 'communications_canonical_read_v2');

  assert.deepEqual(gate, {
    gate: 'communications_canonical_read_v2',
    role: 'domain',
    owner: 'communications',
    state: 'implemented',
    dependsOn: ['client_gateway_v1', 'communications.query.v1'],
  });
  assert.match(adr, /Состояние реализации: implemented/);
  assert.match(proto, /GetMessageRequestV1 get_message = 12/);
  assert.match(proto, /GetMessageResponseV1 get_message = 11/);
  assert.equal((proto.match(/bytes cursor =/g) ?? []).length, 8);
  assert.equal((proto.match(/bytes next_cursor =/g) ?? []).length, 8);
  assert.match(admission, /capability_id: COMMUNICATIONS_QUERY_CAPABILITY_ID[\s\S]*capability_revision: 3/);
  assert.match(admission, /descriptor_revision: 8/);
  assert.match(admission, /ProvidedSurfaceKindV1::QueryRpc/);
  assert.match(moduleQuery, /handle_module_query_delivery_v1/);
  assert.match(moduleQuery, /communications_query_contract_reference_v1/);
  assert.doesNotMatch(moduleQuery, /mail|telegram|whatsapp|zulip/i);
  assert.match(
    admission,
    /\/makosh\.communications\.query\.v1\.CommunicationsQueryService\/Query/,
  );
});

test('Communications canonical read v2 owns keyset persistence and scoped opaque cursors', async () => {
  const [cursor, persistence, migration, storageBundle, managed] = await Promise.all([
    readFile(paths.cursor, 'utf8'),
    readFile(paths.persistence, 'utf8'),
    readFile(paths.migration, 'utf8'),
    readFile(paths.storageBundle, 'utf8'),
    readFile(paths.managed, 'utf8'),
  ]);

  assert.match(cursor, /const PREFIX: &\[u8; 4\] = b"HCR2"/);
  assert.match(cursor, /scope_hash_v1/);
  assert.match(cursor, /WrongScope/);
  assert.doesNotMatch(cursor, /provider|blob_ref|query_text|content/i);
  assert.match(persistence, /LIMIT \$\d/);
  assert.match(persistence, /canonical_message/);
  assert.match(persistence, /canonical_message_evidence_page/);
  assert.equal((migration.match(/CREATE INDEX/g) ?? []).length, 8);
  assert.match(storageBundle, /COMMUNICATIONS_STORAGE_BUNDLE_REVISION_V1: u32 = 15/);
  assert.match(storageBundle, /communications_canonical_read_v2_indexes/);
  assert.match(managed, /assert_communications_canonical_read_v2_pagination/);
  assert.match(managed, /Operation::GetMessage/);
  assert.match(managed, /Operation::ListMessageEvidence/);
});

test('Communications frontend detail is generated owner-only composition with no provider fallback', async () => {
  const sources = await Promise.all([
    readFile(paths.frontendRead, 'utf8'),
    readFile(paths.frontendDetail, 'utf8'),
    readFile(paths.frontendController, 'utf8'),
    readFile(paths.frontendPresentation, 'utf8'),
  ]);
  const [readAdapter, detailAdapter, controller, presentation] = sources;

  for (const source of sources) {
    assert.doesNotMatch(source, /integrations\/(mail|telegram|whatsapp|zulip)/);
    assert.doesNotMatch(source, /\/api\/v1\//);
    assert.doesNotMatch(source, /providerLocator|blobRef|messageBody|contentHtml/);
  }
  for (const operation of [
    'getMessage',
    'getConversation',
    'listConversationParticipants',
    'listMessageAttachmentAnchors',
    'listMessageReferences',
    'listMessageEvidence',
  ]) {
    assert.match(readAdapter, new RegExp(`case: '${operation}'`));
  }
  assert.match(detailAdapter, /Promise\.all/);
  assert.match(controller, /requestGeneration !== generation/);
  assert.doesNotMatch(presentation, /connect\/|queries\/|fetch\(/);
});
