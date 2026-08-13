import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);
const ADAPTER_ROOT = new URL(
  'src/communication-delivery-intent-event-adapters/',
  BACKEND_ROOT,
);

const providers = ['mail', 'telegram', 'whatsapp', 'zulip'];

test('delivery intent event adapters preserve four exact routes without a provider facade', async () => {
  const [
    policy,
    reconstruction,
    workspace,
    manifest,
    adapterCore,
    runtimeManifest,
    runtimeAdapter,
    persistenceManifest,
    persistence,
    migration,
    schema,
    adr,
    ...providerModules
  ] = await Promise.all([
    readFile(new URL('architecture/policy.json', BACKEND_ROOT), 'utf8').then(JSON.parse),
    readFile(
      new URL('architecture/communications-settings-reconstruction.json', BACKEND_ROOT),
      'utf8',
    ).then(JSON.parse),
    readFile(new URL('Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('Cargo.toml', ADAPTER_ROOT), 'utf8'),
    readFile(new URL('src/lib.rs', ADAPTER_ROOT), 'utf8'),
    readFile(
      new URL('src/communication-delivery-intent-runtime/Cargo.toml', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delivery-intent-runtime/src/provider_events.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('src/communication-delivery-intent-persistence/Cargo.toml', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delivery-intent-persistence/src/provider_events.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delivery-intent-persistence/migrations/0002_provider_event_delivery.sql',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('src/communication-delivery-intent-persistence/src/schema.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL(
        'docs/adr/ADR-0332-delivery-intent-transactional-provider-event-adapters.md',
        PROJECT_ROOT,
      ),
      'utf8',
    ),
    ...providers.map((provider) =>
      readFile(new URL(`src/${provider}.rs`, ADAPTER_ROOT), 'utf8'),
    ),
  ]);

  assert.equal(
    policy.implementation.currentSlice,
    'speech_to_text_whisper_admission_v1',
  );
  assert.equal(
    reconstruction.slices.find(({ gate }) => gate === 'communication_delivery_intent_v1')
      ?.state,
    'implemented',
  );
  assert.deepEqual(
    policy.implementation.productionPackages
      .filter(({ owner }) => owner === 'communication_delivery_intent')
      .map(({ name }) => name)
      .filter((name) => name.endsWith('event-adapters')),
    ['makosh-communication-delivery-intent-event-adapters'],
  );

  assert.match(workspace, /"src\/communication-delivery-intent-event-adapters"/);
  assert.match(
    manifest,
    /role = "workflow"[\s\S]*owner = "communication_delivery_intent"[\s\S]*surface = "implementation"/,
  );
  for (const provider of providers) {
    assert.match(manifest, new RegExp(`makosh-${provider}-delivery-intent-contract`));
  }
  assert.doesNotMatch(
    manifest,
    /makosh-(?:communications-domain|communications-persistence|mail-runtime|mail-persistence|telegram-runtime|telegram-persistence|whatsapp-runtime|whatsapp-persistence|zulip-runtime|zulip-persistence)|sqlx|async-nats/,
  );
  assert.doesNotMatch(adapterCore, /enum Provider|ProviderKind|dispatch_provider|execute_any/);

  for (const [index, provider] of providers.entries()) {
    const source = providerModules[index];
    assert.match(source, new RegExp(`makosh_${provider}_delivery_intent_contract`));
    assert.match(source, /pub fn build_execute_outbox_v1/);
    assert.match(source, /pub fn decode_succeeded_v1/);
    assert.match(source, /pub fn decode_rejected_v1/);
    for (const other of providers.filter((value) => value !== provider)) {
      assert.doesNotMatch(
        source,
        new RegExp(`makosh_${other}_delivery_intent_contract`),
      );
    }
    assert.doesNotMatch(source, /communications_(?:domain|persistence)|provider sdk|execute_any/);
  }

  assert.match(
    runtimeManifest,
    /makosh-communication-delivery-intent-event-adapters/,
  );
  assert.doesNotMatch(
    runtimeManifest,
    /makosh-(?:mail|telegram|whatsapp|zulip)-(?:runtime|persistence)/,
  );
  for (const provider of providers) {
    assert.match(runtimeAdapter, new RegExp(`enqueue_${provider}_command_v1`));
    assert.match(runtimeAdapter, new RegExp(`complete_${provider}_publish_v1`));
    assert.match(runtimeAdapter, new RegExp(`apply_${provider}_succeeded_v1`));
    assert.match(runtimeAdapter, new RegExp(`apply_${provider}_rejected_v1`));
  }

  assert.match(persistenceManifest, /makosh-events-protocol/);
  assert.doesNotMatch(
    persistenceManifest,
    /makosh-(?:mail|telegram|whatsapp|zulip)-delivery-intent-contract/,
  );
  assert.match(persistence, /enqueue_provider_command/);
  assert.match(persistence, /pending_provider_commands/);
  assert.match(persistence, /mark_provider_command_published/);
  assert.match(persistence, /apply_terminal_result/);
  assert.match(persistence, /existing_sha256\.as_slice\(\) != record\.envelope_sha256/);
  assert.match(persistence, /ApplyTerminalDeliveryResultOutcomeV1::Duplicate/);
  assert.match(migration, /communication_delivery_intent_provider_outbox/);
  assert.match(migration, /communication_delivery_intent_result_inbox/);
  assert.match(migration, /exact_envelope_bytes/);
  assert.match(migration, /FOREIGN KEY \(command_message_id\)/);
  assert.doesNotMatch(migration, /body_utf8|provider_payload|error_text/);
  assert.match(schema, /STORAGE_BUNDLE_REVISION_V2/);
  assert.match(schema, /0002_provider_event_delivery\.sql/);

  assert.match(adr, /четыре exact workflow-owned command encoder\/result/);
  assert.match(adr, /Нет provider discriminator dispatch/);
  assert.match(adr, /exact duplicate/);
  assert.match(adr, /остаётся[\s\S]*`planned`/);
});
