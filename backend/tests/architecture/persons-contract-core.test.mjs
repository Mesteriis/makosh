import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import test from 'node:test';

const PROJECT_ROOT = resolve(import.meta.dirname, '../../..');
const read = (path) => readFile(resolve(PROJECT_ROOT, path), 'utf8');

test('Persons contract core persistence runtime and assembly are exact admitted production units', async () => {
  const [policySource, apiManifest, coreManifest] = await Promise.all([
    read('backend/architecture/policy.json'),
    read('backend/src/persons-api/Cargo.toml'),
    read('backend/src/persons-core/Cargo.toml'),
  ]);
  const policy = JSON.parse(policySource);
  const personsPackages = policy.implementation.productionPackages.filter(
    ({ name }) => name.startsWith('makosh-persons-'),
  );

  assert.equal(policy.implementation.currentSlice, 'speech_to_text_whisper_admission_v1');
  assert.deepEqual(personsPackages, [
    { name: 'makosh-persons-api', role: 'domain', owner: 'persons', surface: 'contract' },
    { name: 'makosh-persons-core', role: 'domain', owner: 'persons', surface: 'implementation' },
    { name: 'makosh-persons-persistence', role: 'domain', owner: 'persons', surface: 'persistence' },
    { name: 'makosh-persons-runtime', role: 'domain', owner: 'persons', surface: 'runtime' },
    { name: 'makosh-persons-assembly', role: 'domain', owner: 'persons', surface: 'assembly' },
  ]);
  assert.deepEqual(policy.implementation.workspaceDependencyAllowlist['makosh-persons-api'], [
    { name: 'makosh-runtime-protocol', kind: 'normal' },
  ]);
  assert.deepEqual(policy.implementation.workspaceDependencyAllowlist['makosh-persons-core'], [
    { name: 'makosh-persons-api', kind: 'normal' },
  ]);
  assert.deepEqual(policy.implementation.targetPolicy['makosh-persons-api'], {
    primaryKind: 'lib', customBuildAllowed: true,
  });
  assert.deepEqual(policy.implementation.targetPolicy['makosh-persons-core'], {
    primaryKind: 'lib', customBuildAllowed: false,
  });
  assert.equal(policy.implementation.ownerInventory.domains.includes('persons'), true);
  assert.ok(
    policy.implementation.ownerInventory.businessCapabilities.includes('persons.client.v1'),
  );
  assert.match(apiManifest, /role = "domain"[\s\S]*owner = "persons"[\s\S]*surface = "contract"/);
  assert.match(coreManifest, /role = "domain"[\s\S]*owner = "persons"[\s\S]*surface = "implementation"/);
  assert.doesNotMatch(`${apiManifest}\n${coreManifest}`, /makosh-(?:contacts|mail|telegram|whatsapp|zulip|storage|events)/);
});

test('Persons wire contract exposes typed bounded public identities without private provider state', async () => {
  const [proto, apiSource, buildSource] = await Promise.all([
    read('backend/src/persons-api/proto/makosh/persons/v1/persons.proto'),
    read('backend/src/persons-api/src/lib.rs'),
    read('backend/src/persons-api/build.rs'),
  ]);
  const schema = proto.replaceAll(/\/\/.*$/gm, '');

  assert.match(schema, /package makosh\.persons\.v1;/);
  for (const message of [
    'ManualCreatePersonCommandV1',
    'UpdatePersonOwnerProfileCommandV1',
    'ObserveProviderSourceContactCommandV1',
    'UpdateProviderSourceContactCommandV1',
    'RemoveProviderSourceContactCommandV1',
    'ConfirmAttachPersonSourceCommandV1',
    'ConfirmDetachPersonSourceCommandV1',
    'ConfirmMergePersonsCommandV1',
    'ConfirmSplitPersonCommandV1',
    'ReadPersonDirectoryRequestV1',
    'ReadPersonProfileRequestV1',
    'ReadPersonSourceLinksRequestV1',
    'PersonCommandSucceededV1',
    'PersonCommandRejectedV1',
    'PersonChangedEventV1',
    'PersonProfileChangedEventV1',
    'PersonSourceLinkChangedEventV1',
    'PersonLineageChangedEventV1',
    'PersonReviewCandidateRaisedEventV1',
  ]) {
    assert.match(schema, new RegExp(`message ${message} \\{`));
  }
  for (const publicSourceId of [
    'integration_public_id',
    'account_public_id',
    'provider_source_contact_public_id',
  ]) {
    assert.match(schema, new RegExp(`bytes ${publicSourceId} =`));
  }
  for (const snapshotField of [
    'expected_from_person_revision',
    'expected_to_person_revision',
    'expected_person_revision',
    'expected_detached_person_revision',
    'expected_source_person_revision',
    'expected_target_person_revision',
    'expected_merged_person_revision',
    'expected_source_revision',
    'approved_action_digest',
  ]) {
    assert.match(schema, new RegExp(`\\b${snapshotField}\\b`));
  }
  assert.match(schema, /repeated PersonRevisionV1 resulting_person_revisions/);
  assert.doesNotMatch(schema, /uint64 resulting_person_revision\b/);
  assert.match(schema, /repeated SplitPersonSourceSelectionV1 source_selection/);
  assert.match(schema, /repeated SplitProfileFactKindV1 profile_fact_selection/);
  for (const forbidden of [
    'credential', 'session', 'token', 'raw_payload', 'private_locator',
    'provider_locator', 'error_detail', 'map<', 'google_resource_name',
  ]) {
    assert.ok(!schema.toLowerCase().includes(forbidden), `forbidden schema token ${forbidden}`);
  }
  assert.doesNotMatch(schema, /PersonReviewCandidateRaisedEventV1[\s\S]*?(?:normalized_emails|normalized_phones|DecisionProvenanceV1)/);
  assert.match(apiSource, /STABLE_ID_BYTES_V1: usize = 16/);
  assert.match(apiSource, /DIGEST_BYTES_V1: usize = 32/);
  assert.match(buildSource, /file_descriptor_set_path/);
  assert.match(buildSource, /Sha256/);
});

test('Persons core remains pure after runtime and assembly are introduced', async () => {
  const coreSources = await Promise.all([
    read('backend/src/persons-core/src/lib.rs'),
    read('backend/src/persons-core/src/model.rs'),
    read('backend/src/persons-core/src/normalization.rs'),
    read('backend/src/persons-core/src/state.rs'),
    read('backend/src/persons-core/src/transitions.rs'),
  ]);

  assert.doesNotMatch(coreSources.join('\n'), /makosh_contacts|reqwest|sqlx|credential|raw_payload|provider_availability/i);
  assert.match(coreSources.join('\n'), /attach_source_action_digest_v1/);
  assert.match(coreSources.join('\n'), /DecisionReuseConflict/);
  assert.match(coreSources.join('\n'), /EmptySplitSelection/);
});
