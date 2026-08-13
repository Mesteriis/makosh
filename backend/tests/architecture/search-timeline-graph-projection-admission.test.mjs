import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import test from 'node:test';

const root = new URL('../../..', import.meta.url);
const read = (path) => readFileSync(new URL(path, root), 'utf8');
const policy = JSON.parse(read('backend/architecture/policy.json'));

const projections = ['search', 'timeline', 'graph'];

test('Task 24 records Search Timeline and Graph as implemented but not production-admitted', () => {
  assert.equal(policy.implementation.currentSlice, 'speech_to_text_whisper_admission_v1');
  assert.equal(policy.implementation.productionPackages.length, 283);
  assert.equal(policy.implementation.ownerInventory.businessCapabilities.length, 253);
  assert.equal(policy.implementation.ownerInventory.projections, undefined);
  for (const owner of projections) {
    assert.deepEqual(policy.implementation.productionPackages.filter((value) => value.owner === owner), []);
  }
});

test('Task 24 creates exactly fifteen projection packages', () => {
  const metadata = JSON.parse(execFileSync('cargo', ['metadata', '--no-deps', '--format-version', '1'], {
    cwd: new URL('backend', root), encoding: 'utf8', maxBuffer: 16 * 1024 * 1024,
  }));
  assert.equal(metadata.workspace_members.length, 420);
  for (const owner of projections) {
    for (const surface of ['api', 'core', 'persistence', 'runtime', 'assembly']) {
      assert.ok(metadata.packages.some((value) => value.name === `makosh-${owner}-${surface}`), `${owner}/${surface}`);
    }
    assert.ok(!existsSync(new URL(`backend/tests/support/${owner}`, root)));
  }
});

test('Task 24 clients are read-only and projections cannot command owners', () => {
  for (const owner of projections) {
    const proto = read(`backend/src/${owner}-api/proto/makosh/${owner}/v1/${owner}.proto`);
    assert.doesNotMatch(proto, /rpc\s+(Create|Update|Delete|Mutate|Command)/);
    assert.doesNotMatch(proto, /credential|provider_payload|private_locator|confidence|risk|map<|json/i);
    const admission = read(`backend/src/${owner}-runtime/src/admission.rs`);
    assert.doesNotMatch(
      admission,
      /kind:\s*ProvidedSurfaceKindV1::DurablePublisher|ProvidedSurfaceKindV1::Command/,
    );
  }
});

test('Task 24 storage is rebuildable, owner-isolated and deletion-aware', () => {
  for (const owner of projections) {
    const migration = read(`backend/src/${owner}-persistence/migrations/0001_${owner}.sql`);
    assert.match(migration, /projection_generation/);
    assert.match(migration, /source_revision/);
    assert.match(migration, /deleted_at/);
    assert.match(migration, /ENABLE ROW LEVEL SECURITY/);
    assert.match(migration, /FORCE ROW LEVEL SECURITY/);
    assert.match(migration, /current_setting\('makosh\.logical_owner_id'/);
  }
});

test('Task 24 release and frontend expose three exact read projections only', () => {
  const materializer = read('backend/scripts/materialize-dev-release.sh');
  const bootstrap = read('backend/src/api/gateway/contracts/proto/makosh/gateway/v1/client_bootstrap.proto');
  for (const owner of projections) {
    assert.match(materializer, new RegExp(`makosh-${owner}-runtime`));
    assert.match(bootstrap, new RegExp(`${owner}`, 'i'));
    assert.ok(existsSync(new URL(`frontend/src/platform/connect/${owner}Client.ts`, root)));
  }
});
