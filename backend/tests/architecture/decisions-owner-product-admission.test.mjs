import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import test from 'node:test';

const root = new URL('../../..', import.meta.url);
const read = (path) => readFileSync(new URL(path, root), 'utf8');
const policy = JSON.parse(read('backend/architecture/policy.json'));

test('Task 22 lifts only Decisions development authority', () => {
  assert.ok(policy.domains.developmentAllowlist.includes('decisions'));
  assert.ok(!policy.domains.blocked.includes('decisions'));
  assert.match(read('docs/adr/ADR-0407-decisions-owner-alternatives-evidence-and-product-boundary.md'), /Status|Статус/);
});

test('Task 22 records Decisions as implemented but not production-admitted', () => {
  assert.equal(policy.implementation.currentSlice, 'speech_to_text_whisper_admission_v1');
  assert.equal(policy.implementation.productionPackages.length, 283);
  assert.equal(policy.implementation.ownerInventory.businessCapabilities.length, 253);
  assert.deepEqual(policy.implementation.productionPackages.filter(({ owner }) => owner === 'decisions'), []);
  assert.deepEqual(
    policy.implementation.ownerInventory.businessCapabilities.filter((id) => id.startsWith('decisions.')),
    [],
  );
});

test('Task 22 creates exactly five packages without a testkit', () => {
  const metadata = JSON.parse(execFileSync('cargo', ['metadata', '--no-deps', '--format-version', '1'], {
    cwd: new URL('backend', root), encoding: 'utf8', maxBuffer: 16 * 1024 * 1024,
  }));
  assert.equal(metadata.workspace_members.length, 420);
  for (const name of [
    'makosh-decisions-api', 'makosh-decisions-core', 'makosh-decisions-persistence',
    'makosh-decisions-runtime', 'makosh-decisions-assembly',
  ]) assert.ok(metadata.packages.some((value) => value.name === name), name);
  assert.ok(!existsSync(new URL('backend/tests/support/decisions', root)));
});

test('Task 22 contract is typed and excludes Review or generic projection truth', () => {
  const proto = read('backend/src/decisions-api/proto/makosh/decisions/client/v1/decisions.proto');
  assert.match(proto, /DECISION_STATE_DRAFT/);
  assert.match(proto, /DECISION_STATE_DECIDED/);
  assert.match(proto, /message DecisionAlternativeV1/);
  assert.match(proto, /message DecisionEvidenceLinkV1/);
  assert.match(proto, /rpc Decide/);
  assert.match(proto, /rpc Supersede/);
  assert.match(proto, /rpc ListAlternatives/);
  assert.match(proto, /rpc ListEvidence/);
  assert.doesNotMatch(proto, /arbitrary_json|confidence|risk_score|ReviewCandidate|provider_payload/i);
});

test('Task 22 storage, release and frontend are executable owner surfaces', () => {
  const migration = read('backend/src/decisions-persistence/migrations/0001_decisions_owner.sql');
  assert.match(migration, /ENABLE ROW LEVEL SECURITY/);
  assert.match(migration, /FORCE ROW LEVEL SECURITY/);
  assert.match(migration, /current_setting\('makosh\.logical_owner_id'/);
  assert.match(read('backend/scripts/materialize-dev-release.sh'), /makosh-decisions-runtime/);
  assert.match(read('frontend/src/domains/decisions/stores/decisions.ts'), /getDecisionsQueryClient/);
  assert.doesNotMatch(read('frontend/src/domains/decisions/stores/decisions.ts'), /\/api\/v1\/decisions/);
});
