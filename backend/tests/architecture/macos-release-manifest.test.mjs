import assert from 'node:assert/strict';
import test from 'node:test';

import { validateManifest } from '../../scripts/verify-macos-release-manifest.mjs';

const digest = '0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef';

function validManifest() {
  return {
    schema_version: 1,
    deployment_profile: 'macos_tauri_embedded_v1',
    runtime_lifecycle: 'managed_child',
    signing: { team_id: 'AB12CD34EF' },
    artifacts: {
      tauri_bundle: { path: '/Applications/Макошь.app' },
      kernel_sidecar: {
        path: '/Applications/Макошь.app/Contents/MacOS/makosh-kernel',
        sha256: digest,
      },
      release_trust_root: {
        path: '/Applications/Макошь.app/Contents/Resources/makosh-kernel-release/makosh-release-trust-root.pb',
        sha256: digest,
      },
      signed_distribution_manifest: {
        path: '/Applications/Макошь.app/Contents/Resources/makosh-kernel-release/makosh-signed-distribution-manifest.pb',
        sha256: digest,
      },
      distribution_root: {
        path: '/Applications/Макошь.app/Contents/Resources/makosh-kernel-release/distribution',
      },
    },
  };
}

test('accepts a macOS managed-child release manifest with an exact Kernel digest', () => {
  assert.deepEqual(validateManifest(validManifest()), []);
});

test('rejects non-macOS profiles, malformed Team IDs and missing Kernel digest', () => {
  const wrongLifecycle = validManifest();
  wrongLifecycle.runtime_lifecycle = 'external_compose';
  assert.ok(validateManifest(wrongLifecycle).some((error) => error.includes('runtime_lifecycle')));

  const weakIdentity = validManifest();
  weakIdentity.signing.team_id = 'unverified';
  assert.ok(validateManifest(weakIdentity).some((error) => error.includes('team_id')));

  const notAnAppBundle = validManifest();
  notAnAppBundle.artifacts.tauri_bundle.path = '/Applications/Макошь';
  assert.ok(validateManifest(notAnAppBundle).some((error) => error.includes('tauri_bundle')));

  const noDigest = validManifest();
  delete noDigest.artifacts.kernel_sidecar.sha256;
  assert.ok(validateManifest(noDigest).some((error) => error.includes('sha256')));

  const noTrustRoot = validManifest();
  delete noTrustRoot.artifacts.release_trust_root;
  assert.ok(validateManifest(noTrustRoot).some((error) => error.includes('release_trust_root')));

  const noSignedManifest = validManifest();
  delete noSignedManifest.artifacts.signed_distribution_manifest;
  assert.ok(validateManifest(noSignedManifest).some((error) => error.includes('signed_distribution_manifest')));
});

test('rejects a release manifest that points a signed artifact outside its Tauri bundle', () => {
  const detachedSidecar = validManifest();
  detachedSidecar.artifacts.kernel_sidecar.path = '/opt/makosh/makosh-kernel';
  assert.ok(validateManifest(detachedSidecar).some((error) => error.includes('kernel_sidecar')));

  const wrongResourceRoot = validManifest();
  wrongResourceRoot.artifacts.distribution_root.path = '/Applications/Макошь.app/Contents/Resources/distribution';
  assert.ok(validateManifest(wrongResourceRoot).some((error) => error.includes('distribution_root')));
});
