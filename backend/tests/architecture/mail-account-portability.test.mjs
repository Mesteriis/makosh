import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

const paths = {
  adr: new URL(
    'docs/adr/ADR-0294-mail-account-credential-lifecycle-and-portability.md',
    PROJECT_ROOT,
  ),
  vaultAdr: new URL(
    'docs/adr/ADR-0295-owner-write-only-vault-provisioning-through-core-gateway.md',
    PROJECT_ROOT,
  ),
  inventory: new URL(
    'architecture/communications-settings-reconstruction.json',
    BACKEND_ROOT,
  ),
  contract: new URL(
    'src/mail-api/proto/makosh/mail/portability/v1/portability.proto',
    BACKEND_ROOT,
  ),
  validator: new URL('src/mail-api/src/portability.rs', BACKEND_ROOT),
  runtimeSettings: new URL('src/mail-runtime/src/settings.rs', BACKEND_ROOT),
  generatedClient: new URL(
    'frontend/src/gen/makosh/mail/portability/v1/portability_pb.ts',
    PROJECT_ROOT,
  ),
  codec: new URL(
    'frontend/src/integrations/mail/portability/mailAccountPortabilityCodec.ts',
    PROJECT_ROOT,
  ),
  workflow: new URL(
    'frontend/src/integrations/mail/portability/mailAccountPortabilityWorkflow.ts',
    PROJECT_ROOT,
  ),
  workflowTests: new URL(
    'frontend/src/integrations/mail/portability/mailAccountPortabilityWorkflow.test.ts',
    PROJECT_ROOT,
  ),
  ui: new URL(
    'frontend/src/integrations/mail/presentation/MailPortabilityPanel.vue',
    PROJECT_ROOT,
  ),
  settingsPanel: new URL(
    'frontend/src/integrations/mail/presentation/MailSettingsPanel.vue',
    PROJECT_ROOT,
  ),
  androidHost: new URL(
    'frontend/src/platform/vault/ownerVaultProvisioningAndroidHost.ts',
    PROJECT_ROOT,
  ),
  provisioningFactory: new URL(
    'frontend/src/platform/vault/ownerVaultProvisioningHostFactory.ts',
    PROJECT_ROOT,
  ),
  vaultClient: new URL(
    'frontend/src/platform/vault/ownerVaultProvisioningClient.ts',
    PROJECT_ROOT,
  ),
  vaultHost: new URL(
    'frontend/src/platform/vault/ownerVaultProvisioningHost.ts',
    PROJECT_ROOT,
  ),
};

test('Mail account portability is one typed desktop app composition with explicit receipts', async () => {
  const [
    adr,
    vaultAdr,
    inventorySource,
    contract,
    validator,
    runtimeSettings,
    generatedClient,
    codec,
    workflow,
    workflowTests,
    ui,
    settingsPanel,
    androidHost,
    provisioningFactory,
    vaultClient,
    vaultHost,
  ] = await Promise.all(Object.values(paths).map((path) => readFile(path, 'utf8')));
  const inventory = JSON.parse(inventorySource);
  const slices = new Map(inventory.slices.map((slice) => [slice.gate, slice]));

  assert.deepEqual(slices.get('owner_vault_provisioning_desktop_v1'), {
    gate: 'owner_vault_provisioning_desktop_v1',
    role: 'app',
    owner: 'first_party_desktop',
    state: 'implemented',
    dependsOn: ['owner_vault_provisioning_backend_v1'],
  });
  assert.deepEqual(slices.get('owner_vault_provisioning_v1'), {
    gate: 'owner_vault_provisioning_v1',
    role: 'app',
    owner: 'first_party_client',
    state: 'implemented',
    dependsOn: [
      'owner_vault_provisioning_backend_v1',
      'owner_vault_provisioning_desktop_v1',
    ],
  });
  assert.deepEqual(slices.get('mail_account_portability_v1'), {
    gate: 'mail_account_portability_v1',
    role: 'app',
    owner: 'first_party_client',
    state: 'implemented',
    dependsOn: [
      'client_gateway_v1',
      'mail_account_credential_binding_v1',
      'managed_integration_settings_apply_v1',
      'owner_module_settings_export_v1',
      'owner_module_settings_gateway_v1',
      'owner_vault_provisioning_desktop_v1',
    ],
  });
  assert.equal(slices.get('mail_account_lifecycle_v1').state, 'implemented');
  assert.match(adr, /configuration-only successor[\s\S]*credential successor/);
  assert.match(adr, /MailAccountExportV1/);
  assert.match(vaultAdr, /owner_vault_provisioning_desktop_v1[\s\S]*Implemented/);
  assert.match(contract, /message MailAccountExportV1/);
  assert.match(contract, /oneof inbound[\s\S]*MailImapConfigurationV1 imap[\s\S]*MailGmailConfigurationV1 gmail/);
  assert.match(contract, /optional MailSmtpConfigurationV1 smtp/);
  assert.match(contract, /settings_schema_major/);
  assert.match(contract, /effective_settings_revision/);
  assert.doesNotMatch(
    contract,
    /\b(?:password|authorization_code|session|cursor|secret_reference|record_id|wrapping_key|credential_revision)\b/i,
  );
  assert.match(validator, /validate_mail_account_export_v1/);
  assert.match(validator, /valid_account_configuration/);
  assert.match(validator, /valid_gmail_oauth_configuration/);
  assert.match(validator, /profile_matches/);
  assert.match(runtimeSettings, /pub use makosh_mail_api::\{[\s\S]*MAIL_SETTINGS_SCHEMA_MAJOR_V2/);
  assert.match(generatedClient, /export type MailAccountExportV1/);
  assert.match(codec, /ignoreUnknownFields: false/);
  assert.match(codec, /mailAccountExportSettingsInputs/);
  assert.match(workflow, /settingsUpdateReceipt/);
  assert.match(workflow, /configurationApplyReceipt/);
  assert.match(workflow, /vaultReceipt/);
  assert.match(workflow, /bindingReceipt/);
  assert.match(workflow, /activationApplyReceipt/);
  assert.match(workflow, /gmailOAuthStarted/);
  assert.match(workflow, /gmailOAuthAccepted/);
  assert.match(workflow, /gmailOAuthStatus/);
  assert.match(workflowTests, /without provisioning twice/);
  assert.match(workflowTests, /separate receipts/);
  assert.match(ui, /Import receipts/);
  assert.match(ui, /Continue with Google/);
  assert.match(settingsPanel, /MailPortabilityPanel/);
  assert.match(vaultClient, /OwnerVaultProvisioningService/);
  assert.match(androidHost, /AndroidOwnerVaultProvisioningHostV1/);
  assert.match(provisioningFactory, /hasAndroidOwnerVaultProvisioningHostV1/);
  assert.match(provisioningFactory, /new AndroidOwnerVaultProvisioningHostV1/);
  assert.match(vaultHost, /owner_vault_provisioning_host_seal/);
  assert.doesNotMatch(
    `${contract}\n${validator}\n${codec}\n${workflow}\n${ui}`,
    /domains\/communications|makosh_communications|makosh-kernel|mail-runtime|mail-persistence|vault-store/i,
  );
});
