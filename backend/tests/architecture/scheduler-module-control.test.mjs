import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

test('module-originated Scheduler control opens only with exact protocol and managed evidence', async () => {
  const [adr, inventorySource, proto, schedulerProtocol, validation, runtimeContract, admission, mapping, resultEnvelope, migration, authorityMigration, persistence, schedulerConnection, jetstream, runtimeWorker, schedulerRuntimeControl, delayedDeliveryRuntime, delayedDeliverySchedulerResults, delayedDeliveryDueExecution, kernelTopology, schedulerLaunch, schedulerLifecycle, eventCatalog, development, manifest] = await Promise.all([
    readFile(
      new URL(
        'docs/adr/ADR-0342-module-originated-scheduler-control-events.md',
        PROJECT_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'architecture/communications-settings-reconstruction.json',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/platform/scheduler/protocol/proto/makosh/scheduler/v1/job_command.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/platform/scheduler/protocol/src/lib.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/platform/scheduler/protocol/src/validation/schedule_control.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/platform/runtime_protocol/proto/makosh/runtime/v1/scheduler_runtime.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/platform/scheduler/implementation/src/control/admission.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/platform/scheduler/implementation/src/control/one_shot.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/platform/scheduler/implementation/src/control/result.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/platform/scheduler/persistence/migrations/0008_scheduler_schedule_control.sql',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/platform/scheduler/persistence/migrations/0009_scheduler_schedule_control_authority.sql',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/platform/scheduler/persistence/src/store/control/apply.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/platform/scheduler/persistence/src/store/connection.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/platform/scheduler/jetstream/src/transport/schedule_control.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/platform/scheduler/runtime/src/control/schedule_control.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/platform/scheduler/runtime/src/control/runtime.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delayed-delivery-runtime/src/managed_runtime.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delayed-delivery-runtime/src/scheduler_results.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delayed-delivery-runtime/src/due_execution.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/kernel/src/platform/scheduler/schedule_control.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/kernel/src/platform/scheduler/launch.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/kernel/src/platform/scheduler/lifecycle.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/kernel/src/platform/events/catalog/entries.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/kernel/src/platform/development.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/platform/scheduler/protocol/Cargo.toml',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
  ]);
  const inventory = JSON.parse(inventorySource);
  const gate = inventory.slices.find(
    (slice) => slice.gate === 'scheduler_module_schedule_control_v1',
  );

  assert.deepEqual(gate, {
    gate: 'scheduler_module_schedule_control_v1',
    role: 'platform',
    owner: 'scheduler',
    state: 'implemented',
    dependsOn: ['scheduler_v1', 'nats_data_plane_v1'],
  });
  assert.match(adr, /Состояние реализации: реализовано/);
  assert.match(adr, /scheduler_module_schedule_control_v1` открыт/);
  assert.match(adr, /DurableEnvelopeV1/);
  assert.match(proto, /message SchedulerScheduleControlCommandV1/);
  assert.match(
    schedulerProtocol,
    /SCHEDULER_RUNTIME_MODULE_ID_V1: &str = "makosh-scheduler-runtime"/,
  );
  assert.match(proto, /message SchedulerScheduleControlResultV1/);
  assert.match(proto, /EnsureOneShotScheduleV1 ensure_one_shot/);
  assert.match(proto, /CancelOneShotScheduleV1 cancel_one_shot/);
  assert.match(proto, /JobKindV1 job_kind = 3/);
  assert.match(validation, /MAX_ATTEMPTS: u32 = 32/);
  assert.match(runtimeContract, /SchedulerRuntimeScheduleControlBindingV1/);
  assert.match(runtimeContract, /SchedulerRuntimeScheduleControlGrantV1/);
  assert.match(admission, /source_runtime_generation/);
  assert.match(admission, /source_grant_epoch/);
  assert.match(resultEnvelope, /ResultMetadataV1/);
  assert.match(resultEnvelope, /command_message_id: command\.message_id\(\)/);
  assert.match(mapping, /ScheduleTriggerV1::At/);
  assert.match(mapping, /OverlapPolicyV1::Forbid/);
  assert.match(mapping, /MisfirePolicyV1::FireOnce/);
  assert.match(migration, /scheduler_schedule_control_inbox/);
  assert.match(migration, /scheduler_schedule_control_results/);
  assert.match(authorityMigration, /scheduler_schedule_control_authorities/);
  assert.match(persistence, /command_envelope_sha256/);
  assert.match(persistence, /exact_envelope_bytes/);
  assert.match(persistence, /SchedulerScheduleControlDecisionV1::TooLate/);
  assert.match(persistence, /ForeignAuthority/);
  assert.doesNotMatch(schedulerConnection, /tokio::time::sleep/);
  assert.match(jetstream, /SchedulerJetStreamScheduleControlPortV1/);
  assert.match(runtimeWorker, /\.apply_schedule_control\(&request/);
  assert.match(runtimeWorker, /relay_results\(port, store\)\.await\?/);
  assert.match(runtimeWorker, /admit_or_discard/);
  assert.match(runtimeWorker, /delivery\s*\.acknowledge\(\)\s*\.await/);
  assert.doesNotMatch(runtimeWorker, /schedule_control_admission/);
  assert.match(
    schedulerRuntimeControl,
    /SchedulerScheduleControlWorkerConfigV1::from_runtime\(\s*SCHEDULER_RUNTIME_MODULE_ID_V1,/,
  );
  assert.match(
    schedulerRuntimeControl,
    /SchedulerMaterializationSourceV1::new\(\s*SCHEDULER_RUNTIME_MODULE_ID_V1\.to_owned\(\),/,
  );
  assert.match(schedulerRuntimeControl, /STORAGE_CONNECT_ATTEMPTS: u8 = 120/);
  assert.match(
    schedulerRuntimeControl,
    /STORAGE_CONNECT_RETRY_DELAY: Duration = Duration::from_millis\(250\)/,
  );
  assert.match(resultEnvelope, /module_id: source\.runtime_id\(\)\.to_owned\(\)/);
  assert.match(
    delayedDeliveryRuntime,
    /u8::from_str_radix\(&runtime_instance_id\[index \* 2\.\.index \* 2 \+ 2\], 16\)/,
  );
  assert.doesNotMatch(
    delayedDeliveryRuntime,
    /Sha256::digest\(runtime_instance_id\.as_bytes\(\)\)/,
  );
  assert.match(
    delayedDeliverySchedulerResults,
    /discard_invalid_scheduler_result/,
  );
  assert.match(
    delayedDeliverySchedulerResults,
    /delivery\s*\.acknowledge\(\)\s*\.await/,
  );
  assert.match(delayedDeliveryDueExecution, /discard_invalid_due_command/);
  assert.match(
    delayedDeliveryDueExecution,
    /delivery\s*\.acknowledge\(\)\s*\.await/,
  );
  assert.match(kernelTopology, /current_managed_runtime_matches/);
  assert.match(kernelTopology, /command_publishers\.contains/);
  assert.match(kernelTopology, /result_consumers\.contains/);
  assert.match(kernelTopology, /scheduler_catalog::resolve/);
  assert.match(schedulerLaunch, /topology_fingerprint/);
  assert.match(schedulerLaunch, /schedule_control\.grants/);
  assert.match(schedulerLaunch, /dispatch_publishers/);
  assert.match(schedulerLaunch, /receipt_consumers/);
  assert.match(schedulerLifecycle, /expected_topology_fingerprint/);
  assert.match(schedulerLifecycle, /ReconcileOutcome::Refreshed/);
  assert.match(schedulerLifecycle, /capture_active_topology_fingerprint/);
  assert.match(schedulerLifecycle, /TOPOLOGY_STABLE_OBSERVATIONS/);
  assert.match(schedulerLifecycle, /observe_stable_topology/);
  assert.match(schedulerLifecycle, /successor::reserve/);
  assert.match(eventCatalog, /scheduler_dispatch_entries/);
  assert.match(eventCatalog, /module_scheduler_job_requests/);
  assert.match(eventCatalog, /SCHEDULER_DISPATCH_CAPABILITY_ID_V1/);
  assert.doesNotMatch(
    eventCatalog.split('#[cfg(test)]')[0],
    /communication_delayed_delivery|mail|telegram|whatsapp|zulip/,
  );
  assert.match(development, /events\.scheduler\.schedule_control\.command/);
  assert.match(development, /events\.scheduler\.schedule_control\.result/);
  assert.match(development, /StorageBundleV1::decode/);
  assert.match(development, /issue_after_with_bundle/);
  assert.doesNotMatch(
    development,
    /StorageBindingIssueV1::new\(1,\s*1,\s*7,/,
  );
  assert.match(manifest, /role = "platform"/);
  assert.match(manifest, /owner = "scheduler"/);
  assert.match(manifest, /surface = "contract"/);
  const scheduleControlProto = proto.slice(
    proto.indexOf('message EnsureOneShotScheduleV1'),
  );
  assert.doesNotMatch(
    `${scheduleControlProto}\n${validation}`,
    /mail|telegram|whatsapp|zulip|conversation|provider/i,
  );
});
