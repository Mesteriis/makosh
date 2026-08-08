import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

test('loopback development owner-device proof host is a separate development unit', async () => {
  const [
    inventory,
    adr,
    manifest,
    server,
    client,
    factory,
    ownerSettingsClient,
    vaultClient,
    ownerDeviceProofSource,
    vite,
    ensemble,
  ] = await Promise.all([
    readFile(
      new URL('architecture/communications-settings-reconstruction.json', BACKEND_ROOT),
      'utf8',
    ).then(JSON.parse),
    readFile(
      new URL('docs/adr/ADR-0322-loopback-native-owner-device-proof-host.md', PROJECT_ROOT),
      'utf8',
    ),
    readFile(
      new URL('frontend/native/owner-vault-provisioning-host/Cargo.toml', PROJECT_ROOT),
      'utf8',
    ),
    readFile(
      new URL(
        'frontend/native/owner-vault-provisioning-host/src/bin/development_host.rs',
        PROJECT_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('frontend/src/platform/gateway/developmentOwnerDeviceProof.ts', PROJECT_ROOT),
      'utf8',
    ),
    readFile(
      new URL('frontend/src/platform/gateway/ownerDeviceProofFactory.ts', PROJECT_ROOT),
      'utf8',
    ),
    readFile(
      new URL('frontend/src/platform/settings/ownerModuleSettingsClient.ts', PROJECT_ROOT),
      'utf8',
    ),
    readFile(
      new URL('frontend/src/platform/vault/ownerVaultProvisioningClient.ts', PROJECT_ROOT),
      'utf8',
    ),
    readFile(
      new URL('frontend/src/platform/gateway/ownerDeviceProof.ts', PROJECT_ROOT),
      'utf8',
    ),
    readFile(new URL('frontend/vite.config.ts', PROJECT_ROOT), 'utf8'),
    readFile(new URL('scripts/dev-ensemble.sh', BACKEND_ROOT), 'utf8'),
  ]);

  const gate = inventory.slices.find(
    (slice) => slice.gate === 'loopback_native_owner_device_proof_host_v1',
  );
  assert.ok(gate, 'missing loopback native owner-device proof gate');
  assert.equal(gate.role, 'app');
  assert.equal(gate.owner, 'first_party_development');
  assert.equal(gate.state, 'implemented');
  assert.match(gate.dependsOn.join(','), /client_gateway_v1/);
  assert.match(adr, /Состояние реализации: implemented/i);

  assert.match(manifest, /development-server/);
  assert.match(manifest, /makosh-owner-vault-development-host/);

  assert.match(server, /OWNER_DEVICE_PROOF_SIGN_PATH/);
  assert.match(server, /\/__makosh\/owner-device-proof\/v1\/sign/);
  assert.match(server, /sign_challenge\(&exact_array\(request\.challenge_bytes\)\?/);
  assert.match(server, /authorize_request_metadata\(/);
  assert.match(server, /EXACT_BROWSER_ORIGIN/);
  assert.match(server, /x-makosh-development-host-proof/);

  assert.match(client, /credentials: 'same-origin'/);
  assert.match(client, /cache: 'no-store'/);
  assert.match(client, /redirect: 'error'/);
  assert.match(client, /signatureRaw/);
  assert.match(factory, /hasDevelopmentOwnerDeviceProofHostV1/);
  assert.match(factory, /VITE_MAKOSH_DEV_OWNER_DEVICE_PROOF_HOST === '1'/);
  assert.match(ownerDeviceProofSource, /interface OwnerDeviceProofV1/);

  assert.match(ownerSettingsClient, /createOwnerDeviceProofV1\(\)/);
  assert.match(vaultClient, /createOwnerDeviceProofV1\(\)/);
  assert.match(ownerSettingsClient, /class OwnerModuleSettingsClientV1/);
  assert.match(vaultClient, /class OwnerVaultProvisioningClientV1/);

  assert.match(vite, /\/__makosh\/owner-device-proof\/v1/);
  assert.match(vite, /DEVELOPMENT_HOST_PROOF_HEADER/);
  assert.match(vite, /request\.removeHeader\(DEVELOPMENT_HOST_PROOF_HEADER\)/);
  assert.match(vite, /request\.setHeader\(DEVELOPMENT_HOST_PROOF_HEADER, host\.proof\)/);

  assert.match(ensemble, /VITE_MAKOSH_DEV_OWNER_DEVICE_PROOF_HOST=1/);
  assert.match(ensemble, /VITE_MAKOSH_DEV_OWNER_VAULT_HOST=1/);
});
