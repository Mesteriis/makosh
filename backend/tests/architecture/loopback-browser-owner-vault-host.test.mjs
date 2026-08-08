import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

test('make dev uses a separate fail-closed loopback Owner Vault host', async () => {
  const [
    inventory,
    adr,
    manifest,
    server,
    client,
    factory,
    availability,
    vite,
    ensemble,
    probe,
  ] = await Promise.all([
    readFile(
      new URL('architecture/communications-settings-reconstruction.json', BACKEND_ROOT),
      'utf8',
    ).then(JSON.parse),
    readFile(
      new URL(
        'docs/adr/ADR-0309-loopback-browser-owner-vault-provisioning-host.md',
        PROJECT_ROOT,
      ),
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
      new URL(
        'frontend/src/platform/vault/developmentOwnerVaultProvisioningHost.ts',
        PROJECT_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'frontend/src/platform/vault/ownerVaultProvisioningHostFactory.ts',
        PROJECT_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'frontend/src/platform/vault/provisioningHostAvailability.ts',
        PROJECT_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('frontend/vite.config.ts', PROJECT_ROOT), 'utf8'),
    readFile(new URL('scripts/dev-ensemble.sh', BACKEND_ROOT), 'utf8'),
    readFile(new URL('scripts/probe-dev-owner-vault-host.mjs', BACKEND_ROOT), 'utf8'),
  ]);

  const gate = inventory.slices.find(
    (slice) => slice.gate === 'loopback_browser_owner_vault_host_v1',
  );
  assert.ok(gate, 'missing loopback browser Owner Vault host gate');
  assert.equal(gate.role, 'app');
  assert.equal(gate.owner, 'first_party_development');
  assert.equal(gate.state, 'implemented');
  assert.match(adr, /Состояние реализации: Implemented/);

  assert.match(manifest, /development-server/);
  assert.match(manifest, /makosh-owner-vault-development-host/);
  assert.match(server, /DEFAULT_LISTEN_ADDRESS: &str = "127\.0\.0\.1:9445"/);
  assert.match(server, /EXACT_BROWSER_ORIGIN: &str = "http:\/\/127\.0\.0\.1:5173"/);
  assert.match(server, /x-makosh-development-host-proof/);
  assert.match(server, /MAX_REQUEST_BYTES: usize = 256 \* 1024/);
  assert.match(server, /permissions\(\)\.mode\(\) & 0o077 != 0/);
  assert.match(server, /Zeroizing<String>/);
  assert.doesNotMatch(server, /println!\([^)]*(?:request|secret|ciphertext|receipt)/s);

  assert.match(client, /credentials: 'same-origin'/);
  assert.match(client, /cache: 'no-store'/);
  assert.match(client, /redirect: 'error'/);
  assert.match(client, /secretPayload\.fill\(0\)/);
  assert.match(factory, /hasNativeOwnerVaultProvisioningHostV1/);
  assert.match(factory, /hasDevelopmentOwnerVaultProvisioningHostV1/);
  assert.match(factory, /UnavailableOwnerVaultProvisioningHostV1/);
  assert.match(availability, /VITE_MAKOSH_DEV_OWNER_VAULT_HOST === '1'/);

  assert.match(vite, /\/__makosh\/owner-vault-host\/v1/);
  assert.match(vite, /request\.removeHeader\(DEVELOPMENT_HOST_PROOF_HEADER\)/);
  assert.match(vite, /request\.setHeader\(DEVELOPMENT_HOST_PROOF_HEADER, host\.proof\)/);
  assert.match(ensemble, /--features development-server/);
  assert.match(ensemble, /makosh-owner-vault-development-host/);
  assert.match(ensemble, /VITE_MAKOSH_DEV_OWNER_VAULT_HOST=1/);
  assert.match(probe, /\/__makosh\/owner-vault-host\/v1\/start/);
  assert.match(probe, /\/__makosh\/owner-vault-host\/v1\/cancel/);
});
