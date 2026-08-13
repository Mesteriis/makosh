import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import test from 'node:test';

const root = new URL('../../..', import.meta.url);
const read = (path) => readFileSync(new URL(path, root), 'utf8');
const policy = JSON.parse(read('backend/architecture/policy.json'));
const owners = ['zoom', 'telemost', 'omniroute'];

test('Task 26 records external provider contours without production admission', () => {
  assert.equal(policy.implementation.currentSlice, 'speech_to_text_whisper_admission_v1');
  assert.equal(policy.implementation.productionPackages.length, 283);
  assert.equal(policy.implementation.ownerInventory.businessCapabilities.length, 253);
  for (const owner of owners) {
    assert.equal(policy.implementation.ownerInventory.integrations.includes(owner), false);
    assert.deepEqual(policy.implementation.productionPackages.filter((value) => value.owner === owner), []);
  }
});

test('Task 26 creates exactly fifteen isolated packages', () => {
  const metadata = JSON.parse(execFileSync('cargo', ['metadata', '--no-deps', '--format-version', '1'], {
    cwd: new URL('backend', root), encoding: 'utf8', maxBuffer: 16 * 1024 * 1024,
  }));
  assert.equal(metadata.workspace_members.length, 420);
  for (const owner of owners) {
    for (const surface of ['api', 'core', 'persistence', 'runtime', 'assembly']) {
      assert.ok(metadata.packages.some(({ name }) => name === `makosh-${owner}-${surface}`));
    }
  }
});

test('provider boundaries are Vault-only and private-free', () => {
  for (const owner of owners) {
    const admission = read(`backend/src/${owner}-runtime/src/admission.rs`);
    const migration = read(`backend/src/${owner}-persistence/migrations/0001_${owner}.sql`);
    assert.match(admission, /VaultPurposeRequestV1/);
    assert.doesNotMatch(admission, /std::env|API_KEY|TOKEN/);
    assert.match(migration, /ENABLE ROW LEVEL SECURITY/);
    assert.match(migration, /FORCE ROW LEVEL SECURITY/);
    assert.match(migration, /current_setting\('makosh\.logical_owner_id'/);
    assert.doesNotMatch(migration, /credential|access_token|refresh_token|provider_payload|webhook_body|prompt|response_body/i);
  }
});

test('Zoom and Telemost hand off only sanitized Communications call evidence', () => {
  for (const owner of ['zoom', 'telemost']) {
    const manifest = read(`backend/src/${owner}-runtime/Cargo.toml`);
    const runtime = read(`backend/src/${owner}-runtime/src/lib.rs`);
    assert.match(manifest, /makosh-communications-call-evidence-ingress/);
    assert.match(runtime, /build_call_evidence_observed_outbox_record_v1/);
    assert.doesNotMatch(runtime, /communications.*persistence|demo.call|recording_start/i);
  }
});

test('release inventory has six compiler-consumed artifacts and no legacy façade', () => {
  for (const owner of owners) {
    assert.ok(existsSync(new URL(`backend/src/${owner}-assembly`, root)));
    const assembly = read(`backend/src/${owner}-assembly/src/lib.rs`);
    assert.match(assembly, /module_runtime/);
    assert.match(assembly, /storage_bundle/);
  }
  assert.equal(existsSync(new URL('backend/src/demo-calls-runtime', root)), false);
  assert.equal(existsSync(new URL('frontend/src/platform/connect/omnirouteClient.ts', root)), false);
});
