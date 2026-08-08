import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);

test('WhatsApp operational read remains an integration-owned SRP slice', async () => {
  const [
    apiManifest,
    coreManifest,
    persistenceManifest,
    runtimeManifest,
    publicContract,
    coreProjection,
    persistenceProjection,
    runtimeComposition,
    managedRuntime,
  ] = await Promise.all([
    readFile(new URL('src/whatsapp-api/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/whatsapp-core/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/whatsapp-persistence/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/whatsapp-runtime/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL(
        'src/whatsapp-api/proto/makosh/whatsapp/operational/v1/client.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('src/whatsapp-core/src/operational.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL('src/whatsapp-persistence/src/operational.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(new URL('src/whatsapp-runtime/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/whatsapp-runtime/src/managed.rs', BACKEND_ROOT), 'utf8'),
  ]);

  for (const manifest of [
    apiManifest,
    coreManifest,
    persistenceManifest,
    runtimeManifest,
  ]) {
    for (const forbiddenOwner of [
      'makosh-kernel',
      'makosh-gateway',
      'makosh-communications-domain',
      'makosh-communications-persistence',
      'makosh-communications-runtime',
    ]) {
      assert.doesNotMatch(manifest, new RegExp(forbiddenOwner));
    }
  }

  assert.match(publicContract, /oneof query/);
  assert.match(publicContract, /oneof response/);
  assert.match(publicContract, /optional string cursor/);
  assert.match(publicContract, /uint32 limit/);
  assert.doesNotMatch(
    publicContract,
    /google\.protobuf\.Any|\bbytes\b|\bmap\s*</,
  );

  assert.match(coreProjection, /project_operational_host_observation/);
  assert.match(
    coreProjection,
    /metadata_only_message_does_not_invent_operational_content/,
  );
  assert.doesNotMatch(coreProjection, /sqlx|makosh_communications/);

  assert.match(
    persistenceProjection,
    /record_host_observation_projection_and_enqueue/,
  );
  assert.match(persistenceProjection, /execute_operational_query/);
  assert.match(persistenceProjection, /validate_operational_query/);
  assert.match(persistenceProjection, /operational_sha256/);
  assert.match(persistenceProjection, /whatsapp_operational_tombstones/);
  assert.match(persistenceProjection, /ParticipantRemoved/);
  assert.match(persistenceProjection, /delivery_state/);
  assert.match(persistenceProjection, /transaction\s*\.commit\(\)/);
  assert.doesNotMatch(
    persistenceProjection,
    /makosh_data\.communications_|makosh_(?:kernel|gateway)/,
  );

  assert.match(runtimeComposition, /project_operational_host_observation/);
  assert.match(
    runtimeComposition,
    /record_host_observation_projection_and_enqueue/,
  );
  assert.match(managedRuntime, /operational_query_account_id\(query\)/);
  assert.doesNotMatch(managedRuntime, /SELECT |INSERT |UPDATE |DELETE /);
});
