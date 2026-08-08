import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

test('Scheduler gate is backed by hot reconciliation and managed successor recovery evidence', async () => {
  const [adr, policySource, conformance, runner] = await Promise.all([
    readFile(
      new URL(
        'docs/adr/ADR-0214-durable-job-platform-scheduler-and-runtime-reconfiguration.md',
        PROJECT_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('architecture/policy.json', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('scripts/test-authenticated-storage.mjs', BACKEND_ROOT),
      'utf8',
    ),
  ]);
  const policy = JSON.parse(policySource);

  assert.deepEqual(policy.phaseGates.requiredDecisionFields.scheduler_v1, [
    'exact_package_inventory',
    'jobspec_jobkind_and_contract_binding',
    'schedule_run_lease_and_fencing_storage',
    'nats_acceptance_result_and_ack',
    'hot_schedule_reconciliation',
    'retry_idempotency_and_recovery',
    'deterministic_clock_conformance',
    'managed_successor_restart_recovery',
    'revoked_binding_no_resurrection',
    'authenticated_storage_nats_live_conformance',
  ]);
  assert.equal(policy.phaseGates.notAuthorized.includes('scheduler_v1'), false);
  assert.match(adr, /gate реализован/);
  assert.match(
    conformance,
    /managed_scheduler_crash_uses_storage_control_successor_provisioning/,
  );
  assert.match(conformance, /SchedulerScheduleUpsertOutcomeV1::Updated/);
  assert.match(conformance, /assert_recovered_scheduler_delivery/);
  assert.match(conformance, /assert_revoked_binding_does_not_restart/);
  assert.match(
    runner,
    /MAKOSH_STORAGE_MANAGED_TEST_FILTER/,
  );
});
