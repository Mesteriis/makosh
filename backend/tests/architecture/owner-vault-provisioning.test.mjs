import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

const paths = {
  adr: new URL(
    'docs/adr/ADR-0295-owner-write-only-vault-provisioning-through-core-gateway.md',
    PROJECT_ROOT,
  ),
  inventory: new URL(
    'architecture/communications-settings-reconstruction.json',
    BACKEND_ROOT,
  ),
  command: new URL(
    'src/platform/vault/protocol/src/operations/command.rs',
    BACKEND_ROOT,
  ),
  receipt: new URL(
    'src/platform/vault/protocol/src/operations/provisioning.rs',
    BACKEND_ROOT,
  ),
  service: new URL(
    'src/platform/vault/runtime/src/service/runtime.rs',
    BACKEND_ROOT,
  ),
  persistence: new URL(
    'src/platform/vault/store_sqlcipher/src/actor/provisioning.rs',
    BACKEND_ROOT,
  ),
  schema: new URL(
    'src/platform/vault/store_sqlcipher/src/database/store.rs',
    BACKEND_ROOT,
  ),
  gatewayContract: new URL(
    'src/api/gateway/contracts/proto/makosh/gateway/v1/owner_vault_provisioning.proto',
    BACKEND_ROOT,
  ),
  gatewayRouter: new URL(
    'src/api/gateway/runtime/src/browser/owner_vault.rs',
    BACKEND_ROOT,
  ),
  kernelAuthority: new URL(
    'src/kernel/src/platform/vault/owner_provisioning/authorization.rs',
    BACKEND_ROOT,
  ),
  kernelOwnerProof: new URL(
    'src/kernel/src/platform/gateway/owner_device_proof.rs',
    BACKEND_ROOT,
  ),
  kernelCeremony: new URL(
    'src/kernel/src/platform/vault/owner_provisioning/mod.rs',
    BACKEND_ROOT,
  ),
  kernelRoute: new URL(
    'src/kernel/src/platform/vault/owner_provisioning/routes.rs',
    BACKEND_ROOT,
  ),
  liveConformance: new URL(
    'tests/support/kernel-recovery/src/tests/owner_vault_provisioning.rs',
    BACKEND_ROOT,
  ),
};

test('owner Vault provisioning primitive is write-only durable and platform-neutral', async () => {
  const [
    adr,
    inventorySource,
    command,
    receipt,
    service,
    persistence,
    schema,
    gatewayContract,
    gatewayRouter,
    kernelAuthority,
    kernelOwnerProof,
    kernelCeremony,
    kernelRoute,
    liveConformance,
  ] =
    await Promise.all([
      readFile(paths.adr, 'utf8'),
      readFile(paths.inventory, 'utf8'),
      readFile(paths.command, 'utf8'),
      readFile(paths.receipt, 'utf8'),
      readFile(paths.service, 'utf8'),
      readFile(paths.persistence, 'utf8'),
      readFile(paths.schema, 'utf8'),
      readFile(paths.gatewayContract, 'utf8'),
      readFile(paths.gatewayRouter, 'utf8'),
      readFile(paths.kernelAuthority, 'utf8'),
      readFile(paths.kernelOwnerProof, 'utf8'),
      readFile(paths.kernelCeremony, 'utf8'),
      readFile(paths.kernelRoute, 'utf8'),
      readFile(paths.liveConformance, 'utf8'),
    ]);
  const inventory = JSON.parse(inventorySource);
  const backend = inventory.slices.find(
    ({ gate }) => gate === 'owner_vault_provisioning_backend_v1',
  );

  assert.deepEqual(backend, {
    gate: 'owner_vault_provisioning_backend_v1',
    role: 'platform',
    owner: 'vault',
    state: 'implemented',
    dependsOn: ['client_gateway_v1', 'vault_v1'],
  });
  assert.match(command, /ProvisionLease/);
  assert.match(command, /operation_id: \[u8; 16\]/);
  assert.match(receipt, /VaultProvisioningReceiptV1/);
  assert.match(receipt, /secret_revision/);
  assert.doesNotMatch(receipt, /record_id|payload|credential/);
  assert.match(service, /provision_current_once/);
  assert.match(persistence, /vault_owner_provisioning_receipts/);
  assert.match(persistence, /transaction\.commit/);
  assert.match(persistence, /expected_intent_digest/);
  assert.match(schema, /CREATE TABLE vault_owner_provisioning_receipts/);
  assert.match(adr, /Prepare[\s\S]*Authorize[\s\S]*Commit/);
  assert.match(gatewayContract, /service OwnerVaultProvisioningService/);
  assert.match(gatewayContract, /rpc Prepare[\s\S]*rpc Authorize[\s\S]*rpc Commit/);
  assert.match(gatewayContract, /enum OwnerVaultSecretClassV1/);
  assert.match(gatewayContract, /enum OwnerVaultActionV1/);
  assert.doesNotMatch(gatewayContract, /makosh\/runtime\/v1\/recovery\.proto/);
  assert.match(gatewayRouter, /authorize_request/);
  assert.match(gatewayRouter, /is_lan_development/);
  assert.match(gatewayRouter, /require_mutation_origin/);
  assert.match(kernelOwnerProof, /BrowserDeviceStateV1::Active/);
  assert.match(kernelAuthority, /module_vault_purpose_requests/);
  assert.match(kernelOwnerProof, /VerifyingKey::from_sec1_bytes/);
  assert.match(kernelCeremony, /challenge_digest/);
  assert.match(kernelCeremony, /operation_id/);
  assert.match(kernelCeremony, /audience_grant_epoch/);
  assert.match(kernelRoute, /relay_kernel_authorized_route/);
  assert.match(liveConformance, /authenticate_gateway_router/);
  assert.match(liveConformance, /pre-restart provisioning session must be stale/);
  assert.match(liveConformance, /assert_eq!\(replay, first\)/);
  assert.doesNotMatch(
    `${command}\n${receipt}\n${service}\n${persistence}\n${gatewayContract}\n${gatewayRouter}\n${kernelAuthority}\n${kernelOwnerProof}\n${kernelCeremony}\n${kernelRoute}`,
    /makosh_(?:mail|telegram|whatsapp|zulip|communications)|Mail|Telegram|WhatsApp|Zulip/,
  );
});
