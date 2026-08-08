import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

const paths = {
  adr: new URL(
    'docs/adr/ADR-0312-mail-permanent-delete-confirmation-and-provider-authority.md',
    PROJECT_ROOT,
  ),
  inventory: new URL(
    'architecture/communications-settings-reconstruction.json',
    BACKEND_ROOT,
  ),
  proto: new URL(
    'src/mail-api/proto/makosh/mail/message_permanent_delete/v1/client.proto',
    BACKEND_ROOT,
  ),
  oauthProto: new URL(
    'src/mail-api/proto/makosh/mail/v1/client.proto',
    BACKEND_ROOT,
  ),
  contract: new URL('src/mail-api/src/client_contract.rs', BACKEND_ROOT),
  persistence: new URL(
    'src/mail-persistence/src/message_permanent_delete.rs',
    BACKEND_ROOT,
  ),
  schema: new URL('src/mail-persistence/src/schema.rs', BACKEND_ROOT),
  imap: new URL('src/mail-imap/src/lib.rs', BACKEND_ROOT),
  gmail: new URL('src/mail-gmail/src/lib.rs', BACKEND_ROOT),
  runtime: new URL('src/mail-runtime/src/managed.rs', BACKEND_ROOT),
  admission: new URL('src/mail-runtime/src/admission.rs', BACKEND_ROOT),
  managedSetup: new URL(
    'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/mail_managed_setup.rs',
    BACKEND_ROOT,
  ),
  managedFlow: new URL(
    'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/mail_operational_flow.rs',
    BACKEND_ROOT,
  ),
  managedDeleteTest: new URL(
    'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/mail_message_permanent_delete_flow.rs',
    BACKEND_ROOT,
  ),
  managedOAuthTest: new URL(
    'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/mail_gmail_oauth_flow.rs',
    BACKEND_ROOT,
  ),
  fixture: new URL(
    'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/mail_imap_fixture.rs',
    BACKEND_ROOT,
  ),
  generated: new URL(
    'frontend/src/gen/makosh/mail/message_permanent_delete/v1/client_pb.ts',
    PROJECT_ROOT,
  ),
  commandClient: new URL(
    'frontend/src/integrations/mail/api/mailMessagePermanentDeleteCommandClient.ts',
    PROJECT_ROOT,
  ),
  queryClient: new URL(
    'frontend/src/integrations/mail/api/mailMessagePermanentDeleteQueryClient.ts',
    PROJECT_ROOT,
  ),
  gateway: new URL(
    'frontend/src/integrations/mail/api/mailMessagePermanentDeleteGateway.ts',
    PROJECT_ROOT,
  ),
  controller: new URL(
    'frontend/src/integrations/mail/queries/useMailMessagePermanentDelete.ts',
    PROJECT_ROOT,
  ),
  component: new URL(
    'frontend/src/integrations/mail/presentation/MailMessagePermanentDeleteActions.vue',
    PROJECT_ROOT,
  ),
  authorization: new URL(
    'frontend/src/integrations/mail/setup/useMailGmailPermanentDeleteAuthorization.ts',
    PROJECT_ROOT,
  ),
};

test('Mail permanent delete is explicit, provider-owned and live-conformant', async () => {
  const [
    adr,
    inventorySource,
    proto,
    oauthProto,
    contract,
    persistence,
    schema,
    imap,
    gmail,
    runtime,
    admission,
    managedSetup,
    managedFlow,
    managedDeleteTest,
    managedOAuthTest,
    fixture,
    generated,
    commandClient,
    queryClient,
    gateway,
    controller,
    component,
    authorization,
  ] = await Promise.all(
    Object.values(paths).map((path) => readFile(path, 'utf8')),
  );
  const inventory = JSON.parse(inventorySource);
  const permanentDelete = inventory.slices.find(
    ({ gate }) => gate === 'mail_message_permanent_delete_command_v1',
  );
  const operationalCommand = inventory.slices.find(
    ({ gate }) => gate === 'mail_operational_command_v1',
  );

  assert.deepEqual(permanentDelete, {
    gate: 'mail_message_permanent_delete_command_v1',
    role: 'integration',
    owner: 'mail',
    state: 'implemented',
    dependsOn: ['mail_provider_location_identity_v1'],
  });
  assert.equal(operationalCommand.state, 'implemented');
  assert.match(adr, /Состояние реализации: implemented/);
  assert.match(adr, /Gate `mail_message_permanent_delete_command_v1`/);
  assert.match(adr, /UID EXPUNGE/);
  assert.match(adr, /Communications canonical evidence[\s\S]*не удаляются/);

  assert.match(proto, /PERMANENT_DELETE_CONFIRMATION_CONFIRMED/);
  assert.match(proto, /expected_projection_revision/);
  assert.match(proto, /REAUTHORIZATION_REQUIRED/);
  assert.doesNotMatch(proto, /provider_message_id|uid_validity|mailbox_id|gmail_label/i);
  assert.match(
    oauthProto,
    /enum GmailOAuthAuthorityV1[\s\S]*OPERATIONAL[\s\S]*PERMANENT_DELETE/,
  );
  assert.match(contract, /"mail\.message-permanent-delete\.command\.v1"/);
  assert.match(contract, /"mail\.message-permanent-delete\.query\.v1"/);
  assert.match(admission, /MailClientContractV1::MessagePermanentDeleteCommand/);
  assert.match(admission, /MailClientContractV1::MessagePermanentDeleteQuery/);

  assert.match(
    persistence,
    /CREATE TABLE IF NOT EXISTS makosh_data\.mail_message_permanent_delete_operations/,
  );
  assert.match(persistence, /exact_command_bytes BYTEA NOT NULL/);
  assert.match(persistence, /request_sha256 BYTEA NOT NULL/);
  assert.match(persistence, /MAIL_FOLDER_KIND_TRASH_DB_VALUE/);
  assert.match(persistence, /complete_message_permanent_delete_success/);
  assert.match(persistence, /DELETE FROM makosh_data\.mail_operational_messages/);
  assert.doesNotMatch(persistence, /makosh_communications|communications_/);
  assert.match(schema, /MAIL_STORAGE_BUNDLE_REVISION_V17: u32 = 17/);
  assert.match(schema, /MAIL_SCHEMA_V17/);

  assert.match(imap, /has_str\("UIDPLUS"\)/);
  assert.match(imap, /UID STORE \{\} \+FLAGS\.SILENT \(\\\\Deleted\)/);
  assert.match(imap, /UID EXPUNGE \{\}/);
  assert.doesNotMatch(imap, /format!\("EXPUNGE/);
  assert.match(gmail, /GMAIL_PERMANENT_DELETE_OAUTH_SCOPES/);
  assert.match(gmail, /"https:\/\/mail\.google\.com\/"/);
  assert.match(gmail, /permanently_delete_message/);
  assert.match(gmail, /"DELETE"/);
  assert.match(gmail, /204 \| 404 => Ok\(\(\)\)/);

  assert.match(runtime, /submit_message_permanent_delete_command/);
  assert.match(runtime, /execute_next_message_permanent_delete_command/);
  assert.match(runtime, /permanent_delete_authorized/);
  assert.match(runtime, /ReauthorizationRequired/);
  assert.match(runtime, /MailMessagePermanentDeletePersistenceErrorV1::Database/);
  assert.doesNotMatch(runtime, /makosh_communications_runtime|communications-runtime/);
  assert.match(managedSetup, /mail_runtime_storage_bundle_v1/);
  assert.match(managedSetup, /MailClientContractV1::MessagePermanentDeleteCommand/);
  assert.match(managedSetup, /MailClientContractV1::MessagePermanentDeleteQuery/);
  assert.match(
    managedDeleteTest,
    /managed_mail_message_permanent_delete_is_fenced_exact_and_replay_safe/,
  );
  assert.match(managedFlow, /MailMessagePermanentDeleteOperationOutcomeV1::Unsupported/);
  assert.match(managedFlow, /message_permanent_deletions/);
  assert.match(fixture, /UID EXPUNGE/);
  assert.match(fixture, /delete_marked/);
  assert.match(
    managedOAuthTest,
    /GmailOAuthAuthorityV1::PermanentDelete[\s\S]*GmailOAuthOutcomeV1::Rejected/,
  );

  for (const source of [
    generated,
    commandClient,
    queryClient,
    gateway,
    controller,
    component,
    authorization,
  ]) {
    assert.doesNotMatch(source, /provider_message_id|uid_validity|UID EXPUNGE|mailbox_id/i);
  }
  assert.match(commandClient, /MailMessagePermanentDeleteCommandService/);
  assert.match(queryClient, /MailMessagePermanentDeleteQueryService/);
  assert.match(gateway, /getMailMessagePermanentDeleteCommandConnectClient/);
  assert.match(gateway, /getMailMessagePermanentDeleteQueryConnectClient/);
  assert.match(controller, /expectedProjectionRevision/);
  assert.match(controller, /confirmed: true/);
  assert.match(component, /type="checkbox"/);
  assert.match(component, /I understand this cannot be undone at the provider/);
  assert.match(
    authorization,
    /client\.start\([\s\S]*operationId\.value,[\s\S]*requiredConnectionId\(connectionId\.value\),[\s\S]*'permanent-delete'/,
  );
});
