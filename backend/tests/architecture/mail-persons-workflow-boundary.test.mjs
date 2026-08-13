import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import test from 'node:test';

const PROJECT_ROOT = resolve(import.meta.dirname, '../../..');
const read = (path) => readFile(resolve(PROJECT_ROOT, path), 'utf8');

const expectedPackages = [
  { name: 'makosh-mail-persons-sync-api', role: 'workflow', owner: 'mail_persons_sync', surface: 'contract' },
  { name: 'makosh-mail-persons-sync-core', role: 'workflow', owner: 'mail_persons_sync', surface: 'implementation' },
  { name: 'makosh-mail-persons-sync-persistence', role: 'workflow', owner: 'mail_persons_sync', surface: 'persistence' },
  { name: 'makosh-mail-persons-sync-runtime', role: 'workflow', owner: 'mail_persons_sync', surface: 'runtime' },
  { name: 'makosh-mail-persons-sync-assembly', role: 'workflow', owner: 'mail_persons_sync', surface: 'assembly' },
  { name: 'makosh-review-person-match-candidate-api', role: 'domain', owner: 'review', surface: 'contract' },
  { name: 'makosh-review-person-match-candidate-core', role: 'domain', owner: 'review', surface: 'implementation' },
  { name: 'makosh-review-person-match-candidate-persistence', role: 'domain', owner: 'review', surface: 'persistence' },
  { name: 'makosh-review-person-match-candidate-runtime', role: 'domain', owner: 'review', surface: 'runtime' },
  { name: 'makosh-review-person-match-candidate-assembly', role: 'domain', owner: 'review', surface: 'assembly' },
  { name: 'makosh-review-person-match-candidate-promotion-api', role: 'domain', owner: 'review', surface: 'contract' },
  { name: 'makosh-reviewed-person-match-candidate-promotion-core', role: 'workflow', owner: 'reviewed_person_match_candidate_promotion', surface: 'implementation' },
  { name: 'makosh-reviewed-person-match-candidate-promotion-persistence', role: 'workflow', owner: 'reviewed_person_match_candidate_promotion', surface: 'persistence' },
  { name: 'makosh-reviewed-person-match-candidate-promotion-runtime', role: 'workflow', owner: 'reviewed_person_match_candidate_promotion', surface: 'runtime' },
  { name: 'makosh-reviewed-person-match-candidate-promotion-assembly', role: 'workflow', owner: 'reviewed_person_match_candidate_promotion', surface: 'assembly' },
];

test('Task 6 keeps the exact Mail-Persons and Review production package families', async () => {
  const policy = JSON.parse(await read('backend/architecture/policy.json'));
  const names = new Set(expectedPackages.slice(0, 2).map(({ name }) => name));
  assert.deepEqual(
    policy.implementation.productionPackages.filter(({ name }) => names.has(name)),
    expectedPackages.slice(0, 2),
  );
  assert.equal(policy.implementation.productionPackages.length, 283);
});

test('later Task 5 slices declare the remaining dormant package families', async () => {
  const policy = JSON.parse(await read('backend/architecture/policy.json'));
  const remaining = expectedPackages.slice(5);
  const names = new Set(remaining.map(({ name }) => name));
  assert.deepEqual(
    policy.implementation.productionPackages.filter(({ name }) => names.has(name)),
    remaining,
  );
});

test('Task 5B declares the exact dormant Mail workflow contour and testkit', async () => {
  const [policy, workspace, testkit] = await Promise.all([
    read('backend/architecture/policy.json').then(JSON.parse),
    read('backend/Cargo.toml'),
    read('backend/tests/support/mail-persons-sync-persistence/Cargo.toml'),
  ]);
  const task5b = expectedPackages.slice(2, 5);
  const names = new Set(task5b.map(({ name }) => name));
  assert.deepEqual(
    policy.implementation.productionPackages.filter(({ name }) => names.has(name)),
    task5b,
  );
  assert.equal(policy.implementation.productionPackages.length, 283);
  assert.match(workspace, /"src\/mail-persons-sync-persistence"/);
  assert.match(workspace, /"src\/mail-persons-sync-runtime"/);
  assert.match(workspace, /"src\/mail-persons-sync-assembly"/);
  assert.match(workspace, /"tests\/support\/mail-persons-sync-persistence"/);
  assert.match(testkit, /name = "makosh-mail-persons-sync-persistence-testkit"/);
  assert.match(testkit, /sqlx = .*features = \["postgres"/);
});

test('Task 6 activates only the sanitized Mail Person-source producer', async () => {
  const [mailRuntime, oldWorker, policy] = await Promise.all([
    read('backend/src/mail-runtime/src/lib.rs'),
    read('backend/src/mail-runtime/src/address_book_fetch_worker.rs'),
    read('backend/architecture/policy.json').then(JSON.parse),
  ]);
  assert.match(mailRuntime, /person_source_fetch_worker/);
  assert.match(oldWorker, /fetch_provider_person_source_page_v1/);
  assert.doesNotMatch(mailRuntime + oldWorker, /mail_contacts_sync/i);
  assert.equal(policy.implementation.ownerInventory.workflows.includes('mail_persons_sync'), true);
  assert.equal(policy.implementation.currentSlice, 'speech_to_text_whisper_admission_v1');
});

test('Task 5B has one reclaiming run start and durable predecessor supersession path', async () => {
  const [repository, schema] = await Promise.all([
    read('backend/src/mail-persons-sync-persistence/src/repository.rs'),
    read('backend/src/mail-persons-sync-persistence/migrations/0001_mail_persons_sync.sql'),
  ]);
  assert.doesNotMatch(repository, /pub async fn begin_run_once\s*\(/);
  assert.match(repository, /begin_run_reclaiming_expired_once/);
  assert.match(repository, /superseded_by_run_id/);
  assert.match(repository, /semantic_kind\s*<>\s*7/);
  assert.match(schema, /superseded_by_run_id BYTEA/);
  assert.match(schema, /superseded_at_unix_millis BIGINT/);
});

test('Task 5B relays through a locked outbox claim across publish and reclaim', async () => {
  const [repository, runtime] = await Promise.all([
    read('backend/src/mail-persons-sync-persistence/src/repository.rs'),
    read('backend/src/mail-persons-sync-runtime/src/managed_runtime.rs'),
  ]);
  assert.match(repository, /claim_next_pending_outbox/);
  assert.match(repository, /FOR UPDATE SKIP LOCKED/);
  assert.match(repository, /MailPersonsSyncOutboxPublishClaimV1/);
  assert.match(runtime, /claim_next_pending_outbox/);
  assert.doesNotMatch(runtime, /load_pending_outbox/);
  assert.doesNotMatch(runtime, /mark_outbox_published/);
});

test('dormant Mail fetch replay is checked before entropy and provider classification', async () => {
  const repository = await read(
    'backend/src/mail-address-book-persistence/src/person_source_repository.rs',
  );
  assert.match(
    repository,
    /mail_address_book_person_source_fetch_inbox[\s\S]*return Ok\(MailPersonSourceAtomicFetchOutcomeV1[\s\S]*prepare_observations\(\)/,
  );
  assert.doesNotMatch(
    repository,
    /pub struct MailPersonSourceAtomicFetchCommitV1[\s\S]*pub observations:/,
  );
});

test('dormant Mail source mutation is exposed only by the high-level page transaction', async () => {
  const [repository, producer, runtimeManifest] = await Promise.all([
    read('backend/src/mail-address-book-persistence/src/person_source_repository.rs'),
    read('backend/src/mail-runtime/src/person_source_producer.rs'),
    read('backend/src/mail-runtime/Cargo.toml'),
  ]);
  assert.doesNotMatch(
    repository,
    /(?<!#\[cfg\(feature = "conformance-test-support"\)\]\n\s*)pub async fn observe_person_source_contact\s*\(/,
  );
  assert.doesNotMatch(
    repository,
    /(?<!#\[cfg\(feature = "conformance-test-support"\)\]\n\s*)pub async fn ensure_person_source_contact_mapping\s*\(/,
  );
  assert.doesNotMatch(producer, /pub async fn observe_public_source_mapping_v1\s*\(/);
  assert.match(repository, /pub async fn commit_person_source_fetch_atomically_once/);
  assert.doesNotMatch(
    runtimeManifest,
    /makosh-mail-address-book-persistence\/conformance-test-support/,
  );
});

test('dormant Mail producer exact-decodes every sanitized envelope before commit', async () => {
  const repository = await read(
    'backend/src/mail-address-book-persistence/src/person_source_repository.rs',
  );
  assert.match(repository, /DurableEnvelopeV1::decode/);
  assert.match(repository, /validate_fetch_command_envelope_v1/);
  assert.match(repository, /validate_fetch_output_envelopes_v1/);
  assert.match(repository, /validate_snapshot_terminal_envelope_v1/);
  assert.match(repository, /validate_removal_output_envelopes_v1/);
  assert.doesNotMatch(
    repository,
    /pub struct MailPersonSourceSnapshotCommitV1[\s\S]*pub terminal_fingerprint:/,
  );
  assert.match(repository, /mail\.person-source\.terminal-fingerprint\.v1/);
});

test('Mail exposes a sanitized public Person-source contract without provider-private fields', async () => {
  const proto = await read(
    'backend/src/mail-address-book-contract/proto/makosh/mail/address_book/person_source/v1/person_source.proto',
  );
  for (const message of [
    'FetchMailPersonSourcePageCommandV1',
    'MailPersonSourceObservedV1',
    'MailPersonSourceUpdatedV1',
    'MailPersonSourceRemovedV1',
    'MailPersonSourcePageCompletedV1',
    'MailPersonSourcePageRejectedV1',
  ]) assert.match(proto, new RegExp(`message ${message}`));
  assert.doesNotMatch(
    proto,
    /provider_entry_id|provider_etag|continuation_cursor|credential|private_locator|raw_payload|error_detail/i,
  );
});

test('Review owns decisions and the promotion workflow alone publishes confirmed Persons commands', async () => {
  const [reviewProto, promotionProto, promotionRuntime] = await Promise.all([
    read('backend/src/review-person-match-candidate-api/proto/makosh/review/person_match_candidate/v1/person_match_candidate.proto'),
    read('backend/src/review-person-match-candidate-promotion-api/proto/makosh/review/person_match_candidate/promotion/v1/promotion.proto'),
    read('backend/src/reviewed-person-match-candidate-promotion-runtime/src/approval.rs'),
  ]);
  assert.match(reviewProto, /PersonMatchCandidateApprovedForPromotionV1/);
  assert.match(reviewProto, /PERSON_MATCH_CANDIDATE_DECISION_APPROVE/);
  assert.match(reviewProto, /PERSON_MATCH_CANDIDATE_DECISION_REJECT/);
  assert.match(promotionProto, /ReviewPersonMatchCandidatePromotionResultV1/);
  assert.match(promotionRuntime, /build_persons_command_outbox_record_v1/);
  assert.match(promotionRuntime, /approved_action_digest/);
  assert.doesNotMatch(reviewProto + promotionProto, /string normalized_(?:email|phone)|provider_entry_id|provider_etag|continuation_cursor/i);
});

test('Task 6 admits the successor owner set and retires Contacts-era admission', async () => {
  const policy = JSON.parse(await read('backend/architecture/policy.json'));
  const inventory = policy.implementation.ownerInventory;
  assert.equal(policy.implementation.currentSlice, 'speech_to_text_whisper_admission_v1');
  assert.equal(inventory.workflows.includes('mail_contacts_sync'), false);
  assert.equal(inventory.workflows.includes('mail_persons_sync'), true);
  assert.equal(inventory.workflows.includes('reviewed_person_match_candidate_promotion'), true);
  assert.equal(inventory.domains.includes('persons'), true);
  const serialized = JSON.stringify(inventory);
  assert.doesNotMatch(serialized, /mail_contacts_sync|contacts/);
});
