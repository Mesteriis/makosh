import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

const paths = {
  adr: new URL(
    'docs/adr/ADR-0292-managed-integration-settings-apply-and-credential-binding.md',
    PROJECT_ROOT,
  ),
  inventory: new URL(
    'architecture/communications-settings-reconstruction.json',
    BACKEND_ROOT,
  ),
  ownerControl: new URL(
    'src/api/gateway/contracts/proto/makosh/gateway/v1/owner_control.proto',
    BACKEND_ROOT,
  ),
  kernelApply: new URL(
    'src/kernel/src/modules/settings/managed_application.rs',
    BACKEND_ROOT,
  ),
  kernelDispatch: new URL(
    'src/kernel/src/identity/owner_control/dispatch.rs',
    BACKEND_ROOT,
  ),
  accountProto: new URL(
    'src/zulip-api/proto/makosh/zulip/account/v1/client.proto',
    BACKEND_ROOT,
  ),
  clientContract: new URL('src/zulip-api/src/client_contract.rs', BACKEND_ROOT),
  settings: new URL('src/zulip-runtime/src/settings.rs', BACKEND_ROOT),
  persistence: new URL('src/zulip-persistence/src/account.rs', BACKEND_ROOT),
  schema: new URL('src/zulip-persistence/src/schema.rs', BACKEND_ROOT),
  runtime: new URL('src/zulip-runtime/src/managed.rs', BACKEND_ROOT),
  liveFlow: new URL(
    'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/zulip_managed_flow.rs',
    BACKEND_ROOT,
  ),
};

test('managed integration Settings apply remains provider-neutral and fail-closed', async () => {
  const [adr, inventorySource, ownerControl, kernelApply, kernelDispatch] =
    await Promise.all([
      readFile(paths.adr, 'utf8'),
      readFile(paths.inventory, 'utf8'),
      readFile(paths.ownerControl, 'utf8'),
      readFile(paths.kernelApply, 'utf8'),
      readFile(paths.kernelDispatch, 'utf8'),
    ]);
  const inventory = JSON.parse(inventorySource);
  const slice = inventory.slices.find(
    ({ gate }) => gate === 'managed_integration_settings_apply_v1',
  );

  assert.deepEqual(slice, {
    gate: 'managed_integration_settings_apply_v1',
    role: 'platform',
    owner: 'kernel_settings_apply',
    state: 'implemented',
    dependsOn: [
      'client_gateway_v1',
      'managed_launch_trust_v1',
      'module_control_plane_v1',
      'storage_control_v1',
      'vault_v1',
    ],
  });
  assert.match(
    ownerControl,
    /message ApplyManagedIntegrationSettingsRequestV1[\s\S]*expected_desired_revision/,
  );
  assert.match(
    ownerControl,
    /apply_managed_integration_settings = 39[\s\S]*apply_managed_integration_settings = 40/,
  );
  assert.match(kernelApply, /SettingApplyModeV1::RestartModule/);
  assert.match(kernelApply, /successor::reserve/);
  assert.match(kernelApply, /ApplyAcknowledgement::RuntimeApplied/);
  assert.match(kernelApply, /SettingsApplyState::BlockedConfig/);
  assert.match(kernelDispatch, /managed_settings_application::prepare/);
  assert.match(kernelDispatch, /managed_settings_application::wait_for_ready_and_confirm/);
  assert.doesNotMatch(
    `${kernelApply}\n${kernelDispatch}`,
    /makosh_(?:mail|telegram|whatsapp|zulip)|Mail|Telegram|WhatsApp|Zulip/,
  );
  assert.match(adr, /Kernel\/Core согласуют только/);
  assert.match(adr, /импортируют integration packages/);
});

test('Zulip account lifecycle keeps credentials out of Settings and applies by successor', async () => {
  const [
    inventorySource,
    accountProto,
    clientContract,
    settings,
    persistence,
    schema,
    runtime,
    liveFlow,
  ] = await Promise.all([
    readFile(paths.inventory, 'utf8'),
    readFile(paths.accountProto, 'utf8'),
    readFile(paths.clientContract, 'utf8'),
    readFile(paths.settings, 'utf8'),
    readFile(paths.persistence, 'utf8'),
    readFile(paths.schema, 'utf8'),
    readFile(paths.runtime, 'utf8'),
    readFile(paths.liveFlow, 'utf8'),
  ]);
  const inventory = JSON.parse(inventorySource);
  const slice = inventory.slices.find(
    ({ gate }) => gate === 'zulip_account_lifecycle_v1',
  );

  assert.equal(slice.role, 'integration');
  assert.equal(slice.owner, 'zulip');
  assert.equal(slice.state, 'implemented');
  assert.ok(slice.dependsOn.includes('managed_integration_settings_apply_v1'));
  assert.match(
    accountProto,
    /service ZulipAccountLifecycleService[\s\S]*rpc Apply/,
  );
  assert.match(
    accountProto,
    /expected_binding_revision[\s\S]*credential_revision/,
  );
  assert.doesNotMatch(accountProto, /api_key|password|secret_ref|record_id/i);
  assert.match(clientContract, /zulip\.account\.lifecycle\.v1/);
  assert.match(
    clientContract,
    /\/makosh\.zulip\.account\.v1\.ZulipAccountLifecycleService\/Apply/,
  );
  assert.doesNotMatch(settings, /api_key|credential_revision|secret_ref|record_id/i);
  assert.match(settings, /SettingClientVisibilityV1::Editable/);
  assert.match(persistence, /expected_binding_revision/);
  assert.match(persistence, /pending_restart|PendingRestart/);
  assert.match(schema, /ZULIP_STORAGE_BUNDLE_REVISION_V3/);
  assert.match(schema, /zulip_account_credential_bindings/);
  assert.doesNotMatch(
    schema.match(/pub const ZULIP_SCHEMA_V3:[\s\S]*?\"#;/)?.[0] ?? '',
    /api_key|secret_ref|record_id/i,
  );
  assert.match(runtime, /credential_binding\(&account\.account_id\)/);
  assert.match(runtime, /ZulipCredentialBindingStateV1::Retired => None/);
  assert.match(runtime, /mark_credential_binding_active/);
  assert.match(
    liveFlow,
    /managed_zulip_account_rotation_and_retirement_use_settings_successors/,
  );
  assert.match(liveFlow, /SettingsApplyState::BlockedConfig/);
  assert.match(liveFlow, /credential_v2_requests/);
  assert.match(liveFlow, /failed Zulip successor must not reactivate its predecessor/);
});
