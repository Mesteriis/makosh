import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import test from 'node:test';

const root = new URL('../../..', import.meta.url);
const read = (path) => readFileSync(new URL(path, root), 'utf8');
const policy = JSON.parse(read('backend/architecture/policy.json'));
const contours = ['memory', 'consistency', 'risk'];

test('Task 25 records derived contours as implemented but not production-admitted', () => {
  assert.equal(policy.implementation.currentSlice, 'speech_to_text_whisper_admission_v1');
  assert.equal(policy.implementation.productionPackages.length, 283);
  assert.equal(policy.implementation.ownerInventory.businessCapabilities.length, 253);
  assert.equal(policy.implementation.ownerInventory.projections, undefined);
  assert.equal(policy.implementation.ownerInventory.engines.includes('consistency'), false);
  assert.equal(policy.implementation.ownerInventory.engines.includes('risk'), false);
});

test('Task 25 creates exactly fifteen isolated packages', () => {
  const metadata = JSON.parse(execFileSync('cargo', ['metadata', '--no-deps', '--format-version', '1'], {
    cwd: new URL('backend', root), encoding: 'utf8', maxBuffer: 16 * 1024 * 1024,
  }));
  assert.equal(metadata.workspace_members.length, 420);
  for (const owner of contours) {
    for (const surface of ['api', 'core', 'persistence', 'runtime', 'assembly']) {
      assert.ok(metadata.packages.some(({ name }) => name === `makosh-${owner}-${surface}`));
    }
  }
});

test('Task 25 contracts are bounded read models without owner commands or context payloads', () => {
  for (const owner of contours) {
    const proto = read(`backend/src/${owner}-api/proto/makosh/${owner}/v1/${owner}.proto`);
    assert.doesNotMatch(proto, /rpc\s+(Create|Update|Delete|Mutate|Command)/);
    assert.doesNotMatch(proto, /credential|provider_payload|private_locator|plaintext|map<|json|context_pack/i);
    const admission = read(`backend/src/${owner}-runtime/src/admission.rs`);
    assert.doesNotMatch(
      admission,
      /kind:\s*ProvidedSurfaceKindV1::(?:DurablePublisher|Command)/,
    );
  }
});

test('Task 25 storage is rebuildable owner-isolated and expiry or tombstone aware', () => {
  for (const owner of contours) {
    const migration = read(`backend/src/${owner}-persistence/migrations/0001_${owner}.sql`);
    assert.match(migration, /projection_generation/);
    assert.match(migration, /source_revision/);
    assert.match(migration, /deleted_at|expires_at/);
    assert.match(migration, /ENABLE ROW LEVEL SECURITY/);
    assert.match(migration, /FORCE ROW LEVEL SECURITY/);
    assert.match(migration, /current_setting\('makosh\.logical_owner_id'/);
  }
});

test('Signal Hub is app composition and the generic backend facade is absent', () => {
  assert.ok(existsSync(new URL('frontend/src/app/signalHub/signalHubComposition.ts', root)));
  assert.equal(existsSync(new URL('frontend/src/platform/connect/signalHubClient.ts', root)), false);
  assert.equal(existsSync(new URL('frontend/src/gen/makosh/signal_hub/v1/signal_hub_pb.ts', root)), false);
  assert.equal(existsSync(new URL('backend/src/signal-hub-runtime', root)), false);
  const composition = read('frontend/src/app/signalHub/signalHubComposition.ts');
  for (const owner of ['search', 'timeline', 'graph', ...contours]) {
    assert.match(composition, new RegExp(`${owner}Client`, 'i'));
  }
  assert.doesNotMatch(composition, /fetch\(|\/api\//);
});
