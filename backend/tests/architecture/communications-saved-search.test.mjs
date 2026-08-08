import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

const paths = {
  inventory: new URL('architecture/communications-settings-reconstruction.json', BACKEND_ROOT),
  policy: new URL('architecture/policy.json', BACKEND_ROOT),
  manifest: new URL('src/communications-saved-query-api/Cargo.toml', BACKEND_ROOT),
  proto: new URL(
    'src/communications-saved-query-api/proto/makosh/communications/saved_search/v1/saved_search.proto',
    BACKEND_ROOT,
  ),
  migration: new URL(
    'src/communications-persistence/migrations/0012_communications_saved_search_projection.sql',
    BACKEND_ROOT,
  ),
  domain: new URL('src/communications-domain/src/saved_search.rs', BACKEND_ROOT),
  admission: new URL('src/communications-runtime/src/admission.rs', BACKEND_ROOT),
  persistence: new URL('src/communications-persistence/src/saved_search.rs', BACKEND_ROOT),
  runtime: new URL('src/communications-runtime/src/saved_search_port.rs', BACKEND_ROOT),
  managed: new URL(
    'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker.rs',
    BACKEND_ROOT,
  ),
  generator: new URL('frontend/scripts/generate-proto.mjs', PROJECT_ROOT),
  frontend: new URL(
    'frontend/src/domains/communications/queries/canonicalCommunicationsSavedSearches.ts',
    PROJECT_ROOT,
  ),
  controller: new URL(
    'frontend/src/domains/communications/queries/useCanonicalCommunicationsSavedSearches.ts',
    PROJECT_ROOT,
  ),
  presentation: new URL(
    'frontend/src/domains/communications/presentation/CanonicalSavedSearchPanel.vue',
    PROJECT_ROOT,
  ),
  adr: new URL(
    'docs/adr/ADR-0316-communications-saved-search-derived-projection.md',
    PROJECT_ROOT,
  ),
};

test('saved search is one exact Communications build-unit capability', async () => {
  const [inventorySource, policySource, manifest, proto, admission, adr] = await Promise.all([
    readFile(paths.inventory, 'utf8'),
    readFile(paths.policy, 'utf8'),
    readFile(paths.manifest, 'utf8'),
    readFile(paths.proto, 'utf8'),
    readFile(paths.admission, 'utf8'),
    readFile(paths.adr, 'utf8'),
  ]);
  const inventory = JSON.parse(inventorySource);
  const policy = JSON.parse(policySource);
  const gate = inventory.slices.find(({ gate: name }) => name === 'communications_saved_search_v1');

  assert.equal(gate.state, 'implemented');
  assert.equal(
    policy.implementation.currentSlice,
    'call_transcription_managed_conformance_v1',
  );
  assert.ok(policy.implementation.ownerInventory.businessCapabilities.includes(
    'communications.saved-search.v1',
  ));
  assert.match(adr, /Состояние реализации: implemented/);
  assert.match(manifest, /name = "makosh-communications-saved-query-api"/);
  assert.match(manifest, /role = "domain"[\s\S]*owner = "communications"/);
  assert.match(proto, /service CommunicationsSavedSearchService/);
  assert.match(proto, /oneof operation/);
  assert.match(admission, /COMMUNICATIONS_SAVED_SEARCH_CAPABILITY_ID/);
});

test('saved search persistence never stores query plaintext or provider truth', async () => {
  const [proto, migration, domain, persistence, runtime] = await Promise.all([
    readFile(paths.proto, 'utf8'),
    readFile(paths.migration, 'utf8'),
    readFile(paths.domain, 'utf8'),
    readFile(paths.persistence, 'utf8'),
    readFile(paths.runtime, 'utf8'),
  ]);

  assert.match(migration, /communications_saved_query_token_digests/);
  assert.match(migration, /communications_saved_query_audit/);
  assert.doesNotMatch(migration, /query_text|provider|source_cursor|blob_ref|metadata/);
  assert.match(domain, /normalize_search_query_v1/);
  assert.match(runtime, /ensure_index_key/);
  assert.match(runtime, /keyed_search_token_digest_v1/);
  assert.match(persistence, /expected_revision/);
  assert.match(persistence, /lifecycle_state/);
  assert.doesNotMatch(persistence, /query_text|provider|source_cursor|blob_ref/);
  assert.match(proto, /string query/);
  assert.doesNotMatch(
    proto.match(/message SavedSearchSummaryV1 \{[\s\S]*?\n\}/)?.[0] ?? '',
    /query|digest|provider|locator|credential/i,
  );
});

test('saved search has managed Gateway and owner-local frontend evidence', async () => {
  const [managed, generator, frontend, controller, presentation] = await Promise.all([
    readFile(paths.managed, 'utf8'),
    readFile(paths.generator, 'utf8'),
    readFile(paths.frontend, 'utf8'),
    readFile(paths.controller, 'utf8'),
    readFile(paths.presentation, 'utf8'),
  ]);

  assert.match(managed, /SAVED_SEARCH_CONNECT_PATH_V1/);
  assert.match(managed, /SavedSearchErrorCodeRevisionConflict/);
  assert.match(generator, /communications-saved-query-api/);
  assert.match(frontend, /getCommunicationsSavedSearchConnectClient/);
  assert.match(controller, /canManage/);
  assert.match(controller, /requestGeneration/);
  assert.doesNotMatch(presentation, /fetch\(|connect\/|v-html/);
  for (const source of [frontend, controller, presentation]) {
    assert.doesNotMatch(source, /integrations\/(mail|telegram|whatsapp|zulip)/);
    assert.doesNotMatch(source, /api\/v1\/communications/);
  }
});
