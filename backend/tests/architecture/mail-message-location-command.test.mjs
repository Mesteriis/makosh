import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

const paths = {
  adr: new URL(
    'docs/adr/ADR-0308-mail-message-identity-imap-mailbox-roles-and-location-authority.md',
    PROJECT_ROOT,
  ),
  inventory: new URL(
    'architecture/communications-settings-reconstruction.json',
    BACKEND_ROOT,
  ),
  proto: new URL(
    'src/mail-api/proto/makosh/mail/message_location/v1/client.proto',
    BACKEND_ROOT,
  ),
  contract: new URL('src/mail-api/src/client_contract.rs', BACKEND_ROOT),
  persistence: new URL(
    'src/mail-persistence/src/message_location.rs',
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
  managedTest: new URL(
    'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/mail_message_location_flow.rs',
    BACKEND_ROOT,
  ),
  fixture: new URL(
    'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/mail_imap_fixture.rs',
    BACKEND_ROOT,
  ),
  generated: new URL(
    'frontend/src/gen/makosh/mail/message_location/v1/client_pb.ts',
    PROJECT_ROOT,
  ),
  commandClient: new URL(
    'frontend/src/integrations/mail/api/mailMessageLocationCommandClient.ts',
    PROJECT_ROOT,
  ),
  queryClient: new URL(
    'frontend/src/integrations/mail/api/mailMessageLocationQueryClient.ts',
    PROJECT_ROOT,
  ),
  gateway: new URL(
    'frontend/src/integrations/mail/api/mailMessageLocationGateway.ts',
    PROJECT_ROOT,
  ),
  controller: new URL(
    'frontend/src/integrations/mail/queries/useMailMessageLocation.ts',
    PROJECT_ROOT,
  ),
  component: new URL(
    'frontend/src/integrations/mail/presentation/MailMessageLocationActions.vue',
    PROJECT_ROOT,
  ),
};

test('Mail message location commands are reversible, provider-owned and live-conformant', async () => {
  const [
    adr,
    inventorySource,
    proto,
    contract,
    persistence,
    schema,
    imap,
    gmail,
    runtime,
    admission,
    managedSetup,
    managedFlow,
    managedTest,
    fixture,
    generated,
    commandClient,
    queryClient,
    gateway,
    controller,
    component,
  ] = await Promise.all(
    Object.values(paths).map((path) => readFile(path, 'utf8')),
  );
  const inventory = JSON.parse(inventorySource);
  const location = inventory.slices.find(
    ({ gate }) => gate === 'mail_message_location_command_v1',
  );
  const permanentDelete = inventory.slices.find(
    ({ gate }) => gate === 'mail_message_permanent_delete_command_v1',
  );

  assert.deepEqual(location, {
    gate: 'mail_message_location_command_v1',
    role: 'integration',
    owner: 'mail',
    state: 'implemented',
    dependsOn: ['mail_provider_location_identity_v1'],
  });
  assert.equal(permanentDelete.state, 'implemented');
  assert.match(adr, /Gate `mail_message_location_command_v1`/);
  assert.match(adr, /Mail storage bundle V15/);
  assert.match(adr, /Permanent delete остаётся отдельным gate/);

  for (const kind of ['ARCHIVE', 'TRASH', 'RESTORE', 'MOVE']) {
    assert.match(proto, new RegExp(`MAIL_MESSAGE_LOCATION_KIND_${kind}`));
  }
  assert.doesNotMatch(proto, /PERMANENT_DELETE|EXPUNGE/);
  assert.match(contract, /"mail\.message-location\.command\.v1"/);
  assert.match(contract, /"mail\.message-location\.query\.v1"/);
  assert.match(admission, /MailClientContractV1::MessageLocationCommand/);
  assert.match(admission, /MailClientContractV1::MessageLocationQuery/);

  assert.match(
    persistence,
    /CREATE TABLE IF NOT EXISTS makosh_data\.mail_message_location_operations/,
  );
  assert.match(persistence, /exact_command_bytes BYTEA NOT NULL/);
  assert.match(persistence, /request_sha256 BYTEA NOT NULL/);
  assert.match(persistence, /complete_message_location_success/);
  assert.match(persistence, /transaction\s*\.commit\(\)/);
  assert.doesNotMatch(persistence, /permanent[_ -]?delete|EXPUNGE/i);
  assert.match(schema, /MAIL_STORAGE_BUNDLE_REVISION_V15: u32 = 15/);
  assert.match(schema, /mail_message_location_operations/);

  assert.match(imap, /has_str\("MOVE"\)/);
  assert.match(imap, /has_str\("UIDPLUS"\)/);
  assert.match(imap, /ResponseCode::CopyUid/);
  assert.match(imap, /UIDVALIDITY does not match the stored locator/);
  assert.doesNotMatch(imap, /UID COPY[\s\S]*UID STORE.*\\Deleted/);
  assert.match(gmail, /post_message_action[\s\S]*"trash"/);
  assert.match(gmail, /post_message_action[\s\S]*"untrash"/);
  assert.match(gmail, /fetch_message_location/);
  assert.match(gmail, /remove_label_ids/);

  assert.match(runtime, /submit_message_location_command/);
  assert.match(runtime, /execute_next_message_location_command/);
  assert.match(runtime, /makosh_mail_imap::move_message/);
  assert.match(runtime, /client\.restore_message/);
  assert.doesNotMatch(runtime, /makosh_communications_runtime|communications-runtime/);
  assert.match(managedSetup, /mail_runtime_storage_bundle_v1/);
  assert.match(managedSetup, /MailClientContractV1::MessageLocationCommand/);
  assert.match(managedSetup, /MailClientContractV1::MessageLocationQuery/);
  assert.match(managedFlow, /assert_mail_message_archive/);
  assert.match(
    managedFlow,
    /assert_mail_message_location_survives_restart_and_fails_closed/,
  );
  assert.match(managedFlow, /message_location_mutations/);
  assert.match(managedFlow, /MailMessageLocationOperationOutcomeV1::Unsupported/);
  assert.match(managedTest, /managed_mail_message_location_is_exact_replay_safe_and_restart_safe/);
  assert.match(fixture, /UID MOVE/);
  assert.match(fixture, /COPYUID/);
  assert.match(fixture, /set_move_supported/);

  for (const source of [
    generated,
    commandClient,
    queryClient,
    gateway,
    controller,
    component,
  ]) {
    assert.doesNotMatch(source, /provider_message_id|uid_validity|COPYUID/i);
  }
  assert.match(commandClient, /MailMessageLocationCommandService/);
  assert.match(queryClient, /MailMessageLocationQueryService/);
  assert.match(gateway, /getMailMessageLocationCommandConnectClient/);
  assert.match(gateway, /getMailMessageLocationQueryConnectClient/);
  assert.match(controller, /archive/);
  assert.match(controller, /trash/);
  assert.match(controller, /restore/);
  assert.match(controller, /move/);
  assert.doesNotMatch(component, /permanent|delete|expunge/i);
});
