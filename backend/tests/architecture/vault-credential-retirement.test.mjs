import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

const paths = {
  adr: new URL(
    'docs/adr/ADR-0293-scoped-vault-credential-retirement-and-deletion.md',
    PROJECT_ROOT,
  ),
  inventory: new URL(
    'architecture/communications-settings-reconstruction.json',
    BACKEND_ROOT,
  ),
  protocol: new URL(
    'src/platform/vault/protocol/src/operations/command.rs',
    BACKEND_ROOT,
  ),
  client: new URL(
    'src/platform/vault/managed_client/src/lib.rs',
    BACKEND_ROOT,
  ),
  service: new URL(
    'src/platform/vault/runtime/src/service/runtime.rs',
    BACKEND_ROOT,
  ),
  store: new URL(
    'src/platform/vault/store_sqlcipher/src/actor/handle.rs',
    BACKEND_ROOT,
  ),
  lifecycle: new URL(
    'src/platform/vault/store_sqlcipher/src/actor/lifecycle.rs',
    BACKEND_ROOT,
  ),
  schema: new URL(
    'src/platform/vault/store_sqlcipher/src/database/store.rs',
    BACKEND_ROOT,
  ),
};

test('Vault retirement is an exact platform lifecycle with durable tombstones', async () => {
  const [adr, inventorySource, protocol, client, service, store, lifecycle, schema] =
    await Promise.all([
      readFile(paths.adr, 'utf8'),
      readFile(paths.inventory, 'utf8'),
      readFile(paths.protocol, 'utf8'),
      readFile(paths.client, 'utf8'),
      readFile(paths.service, 'utf8'),
      readFile(paths.store, 'utf8'),
      readFile(paths.lifecycle, 'utf8'),
      readFile(paths.schema, 'utf8'),
    ]);
  const inventory = JSON.parse(inventorySource);
  const slice = inventory.slices.find(
    ({ gate }) => gate === 'vault_credential_retirement_v1',
  );

  assert.deepEqual(slice, {
    gate: 'vault_credential_retirement_v1',
    role: 'platform',
    owner: 'vault',
    state: 'implemented',
    dependsOn: ['module_control_plane_v1', 'vault_v1'],
  });
  assert.match(protocol, /RetireLease[\s\S]*DeleteLease/);
  assert.match(client, /pub fn retire_once\(/);
  assert.match(client, /pub fn delete_once\(/);
  assert.match(service, /VaultActionV1::Retire/);
  assert.match(service, /VaultActionV1::Delete/);
  assert.match(store, /lifecycle::mutate\(/);
  assert.match(lifecycle, /pub\(super\) fn mutate\(/);
  assert.match(lifecycle, /DELETE FROM vault_secret_records/);
  assert.match(lifecycle, /INSERT INTO vault_secret_tombstones/);
  assert.match(schema, /const SCHEMA_VERSION: i64 = 4/);
  assert.match(schema, /CREATE TRIGGER vault_secret_records_reject_tombstone/);
  assert.match(adr, /Kernel не декодирует provider lifecycle command/);
  assert.doesNotMatch(
    `${protocol}\n${client}\n${service}\n${store}\n${lifecycle}\n${schema}`,
    /makosh_(?:mail|telegram|whatsapp|zulip|communications)|Mail|Telegram|WhatsApp|Zulip/,
  );
});
