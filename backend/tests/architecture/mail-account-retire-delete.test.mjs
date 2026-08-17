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
    'src/mail-api/proto/makosh/mail/account_lifecycle/v1/client.proto',
    BACKEND_ROOT,
  ),
  api: new URL('src/mail-api/src/account_lifecycle.rs', BACKEND_ROOT),
  wire: new URL('src/mail-api/src/account_lifecycle_wire.rs', BACKEND_ROOT),
  contract: new URL('src/mail-api/src/client_contract.rs', BACKEND_ROOT),
  persistence: new URL('src/mail-persistence/src/lifecycle.rs', BACKEND_ROOT),
  schema: new URL('src/mail-persistence/src/schema.rs', BACKEND_ROOT),
  coordinator: new URL('src/mail-runtime/src/account_lifecycle.rs', BACKEND_ROOT),
  runtime: new URL('src/mail-runtime/src/managed.rs', BACKEND_ROOT),
  gmail: new URL('src/mail-runtime/src/gmail_oauth.rs', BACKEND_ROOT),
  clientPort: new URL('src/mail-runtime/src/client_port.rs', BACKEND_ROOT),
  admission: new URL('src/mail-runtime/src/admission.rs', BACKEND_ROOT),
  live: new URL(
    'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/mail_account_credential_flow.rs',
    BACKEND_ROOT,
  ),
};

test('Mail retire and delete are exact owner-local lifecycle operations', async () => {
  const [
    adr,
    inventorySource,
    proto,
    api,
    wire,
    contract,
    persistence,
    schema,
    coordinator,
    runtime,
    gmail,
    clientPort,
    admission,
    live,
  ] = await Promise.all([
    readFile(paths.adr, 'utf8'),
    readFile(paths.inventory, 'utf8'),
    readFile(paths.proto, 'utf8'),
    readFile(paths.api, 'utf8'),
    readFile(paths.wire, 'utf8'),
    readFile(paths.contract, 'utf8'),
    readFile(paths.persistence, 'utf8'),
    readFile(paths.schema, 'utf8'),
    readFile(paths.coordinator, 'utf8'),
    readFile(paths.runtime, 'utf8'),
    readFile(paths.gmail, 'utf8'),
    readFile(paths.clientPort, 'utf8'),
    readFile(paths.admission, 'utf8'),
    readFile(paths.live, 'utf8'),
  ]);
  const inventory = JSON.parse(inventorySource);
  const slice = inventory.slices.find(
    ({ gate }) => gate === 'mail_account_retire_delete_v1',
  );

  assert.deepEqual(slice, {
    gate: 'mail_account_retire_delete_v1',
    role: 'integration',
    owner: 'mail',
    state: 'implemented',
    dependsOn: [
      'mail_account_credential_binding_v1',
      'vault_credential_retirement_v1',
    ],
  });
  assert.match(
    adr,
    /Phase 1 `mail_account_credential_binding_v1`, Phase 2\s+`mail_account_retire_delete_v1`[\s\S]*`mail_account_lifecycle_v1` реализованы/,
  );
  assert.match(adr, /Runtime не становится assembly, integration не становится domain/);

  assert.match(proto, /service MailAccountRetireService[\s\S]*rpc Retire/);
  assert.match(proto, /service MailAccountDeleteService[\s\S]*rpc Delete/);
  assert.match(proto, /service MailAccountLifecycleRetryService[\s\S]*rpc Retry/);
  assert.match(proto, /service MailAccountLifecycleStatusService[\s\S]*rpc Get/);
  const command = proto.match(
    /message MailAccountLifecycleCommandV1 \{[\s\S]*?\n\}/,
  )?.[0];
  assert.ok(command, 'missing typed Mail lifecycle command');
  assert.match(command, /string operation_id = 1/);
  assert.match(command, /string connection_id = 2/);
  assert.match(command, /uint64 expected_lifecycle_revision = 3/);
  assert.doesNotMatch(
    command,
    /password|token|secret|record|bytes|payload|purpose|credential_revision/i,
  );
  assert.match(proto, /MailLifecycleCredentialPurposeV1 purpose = 1/);
  assert.match(proto, /optional uint64 binding_revision = 3/);
  assert.match(proto, /uint64 credential_revision = 4/);
  assert.match(api, /credentials\.len\(\) > 4/);
  assert.match(
    api,
    /state == MailCredentialLifecycleStateV1::OutcomeUnknown[\s\S]*return MailAccountLifecycleStateV1::OutcomeUnknown;[\s\S]*state == MailCredentialLifecycleStateV1::Rejected/,
  );
  assert.match(wire, /validate_lifecycle_receipt/);

  for (const route of [
    'mail.account.retire.v1',
    'mail.account.delete.v1',
    'mail.account.lifecycle.retry.v1',
    'mail.account.lifecycle.query.v1',
  ]) {
    assert.match(contract, new RegExp(route.replaceAll('.', '\\.')));
  }
  assert.match(contract, /MAIL_CLIENT_CONTRACT_REVISION: u32 = 15/);
  assert.match(clientPort, /MailClientContractV1::AccountRetire/);
  assert.match(clientPort, /MailClientContractV1::AccountDelete/);
  assert.match(clientPort, /MailClientContractV1::AccountLifecycleRetry/);
  assert.match(clientPort, /MailClientContractV1::AccountLifecycleQuery/);

  assert.match(persistence, /mail_account_lifecycle_operations/);
  assert.match(persistence, /mail_account_lifecycle_credentials/);
  assert.match(persistence, /mail_account_tombstones/);
  assert.match(
    persistence,
    /current_revision != command\.expected_lifecycle_revision/,
  );
  assert.match(persistence, /state IN \(1, 4\)/);
  assert.match(persistence, /mail_gmail_oauth_credential_bindings/);
  assert.match(persistence, /if tombstoned[\s\S]*InvalidRow/);
  assert.match(schema, /MAIL_STORAGE_BUNDLE_REVISION_V8/);
  assert.match(schema, /migration_id: "mail_account_lifecycle"/);

  assert.match(
    coordinator,
    /self\.provider_io_quiesced = true;[\s\S]*begin_account_lifecycle/,
  );
  assert.match(coordinator, /MailCredentialPurpose::ImapPassword/);
  assert.match(coordinator, /MailCredentialPurpose::SmtpPassword/);
  assert.match(coordinator, /MailCredentialPurpose::GmailAccessToken/);
  assert.match(coordinator, /MailCredentialPurpose::GmailRefreshCredential/);
  assert.match(
    coordinator,
    /GmailRefreshCredential,[\s\S]*SecretClassV1::OAuthRefreshCredential/,
  );
  assert.match(coordinator, /retire_once/);
  assert.match(coordinator, /delete_once/);
  assert.match(
    coordinator,
    /Rejected[\s\S]*Unavailable[\s\S]*MailCredentialLifecycleStateV1::OutcomeUnknown/,
  );
  assert.match(
    runtime,
    /self\.imap_password = None;[\s\S]*self\.smtp_password = None;[\s\S]*self\.gmail_oauth_operation_in_flight = None;[\s\S]*\.begin\(/,
  );
  assert.match(runtime, /latest_account_lifecycle/);
  assert.match(runtime, /lifecycle_account_readiness/);
  assert.match(gmail, /if !self\.provider_io_permitted\(\)/);

  for (const capability of [
    'MAIL_GMAIL_CREDENTIAL_LIFECYCLE_CAPABILITY_ID',
    'MAIL_GMAIL_REFRESH_CREDENTIAL_LIFECYCLE_CAPABILITY_ID',
    'MAIL_IMAP_CREDENTIAL_LIFECYCLE_CAPABILITY_ID',
    'MAIL_SMTP_CREDENTIAL_LIFECYCLE_CAPABILITY_ID',
  ]) {
    assert.match(admission, new RegExp(capability));
  }
  assert.match(
    admission,
    /&\[VaultActionV1::Retire, VaultActionV1::Delete\]/,
  );

  assert.match(live, /MailAccountLifecycleStateV1::OutcomeUnknown/);
  assert.match(live, /assert_eq!\(replayed_retire, retire_unknown\)/);
  assert.match(live, /stop Storage before rebinding the successor Vault generation/);
  assert.match(live, /MailClientContractV1::AccountLifecycleRetry/);
  assert.match(live, /MailClientContractV1::AccountLifecycleQuery/);
  assert.match(live, /assert_lifecycle_completed\(&delete/);
  assert.match(live, /account_is_tombstoned/);
  assert.match(live, /mail-account-delete-after-tombstone/);
  assert.match(live, /imap\.accepted_connections\(\), 1/);
  assert.match(live, /smtp\.accepted_messages\(\), 1/);

  const ownerBoundary = `${proto}\n${api}\n${wire}\n${persistence}\n${coordinator}`;
  assert.doesNotMatch(
    ownerBoundary,
    /makosh_(?:communications|telegram|whatsapp|zulip)/,
  );
});
