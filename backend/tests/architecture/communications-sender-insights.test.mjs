import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

const paths = {
  inventory: new URL('architecture/communications-settings-reconstruction.json', BACKEND_ROOT),
  policy: new URL('architecture/policy.json', BACKEND_ROOT),
  manifest: new URL('src/communications-sender-insights-api/Cargo.toml', BACKEND_ROOT),
  proto: new URL(
    'src/communications-sender-insights-api/proto/makosh/communications/sender_insights/v1/sender_insights.proto',
    BACKEND_ROOT,
  ),
  migration: new URL(
    'src/communications-persistence/migrations/0013_communications_sender_insights_projection.sql',
    BACKEND_ROOT,
  ),
  persistence: new URL('src/communications-persistence/src/sender_insights.rs', BACKEND_ROOT),
  durable: new URL('src/communications-persistence/src/durable.rs', BACKEND_ROOT),
  mail: new URL('src/mail-core/src/lib.rs', BACKEND_ROOT),
  telegram: new URL('src/telegram-core/src/lib.rs', BACKEND_ROOT),
  admission: new URL('src/communications-runtime/src/admission.rs', BACKEND_ROOT),
  managed: new URL(
    'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker.rs',
    BACKEND_ROOT,
  ),
  generator: new URL('frontend/scripts/generate-proto.mjs', PROJECT_ROOT),
  frontend: new URL(
    'frontend/src/domains/communications/queries/canonicalCommunicationsSenderInsights.ts',
    PROJECT_ROOT,
  ),
  controller: new URL(
    'frontend/src/domains/communications/queries/useCanonicalCommunicationsSenderInsights.ts',
    PROJECT_ROOT,
  ),
  presentation: new URL(
    'frontend/src/domains/communications/presentation/CanonicalSenderInsightsPanel.vue',
    PROJECT_ROOT,
  ),
  layout: new URL('frontend/src/app/layout/AppLayoutRoot.vue', PROJECT_ROOT),
  adr: new URL(
    'docs/adr/ADR-0317-communications-sender-insights-derived-projection.md',
    PROJECT_ROOT,
  ),
};

test('sender insights is one exact Communications build-unit capability', async () => {
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
  const gate = inventory.slices.find(({ gate: name }) => name === 'communications_sender_insights_v1');

  assert.equal(gate.state, 'implemented');
  assert.equal(
    policy.implementation.currentSlice,
    'call_transcription_managed_conformance_v1',
  );
  assert.ok(policy.implementation.ownerInventory.businessCapabilities.includes(
    'communications.sender-insights.v1',
  ));
  assert.match(adr, /Состояние реализации: implemented/);
  assert.match(manifest, /name = "makosh-communications-sender-insights-api"/);
  assert.match(manifest, /role = "domain"[\s\S]*owner = "communications"/);
  assert.match(proto, /service CommunicationsSenderInsightsService/);
  assert.match(admission, /COMMUNICATIONS_SENDER_INSIGHTS_CAPABILITY_ID/);
});

test('sender projection is incoming-only and excludes provider or Review truth', async () => {
  const [proto, migration, persistence, durable, mail, telegram] = await Promise.all([
    readFile(paths.proto, 'utf8'),
    readFile(paths.migration, 'utf8'),
    readFile(paths.persistence, 'utf8'),
    readFile(paths.durable, 'utf8'),
    readFile(paths.mail, 'utf8'),
    readFile(paths.telegram, 'utf8'),
  ]);

  assert.match(migration, /communications_sender_profiles/);
  assert.match(migration, /communications_message_sender_facts/);
  assert.match(durable, /message\.direction == CommunicationDirectionV1::Incoming/);
  assert.match(persistence, /COUNT\(\*\).*message_count/s);
  assert.match(persistence, /COUNT\(DISTINCT messages\.conversation_id\)/);
  assert.match(durable, /participant_display_label/);
  assert.match(mail, /draft_ingress_observation_with_sender_subject_body/);
  assert.match(telegram, /sender_display_name/);
  for (const source of [migration, persistence, proto]) {
    assert.doesNotMatch(source, /provider_locator|source_cursor|message_body|importance_score/i);
  }
});

test('sender insights has managed Gateway and owner-local frontend evidence', async () => {
  const [managed, generator, frontend, controller, presentation, layout] = await Promise.all([
    readFile(paths.managed, 'utf8'),
    readFile(paths.generator, 'utf8'),
    readFile(paths.frontend, 'utf8'),
    readFile(paths.controller, 'utf8'),
    readFile(paths.presentation, 'utf8'),
    readFile(paths.layout, 'utf8'),
  ]);

  assert.match(managed, /SENDER_INSIGHTS_CONNECT_PATH_V1/);
  assert.match(managed, /sender-insights response must not reveal provider locators/);
  assert.match(generator, /communications-sender-insights-api/);
  assert.match(frontend, /getCommunicationsSenderInsightsConnectClient/);
  assert.match(controller, /requestGeneration/);
  assert.match(layout, /'communications\.sender-insights\.v1'/);
  assert.doesNotMatch(presentation, /fetch\(|connect\/|v-html/);
  for (const source of [frontend, controller, presentation]) {
    assert.doesNotMatch(source, /integrations\/(mail|telegram|whatsapp|zulip)/);
    assert.doesNotMatch(source, /api\/v1\/communications/);
  }
});
