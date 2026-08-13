import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import test from 'node:test';

const root = new URL('../../..', import.meta.url);
const read = (path) => readFileSync(new URL(path, root), 'utf8');
const policy = JSON.parse(read('backend/architecture/policy.json'));

test('Task 23 records Identity Resolution as implemented but not production-admitted', () => {
  assert.equal(policy.implementation.currentSlice, 'speech_to_text_whisper_admission_v1');
  assert.equal(policy.implementation.productionPackages.length, 283);
  assert.equal(policy.implementation.ownerInventory.businessCapabilities.length, 253);
  assert.equal(policy.implementation.ownerInventory.engines.includes('identity_resolution'), false);
  assert.deepEqual(
    policy.implementation.productionPackages.filter(({ owner }) => owner === 'identity_resolution'),
    [],
  );
});

test('Task 23 creates exactly five engine packages without a testkit', () => {
  const metadata = JSON.parse(execFileSync('cargo', ['metadata', '--no-deps', '--format-version', '1'], {
    cwd: new URL('backend', root), encoding: 'utf8', maxBuffer: 16 * 1024 * 1024,
  }));
  assert.equal(metadata.workspace_members.length, 420);
  for (const name of [
    'makosh-identity-resolution-api', 'makosh-identity-resolution-core',
    'makosh-identity-resolution-persistence', 'makosh-identity-resolution-runtime',
    'makosh-identity-resolution-assembly',
  ]) assert.ok(metadata.packages.some((value) => value.name === name), name);
  assert.ok(!existsSync(new URL('backend/tests/support/identity-resolution', root)));
});

test('Task 23 contract is typed, private-free and cannot command Persons', () => {
  const proto = read('backend/src/identity-resolution-api/proto/makosh/identity_resolution/v1/identity_resolution.proto');
  assert.match(proto, /message PersonLinkMergeCandidateProposedEventV1/);
  assert.match(proto, /evidence_event_id/);
  assert.match(proto, /candidate_id/);
  assert.match(proto, /match_kind/);
  assert.doesNotMatch(proto, /raw_email|raw_phone|email_value|phone_value|credential|provider_payload|private_locator|confidence|risk|map<|json/i);
  const runtime = read('backend/src/identity-resolution-runtime/src/admission.rs');
  assert.doesNotMatch(runtime, /persons_command_(?:consume|publish)_request_v1/);
  assert.doesNotMatch(runtime, /client_rpc_route:\s*Some/);
});

test('Task 23 Review consumes only the engine proposal boundary', () => {
  const admission = read('backend/src/review-person-match-candidate-runtime/src/admission.rs');
  assert.match(admission, /identity_resolution_person_match_candidate_contract_reference_v1/);
  assert.doesNotMatch(admission, /persons_review_candidate_contract_reference_v1/);
});

test('Task 23 persistence and release are executable without frontend surface', () => {
  const migration = read('backend/src/identity-resolution-persistence/migrations/0001_identity_resolution.sql');
  assert.match(migration, /ENABLE ROW LEVEL SECURITY/);
  assert.match(migration, /FORCE ROW LEVEL SECURITY/);
  assert.match(migration, /current_setting\('makosh\.logical_owner_id'/);
  assert.match(read('backend/scripts/materialize-dev-release.sh'), /makosh-identity-resolution-runtime/);
  assert.doesNotMatch(read('backend/src/api/gateway/contracts/proto/makosh/gateway/v1/client_bootstrap.proto'), /IdentityResolution/);
  assert.ok(!existsSync(new URL('frontend/src/platform/connect/identityResolutionClient.ts', root)));
});
