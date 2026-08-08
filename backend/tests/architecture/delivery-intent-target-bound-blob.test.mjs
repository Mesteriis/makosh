import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

test('delivery intent body is materialized once into an exact provider-bound Blob receipt', async () => {
  const [
    runtimeManifest,
    admission,
    materializer,
    coordinator,
    runtime,
    persistence,
    providerEvents,
    migration,
    adr,
  ] = await Promise.all([
    readFile(
      new URL('src/communication-delivery-intent-runtime/Cargo.toml', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/communication-delivery-intent-runtime/src/admission.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delivery-intent-runtime/src/body_materializer.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delivery-intent-runtime/src/coordinator.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('src/communication-delivery-intent-runtime/src/runtime.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delivery-intent-persistence/src/intents.rs',
        BACKEND_ROOT,
      ),
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
      new URL(
        'src/communication-delivery-intent-persistence/migrations/0001_delivery_intent_state.sql',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'docs/adr/ADR-0333-delivery-intent-target-bound-blob-materialization.md',
        PROJECT_ROOT,
      ),
      'utf8',
    ),
  ]);

  assert.match(runtimeManifest, /makosh-blob-client/);
  for (const provider of ['mail', 'telegram', 'whatsapp', 'zulip']) {
    assert.match(
      runtimeManifest,
      new RegExp(`makosh-${provider}-delivery-intent-contract`),
    );
    assert.doesNotMatch(
      runtimeManifest,
      new RegExp(`makosh-${provider}-(?:runtime|persistence)`),
    );
  }

  assert.match(admission, /communication_delivery_intent\.blob\.v1/);
  assert.match(admission, /BlobQuotaOperationV1::ReadRange/);
  assert.match(admission, /BlobQuotaOperationV1::Write/);

  for (const exactTarget of [
    'MAIL_DELIVERY_INTENT_TARGET_BLOB_CAPABILITY_ID_V1',
    'TELEGRAM_DELIVERY_INTENT_TARGET_BLOB_CAPABILITY_ID_V1',
    'WHATSAPP_DELIVERY_INTENT_TARGET_BLOB_CAPABILITY_ID_V1',
    'ZULIP_DELIVERY_INTENT_TARGET_BLOB_CAPABILITY_ID_V1',
  ]) {
    assert.match(materializer, new RegExp(exactTarget));
  }
  assert.match(materializer, /ManagedBlobCustodyTargetV1/);
  assert.match(materializer, /BlobDataOperationWriteV1/);
  assert.match(materializer, /custody_transfer_source_proof/);
  assert.doesNotMatch(materializer, /execute_any|ProviderFacade|Vault/);

  assert.match(coordinator, /DeliveryIntentBodyMaterializerV1/);
  assert.match(runtime, /create_delivery_intent_v1/);
  assert.match(runtime, /ManagedDeliveryIntentBodyMaterializerV1/);
  assert.match(providerEvents, /claim\.body_receipt/);

  assert.match(persistence, /DeliveryIntentBodyBlobReceiptV1/);
  assert.doesNotMatch(
    persistence,
    /SealedDeliveryBodyV1|body_ciphertext|body_nonce|body_key_epoch|pub body_utf8:/,
  );
  assert.doesNotMatch(migration, /body_ciphertext|body_nonce|body_key_epoch|body_utf8/);
  for (const column of [
    'body_reference_id',
    'body_declared_bytes',
    'body_sha256',
    'body_custody_source_proof',
  ]) {
    assert.match(migration, new RegExp(column));
  }

  assert.match(adr, /target-bound custody source proof/);
  assert.match(adr, /[Оо]бщего provider facade/);
  assert.match(adr, /остаётся[\s\S]*`planned`/);
});
