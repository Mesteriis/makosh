import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

const paths = {
  policy: new URL('architecture/policy.json', BACKEND_ROOT),
  sourceManifest: new URL(
    'src/communications-evidence-export-source-api/Cargo.toml',
    BACKEND_ROOT,
  ),
  sourceContract: new URL(
    'src/communications-evidence-export-source-api/proto/makosh/communications/evidence_export_source/v1/evidence_export_source.proto',
    BACKEND_ROOT,
  ),
  communicationsManifest: new URL('src/communications-runtime/Cargo.toml', BACKEND_ROOT),
  communicationsAdmission: new URL(
    'src/communications-runtime/src/admission.rs',
    BACKEND_ROOT,
  ),
  workflowManifest: new URL('src/communications-export-runtime/Cargo.toml', BACKEND_ROOT),
  workflowAdmission: new URL(
    'src/communications-export-runtime/src/admission.rs',
    BACKEND_ROOT,
  ),
  workflowRuntime: new URL(
    'src/communications-export-runtime/src/main.rs',
    BACKEND_ROOT,
  ),
  workflowManagedRuntime: new URL(
    'src/communications-export-runtime/src/runtime.rs',
    BACKEND_ROOT,
  ),
  workflowRealtime: new URL(
    'src/communications-export-runtime/src/client_realtime.rs',
    BACKEND_ROOT,
  ),
  workflowPersistenceRealtime: new URL(
    'src/communications-export-persistence/src/realtime.rs',
    BACKEND_ROOT,
  ),
  workflowAssembly: new URL(
    'src/communications-export-assembly/Cargo.toml',
    BACKEND_ROOT,
  ),
  kernelLaunch: new URL('src/kernel/src/platform/macos/managed_launch.rs', BACKEND_ROOT),
  runtimeDescriptorValidation: new URL(
    'src/platform/runtime_protocol/src/validation/descriptor.rs',
    BACKEND_ROOT,
  ),
  controlStoreClientBlob: new URL(
    'src/kernel/control_store/sqlite/src/module_state/client_blob_route.rs',
    BACKEND_ROOT,
  ),
  controlStoreClientBlobSchema: new URL(
    'src/kernel/control_store/sqlite/src/schema/v48_to_v49.rs',
    BACKEND_ROOT,
  ),
  gatewayClientBlob: new URL(
    'src/api/gateway/runtime/src/browser/client_blob.rs',
    BACKEND_ROOT,
  ),
  ownerControl: new URL(
    'src/api/gateway/contracts/proto/makosh/gateway/v1/owner_control.proto',
    BACKEND_ROOT,
  ),
  devAssembly: new URL('development/assembly/src/main.rs', BACKEND_ROOT),
  release: new URL('scripts/materialize-dev-release.sh', BACKEND_ROOT),
  frontendRoute: new URL(
    'frontend/src/domains/communications/views/CanonicalCommunicationsRoute.vue',
    PROJECT_ROOT,
  ),
  frontendApp: new URL('frontend/src/app/layout/AppLayoutRoot.vue', PROJECT_ROOT),
  frontendWorkflow: new URL(
    'frontend/src/workflows/communications-export/api/communicationsEvidenceExport.ts',
    PROJECT_ROOT,
  ),
  frontendController: new URL(
    'frontend/src/workflows/communications-export/queries/useCommunicationsEvidenceExport.ts',
    PROJECT_ROOT,
  ),
  frontendViteConfig: new URL('frontend/vite.config.ts', PROJECT_ROOT),
  managedExportTest: new URL(
    'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker.rs',
    BACKEND_ROOT,
  ),
  managedExportRace: new URL(
    'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/communications_export_race.rs',
    BACKEND_ROOT,
  ),
  authenticatedStorageHarness: new URL(
    'scripts/test-authenticated-storage.mjs',
    BACKEND_ROOT,
  ),
  adr: new URL(
    'docs/adr/ADR-0318-communications-evidence-export-workflow.md',
    PROJECT_ROOT,
  ),
  realtimeAdr: new URL(
    'docs/adr/ADR-0393-communications-export-status-over-shared-client-realtime.md',
    PROJECT_ROOT,
  ),
};

test('Communications export is one exact workflow family with a public domain source port', async () => {
  const [
    policySource,
    sourceManifest,
    sourceContract,
    communicationsManifest,
    communicationsAdmission,
    workflowManifest,
    workflowAdmission,
    workflowAssembly,
    adr,
  ] = await Promise.all([
    readFile(paths.policy, 'utf8'),
    readFile(paths.sourceManifest, 'utf8'),
    readFile(paths.sourceContract, 'utf8'),
    readFile(paths.communicationsManifest, 'utf8'),
    readFile(paths.communicationsAdmission, 'utf8'),
    readFile(paths.workflowManifest, 'utf8'),
    readFile(paths.workflowAdmission, 'utf8'),
    readFile(paths.workflowAssembly, 'utf8'),
    readFile(paths.adr, 'utf8'),
  ]);
  const policy = JSON.parse(policySource);
  const sourceSchema = sourceContract.replaceAll(/\/\/.*$/gm, '');
  assert.equal(
    policy.implementation.currentSlice,
    'speech_to_text_whisper_admission_v1',
  );
  assert.deepEqual(policy.implementation.ownerInventory.workflows, [
    'attachment_preview',
    'attachment_preview_evidence_replay',
    'attachment_text_extraction',
    'attachment_translation',
    'call_transcription',
    'communication_bulk_action',
    'communication_cross_channel_forward',
    'communication_delayed_delivery',
    'communication_delivery_intent',
    'communication_explanation',
    'communication_note_candidate_extraction',
    'communication_recipient_suggestion',
    'communication_reply_suggestion',
    'communication_summary',
    'communication_task_candidate_extraction',
    'communication_translation',
    'communications_export',
    'mail_persons_sync',
    'reviewed_note_candidate_promotion',
    'reviewed_person_match_candidate_promotion',
    'reviewed_task_candidate_promotion',
  ]);
  assert.deepEqual(
    policy.implementation.productionPackages
      .filter(({ owner }) => owner === 'communications_export')
      .map(({ name, surface }) => `${name}:${surface}`),
    [
      'makosh-communications-export-api:contract',
      'makosh-communications-export-core:implementation',
      'makosh-communications-export-persistence:persistence',
      'makosh-communications-export-runtime:runtime',
      'makosh-communications-export-assembly:assembly',
    ],
  );
  assert.match(sourceManifest, /role = "domain"[\s\S]*owner = "communications"/);
  assert.match(sourceSchema, /message PrepareEvidenceExportCommandV1/);
  assert.match(sourceSchema, /message EvidenceExportPreparedV1/);
  assert.doesNotMatch(sourceSchema, /\b(?:provider|locator|map)\b/i);
  assert.match(communicationsManifest, /makosh-communications-evidence-export-source-api/);
  assert.doesNotMatch(
    communicationsManifest,
    /makosh-communications-export-(?:api|core|persistence|runtime|assembly)/,
  );
  assert.match(communicationsAdmission, /communications\.export-source\.v1/);
  assert.match(workflowManifest, /role = "workflow"[\s\S]*owner = "communications_export"/);
  assert.doesNotMatch(
    workflowManifest,
    /makosh-(?:communications-domain|communications-persistence|mail|telegram|whatsapp|zulip)/,
  );
  assert.match(workflowAdmission, /ModuleKindV1::Workflow/);
  assert.match(workflowAssembly, /surface = "assembly"/);
  assert.match(adr, /makosh-communications-export-assembly/);
});

test('Kernel launches workflow through a distinct provider-neutral configuration', async () => {
  const [
    kernelLaunch,
    ownerControl,
    workflowRuntime,
    workflowManagedRuntime,
    devAssembly,
    release,
  ] = await Promise.all([
    readFile(paths.kernelLaunch, 'utf8'),
    readFile(paths.ownerControl, 'utf8'),
    readFile(paths.workflowRuntime, 'utf8'),
    readFile(paths.workflowManagedRuntime, 'utf8'),
    readFile(paths.devAssembly, 'utf8'),
    readFile(paths.release, 'utf8'),
  ]);
  assert.match(kernelLaunch, /fn start_reserved_workflow/);
  assert.match(kernelLaunch, /expected_module_kind: ModuleKindV1::Workflow/);
  const workflowLaunch = kernelLaunch.match(
    /pub\(crate\) fn start_reserved_workflow\([\s\S]*?\n\}/,
  )?.[0] ?? '';
  assert.doesNotMatch(workflowLaunch, /ManagedIntegration|ManagedDomain/);
  assert.match(workflowLaunch, /configuration,\s+None,\s+granted_capability_ids/);
  assert.match(kernelLaunch, /fn start_reserved_workflow_with_settings/);
  assert.doesNotMatch(workflowLaunch, /host_bridge_configuration/);
  assert.match(ownerControl, /StartReservedWorkflowRuntimeRequestV1/);
  assert.match(workflowRuntime, /ManagedWorkflowRuntimeConfigurationV1/);
  const eventAccess = workflowManagedRuntime.match(
    /request_managed_runtime_event_access_v2\([\s\S]*?\)\n\s*\.map_err/,
  )?.[0] ?? '';
  assert.match(eventAccess, /&storage_configuration\.logical_owner_id/);
  assert.doesNotMatch(eventAccess, /&admission\.logical_owner_id/);
  assert.match(devAssembly, /ModuleRuntimeKindV1::Workflow/);
  assert.match(devAssembly, /start_reserved_workflow_runtime/);
  assert.match(release, /makosh-communications-export-assembly/);
  assert.match(release, /communications_export\.release-artifacts\.json/);
});

test('export artifact stays within one exact platform client Blob ceiling', async () => {
  const [
    runtimeDescriptorValidation,
    controlStoreClientBlob,
    controlStoreClientBlobSchema,
    gatewayClientBlob,
    workflowAdmission,
  ] = await Promise.all([
      readFile(paths.runtimeDescriptorValidation, 'utf8'),
      readFile(paths.controlStoreClientBlob, 'utf8'),
      readFile(paths.controlStoreClientBlobSchema, 'utf8'),
      readFile(paths.gatewayClientBlob, 'utf8'),
      readFile(paths.workflowAdmission, 'utf8'),
    ]);
  const exactCeiling = /const MAX_CLIENT_BLOB_RESPONSE_BYTES: u64 = 32 \* 1024 \* 1024;/;
  assert.match(runtimeDescriptorValidation, exactCeiling);
  assert.match(controlStoreClientBlob, exactCeiling);
  assert.match(
    controlStoreClientBlobSchema,
    /max_response_bytes BETWEEN 1 AND 33554432/,
  );
  assert.match(gatewayClientBlob, exactCeiling);
  assert.match(
    workflowAdmission,
    /max_response_bytes: COMMUNICATIONS_EXPORT_MAX_ARTIFACT_BYTES_V1/,
  );
});

test('Frontend composition passes canonical IDs into a generated workflow without domain imports', async () => {
  const [route, app, workflow, controller, viteConfig] = await Promise.all([
    readFile(paths.frontendRoute, 'utf8'),
    readFile(paths.frontendApp, 'utf8'),
    readFile(paths.frontendWorkflow, 'utf8'),
    readFile(paths.frontendController, 'utf8'),
    readFile(paths.frontendViteConfig, 'utf8'),
  ]);
  assert.match(route, /canonicalMessageSelected/);
  assert.doesNotMatch(route, /workflows\/communications-export|communicationsEvidenceExport/);
  assert.match(app, /CommunicationsEvidenceExportWorkflow/);
  assert.match(app, /communications\.export\.v1/);
  assert.match(workflow, /getCommunicationsExportCommandClient/);
  assert.match(workflow, /getCommunicationsExportQueryClient/);
  assert.match(workflow, /getCommunicationsExportTicketClient/);
  assert.match(workflow, /BrowserGatewayFetch/);
  assert.match(workflow, /getBrowserGatewayRealtimeHub/);
  assert.match(controller, /openCommunicationsEvidenceExportRealtime/);
  assert.doesNotMatch(controller, /setInterval|setTimeout|poll/i);
  assert.match(viteConfig, /'\/api\/blobs\/': developmentGatewayProxy\(developmentGateway\)/);
  assert.doesNotMatch(workflow, /integrations\/(?:mail|telegram|whatsapp|zulip)|provider/);
});

test('export status is owner-local replay and publishes only through shared client realtime', async () => {
  const [admission, runtime, realtime, persistence, proto, adr] = await Promise.all([
    readFile(paths.workflowAdmission, 'utf8'),
    readFile(paths.workflowManagedRuntime, 'utf8'),
    readFile(paths.workflowRealtime, 'utf8'),
    readFile(paths.workflowPersistenceRealtime, 'utf8'),
    readFile(new URL('src/communications-export-api/proto/makosh/communications_export/v1/export.proto', BACKEND_ROOT), 'utf8'),
    readFile(paths.realtimeAdr, 'utf8'),
  ]);
  assert.match(admission, /ProvidedSurfaceKindV1::ClientRealtime/);
  assert.match(admission, /communications_export_realtime_contract_reference_v1/);
  assert.match(runtime, /pump_client_realtime_once/);
  assert.match(realtime, /ManagedRuntimeClientRealtimePublishRequestV1/);
  assert.match(realtime, /communications-export\/\{\}/);
  assert.match(persistence, /client_realtime_window/);
  assert.match(persistence, /logical_owner_id/);
  assert.match(proto, /message EvidenceExportStatusChangedV1/);
  assert.doesNotMatch(proto, /artifact_sha256|blob|provider|content/);
  assert.match(adr, /periodic polling/);
});

test('managed gate deterministically rejects a canonical revision race', async () => {
  const [managedTest, race, harness, adr] = await Promise.all([
    readFile(paths.managedExportTest, 'utf8'),
    readFile(paths.managedExportRace, 'utf8'),
    readFile(paths.authenticatedStorageHarness, 'utf8'),
    readFile(paths.adr, 'utf8'),
  ]);
  assert.match(managedTest, /revision_race\.arm\(/);
  assert.match(managedTest, /EvidenceExportRejectCodeStaleRevision/);
  assert.match(managedTest, /communications_export_rejection_code/);
  assert.match(race, /COMMUNICATIONS_EXPORT_SOURCE_BLOB_CAPABILITY_ID/);
  assert.match(
    race,
    /SET canonical_revision = canonical_revision \+ 1[\s\S]*RETURNING canonical_revision/,
  );
  assert.match(race, /impl ManagedRuntimeBlobSessionHandler/);
  assert.match(harness, /MAKOSH_ATTACHMENT_SECURITY_CLAMAV_PORT: String\(secrets\.clamavPort\)/);
  assert.match(adr, /Состояние реализации: implemented/);
  assert.doesNotMatch(adr, /Gate всё ещё не `implemented`/);
});
