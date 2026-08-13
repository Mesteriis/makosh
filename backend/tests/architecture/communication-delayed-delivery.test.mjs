import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

test('delayed delivery admits exact due commands and durable Blob cleanup as an implemented workflow', async () => {
  const [
    adr,
    storeAdapterAdr,
    inventorySource,
    schedulerAdr,
    apiManifest,
    apiSource,
    apiProto,
    coreManifest,
    coreSource,
    persistenceManifest,
    persistenceSource,
    persistenceConformance,
    persistenceOperations,
    persistenceExecution,
    persistenceRelay,
    persistenceStatus,
    persistenceRealtime,
    persistenceCleanup,
    persistenceMigration,
    schedulerReceiptMigration,
    clientRealtimeMigration,
    bodyCleanupMigration,
    ownerRlsMigration,
    executionManifest,
    executionSource,
    executionPorts,
    executionWorker,
    executionCleanup,
    eventAdaptersManifest,
    eventAdaptersSource,
    dueEventAdapterSource,
    runtimeAdaptersManifest,
    runtimeAdaptersSource,
    storeAdaptersManifest,
    storeAdaptersSource,
    runtimeManifest,
    runtimeAdmission,
    runtimeClientPort,
    runtimeClientRealtime,
    runtimeSchedulerOutbox,
    runtimeSchedulerResults,
    runtimeDueExecution,
    runtimeBodyCleanup,
    managedRuntime,
    runtimeMain,
    assemblyManifest,
    assemblySource,
    assemblyMain,
    developmentReleaseScript,
    authenticatedStorageRunner,
    managedSetup,
    managedFlow,
    conformanceManifest,
    conformanceTest,
    methodRoutingAdr,
  ] = await Promise.all([
    readFile(
      new URL(
        'docs/adr/ADR-0341-scheduled-communication-delivery-workflow.md',
        PROJECT_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'docs/adr/ADR-0344-delayed-delivery-execution-store-adapter.md',
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
        'docs/adr/ADR-0214-durable-job-platform-scheduler-and-runtime-reconfiguration.md',
        PROJECT_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('src/communication-delayed-delivery-api/Cargo.toml', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/communication-delayed-delivery-api/src/lib.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delayed-delivery-api/proto/makosh/communication_delayed_delivery/v1/delivery.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('src/communication-delayed-delivery-core/Cargo.toml', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/communication-delayed-delivery-core/src/lib.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delayed-delivery-persistence/Cargo.toml',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delayed-delivery-persistence/src/lib.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delayed-delivery-persistence/src/conformance.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delayed-delivery-persistence/src/operations.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delayed-delivery-persistence/src/execution.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delayed-delivery-persistence/src/relay.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delayed-delivery-persistence/src/status.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delayed-delivery-persistence/src/realtime.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delayed-delivery-persistence/src/cleanup.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delayed-delivery-persistence/migrations/0001_delayed_delivery_state.sql',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delayed-delivery-persistence/migrations/0002_scheduler_receipt_outbox.sql',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delayed-delivery-persistence/migrations/0003_client_realtime_replay.sql',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delayed-delivery-persistence/migrations/0004_body_cleanup_queue.sql',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delayed-delivery-persistence/migrations/0005_owner_rls.sql',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('src/communication-delayed-delivery-execution/Cargo.toml', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/communication-delayed-delivery-execution/src/lib.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/communication-delayed-delivery-execution/src/ports.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/communication-delayed-delivery-execution/src/worker.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/communication-delayed-delivery-execution/src/cleanup.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delayed-delivery-event-adapters/Cargo.toml',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delayed-delivery-event-adapters/src/lib.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delayed-delivery-event-adapters/src/due.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delayed-delivery-runtime-adapters/Cargo.toml',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delayed-delivery-runtime-adapters/src/lib.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delayed-delivery-store-adapters/Cargo.toml',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delayed-delivery-store-adapters/src/lib.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('src/communication-delayed-delivery-runtime/Cargo.toml', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/communication-delayed-delivery-runtime/src/admission.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/communication-delayed-delivery-runtime/src/client_port.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delayed-delivery-runtime/src/client_realtime.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delayed-delivery-runtime/src/scheduler_outbox.rs',
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
        'src/communication-delayed-delivery-runtime/src/body_cleanup.rs',
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
      new URL('src/communication-delayed-delivery-runtime/src/main.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delayed-delivery-assembly/Cargo.toml',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delayed-delivery-assembly/src/lib.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delayed-delivery-assembly/src/main.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('scripts/materialize-dev-release.sh', BACKEND_ROOT), 'utf8'),
    readFile(new URL('scripts/test-authenticated-storage.mjs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/delayed_delivery_managed_setup.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/delayed_delivery_managed_flow.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'tests/support/communication-delayed-delivery/Cargo.toml',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'tests/support/communication-delayed-delivery/tests/postgres_live.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'docs/adr/ADR-0345-method-exact-delayed-delivery-client-command-routing.md',
        PROJECT_ROOT,
      ),
      'utf8',
    ),
  ]);

  const inventory = JSON.parse(inventorySource);
  const gate = inventory.slices.find(
    (capability) => capability.gate === 'communication_delayed_delivery_v1',
  );

  assert.deepEqual(gate, {
    gate: 'communication_delayed_delivery_v1',
    role: 'workflow',
    owner: 'communication_delayed_delivery',
    state: 'implemented',
    dependsOn: [
      'communication_delivery_intent_v1',
      'scheduler_module_schedule_control_v1',
    ],
  });
  assert.equal(
    JSON.parse(await readFile(new URL('architecture/policy.json', BACKEND_ROOT), 'utf8'))
      .implementation.currentSlice,
    'speech_to_text_whisper_admission_v1',
  );
  assert.match(adr, /Состояние реализации: реализовано/);
  assert.match(
    apiManifest,
    /owner = "communication_delayed_delivery"[\s\S]*surface = "contract"/,
  );
  assert.match(
    coreManifest,
    /owner = "communication_delayed_delivery"[\s\S]*surface = "implementation"/,
  );
  assert.match(apiSource, /COMMUNICATION_DELAYED_DELIVERY_MAX_REQUEST_BYTES_V1/);
  assert.match(apiProto, /rpc Schedule/);
  assert.match(apiProto, /rpc Cancel/);
  assert.match(apiProto, /rpc GetStatus/);
  assert.match(coreSource, /MIN_DELIVERY_DELAY_MILLIS_V1: u64 = 5_000/);
  assert.match(coreSource, /MAX_DELIVERY_DELAY_MILLIS_V1/);
  assert.match(coreSource, /SchedulerCancelOutcomeV1::TooLate/);
  assert.match(
    persistenceManifest,
    /owner = "communication_delayed_delivery"[\s\S]*surface = "persistence"/,
  );
  assert.match(persistenceSource, /DelayedDeliveryBodyReceiptV1/);
  assert.match(persistenceSource, /SchedulerExecutionFenceV1/);
  assert.match(persistenceManifest, /conformance-test-support = \[\]/);
  assert.match(persistenceConformance, /DelayedDeliveryPersistenceConformanceV1/);
  assert.match(persistenceConformance, /PgPoolOptions::new\(\)/);
  assert.match(persistenceOperations, /pub async fn create_operation/);
  assert.match(persistenceOperations, /pub async fn request_cancellation/);
  assert.match(persistenceOperations, /pub async fn apply_scheduler_result/);
  assert.match(persistenceOperations, /ON CONFLICT \(logical_owner_id, message_id\)/);
  assert.match(persistenceOperations, /state_revision = state_revision \+ 1/);
  assert.match(persistenceExecution, /pub async fn claim_due_execution/);
  assert.match(persistenceExecution, /pub async fn mark_delivery_accepted/);
  assert.match(persistenceExecution, /pub async fn mark_delivery_failed/);
  assert.match(
    persistenceExecution,
    /scheduler_lease_expires_at_unix_millis > \$4/,
  );
  assert.match(schedulerReceiptMigration, /scheduler\.job_run\.acceptance\.v1/);
  assert.match(schedulerReceiptMigration, /scheduler\.job_run\.result\.v1/);
  assert.match(persistenceRelay, /pub async fn pending_scheduler_commands/);
  assert.match(persistenceRelay, /pub async fn pending_scheduler_receipts/);
  assert.match(persistenceRelay, /pub async fn mark_scheduler_message_published/);
  assert.match(persistenceRelay, /envelope_sha256 = \$3/);
  assert.match(persistenceStatus, /pub async fn status/);
  assert.match(persistenceStatus, /created_at_unix_millis/);
  assert.match(persistenceRealtime, /pub async fn client_realtime_window/);
  assert.match(persistenceRealtime, /insert_operation_transition/);
  assert.match(persistenceCleanup, /pub async fn next_body_cleanup/);
  assert.match(persistenceCleanup, /pub async fn complete_body_cleanup/);
  assert.match(persistenceCleanup, /pub async fn reschedule_body_cleanup/);
  assert.match(persistenceCleanup, /reconcile_terminal_cleanup/);
  assert.match(persistenceCleanup, /LEAST\(attempt_count \+ 1, 32\)/);
  assert.match(clientRealtimeMigration, /realtime_sequence/);
  assert.match(bodyCleanupMigration, /communication_delayed_delivery_body_cleanup/);
  assert.match(bodyCleanupMigration, /completed_at_unix_millis IS NULL/);
  for (const table of [
    'communication_delayed_delivery_operations',
    'communication_delayed_delivery_scheduler_inbox',
    'communication_delayed_delivery_outbox',
    'communication_delayed_delivery_scheduler_receipt_outbox',
    'communication_delayed_delivery_realtime',
    'communication_delayed_delivery_body_cleanup',
  ]) {
    assert.match(ownerRlsMigration, new RegExp(`ALTER TABLE makosh_data\\.${table} ENABLE ROW LEVEL SECURITY`));
    assert.match(ownerRlsMigration, new RegExp(`ALTER TABLE makosh_data\\.${table} FORCE ROW LEVEL SECURITY`));
  }
  assert.equal(ownerRlsMigration.match(/CREATE POLICY /g)?.length, 6);
  assert.doesNotMatch(bodyCleanupMigration, /body_utf8|provider_id|account_id/);
  assert.doesNotMatch(clientRealtimeMigration, /body_utf8|provider_id|account_id/);
  assert.doesNotMatch(schedulerReceiptMigration, /body_utf8|provider_id|account_id/);
  assert.match(
    persistenceMigration,
    /communication_delayed_delivery_scheduler_inbox/,
  );
  assert.match(persistenceMigration, /communication_delayed_delivery_outbox/);
  assert.doesNotMatch(persistenceMigration, /body_utf8|provider_id|account_id/);
  assert.match(
    executionManifest,
    /owner = "communication_delayed_delivery"[\s\S]*surface = "implementation"/,
  );
  assert.match(executionPorts, /pub trait BodyReadPortV1/);
  assert.match(executionPorts, /pub trait BodyCleanupPortV1/);
  assert.match(executionPorts, /pub trait SchedulerReceiptFactoryPortV1/);
  assert.match(executionWorker, /pub async fn execute_due_delivery_v1/);
  assert.match(executionWorker, /Sha256::digest/);
  assert.doesNotMatch(executionWorker, /cleanup_pending|request_cleanup/);
  assert.match(executionCleanup, /pub async fn process_body_cleanup_once_v1/);
  assert.match(executionCleanup, /complete_cleanup/);
  assert.match(executionCleanup, /reschedule_cleanup/);
  assert.match(executionCleanup, /MAX_RETRY_MILLIS/);
  assert.match(
    eventAdaptersManifest,
    /owner = "communication_delayed_delivery"[\s\S]*surface = "implementation"/,
  );
  assert.match(eventAdaptersSource, /pub fn build_scheduler_command_v1/);
  assert.match(eventAdaptersSource, /pub fn decode_scheduler_result_v1/);
  assert.match(eventAdaptersSource, /FenceKindV1::GrantEpoch/);
  assert.match(eventAdaptersSource, /expected_command_message_id/);
  assert.match(dueEventAdapterSource, /validate_scheduled_job_command_v1/);
  assert.match(dueEventAdapterSource, /JobTriggerKindV1::Time/);
  assert.match(dueEventAdapterSource, /JOB_EXECUTE_CAPABILITY_V1/);
  assert.match(dueEventAdapterSource, /FenceKindV1::RuntimeLease/);
  assert.match(dueEventAdapterSource, /JobRunOutcomeV1::Accepted/);
  assert.match(dueEventAdapterSource, /JobRunOutcomeV1::Succeeded/);
  assert.match(dueEventAdapterSource, /ReceiptBindingV1/);
  assert.doesNotMatch(dueEventAdapterSource, /job_kind: None/);
  assert.doesNotMatch(
    `${eventAdaptersSource}\n${dueEventAdapterSource}`,
    /scheduler_(?:implementation|persistence)|kernel::|communications_runtime|mail_runtime|telegram_runtime|whatsapp_runtime|zulip_runtime/,
  );
  assert.match(
    runtimeAdaptersManifest,
    /owner = "communication_delayed_delivery"[\s\S]*surface = "implementation"/,
  );
  assert.match(runtimeAdaptersSource, /impl BodyReadPortV1/);
  assert.match(runtimeAdaptersSource, /impl BodyCleanupPortV1/);
  assert.match(runtimeAdaptersSource, /impl DeliveryIntentRequestPortV1/);
  assert.match(runtimeAdaptersSource, /request_managed_blob_custody_release_v2/);
  assert.match(runtimeAdaptersSource, /Operation::RouteModuleRequest/);
  assert.doesNotMatch(
    runtimeAdaptersSource,
    /scheduler_(?:implementation|persistence)|kernel::|communications_runtime|mail_runtime|telegram_runtime|whatsapp_runtime|zulip_runtime/,
  );
  assert.match(
    storeAdaptersManifest,
    /owner = "communication_delayed_delivery"[\s\S]*surface = "persistence"/,
  );
  assert.match(storeAdaptersSource, /impl execution::ExecutionStorePortV1/);
  assert.match(storeAdaptersSource, /claim_due_execution/);
  assert.match(storeAdaptersSource, /mark_delivery_accepted/);
  assert.match(storeAdaptersSource, /mark_delivery_failed/);
  assert.match(storeAdaptersSource, /ClaimDueExecutionOutcomeV1::Duplicate/);
  assert.match(storeAdaptersSource, /impl execution::CleanupStorePortV1/);
  assert.match(storeAdaptersSource, /reschedule_body_cleanup/);
  assert.doesNotMatch(
    storeAdaptersSource,
    /sqlx|async_nats|scheduler_(?:implementation|persistence)|kernel::|communications_runtime|mail_runtime|telegram_runtime|whatsapp_runtime|zulip_runtime/,
  );
  assert.match(storeAdapterAdr, /не содержит SQL/);
  assert.match(storeAdapterAdr, /не repository facade/);
  assert.match(
    runtimeManifest,
    /owner = "communication_delayed_delivery"[\s\S]*surface = "runtime"/,
  );
  assert.match(runtimeAdmission, /communication_delayed_delivery_module_descriptor_v1/);
  assert.match(runtimeAdmission, /descriptor_revision: 2/);
  assert.match(
    runtimeAdmission,
    /COMMUNICATION_DELAYED_DELIVERY_DELIVERY_DEPENDENCY_CAPABILITY_ID_V1[\s\S]*dependencies: vec!\[delivery_intent_command_contract_v1\(\)\]/,
  );
  assert.match(runtimeAdmission, /ProvidedSurfaceKindV1::ClientRealtime/);
  assert.match(runtimeAdmission, /ClockTimerRequestV1/);
  assert.match(runtimeAdmission, /BlobQuotaOperationV1::ReleaseCustody/);
  assert.match(runtimeAdmission, /EventRouteDirectionV1::Publish/);
  assert.match(runtimeAdmission, /EventRouteDirectionV1::Consume/);
  assert.match(runtimeClientPort, /schedule_delayed_delivery_payload_v1/);
  assert.match(runtimeClientPort, /cancel_delayed_delivery_payload_v1/);
  assert.match(runtimeClientRealtime, /ManagedRuntimeClientRealtimePublishRequestV1/);
  assert.match(runtimeClientRealtime, /communication-delayed-delivery\/\{\}/);
  assert.match(runtimeSchedulerOutbox, /publish_exact/);
  assert.match(runtimeSchedulerOutbox, /mark_scheduler_message_published/);
  assert.match(runtimeSchedulerResults, /scheduler_result_causation_id_v1/);
  assert.match(runtimeSchedulerResults, /owns_scheduler_command/);
  assert.match(runtimeDueExecution, /decode_delayed_delivery_due_command_v1/);
  assert.match(runtimeDueExecution, /execute_due_delivery_v1/);
  assert.match(runtimeDueExecution, /DelayedDeliveryExecutionOutcomeV1::Retryable/);
  assert.match(runtimeDueExecution, /\.acknowledge\(\)/);
  assert.match(runtimeDueExecution, /build_delayed_delivery_terminal_receipt_v1/);
  assert.match(runtimeBodyCleanup, /process_body_cleanup_once_v1/);
  assert.match(managedRuntime, /Operation::ClientDelivery/);
  assert.match(managedRuntime, /pump_client_realtime_once/);
  assert.match(managedRuntime, /consume_due_delivery_once/);
  assert.match(managedRuntime, /process_body_cleanup_once/);
  assert.match(runtimeMain, /serve-inherited/);
  assert.match(runtimeMain, /as_millis/);
  assert.match(runtimeMain, /process_body_cleanup_once/);
  assert.match(
    assemblyManifest,
    /owner = "communication_delayed_delivery"[\s\S]*surface = "assembly"/,
  );
  assert.match(assemblySource, /materialize_delayed_delivery_release_assembly_v1/);
  assert.match(assemblySource, /communication_delayed_delivery\.runtime\.v1/);
  assert.match(assemblySource, /communication_delayed_delivery\.storage\.v1/);
  assert.match(assemblySource, /write_new_private_file/);
  assert.match(assemblyMain, /--runtime/);
  assert.match(
    developmentReleaseScript,
    /--package makosh-communication-delayed-delivery-assembly/,
  );
  assert.match(
    developmentReleaseScript,
    /communication_delayed_delivery\.release-artifacts\.json/,
  );
  assert.match(
    conformanceManifest,
    /name = "makosh-communication-delayed-delivery-testkit"/,
  );
  assert.match(
    conformanceManifest,
    /makosh-communication-delayed-delivery-persistence[\s\S]*features = \["conformance-test-support"\]/,
  );
  assert.match(conformanceTest, /durable_lifecycle_survives_restart/);
  assert.match(conformanceTest, /ClaimDueExecutionOutcomeV1::Duplicate/);
  assert.match(conformanceTest, /SchedulerScheduleResultV1::TooLate/);
  assert.match(conformanceTest, /client_realtime_window/);
  assert.match(conformanceTest, /cleanup retry must survive reconnect/);
  assert.match(conformanceTest, /DeliveryCancelled/);
  assert.match(
    authenticatedStorageRunner,
    /MAKOSH_COMMUNICATION_DELAYED_DELIVERY_POSTGRES_TEST_FILTER/,
  );
  assert.match(
    authenticatedStorageRunner,
    /makosh-communication-delayed-delivery-testkit/,
  );
  assert.match(
    authenticatedStorageRunner,
    /MAKOSH_COMMUNICATION_DELAYED_DELIVERY_RUNTIME_BIN/,
  );
  assert.match(
    authenticatedStorageRunner,
    /managed_delayed_delivery_starts_with_scheduler_and_delivery_intent/,
  );
  assert.match(
    managedSetup,
    /installed_delayed_delivery_conformance_release/,
  );
  assert.match(managedSetup, /scheduler_release_artifact/);
  assert.match(managedSetup, /delivery_intent_release_artifact/);
  assert.match(managedSetup, /communication_delayed_delivery_module_descriptor_v1/);
  assert.match(managedSetup, /communication_delayed_delivery_storage_bundle_v1/);
  assert.match(managedSetup, /start_reserved_workflow/);
  assert.match(
    managedFlow,
    /managed_delayed_delivery_starts_with_scheduler_and_delivery_intent/,
  );
  assert.match(managedFlow, /start_communications_domain/);
  assert.match(managedFlow, /start_delivery_intent_runtime/);
  assert.match(managedFlow, /start_delayed_delivery_runtime/);
  assert.match(managedFlow, /scheduler_launch::start_from_reservation/);
  assert.match(managedFlow, /supervisor[\s\S]*is_active/);
  assert.match(managedFlow, /assert_blob_release_committed/);
  assert.match(managedFlow, /Blob deletion reservation must be committed/);
  assert.doesNotMatch(
    `${assemblySource}\n${assemblyMain}`,
    /scheduler_(?:implementation|persistence)|communications_runtime|mail_runtime|telegram_runtime|whatsapp_runtime|zulip_runtime/,
  );
  assert.match(methodRoutingAdr, /Schedule -> communication\.delayed_delivery\.schedule@1/);
  assert.match(methodRoutingAdr, /Cancel   -> communication\.delayed_delivery\.cancel@1/);
  assert.doesNotMatch(
    `${runtimeAdmission}\n${runtimeClientPort}\n${runtimeClientRealtime}\n${runtimeDueExecution}\n${managedRuntime}`,
    /setInterval|polling|communications_runtime|mail_runtime|telegram_runtime|whatsapp_runtime|zulip_runtime/,
  );
  for (const source of [executionSource, executionPorts, executionWorker]) {
    assert.doesNotMatch(
      source,
      /scheduler_(?:implementation|persistence)|kernel::|communications_runtime|mail_runtime|telegram_runtime|whatsapp_runtime|zulip_runtime/,
    );
  }
  assert.doesNotMatch(apiProto, /provider_id|account_id|map</);
  for (const source of [apiSource, coreSource]) {
    assert.doesNotMatch(source, /async_nats|sqlx|kernel::/);
  }
  assert.match(adr, /scheduler\.schedule\.command\.v1/);
  assert.match(adr, /scheduler\.schedule\.result\.v1/);
  assert.match(adr, /ScheduledJobCommandV1/);
  assert.match(adr, /communication\.delivery_intent\.command/);
  assert.match(adr, /DurableEnvelopeV1/);
  assert.match(adr, /workflow-owned encrypted Blob custody/);
  assert.match(adr, /не вызывает Gateway/);
  assert.match(adr, /не импортирует Communications implementation/);
  assert.match(schedulerAdr, /gate реализован/);
  assert.doesNotMatch(
    adr,
    /direct (?:domain|integration|Scheduler) (?:call|socket|SQL)/i,
  );
});
