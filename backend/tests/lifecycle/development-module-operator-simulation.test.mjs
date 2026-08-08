import assert from 'node:assert/strict';
import { mkdtemp, readFile, rm, symlink, writeFile } from 'node:fs/promises';
import { spawnSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';
test('development operator simulates module trust lifecycle outside production Kernel', async () => {
  const root = await mkdtemp(join(tmpdir(), 'makosh-kernel-development-registration-'));
  const dataDir = join(root, 'data');
  const descriptor = join(root, 'module-descriptor.bin');
  const distributionArtifact = join(root, 'compose-postgres-artifact');
  const text = (value) => [value.length, ...Buffer.from(value, 'ascii')];
  const capability = [0x0a, ...text('capability.read'), 0x10, 0x01, 0x18, 0x01];
  const bytes = Buffer.from([
    0x08, 0x01, 0x10, 0x01,
    0x1a, ...text('module-1'),
    0x22, ...text('owner-1'),
    0x28, 0x02,
    0x32, ...text('1'),
    0x3a, ...text('build-1'),
    0x4a, capability.length, ...capability,
  ]);
  try {
    await writeFile(descriptor, bytes, { mode: 0o600 });
    await writeFile(distributionArtifact, 'development distribution bytes', { mode: 0o600 });
    const enrolled = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'makosh-kernel', '--', '--data-dir', dataDir, 'initial-owner-enroll', '--owner-id', 'dev-owner', '--device-id', 'dev-device'],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.equal(enrolled.status, 0, enrolled.stderr);
    const missingProfile = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'makosh-kernel', '--', '--data-dir', dataDir, 'module-register', '--descriptor', descriptor],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.notEqual(missingProfile.status, 0);
    assert.match(missingProfile.stderr, /unrecognized subcommand/);
    const registered = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'makosh-development-kernel-operator', '--', '--data-dir', dataDir, 'module-register', '--descriptor', descriptor],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.equal(registered.status, 0, registered.stderr);
    assert.match(registered.stdout, /^module_registration_id=[0-9a-f]{32}\nmodule_registration_state=pending\n$/);
    const registrationId = registered.stdout.match(/^module_registration_id=([0-9a-f]{32})$/m)?.[1];
    assert.ok(registrationId);
    const approved = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'makosh-development-kernel-operator', '--', '--data-dir', dataDir, 'module-approve', '--registration-id', registrationId, '--capability', 'capability.read'],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.equal(approved.status, 0, approved.stderr);
    assert.match(approved.stdout, new RegExp(`^module_registration_id=${registrationId}\\nmodule_grant_epoch=2\\neffective_capability_count=1\\n$`));
    const pinned = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'makosh-development-kernel-operator', '--', '--data-dir', dataDir, 'module-owner-pin-artifact', '--registration-id', registrationId, '--artifact', distributionArtifact],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.equal(pinned.status, 0, pinned.stderr);
    assert.match(pinned.stdout, new RegExp(`^module_registration_id=${registrationId}\\nowner_pinned_artifact_binding_revision=1\\nowner_pinned_artifact_sha256=[0-9a-f]{64}\\n$`));
    const verifiedPinnedArtifact = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'makosh-development-kernel-operator', '--', '--data-dir', dataDir, 'module-owner-pinned-preflight', '--registration-id', registrationId],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.equal(verifiedPinnedArtifact.status, 0, verifiedPinnedArtifact.stderr);
    assert.match(verifiedPinnedArtifact.stdout, new RegExp(`^module_registration_id=${registrationId}\\nowner_pinned_artifact_binding_revision=1\\nowner_pinned_artifact_preflight=verified\\n$`));
    await writeFile(distributionArtifact, 'changed development distribution bytes', { mode: 0o600 });
    const stalePinnedArtifact = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'makosh-development-kernel-operator', '--', '--data-dir', dataDir, 'module-owner-pinned-preflight', '--registration-id', registrationId],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.notEqual(stalePinnedArtifact.status, 0);
    assert.match(stalePinnedArtifact.stderr, /no longer matches its approved binding/);
    const repinned = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'makosh-development-kernel-operator', '--', '--data-dir', dataDir, 'module-owner-pin-artifact', '--registration-id', registrationId, '--artifact', distributionArtifact],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.equal(repinned.status, 0, repinned.stderr);
    assert.match(repinned.stdout, new RegExp(`^module_registration_id=${registrationId}\\nowner_pinned_artifact_binding_revision=2\\nowner_pinned_artifact_sha256=[0-9a-f]{64}\\n$`));
    const attested = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'makosh-development-kernel-operator', '--', '--data-dir', dataDir, 'module-external-attest', '--registration-id', registrationId, '--runtime-id', 'compose-postgres', '--runtime-generation', '1', '--distribution-artifact', distributionArtifact],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.equal(attested.status, 0, attested.stderr);
    assert.match(attested.stdout, new RegExp(`^module_registration_id=${registrationId}\\nexternal_runtime_id=compose-postgres\\nexternal_runtime_generation=1\\nexternal_runtime_grant_epoch=2\\n$`));
    const attestationReplay = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'makosh-development-kernel-operator', '--', '--data-dir', dataDir, 'module-external-attest', '--registration-id', registrationId, '--runtime-id', 'compose-postgres', '--runtime-generation', '1', '--distribution-artifact', distributionArtifact],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.notEqual(attestationReplay.status, 0);
    assert.match(attestationReplay.stderr, /StaleExternalRuntimeAttestation/);
    const distributionSymlink = join(root, 'compose-postgres-artifact-link');
    await symlink(distributionArtifact, distributionSymlink);
    const symlinkedArtifact = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'makosh-development-kernel-operator', '--', '--data-dir', dataDir, 'module-external-attest', '--registration-id', registrationId, '--runtime-id', 'compose-postgres', '--runtime-generation', '2', '--distribution-artifact', distributionSymlink],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.notEqual(symlinkedArtifact.status, 0);
    assert.match(symlinkedArtifact.stderr, /distribution artifact must not be a symlink/);
    const deviceKeyPath = join(dataDir, 'device-es256.key');
    const originalDeviceKey = await readFile(deviceKeyPath);
    await writeFile(deviceKeyPath, Buffer.alloc(32), { mode: 0o600 });
    const rejectedWithReplacedKey = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'makosh-development-kernel-operator', '--', '--data-dir', dataDir, 'module-transition', '--registration-id', registrationId, '--state', 'suspended'],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.notEqual(rejectedWithReplacedKey.status, 0);
    assert.match(rejectedWithReplacedKey.stderr, /device key is invalid/);
    await writeFile(deviceKeyPath, originalDeviceKey, { mode: 0o600 });
    const suspended = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'makosh-development-kernel-operator', '--', '--data-dir', dataDir, 'module-transition', '--registration-id', registrationId, '--state', 'suspended'],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.equal(suspended.status, 0, suspended.stderr);
    assert.match(suspended.stdout, new RegExp(`^module_registration_id=${registrationId}\\nmodule_registration_state=suspended\\nmodule_grant_epoch=3\\n$`));
    const status = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'makosh-development-kernel-operator', '--', '--data-dir', dataDir, 'module-status', '--registration-id', registrationId],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.equal(status.status, 0, status.stderr);
    assert.match(status.stdout, new RegExp(`^module_registration_id=${registrationId}\\nmodule_registration_state=suspended\\nmodule_grant_epoch=3\\neffective_capability_count=0\\nexternal_runtime_attested=false\\n$`));
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});
