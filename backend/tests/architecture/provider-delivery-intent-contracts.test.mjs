import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

const providers = [
  {
    owner: 'mail',
    module: 'makosh-mail-runtime',
    blobCapability: 'mail.blob.v1',
    custodyScope: 'mail.delivery-intent-body.v1',
    messagePrefix: 'Mail',
    protoPackage: 'makosh.mail.delivery_intent.v1',
  },
  {
    owner: 'telegram',
    module: 'makosh-telegram-runtime',
    blobCapability: 'telegram.blob.v1',
    custodyScope: 'telegram.delivery-intent-body.v1',
    messagePrefix: 'Telegram',
    protoPackage: 'makosh.telegram.delivery_intent.v1',
  },
  {
    owner: 'whatsapp',
    module: 'makosh-whatsapp-runtime',
    blobCapability: 'whatsapp.blob.v1',
    custodyScope: 'whatsapp.delivery-intent-body.v1',
    messagePrefix: 'WhatsApp',
    protoPackage: 'makosh.whatsapp.delivery_intent.v1',
  },
  {
    owner: 'zulip',
    module: 'makosh-zulip-runtime',
    blobCapability: 'zulip.blob.v1',
    custodyScope: 'zulip.delivery-intent-body.v1',
    messagePrefix: 'Zulip',
    protoPackage: 'makosh.zulip.delivery_intent.v1',
  },
];

function cratePath(owner, path) {
  return new URL(`src/${owner}-delivery-intent-contract/${path}`, BACKEND_ROOT);
}

test('provider delivery intents are four separate integration-owned contract build units', async () => {
  const policy = JSON.parse(
    await readFile(new URL('architecture/policy.json', BACKEND_ROOT), 'utf8'),
  );
  const reconstruction = JSON.parse(
    await readFile(
      new URL('architecture/communications-settings-reconstruction.json', BACKEND_ROOT),
      'utf8',
    ),
  );
  const workspace = await readFile(new URL('Cargo.toml', BACKEND_ROOT), 'utf8');
  const runtimeManifest = await readFile(
    new URL('src/communication-delivery-intent-runtime/Cargo.toml', BACKEND_ROOT),
    'utf8',
  );
  const workflowManifests = await Promise.all(
    [
      'api',
      'core',
      'persistence',
      'assembly',
    ].map((surface) =>
      readFile(
        new URL(`src/communication-delivery-intent-${surface}/Cargo.toml`, BACKEND_ROOT),
        'utf8',
      ),
    ),
  );

  assert.equal(
    policy.implementation.currentSlice,
    'speech_to_text_whisper_admission_v1',
  );
  assert.equal(
    reconstruction.slices.find(({ gate }) => gate === 'communication_delivery_intent_v1')
      ?.state,
    'implemented',
  );
  assert.deepEqual(policy.implementation.ownerInventory.integrations, [
    'desktop_call_recording',
    'mail',
    'ollama',
    'whisper_stt',
  ]);
  assert.deepEqual(
    policy.implementation.productionPackages
      .filter(({ name }) => name.endsWith('-delivery-intent-contract'))
      .map(({ name }) => name),
    providers.map(({ owner }) => `makosh-${owner}-delivery-intent-contract`),
  );

  for (const provider of providers) {
    const packageName = `makosh-${provider.owner}-delivery-intent-contract`;
    const [manifest, contract, proto] = await Promise.all([
      readFile(cratePath(provider.owner, 'Cargo.toml'), 'utf8'),
      readFile(cratePath(provider.owner, 'src/lib.rs'), 'utf8'),
      readFile(
        cratePath(
          provider.owner,
          `proto/makosh/${provider.owner}/delivery_intent/v1/delivery_intent.proto`,
        ),
        'utf8',
      ),
    ]);
    const wire = proto.replaceAll(/\/\/.*$/gm, '');

    assert.match(workspace, new RegExp(`"src/${provider.owner}-delivery-intent-contract"`));
    assert.match(manifest, new RegExp(`name = "${packageName}"`));
    assert.match(
      manifest,
      new RegExp(
        `role = "integration"[\\s\\S]*owner = "${provider.owner}"[\\s\\S]*surface = "contract"`,
      ),
    );
    assert.match(manifest, /makosh-runtime-protocol/);
    assert.doesNotMatch(
      manifest,
      /makosh-(?:communications|communication-delivery-intent|mail|telegram|whatsapp|zulip)-(?:api|core|runtime|persistence|assembly)|sqlx|async-nats/,
    );

    assert.match(wire, new RegExp(`package ${provider.protoPackage.replaceAll('.', '\\.')};`));
    assert.match(wire, new RegExp(`message Execute${provider.messagePrefix}DeliveryIntentCommandV1`));
    assert.match(wire, /bytes intent_id = 1;/);
    assert.match(wire, /string logical_owner_id = 2;/);
    assert.match(wire, /bytes account_source_cursor = 3;/);
    assert.match(wire, /bytes conversation_source_cursor = 4;/);
    assert.match(wire, /optional bytes reply_to_source_cursor = 5;/);
    assert.match(wire, /bytes reference_id = 1;/);
    assert.match(wire, /uint64 declared_bytes = 2;/);
    assert.match(wire, /bytes sha256 = 3;/);
    assert.match(wire, /bytes custody_transfer_source_proof = 4;/);
    assert.doesNotMatch(
      wire,
      /\b(?:map|Any|provider_kind|provider_id|account_id|chat_id|email|phone|recipient|subject|body_utf8|text_body|error_text)\b/,
    );

    assert.match(contract, new RegExp(`"${provider.module}"`));
    assert.match(contract, new RegExp(`"${provider.blobCapability.replaceAll('.', '\\.')}"`));
    assert.match(contract, new RegExp(`"${provider.custodyScope.replaceAll('.', '\\.')}"`));
    assert.match(contract, /DurableEnvelopeKindV1::Command/);
    assert.match(contract, /DurableEnvelopeKindV1::Result/);
    assert.match(
      contract,
      new RegExp(`validate_${provider.owner}_delivery_intent_execute_v1`),
    );
    assert.match(contract, /MAX_SOURCE_PROOF_BYTES_V1/);
    assert.match(contract, /valid_fixed_id/);
    assert.doesNotMatch(contract, /enum Provider|ProviderKind|generic|facade/);
  }

  assert.doesNotMatch(
    workflowManifests.join('\n'),
    /makosh-(?:mail|telegram|whatsapp|zulip)-delivery-intent-contract/,
  );
  for (const provider of providers) {
    assert.match(
      runtimeManifest,
      new RegExp(`makosh-${provider.owner}-delivery-intent-contract`),
    );
  }
  assert.doesNotMatch(
    runtimeManifest,
    /makosh-(?:mail|telegram|whatsapp|zulip)-(?:runtime|persistence)/,
  );
});

test('ADR keeps core owner-neutral while admission waits for provider runtime consumers', async () => {
  const [adr, workflowAdr, adapterAdr] = await Promise.all([
    readFile(
      new URL(
        'docs/adr/ADR-0331-provider-owned-delivery-intent-event-contracts.md',
        PROJECT_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'docs/adr/ADR-0330-provider-neutral-communication-delivery-intent-workflow.md',
        PROJECT_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'docs/adr/ADR-0332-delivery-intent-transactional-provider-event-adapters.md',
        PROJECT_ROOT,
      ),
      'utf8',
    ),
  ]);

  assert.match(adr, /четыре самостоятельные единицы сборки/);
  assert.match(adr, /Общей[\s\S]*facade нет/);
  assert.match(adr, /Kernel\/Core[\s\S]*не декодирует body/);
  assert.match(adr, /остаётся `planned`/);
  assert.match(workflowAdr, /ADR-0331/);
  assert.match(workflowAdr, /ADR-0332/);
  assert.match(adapterAdr, /provider runtime inbox consumers/);
});
