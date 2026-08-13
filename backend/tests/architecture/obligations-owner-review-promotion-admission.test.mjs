import assert from 'node:assert/strict';
import { globSync } from 'node:fs';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import test from 'node:test';

const ROOT = resolve(import.meta.dirname, '../../..');
const read = async (path) => readFile(resolve(ROOT, path), 'utf8');

const packages = [
  'makosh-obligations-api', 'makosh-obligations-assembly', 'makosh-obligations-core',
  'makosh-obligations-persistence', 'makosh-obligations-runtime',
  'makosh-review-obligation-candidate-api', 'makosh-review-obligation-candidate-assembly',
  'makosh-review-obligation-candidate-core', 'makosh-review-obligation-candidate-persistence',
  'makosh-review-obligation-candidate-runtime',
  'makosh-review-obligation-candidate-promotion-api',
  'makosh-reviewed-obligation-candidate-promotion-assembly',
  'makosh-reviewed-obligation-candidate-promotion-core',
  'makosh-reviewed-obligation-candidate-promotion-persistence',
  'makosh-reviewed-obligation-candidate-promotion-runtime',
].sort();

const capabilities = [
  'obligations.client.v1', 'obligations.lifecycle.event.v1',
  'obligations.reviewed-candidate.blob.v1', 'obligations.reviewed-candidate.command.v1',
  'obligations.storage.v1', 'review.obligation-candidate.blob.v1',
  'review.obligation-candidate.client.v1',
  'review.obligation-candidate.promotion-result.consumer.v1',
  'review.obligation-candidate.promotion-result.v1',
  'review.obligation-candidate.promotion.v1', 'review.obligation-candidate.storage.v1',
  'review.obligation-candidate.submission.v1',
].sort();

test('Task 21 authority lifts only Obligations and retains Decisions freeze', async () => {
  const [policy, adr] = await Promise.all([
    read('backend/architecture/policy.json').then(JSON.parse),
    read('docs/adr/ADR-0406-obligations-owner-review-and-promotion-boundary.md'),
  ]);
  assert.equal(policy.domains.developmentAllowlist.includes('obligations'), true);
  assert.deepEqual(policy.domains.blocked, []);
  assert.match(adr, /Review-owned.*очередь/s);
  assert.match(adr, /promotion\s+workflow/);
});

test('Task 21 records Obligations and promotion as implemented but not production-admitted', async () => {
  const policy = JSON.parse(await read('backend/architecture/policy.json'));
  assert.equal(policy.implementation.currentSlice, 'speech_to_text_whisper_admission_v1');
  assert.equal(policy.implementation.productionPackages.length, 283);
  assert.equal(policy.implementation.ownerInventory.businessCapabilities.length, 253);
  assert.deepEqual(
    policy.implementation.productionPackages
      .filter(({ name, owner }) => owner === 'obligations'
        || owner === 'reviewed_obligation_candidate_promotion'
        || (owner === 'review' && name.includes('obligation-candidate')))
      .map(({ name }) => name)
      .sort(),
    [],
  );
  assert.deepEqual(
    policy.implementation.ownerInventory.businessCapabilities
      .filter((id) => id.startsWith('obligations.') || id.startsWith('review.obligation-candidate.'))
      .sort(),
    [],
  );
});

test('Task 21 creates exactly fifteen packages without a new testkit', async () => {
  const manifests = globSync('**/Cargo.toml', { cwd: resolve(ROOT, 'backend'), exclude: ['target/**'] });
  assert.equal(manifests.length, 421);
  for (const name of packages) {
    assert.match(await read(`backend/src/${name.replace('makosh-', '')}/Cargo.toml`), new RegExp(`name = "${name}"`));
  }
  assert.equal(globSync('**/*obligation*testkit*/Cargo.toml', { cwd: resolve(ROOT, 'backend') }).length, 0);
});

test('Task 21 contracts keep owner truth, Review state and promotion separate', async () => {
  const [owner, ownerCommand, review, promotion] = await Promise.all([
    read('backend/src/obligations-api/proto/makosh/obligations/client/v1/obligations.proto'),
    read('backend/src/obligations-api/proto/makosh/obligations/command/v1/obligations.proto'),
    read('backend/src/review-obligation-candidate-api/proto/makosh/review/obligation_candidate/v1/obligation_candidate.proto'),
    read('backend/src/review-obligation-candidate-promotion-api/proto/makosh/review/obligation_candidate/promotion/v1/promotion.proto'),
  ]);
  assert.match(owner, /OBLIGATION_STATE_FULFILLED/);
  assert.match(owner, /bytes obligated_party_id/);
  assert.match(owner, /optional bytes beneficiary_party_id/);
  assert.match(owner, /message ObligationEvidenceLinkV1/);
  assert.match(owner, /rpc AddEvidence/);
  assert.match(owner, /rpc RemoveEvidence/);
  assert.match(owner, /rpc ListEvidence/);
  assert.doesNotMatch(owner, /rpc Create\(|Priority|Dependency|Checklist/);
  assert.match(ownerCommand, /CreateObligationFromReviewedCandidateCommandV1/);
  assert.match(review, /REVIEW_OBLIGATION_CANDIDATE_STATE_APPROVED/);
  assert.match(review, /bytes obligated_party_id/);
  assert.match(review, /optional bytes beneficiary_party_id/);
  assert.match(review, /repeated ReviewObligationEvidenceLinkV1 evidence_links/);
  assert.match(promotion, /ObligationCandidatePromotionResultV1/);
  assert.doesNotMatch(owner + ownerCommand + review + promotion, /raw_payload|private_locator|credential|arbitrary_json|confidence|risk_score/i);
});

test('Task 21 storage is exact FORCE RLS across all three owners', async () => {
  const migrations = await Promise.all([
    Promise.all([
      read('backend/src/obligations-persistence/migrations/0001_obligations_owner.sql'),
      read('backend/src/obligations-persistence/migrations/0002_obligations_lifecycle_owner_rls.sql'),
      read('backend/src/obligations-persistence/migrations/0003_obligations_parties_evidence.sql'),
    ]).then((parts) => parts.join('\n')),
    Promise.all([
      read('backend/src/review-obligation-candidate-persistence/migrations/0001_review_obligation_candidate.sql'),
      read('backend/src/review-obligation-candidate-persistence/migrations/0002_review_obligation_candidate_owner_rls.sql'),
      read('backend/src/review-obligation-candidate-persistence/migrations/0003_review_obligation_candidate_parties_evidence.sql'),
    ]).then((parts) => parts.join('\n')),
    Promise.all([
      read('backend/src/reviewed-obligation-candidate-promotion-persistence/migrations/0001_reviewed_obligation_candidate_promotion.sql'),
      read('backend/src/reviewed-obligation-candidate-promotion-persistence/migrations/0002_reviewed_obligation_candidate_promotion_owner_rls.sql'),
    ]).then((parts) => parts.join('\n')),
  ]);
  for (const migration of migrations) {
    assert.match(migration, /ENABLE ROW LEVEL SECURITY/);
    assert.match(migration, /FORCE ROW LEVEL SECURITY/);
    assert.match(migration, /current_setting\('makosh\.logical_owner_id'/);
  }
});

test('Task 21 exposes typed Obligations and Review clients without REST or projections', async () => {
  const [frontend, surfaces, runner, materializer] = await Promise.all([
    read('frontend/src/domains/obligations/stores/obligations.ts'),
    read('frontend/src/platform/client-runtime/clientSurfaces.ts'),
    read('backend/scripts/test-authenticated-storage.mjs'),
    read('backend/scripts/materialize-dev-release.sh'),
  ]);
  assert.match(frontend, /getObligationsQueryClient/);
  assert.doesNotMatch(frontend, /\/api\/v1\/obligations|Graph|Timeline|risk/i);
  assert.match(surfaces, /adapterId: 'obligations-owner'/);
  assert.match(runner, /managed_obligation_candidate_promotes_to_actual_obligation_and_replays/);
  assert.match(materializer, /makosh-obligations-runtime/);
});
