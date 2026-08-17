import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import test from 'node:test';

const PROJECT_ROOT = resolve(import.meta.dirname, '../../..');
const absolute = (path) => resolve(PROJECT_ROOT, path);
const read = (path) => readFile(absolute(path), 'utf8');

const retiredProductionPackages = [
  'makosh-contacts-command-api',
  'makosh-contacts-mail-sync-source-api',
  'makosh-contacts-core',
  'makosh-contacts-persistence',
  'makosh-contacts-runtime',
  'makosh-contacts-assembly',
  'makosh-mail-contacts-sync-api',
  'makosh-mail-contacts-sync-core',
  'makosh-mail-contacts-sync-persistence',
  'makosh-mail-contacts-sync-runtime',
  'makosh-mail-contacts-sync-assembly',
];

const successorOwners = {
  domains: ['persons'],
  workflows: ['mail_persons_sync', 'reviewed_person_match_candidate_promotion'],
};

const task6CapabilityIds = [
  'mail.person-source.provider.v1',
  'mail_persons_sync.mail.account-ready.v1',
  'mail_persons_sync.mail.account-retired.v1',
  'mail_persons_sync.mail.fetch-page.v1',
  'mail_persons_sync.mail.page-completed.v1',
  'mail_persons_sync.mail.page-rejected.v1',
  'mail_persons_sync.mail.source-observed.v1',
  'mail_persons_sync.mail.source-removed.v1',
  'mail_persons_sync.mail.source-updated.v1',
  'mail_persons_sync.page-receipt.v1',
  'mail_persons_sync.persons.command-rejected.v1',
  'mail_persons_sync.persons.command-succeeded.v1',
  'mail_persons_sync.persons.command.v1',
  'mail_persons_sync.run-result.v1',
  'mail_persons_sync.scheduler.receipt.v1',
  'mail_persons_sync.scheduler.v1',
  'mail_persons_sync.scheduler_schedule_command.v1',
  'mail_persons_sync.scheduler_schedule_result.v1',
  'mail_persons_sync.storage.v1',
  'persons.client.v1',
  'persons.command-rejected.v1',
  'persons.command-succeeded.v1',
  'persons.command.v1',
  'persons.owner-event.v1',
  'persons.review-candidate.v1',
  'persons.storage.v1',
  'review.person-match-candidate.approved.publisher.v1',
  'review.person-match-candidate.client.v1',
  'review.person-match-candidate.decision.consumer.v1',
  'review.person-match-candidate.persons-candidate.consumer.v1',
  'review.person-match-candidate.promotion-result.consumer.v1',
  'review.person-match-candidate.storage.v1',
  'review.person-match-candidate.submission-rejected.publisher.v1',
  'review.person-match-candidate.submitted.publisher.v1',
  'reviewed-person-match-candidate-promotion.approval.consumer.v1',
  'reviewed-person-match-candidate-promotion.persons-command.publisher.v1',
  'reviewed-person-match-candidate-promotion.persons-rejected.consumer.v1',
  'reviewed-person-match-candidate-promotion.persons-succeeded.consumer.v1',
  'reviewed-person-match-candidate-promotion.result.publisher.v1',
  'reviewed-person-match-candidate-promotion.storage.v1',
];

test('Task 6 atomically replaces Contacts admission with Persons admission', async () => {
  const policy = JSON.parse(await read('backend/architecture/policy.json'));
  const implementation = policy.implementation;
  const packageNames = new Set(implementation.productionPackages.map(({ name }) => name));

  assert.equal(implementation.currentSlice, 'speech_to_text_whisper_admission_v1');
  assert.equal(implementation.productionPackages.length, 283);
  for (const name of retiredProductionPackages) assert.equal(packageNames.has(name), false, name);
  assert.equal(implementation.ownerInventory.domains.includes('contacts'), false);
  assert.equal(implementation.ownerInventory.domains.includes('persons'), true);
  assert.equal(implementation.ownerInventory.workflows.includes('mail_contacts_sync'), false);
  for (const owner of successorOwners.workflows) {
    assert.equal(implementation.ownerInventory.workflows.includes(owner), true, owner);
  }
  assert.equal(implementation.ownerInventory.businessCapabilities.length, 253);
  const admitted = implementation.ownerInventory.businessCapabilities;
  assert.deepEqual(
    admitted.filter((id) => task6CapabilityIds.includes(id)),
    task6CapabilityIds,
  );
  assert.equal(task6CapabilityIds.length, 40);
  for (const values of Object.values(policy.dependencies)) {
    assert.equal(values.includes('makosh-contacts-mail-sync-source-api'), false);
  }
});

test('Task 6 removes obsolete Cargo packages and keeps exact package counts', async () => {
  const workspace = await read('backend/Cargo.toml');
  const members = [...workspace.matchAll(/^\s*"([^"]+)",?$/gm)].map((match) => match[1]);
  const manifests = await import('node:fs').then(({ globSync }) =>
    globSync('**/Cargo.toml', { cwd: absolute('backend'), exclude: ['target/**'] }),
  );

  assert.equal(members.length, 420);
  assert.equal(manifests.length, 421);
  assert.doesNotMatch(workspace, /src\/(?:contacts|mail-contacts-sync)/);
  assert.doesNotMatch(workspace, /tests\/support\/(?:contacts|mail-contacts-sync)/);
  for (const path of [
    'backend/src/contacts-runtime',
    'backend/src/mail-contacts-sync-runtime',
    'backend/tests/support/contacts',
    'backend/tests/support/mail-contacts-sync',
  ]) assert.equal(existsSync(absolute(path)), false, path);
});

test('Task 6 exposes exact Persons and Review Connect routes', async () => {
  const [personsProto, reviewProto, personsAdmission, reviewAdmission] = await Promise.all([
    read('backend/src/persons-api/proto/makosh/persons/v1/persons.proto'),
    read('backend/src/review-person-match-candidate-api/proto/makosh/review/person_match_candidate/v1/person_match_candidate.proto'),
    read('backend/src/persons-runtime/src/admission.rs'),
    read('backend/src/review-person-match-candidate-runtime/src/admission.rs'),
  ]);

  assert.match(personsProto, /service PersonsCommandService/);
  assert.match(personsProto, /rpc Create\(/);
  assert.match(personsProto, /rpc UpdateOwnerProfile\(/);
  assert.match(personsProto, /service PersonsQueryService/);
  for (const rpc of ['ListDirectory', 'GetProfile', 'ListSourceLinks']) {
    assert.match(personsProto, new RegExp(`rpc ${rpc}\\(`));
  }
  assert.match(reviewProto, /service ReviewPersonMatchCandidateCommandService/);
  assert.match(reviewProto, /service ReviewPersonMatchCandidateQueryService/);
  assert.match(reviewProto, /rpc List\(/);
  assert.match(personsAdmission, /PERSONS_CLIENT_CAPABILITY_ID_V1/);
  assert.match(reviewAdmission, /REVIEW_PERSON_MATCH_CANDIDATE_CLIENT_CAPABILITY_ID_V1/);
  assert.match(personsAdmission + reviewAdmission, /ProvidedSurfaceKindV1::ClientRpc/);
});

test('Task 6 wires sanitized Mail account binding and append-only storage', async () => {
  const [proto, mailAdmission, mailStorage, workflowAdmission, workflowSchema] = await Promise.all([
    read('backend/src/mail-address-book-contract/proto/makosh/mail/address_book/person_source/v1/person_source.proto'),
    read('backend/src/mail-runtime/src/admission.rs'),
    read('backend/src/mail-runtime/src/storage_bundle.rs'),
    read('backend/src/mail-persons-sync-runtime/src/admission.rs'),
    read('backend/src/mail-persons-sync-persistence/src/schema.rs'),
  ]);

  assert.match(proto, /message MailPersonSourceAccountReadyV1/);
  assert.match(proto, /message MailPersonSourceAccountRetiredV1/);
  const protoDeclarations = proto.replaceAll(/\/\/.*$/gm, '');
  assert.doesNotMatch(protoDeclarations, /private_account|provider_account|credential|cursor|locator|raw_payload/i);
  assert.match(mailAdmission, /MAIL_PERSON_SOURCE_CAPABILITY_ID_V1/);
  assert.doesNotMatch(mailAdmission, /MAIL_ADDRESS_BOOK_CAPABILITY_ID_V1/);
  assert.match(mailStorage, /revision, 32|revision\s*==\s*32|assert_eq!\(bundle\.revision, 32\)/);
  assert.match(workflowAdmission, /mail_persons_sync\.mail\.account-ready\.v1/);
  assert.match(workflowAdmission, /mail_persons_sync\.mail\.account-retired\.v1/);
  assert.match(workflowAdmission, /mail_persons_sync\.scheduler_schedule_command\.v1/);
  assert.match(workflowAdmission, /mail_persons_sync\.scheduler_schedule_result\.v1/);
  assert.match(workflowSchema, /MAIL_PERSONS_SYNC_STORAGE_BUNDLE_REVISION_V1:\s*u32\s*=\s*2/);
});

test('Task 6 release and Personas surface contain only compiled successor paths', async () => {
  const [materializer, development, surfaces, personasApi, personasBoundary] = await Promise.all([
    read('backend/scripts/materialize-dev-release.sh'),
    read('backend/development/assembly/src/main.rs'),
    read('frontend/src/platform/client-runtime/clientSurfaces.ts'),
    read('frontend/src/domains/personas/api/personas.ts'),
    read('frontend/src/domains/personas/views/PersonasPage.boundary.test.ts'),
  ]);
  const release = `${materializer}\n${development}`;

  for (const owner of [
    'persons',
    'mail_persons_sync',
    'review_person_match_candidate',
    'reviewed_person_match_candidate_promotion',
  ]) assert.match(release, new RegExp(owner));
  const activeModulePlan = development.slice(
    development.indexOf('const MODULE_PLAN:'),
    development.indexOf('const LEGACY_HERMES_PRE_PERSONS_MODULE_PLAN_V3:'),
  );
  assert.doesNotMatch(materializer, /mail_contacts_sync|contacts\.runtime|contacts\.storage/);
  assert.doesNotMatch(activeModulePlan, /mail_contacts_sync|contacts\.runtime|contacts\.storage/);
  assert.match(development, /fn legacy_hermes_pre_persons_successor/);
  assert.match(surfaces, /'persons-owner'/);
  assert.match(surfaces, /routeId: 'personas'[\s\S]*adapterId: 'persons-owner'/);
  assert.doesNotMatch(personasApi, /ApiClient|\/api\/v1\/(?:personas|identity-candidates|identity-traces|relationships)/);
  assert.match(personasApi, /PersonsQueryService|personsQueryClient/);
  assert.match(personasApi, /ReviewPersonMatchCandidate/);
  assert.match(personasBoundary, /relationships.*unavailable|unavailable.*relationships/i);
  assert.doesNotMatch(personasBoundary, /Add to contacts|address-book|identity-traces|\/api\/v1\/personas/);
  assert.equal(existsSync(absolute('frontend/src/gen/makosh/mail_contacts_sync/v1/sync_pb.ts')), false);
  assert.equal(existsSync(absolute('frontend/src/workflows/mail-contacts-sync')), false);
});

test('Task 6 has an empty-start provider resync proof with no Contacts import', () => {
  const harnessPath = absolute(
    'backend/tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/persons_admission_cutover_flow.rs',
  );
  assert.equal(existsSync(harnessPath), true);
  const harness = readFileSync(harnessPath, 'utf8');
  assert.match(harness, /empty_start_provider_resync_without_contacts_import/);
  assert.match(harness, /MailPersonSourceAccountReadyV1/);
  assert.match(harness, /scheduler_launch::start_from_reservation/);
  assert.match(harness, /restart|successor/i);
  assert.match(harness, /source.*remov|remov.*source/i);
  assert.match(harness, /retains|retained/i);
  assert.doesNotMatch(harness, /makosh_contacts|mail_contacts_sync|legacy.*(?:read|import)|dual_(?:read|write)/i);
});
