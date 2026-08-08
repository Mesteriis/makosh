import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

const paths = {
  adr: new URL(
    'docs/adr/ADR-0296-owner-module-settings-through-core-gateway.md',
    PROJECT_ROOT,
  ),
  inventory: new URL(
    'architecture/communications-settings-reconstruction.json',
    BACKEND_ROOT,
  ),
  contract: new URL(
    'src/api/gateway/contracts/proto/makosh/gateway/v1/owner_module_settings.proto',
    BACKEND_ROOT,
  ),
  router: new URL(
    'src/api/gateway/runtime/src/browser/owner_settings.rs',
    BACKEND_ROOT,
  ),
  application: new URL(
    'src/api/gateway/runtime/src/application/mod.rs',
    BACKEND_ROOT,
  ),
  proof: new URL(
    'src/kernel/src/platform/gateway/owner_device_proof.rs',
    BACKEND_ROOT,
  ),
  authority: new URL(
    'src/kernel/src/modules/settings/owner_gateway/mod.rs',
    BACKEND_ROOT,
  ),
  authorization: new URL(
    'src/kernel/src/modules/settings/owner_gateway/authorization.rs',
    BACKEND_ROOT,
  ),
  values: new URL(
    'src/kernel/src/modules/settings/owner_gateway/values.rs',
    BACKEND_ROOT,
  ),
  operation: new URL(
    'src/kernel/src/modules/settings/owner_gateway/operation.rs',
    BACKEND_ROOT,
  ),
  state: new URL(
    'src/kernel/src/modules/settings/owner_gateway/state.rs',
    BACKEND_ROOT,
  ),
  launch: new URL(
    'src/kernel/src/runtime/lifecycle/integration_launch.rs',
    BACKEND_ROOT,
  ),
  workflowLaunch: new URL(
    'src/kernel/src/runtime/lifecycle/workflow_launch.rs',
    BACKEND_ROOT,
  ),
  workflowAdr: new URL(
    'docs/adr/ADR-0385-owner-authorized-managed-workflow-settings-application.md',
    PROJECT_ROOT,
  ),
  gatewayComposition: new URL(
    'src/kernel/src/platform/gateway.rs',
    BACKEND_ROOT,
  ),
  conformance: new URL(
    'tests/support/kernel-recovery/src/tests/owner_module_settings.rs',
    BACKEND_ROOT,
  ),
  bootstrapContract: new URL(
    'src/api/gateway/contracts/proto/makosh/gateway/v1/client_bootstrap.proto',
    BACKEND_ROOT,
  ),
  bootstrapProjection: new URL(
    'src/kernel/src/identity/browser_gateway.rs',
    BACKEND_ROOT,
  ),
  bootstrapConformance: new URL(
    'tests/support/kernel-recovery/src/tests/browser_gateway_session/connect_status.rs',
    BACKEND_ROOT,
  ),
};

test('owner module Settings is one provider-neutral fresh-proof Gateway authority', async () => {
  const [
    adr,
    inventorySource,
    contract,
    router,
    application,
    proof,
    authority,
    authorization,
    values,
    operation,
    state,
    launch,
    workflowLaunch,
    workflowAdr,
    gatewayComposition,
    conformance,
    bootstrapContract,
    bootstrapProjection,
    bootstrapConformance,
  ] = await Promise.all(Object.values(paths).map((path) => readFile(path, 'utf8')));
  const inventory = JSON.parse(inventorySource);
  const slice = inventory.slices.find(
    ({ gate }) => gate === 'owner_module_settings_gateway_v1',
  );

  assert.deepEqual(slice, {
    gate: 'owner_module_settings_gateway_v1',
    role: 'platform',
    owner: 'kernel_settings_registry',
    state: 'implemented',
    dependsOn: ['client_gateway_v1', 'managed_integration_settings_apply_v1'],
  });
  assert.match(adr, /OwnerModuleSettingsService[\s\S]*Prepare[\s\S]*Commit/);
  assert.match(contract, /service OwnerModuleSettingsService/);
  assert.match(contract, /oneof operation/);
  assert.match(contract, /oneof value/);
  assert.doesNotMatch(contract, /owner_control|runtime_protocol|provider|credential/i);
  assert.match(router, /is_lan_development/);
  assert.match(router, /require_mutation_origin/);
  assert.match(router, /authorize_request/);
  assert.match(router, /spawn_blocking/);
  assert.match(application, /with_owner_module_settings/);
  assert.match(proof, /BrowserDeviceStateV1::Active/);
  assert.match(proof, /VerifyingKey::from_sec1_bytes/);
  assert.match(authority, /challenge_digest/);
  assert.match(authority, /control_generation/);
  assert.match(authority, /identity_epoch/);
  assert.match(authority, /verify_fresh_proof/);
  assert.match(authorization, /ModuleRegistrationState::Approved/);
  assert.match(authorization, /settings_schema_binding/);
  assert.match(values, /SettingsSnapshotV1/);
  assert.match(values, /UnsignedIntegerValue/);
  assert.match(operation, /commit_after_owner_authorization/);
  assert.match(operation, /managed_application::prepare/);
  assert.match(operation, /integration_launch::launch_reserved/);
  assert.match(contract, /ApplyOwnerManagedWorkflowSettingsV1/);
  assert.match(contract, /ApplyOwnerManagedWorkflowSettingsReceiptV1/);
  assert.match(operation, /managed_application::prepare/);
  assert.match(operation, /workflow_launch::launch_reserved/);
  assert.match(operation, /Result::WorkflowApplied/);
  assert.match(workflowLaunch, /ManagedWorkflowRuntimeConfigurationV1/);
  assert.match(workflowLaunch, /start_reserved_workflow_with_settings/);
  assert.doesNotMatch(workflowLaunch, /host_bridge|integration_state_root/);
  assert.match(workflowAdr, /generic `apply_module`/);
  assert.match(workflowAdr, /workflow apply не является integration, domain или assembly unit/);
  assert.match(operation, /wait_for_ready_and_confirm/);
  assert.match(state, /MAX_PENDING_CHALLENGES: usize = 64/);
  assert.match(state, /pending\.remove/);
  assert.match(launch, /ManagedIntegrationRuntimeConfigurationV1/);
  assert.match(gatewayComposition, /new_lan_development/);
  assert.match(gatewayComposition, /with_lan_development_policy/);
  assert.match(conformance, /fresh_proof_and_preserves_schema_cas/);
  assert.match(conformance, /denies_lan_mode/);
  assert.match(conformance, /challenge must be single-use/);
  assert.match(bootstrapContract, /repeated ClientModuleSettingsTargetBootstrapV1 settings_targets = 7/);
  assert.match(bootstrapProjection, /settings_configuration_targets/);
  assert.match(bootstrapProjection, /visible_settings_for_target/);
  assert.match(bootstrapConformance, /settings_targets\.len\(\), 2/);
  assert.match(bootstrapConformance, /configuration-current/);
  assert.doesNotMatch(
    `${contract}\n${router}\n${proof}\n${authority}\n${authorization}\n${values}\n${operation}\n${state}\n${launch}\n${workflowLaunch}`,
    /makosh_(?:mail|telegram|whatsapp|zulip|communications)|Mail|Telegram|WhatsApp|Zulip/,
  );
});
