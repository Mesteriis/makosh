import assert from 'node:assert/strict';
import { globSync } from 'node:fs';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import test from 'node:test';

const PROJECT_ROOT = resolve(import.meta.dirname, '../../..');
const absolute = (path) => resolve(PROJECT_ROOT, path);
const read = async (path) => readFile(absolute(path), 'utf8');

const packages = [
  'makosh-calendar-api',
  'makosh-calendar-assembly',
  'makosh-calendar-core',
  'makosh-calendar-persistence',
  'makosh-calendar-runtime',
];
const capabilities = [
  'calendar.client.v1',
  'calendar.lifecycle.event.v1',
  'calendar.scheduler.due.v1',
  'calendar.scheduler.receipt.v1',
  'calendar.scheduler.schedule-command.v1',
  'calendar.scheduler.schedule-result.v1',
  'calendar.storage.v1',
];

test('Task 16 records Calendar as implemented but not production-admitted', async () => {
  const policy = JSON.parse(await read('backend/architecture/policy.json'));
  assert.equal(policy.implementation.currentSlice, 'speech_to_text_whisper_admission_v1');
  assert.equal(policy.implementation.productionPackages.length, 283);
  assert.equal(policy.implementation.ownerInventory.businessCapabilities.length, 253);
  assert.deepEqual(
    policy.implementation.productionPackages
      .filter(({ owner }) => owner === 'calendar')
      .map(({ name }) => name)
      .sort(),
    [],
  );
  assert.deepEqual(
    policy.implementation.ownerInventory.businessCapabilities.filter((id) => id.startsWith('calendar.')),
    [],
  );
  assert.equal(policy.implementation.ownerInventory.domains.includes('calendar'), false);
});

test('Task 16 creates exactly five Calendar packages without direct providers', async () => {
  const manifests = globSync('**/Cargo.toml', { cwd: absolute('backend'), exclude: ['target/**'] });
  assert.equal(manifests.length, 421);
  for (const name of packages) {
    const manifest = await read(`backend/src/${name.replace('makosh-', '')}/Cargo.toml`);
    assert.match(manifest, new RegExp(`name = "${name}"`));
    assert.doesNotMatch(manifest, /google|apple|caldav|provider/i);
  }
  const workspace = await read('backend/Cargo.toml');
  assert.equal(packages.every((name) => workspace.includes(`src/${name.replace('makosh-', '')}`)), true);
});

test('Task 16 owns typed lifecycle client and Scheduler boundaries', async () => {
  const [proto, admission] = await Promise.all([
    read('backend/src/calendar-api/proto/makosh/calendar/client/v1/calendar.proto'),
    read('backend/src/calendar-runtime/src/admission.rs'),
  ]);
  for (const rpc of [
    'Create', 'Update', 'SetState', 'AddParticipant', 'UpdateParticipant',
    'RemoveParticipant', 'SetConstraints', 'AddReminder', 'RemoveReminder',
    'RecordOutcome', 'Get', 'List', 'Search', 'ListParticipants',
    'ListReminders', 'ListOutcomes',
  ]) assert.match(proto, new RegExp(`rpc ${rpc}\\(`));
  for (const capability of capabilities) assert.equal(admission.includes(capability), true);
  assert.doesNotMatch(proto, /provider|credential|private_locator|google|apple|caldav/i);
});

test('Task 16 storage is owner-local FORCE RLS and relay-safe', async () => {
  const [migration, repository] = await Promise.all([
    read('backend/src/calendar-persistence/migrations/0001_calendar_owner.sql'),
    read('backend/src/calendar-persistence/src/repository.rs'),
  ]);
  for (const table of [
    'calendar_events', 'calendar_participants', 'calendar_constraints',
    'calendar_reminders', 'calendar_outcomes', 'calendar_client_operations',
    'calendar_scheduler_inbox', 'calendar_outbox',
  ]) {
    assert.match(migration, new RegExp(`CREATE TABLE makosh_data\\.${table}`));
    assert.match(migration, new RegExp(`ALTER TABLE makosh_data\\.${table} FORCE ROW LEVEL SECURITY`));
  }
  assert.match(repository, /set_config\('makosh\.logical_owner_id'/);
  assert.match(repository, /FOR UPDATE SKIP LOCKED/);
});

test('Task 16 compiles frontend and actual managed/release evidence', async () => {
  const [generator, surfaces, adapters, gateway, runner, flow, materializer] = await Promise.all([
    read('frontend/scripts/generate-proto.mjs'),
    read('frontend/src/platform/client-runtime/clientSurfaces.ts'),
    read('frontend/src/app/client-surfaces/compiledClientSurfaceAdapters.ts'),
    read('backend/src/api/gateway/session_contract/src/browser/client_bootstrap.rs'),
    read('backend/scripts/test-authenticated-storage.mjs'),
    read('backend/tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/calendar_managed_flow.rs'),
    read('backend/scripts/materialize-dev-release.sh'),
  ]);
  assert.equal(generator.includes('calendar-api'), true);
  assert.match(surfaces, /routeId: 'calendar'[\s\S]*adapterId: 'calendar-owner'/);
  assert.match(adapters, /'calendar-owner'/);
  assert.match(gateway, /Self::Calendar => Some\("calendar\.client\.v1"\)/);
  assert.equal(runner.includes('managed_calendar_lifecycle_reminder_replays_and_restarts_with_owner_rls'), true);
  assert.match(flow, /NOBYPASSRLS/);
  assert.match(flow, /Scheduler|scheduler/);
  assert.match(materializer, /makosh-calendar-runtime/);
  assert.deepEqual(globSync('src/domains/calendar/api/**/*', { cwd: absolute('frontend') }), []);
});
