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
  inventory: new URL(
    'architecture/communications-settings-reconstruction.json',
    BACKEND_ROOT,
  ),
  proto: new URL(
    'src/mail-api/proto/makosh/mail/account/v1/client.proto',
    BACKEND_ROOT,
  ),
  account: new URL('src/mail-api/src/account.rs', BACKEND_ROOT),
  portability: new URL('src/mail-api/src/portability.rs', BACKEND_ROOT),
  contract: new URL('src/mail-api/src/client_contract.rs', BACKEND_ROOT),
  persistence: new URL('src/mail-persistence/src/account.rs', BACKEND_ROOT),
  schema: new URL('src/mail-persistence/src/schema.rs', BACKEND_ROOT),
  settings: new URL('src/mail-runtime/src/settings.rs', BACKEND_ROOT),
  runtime: new URL('src/mail-runtime/src/managed.rs', BACKEND_ROOT),
  admission: new URL('src/mail-runtime/src/admission.rs', BACKEND_ROOT),
  assembly: new URL('src/mail-assembly/src/lib.rs', BACKEND_ROOT),
  live: new URL(
    'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/mail_account_credential_flow.rs',
    BACKEND_ROOT,
  ),
};

test('Mail credential binding is owner-local, configuration-only and successor-activated', async () => {
  const [
    adr,
    inventorySource,
    proto,
    account,
    portability,
    contract,
    persistence,
    schema,
    settings,
    runtime,
    admission,
    assembly,
    live,
  ] = await Promise.all([
    readFile(paths.adr, 'utf8'),
    readFile(paths.inventory, 'utf8'),
    readFile(paths.proto, 'utf8'),
    readFile(paths.account, 'utf8'),
    readFile(paths.portability, 'utf8'),
    readFile(paths.contract, 'utf8'),
    readFile(paths.persistence, 'utf8'),
    readFile(paths.schema, 'utf8'),
    readFile(paths.settings, 'utf8'),
    readFile(paths.runtime, 'utf8'),
    readFile(paths.admission, 'utf8'),
    readFile(paths.assembly, 'utf8'),
    readFile(paths.live, 'utf8'),
  ]);
  const inventory = JSON.parse(inventorySource);
  const slice = inventory.slices.find(
    ({ gate }) => gate === 'mail_account_credential_binding_v1',
  );

  assert.deepEqual(slice, {
    gate: 'mail_account_credential_binding_v1',
    role: 'integration',
    owner: 'mail',
    state: 'implemented',
    dependsOn: [
      'client_gateway_v1',
      'managed_integration_settings_apply_v1',
      'vault_v1',
    ],
  });
  assert.match(
    adr,
    /Состояние реализации: Phase 1 `mail_account_credential_binding_v1`, Phase 2\s+`mail_account_retire_delete_v1`[\s\S]*`mail_account_lifecycle_v1` реализованы/,
  );
  assert.match(adr, /Communications не хранит Mail settings, credentials/);
  assert.match(adr, /Runtime не становится assembly, integration не становится domain/);

  assert.match(
    proto,
    /service MailAccountCredentialBindingService[\s\S]*rpc Bind/,
  );
  assert.match(proto, /service MailAccountQueryService[\s\S]*rpc Get/);
  assert.match(proto, /MailConnectorProfileV1 connector_profile = 5/);
  assert.match(proto, /MailProviderPathReadinessV1 sync_readiness = 6/);
  assert.match(proto, /MailProviderPathReadinessV1 delivery_readiness = 7/);
  const bindMessage = proto.match(
    /message MailBindCredentialRequestV1 \{[\s\S]*?\n\}/,
  )?.[0];
  assert.ok(bindMessage, 'missing typed Mail credential bind request');
  assert.match(bindMessage, /string connection_id = 1/);
  assert.match(bindMessage, /MailCredentialPurposeV1 purpose = 2/);
  assert.match(bindMessage, /uint64 expected_binding_revision = 3/);
  assert.match(bindMessage, /uint64 credential_revision = 4/);
  assert.doesNotMatch(bindMessage, /secret|record|location|token|bytes|payload/i);
  assert.match(account, /bindable_by_client[\s\S]*ImapPassword \| Self::SmtpPassword/);
  assert.match(account, /status\.bindings\.len\(\) > 4/);
  assert.match(contract, /mail\.account\.credential\.bind\.v1/);
  assert.match(contract, /mail\.account\.query\.v1/);
  assert.match(admission, /MailClientContractV1::AccountCredentialBind/);
  assert.match(admission, /MailClientContractV1::AccountQuery/);
  assert.match(admission, /mail\.imap\.credential-provisioning\.v1/);
  assert.match(admission, /mail\.smtp\.credential-provisioning\.v1/);
  assert.match(
    admission,
    /\&\[VaultActionV1::Create, VaultActionV1::ReplaceCas\]/,
  );

  assert.match(portability, /MAIL_SETTINGS_SCHEMA_MAJOR_V2: u32 = 2/);
  assert.match(
    settings,
    /pub use makosh_mail_api::\{MAIL_SETTINGS_SCHEMA_MAJOR_V2, MAIL_SETTINGS_SCHEMA_REVISION_V2\}/,
  );
  assert.match(
    settings,
    /client_visibility: SettingClientVisibilityV1::Editable/,
  );
  assert.doesNotMatch(settings, /password_revision|credential_revisions|secret_ref/);
  assert.match(persistence, /mail_account_credential_bindings/);
  assert.match(
    persistence,
    /expected_binding_revision == 0[\s\S]*ON CONFLICT \(connection_id, purpose\) DO NOTHING/,
  );
  assert.match(
    persistence,
    /binding_revision = binding_revision \+ 1[\s\S]*AND binding_revision = \$6/,
  );
  assert.match(schema, /MAIL_STORAGE_BUNDLE_REVISION_V7/);

  assert.match(runtime, /activate_bound_account_credential\(/);
  assert.match(runtime, /signal_ready\(/);
  assert.match(
    runtime,
    /bind_account_credential[\s\S]*MailCredentialPurposeV1::ImapPassword => self\.imap_password = None/,
  );
  assert.match(
    runtime,
    /MailCredentialPurposeV1::SmtpPassword => self\.smtp_password = None/,
  );
  assert.match(
    runtime,
    /MailAccountReadinessV1::ConfigurationOnly[\s\S]*MailAccountReadinessV1::Degraded/,
  );
  assert.match(runtime, /MailConnectorProfileV1::ImapSmtp/);
  assert.match(runtime, /MailProviderPathReadinessV1::CredentialRequired/);
  assert.match(assembly, /mail_settings_schema_v2/);
  assert.match(assembly, /mail_runtime_storage_bundle_v1/);

  assert.match(
    live,
    /managed_mail_credential_rotation_quiesces_until_settings_successor/,
  );
  assert.match(live, /MailAccountReadinessV1::PendingRestart/);
  assert.match(live, /apply_managed_integration_settings/);
  assert.match(live, /credential_revision == Some\(2\)/);
  assert.match(live, /stale Mail generation must not retain its query route/);

  assert.doesNotMatch(
    `${proto}\n${account}\n${persistence}\n${settings}`,
    /makosh_(?:communications|telegram|whatsapp|zulip)/,
  );
});
