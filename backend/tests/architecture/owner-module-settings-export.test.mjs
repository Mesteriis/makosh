import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

const paths = {
  adr: new URL(
    'docs/adr/ADR-0297-fresh-owner-proof-effective-module-settings-export.md',
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
  exportAuthority: new URL(
    'src/kernel/src/modules/settings/owner_gateway/export.rs',
    BACKEND_ROOT,
  ),
  values: new URL(
    'src/kernel/src/modules/settings/owner_gateway/values.rs',
    BACKEND_ROOT,
  ),
  conformance: new URL(
    'tests/support/kernel-recovery/src/tests/owner_module_settings.rs',
    BACKEND_ROOT,
  ),
  generatedClient: new URL(
    'frontend/src/gen/makosh/gateway/v1/owner_module_settings_pb.ts',
    PROJECT_ROOT,
  ),
  desktopAdapter: new URL(
    'frontend/src/platform/settings/ownerModuleSettingsClient.ts',
    PROJECT_ROOT,
  ),
  deviceProof: new URL(
    'frontend/src/platform/gateway/ownerDeviceProof.ts',
    PROJECT_ROOT,
  ),
  deviceProofFactory: new URL(
    'frontend/src/platform/gateway/ownerDeviceProofFactory.ts',
    PROJECT_ROOT,
  ),
};

test('effective Settings export is fresh-proof, current, typed and provider-neutral', async () => {
  const [
    adr,
    inventorySource,
    contract,
    exportAuthority,
    values,
    conformance,
    generatedClient,
    desktopAdapter,
    deviceProof,
    deviceProofFactory,
  ] = await Promise.all(Object.values(paths).map((path) => readFile(path, 'utf8')));
  const inventory = JSON.parse(inventorySource);
  const slice = inventory.slices.find(
    ({ gate }) => gate === 'owner_module_settings_export_v1',
  );

  assert.deepEqual(slice, {
    gate: 'owner_module_settings_export_v1',
    role: 'platform',
    owner: 'kernel_settings_registry',
    state: 'implemented',
    dependsOn: ['owner_module_settings_gateway_v1'],
  });
  assert.match(adr, /Backend authority, generated desktop adapter[\s\S]*реализованы/);
  assert.match(contract, /ExportEffectiveOwnerModuleSettingsV1/);
  assert.match(contract, /expected_effective_revision/);
  assert.match(contract, /ExportEffectiveOwnerModuleSettingsReceiptV1/);
  assert.match(contract, /repeated OwnerSettingEntryV1 values/);
  assert.doesNotMatch(
    contract,
    /\b(?:raw_snapshot|snapshot_bytes|schema_bytes|credential_revision|secret_reference|storage_location)\b/i,
  );
  assert.match(exportAuthority, /SettingsApplyState::Current/);
  assert.match(exportAuthority, /target\.desired_revision\(\) != target\.effective_revision\(\)/);
  assert.match(exportAuthority, /snapshot\.target_id != export\.configuration_instance_id/);
  assert.match(exportAuthority, /settings_schema_artifact/);
  assert.match(exportAuthority, /schema_sha256 != \*binding\.schema_sha256\(\)/);
  assert.match(exportAuthority, /validate_settings_snapshot_against_schema_v1/);
  assert.match(exportAuthority, /visible_public_values/);
  assert.match(values, /SettingClientVisibilityV1::Editable/);
  assert.match(values, /SettingClientVisibilityV1::ReadOnly/);
  assert.doesNotMatch(values, /SettingClientVisibilityV1::Hidden[\s\S]*public_entry/);
  assert.match(
    conformance,
    /owner_settings_export_returns_only_current_client_visible_values/,
  );
  assert.match(conformance, /stale export revision must fail/);
  assert.match(conformance, /non-current Settings must not export/);
  assert.match(conformance, /client_visibility: SettingClientVisibilityV1::Hidden/);
  assert.match(generatedClient, /export const OwnerModuleSettingsService/);
  assert.match(generatedClient, /case: "exportEffective"/);
  assert.match(desktopAdapter, /createClient\([\s\S]*OwnerModuleSettingsService/);
  assert.match(desktopAdapter, /createOwnerDeviceProofV1/);
  assert.match(deviceProofFactory, /DevelopmentOwnerDeviceProofV1/);
  assert.match(deviceProofFactory, /BrowserOwnerDeviceProofV1/);
  assert.match(desktopAdapter, /async exportEffective/);
  assert.match(deviceProof, /signBrowserLocalDeviceChallenge/);
  assert.doesNotMatch(
    `${contract}\n${exportAuthority}\n${values}\n${desktopAdapter}\n${deviceProof}\n${deviceProofFactory}`,
    /makosh_(?:mail|telegram|whatsapp|zulip|communications)|Mail|Telegram|WhatsApp|Zulip/,
  );
});
