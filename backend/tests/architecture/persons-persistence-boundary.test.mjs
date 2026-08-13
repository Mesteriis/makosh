import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import test from 'node:test';

const PROJECT_ROOT = resolve(import.meta.dirname, '../../..');
const read = (path) => readFile(resolve(PROJECT_ROOT, path), 'utf8');

test('Persons persistence is one exact admitted production package', async () => {
  const [policySource, manifest] = await Promise.all([
    read('backend/architecture/policy.json'),
    read('backend/src/persons-persistence/Cargo.toml'),
  ]);
  const policy = JSON.parse(policySource);
  assert.deepEqual(
    policy.implementation.productionPackages.filter(({ name }) => name === 'makosh-persons-persistence'),
    [{
      name: 'makosh-persons-persistence', role: 'domain', owner: 'persons', surface: 'persistence',
    }],
  );
  assert.equal(policy.implementation.currentSlice, 'speech_to_text_whisper_admission_v1');
  assert.equal(policy.implementation.ownerInventory.domains.includes('persons'), true);
  assert.match(manifest, /makosh-persons-core/);
  assert.match(manifest, /makosh-storage-protocol/);
  assert.match(manifest, /sqlx/);
  assert.doesNotMatch(manifest, /makosh-(?:contacts|mail|events|runtime)-/);
});

test('Persons forward schema is typed private and forces owner RLS on every table', async () => {
  const [initial, upgrade] = await Promise.all([
    read('backend/src/persons-persistence/migrations/0001_persons.sql'),
    read('backend/src/persons-persistence/migrations/0002_persons_durable.sql'),
  ]);
  const sql = `${initial}\n${upgrade}`;
  for (const table of [
    'persons_owner_aggregates', 'persons_current', 'persons_profiles', 'persons_sources',
    'persons_lineage', 'persons_lineage_sources', 'persons_decision_receipts',
    'persons_decision_outcomes', 'persons_command_inbox', 'persons_outbox',
  ]) {
    assert.match(sql, new RegExp(`CREATE TABLE makosh_data\\.${table}`));
    assert.match(sql, new RegExp(`ALTER TABLE makosh_data\\.${table} ENABLE ROW LEVEL SECURITY`));
    assert.match(sql, new RegExp(`ALTER TABLE makosh_data\\.${table} FORCE ROW LEVEL SECURITY`));
  }
  assert.match(sql, /current_setting\('makosh\.logical_owner_id', true\)/);
  assert.match(sql, /USING \(logical_owner_id =/);
  assert.match(sql, /WITH CHECK \(logical_owner_id =/);
  assert.match(sql, /octet_length\([^)]*\) = 16/);
  assert.match(sql, /octet_length\([^)]*\) = 32/);
  assert.match(sql, /UNIQUE \(integration_public_id, account_public_id, provider_source_contact_public_id\)/);
  assert.match(sql, /FOREIGN KEY \(logical_owner_id, merged_into_person_id\)[\s\S]*DEFERRABLE INITIALLY DEFERRED/);
  assert.match(sql, /CREATE FUNCTION makosh_data\.persons_reject_profile_history_mutation/);
  assert.match(sql, /BEFORE UPDATE OR DELETE ON makosh_data\.persons_profiles/);
  assert.doesNotMatch(sql, /jsonb?|makosh_contacts|contacts_legacy|contacts_schema|credential|session|token|raw_payload|private_locator/i);
  assert.doesNotMatch(sql, /CREATE ROLE|ALTER ROLE|SUPERUSER|BYPASSRLS|GRANT ALL/i);
});

test('Persons repository sets transaction owner context and testkit proves non-bypass RLS', async () => {
  const [repository, live, conformance, makefile, managedRunner] = await Promise.all([
    read('backend/src/persons-persistence/src/repository.rs'),
    read('backend/tests/support/persons-persistence/tests/postgres_live.rs'),
    read('backend/src/persons-persistence/src/conformance.rs'),
    read('backend/Makefile'),
    read('backend/scripts/test-authenticated-storage.mjs'),
  ]);
  assert.match(repository, /set_config\('makosh\.logical_owner_id'/);
  assert.match(repository, /FOR UPDATE/);
  assert.match(repository, /mark_outbox_published[\s\S]*envelope_sha256[\s\S]*FOR UPDATE/);
  assert.match(repository, /StorageBindingV1/);
  assert.doesNotMatch(repository, /makosh_contacts|contacts_|serde_json|SELECT \* FROM/i);
  const rlsEvidence = `${live}\n${conformance}`;
  assert.match(rlsEvidence, /NOSUPERUSER/);
  assert.match(rlsEvidence, /NOBYPASSRLS/);
  assert.match(rlsEvidence, /SET LOCAL ROLE/);
  assert.match(rlsEvidence, /WHERE logical_owner_id/);
  assert.match(rlsEvidence, /omits an owner predicate|without owner predicate/);
  assert.match(makefile, /MAKOSH_PERSONS_POSTGRES_TEST_FILTER/);
  assert.doesNotMatch(makefile, /persons-postgres-conformance:[\s\S]*MAKOSH_PERSONS_POSTGRES_URL/);
  assert.match(makefile, /persons-postgres-conformance:[\s\S]*test-authenticated-storage\.mjs/);
  assert.match(managedRunner, /MAKOSH_PERSONS_POSTGRES_TEST_FILTER/);
  assert.match(managedRunner, /makosh_persons_conformance_/);
  assert.match(managedRunner, /makosh_persons_disposable_sentinel/);
  assert.match(managedRunner, /--test-threads=1/);
  assert.match(managedRunner, /DROP DATABASE/);
  assert.match(conformance, /makosh_persons_disposable_sentinel/);
  assert.match(conformance, /current_database/);
  assert.match(rlsEvidence, /own_profile_update_blocked/);
  assert.match(rlsEvidence, /own_profile_delete_blocked/);
});
