import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

const paths = {
  adr: new URL(
    'docs/adr/ADR-0307-mail-message-flag-mutations-and-provider-reconciliation.md',
    PROJECT_ROOT,
  ),
  inventory: new URL(
    'architecture/communications-settings-reconstruction.json',
    BACKEND_ROOT,
  ),
  proto: new URL(
    'src/mail-api/proto/makosh/mail/message_flags/v1/client.proto',
    BACKEND_ROOT,
  ),
  validator: new URL('src/mail-api/src/message_flags.rs', BACKEND_ROOT),
  wire: new URL('src/mail-api/src/message_flags_wire.rs', BACKEND_ROOT),
  contract: new URL('src/mail-api/src/client_contract.rs', BACKEND_ROOT),
  persistence: new URL(
    'src/mail-persistence/src/message_flags.rs',
    BACKEND_ROOT,
  ),
  schema: new URL('src/mail-persistence/src/schema.rs', BACKEND_ROOT),
  gmail: new URL('src/mail-gmail/src/lib.rs', BACKEND_ROOT),
  imap: new URL('src/mail-imap/src/lib.rs', BACKEND_ROOT),
  runtime: new URL('src/mail-runtime/src/managed.rs', BACKEND_ROOT),
  clientPort: new URL('src/mail-runtime/src/client_port.rs', BACKEND_ROOT),
  admission: new URL('src/mail-runtime/src/admission.rs', BACKEND_ROOT),
  managedSetup: new URL(
    'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/mail_managed_setup.rs',
    BACKEND_ROOT,
  ),
  managedFlow: new URL(
    'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/mail_message_flag_flow.rs',
    BACKEND_ROOT,
  ),
  imapFixture: new URL(
    'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/mail_imap_fixture.rs',
    BACKEND_ROOT,
  ),
  generatedClient: new URL(
    'frontend/src/gen/makosh/mail/message_flags/v1/client_pb.ts',
    PROJECT_ROOT,
  ),
  frontendCommandClient: new URL(
    'frontend/src/integrations/mail/api/mailMessageFlagCommandClient.ts',
    PROJECT_ROOT,
  ),
  frontendQueryClient: new URL(
    'frontend/src/integrations/mail/api/mailMessageFlagQueryClient.ts',
    PROJECT_ROOT,
  ),
  frontendGateway: new URL(
    'frontend/src/integrations/mail/api/mailMessageFlagsGateway.ts',
    PROJECT_ROOT,
  ),
  frontendController: new URL(
    'frontend/src/integrations/mail/queries/useMailMessageFlags.ts',
    PROJECT_ROOT,
  ),
  frontendPresentation: new URL(
    'frontend/src/integrations/mail/presentation/MailMessageFlagActions.vue',
    PROJECT_ROOT,
  ),
  frontendRoute: new URL(
    'frontend/src/integrations/mail/views/MailOperationalRoute.vue',
    PROJECT_ROOT,
  ),
  frontendLayout: new URL(
    'frontend/src/app/layout/AppLayoutRoot.vue',
    PROJECT_ROOT,
  ),
};

test('Mail message flags stay provider-owned, durable, typed and separately admitted', async () => {
  const sources = await Promise.all(
    Object.values(paths).map((path) => readFile(path, 'utf8')),
  );
  const [
    adr,
    inventorySource,
    proto,
    validator,
    wire,
    contract,
    persistence,
    schema,
    gmail,
    imap,
    runtime,
    clientPort,
    admission,
    managedSetup,
    managedFlow,
    imapFixture,
    generatedClient,
    frontendCommandClient,
    frontendQueryClient,
    frontendGateway,
    frontendController,
    frontendPresentation,
    frontendRoute,
    frontendLayout,
  ] = sources;
  const inventory = JSON.parse(inventorySource);
  const flags = inventory.slices.find(
    ({ gate }) => gate === 'mail_message_flags_command_v1',
  );
  const location = inventory.slices.find(
    ({ gate }) => gate === 'mail_message_location_command_v1',
  );
  const closure = inventory.slices.find(
    ({ gate }) => gate === 'mail_operational_command_v1',
  );

  assert.deepEqual(flags, {
    gate: 'mail_message_flags_command_v1',
    role: 'integration',
    owner: 'mail',
    state: 'implemented',
    dependsOn: ['mail_operational_read_v1', 'mail_gmail_oauth_v1'],
  });
  assert.equal(location.state, 'implemented');
  assert.equal(closure.state, 'implemented');
  assert.deepEqual(closure.dependsOn, [
    'mail.delivery.v1',
    'mail_message_flags_command_v1',
    'mail_message_location_command_v1',
    'mail_message_permanent_delete_command_v1',
  ]);

  assert.match(adr, /реализовано полностью/);
  assert.match(adr, /mail_message_location_command_v1` также\s+реализован/);
  assert.match(adr, /provider-side-effect-free exact\s+replay/);
  assert.match(adr, /Communications не владеет provider flags/);

  assert.match(proto, /package makosh\.mail\.message_flags\.v1/);
  assert.match(proto, /enum MailMessageFlagKindV1[\s\S]*READ[\s\S]*STARRED/);
  assert.match(proto, /service MailMessageFlagCommandService/);
  assert.match(proto, /service MailMessageFlagQueryService/);
  assert.doesNotMatch(
    proto,
    /\b(?:password|secret|token|cookie|body|provider_payload|metadata)\b/i,
  );
  assert.doesNotMatch(proto, /\bmap\s*</);

  assert.match(validator, /validate_message_flag_command/);
  assert.match(validator, /validate_message_flag_status/);
  assert.match(wire, /encode_message_flag_command/);
  assert.match(wire, /decode_message_flag_status_response/);
  assert.match(wire, /encode_message_flag_command\(&command\)\? != bytes/);
  assert.match(contract, /MAIL_CLIENT_CONTRACT_REVISION: u32 = 14/);
  assert.match(contract, /mail\.message-flags\.command\.v1/);
  assert.match(contract, /mail\.message-flags\.query\.v1/);

  assert.match(
    persistence,
    /CREATE TABLE IF NOT EXISTS makosh_data\.mail_message_flag_operations/,
  );
  assert.match(persistence, /exact_command_bytes BYTEA/);
  assert.match(persistence, /request_sha256 BYTEA/);
  assert.match(persistence, /if changed \{/);
  assert.doesNotMatch(persistence, /REFERENCES makosh_data/);
  assert.match(schema, /MAIL_STORAGE_BUNDLE_REVISION_V12: u32 = 12/);

  assert.match(gmail, /gmail\.modify/);
  assert.match(gmail, /set_message_flag/);
  assert.match(gmail, /UNREAD/);
  assert.match(gmail, /STARRED/);
  assert.match(imap, /set_message_flag/);
  assert.match(imap, /UID STORE/);
  assert.match(imap, /\\Seen/);
  assert.match(imap, /\\Flagged/);

  assert.match(runtime, /submit_message_flag_command/);
  assert.match(runtime, /execute_next_message_flag_command/);
  assert.match(runtime, /complete_message_flag_success/);
  assert.match(runtime, /ProviderOutcomeUnknown/);
  assert.match(clientPort, /MailClientRequestV1::MessageFlagCommand/);
  assert.match(clientPort, /MailClientRequestV1::MessageFlagStatus/);
  assert.match(admission, /MailClientContractV1::MessageFlagCommand/);
  assert.match(admission, /MailClientContractV1::MessageFlagQuery/);
  assert.match(managedSetup, /mail_runtime_storage_bundle_v1/);
  assert.match(managedSetup, /MailClientContractV1::MessageFlagCommand/);
  assert.match(managedFlow, /managed_mail_message_flags_reconcile_provider_and_projection/);
  assert.match(managedFlow, /assert_mail_message_flags/);
  assert.match(imapFixture, /UID STORE completed/);

  assert.match(generatedClient, /export const MailMessageFlagCommandService/);
  assert.match(generatedClient, /export const MailMessageFlagQueryService/);
  assert.match(frontendCommandClient, /MailMessageFlagCommandService/);
  assert.match(frontendQueryClient, /MailMessageFlagQueryService/);
  assert.match(frontendGateway, /MailMessageFlagCommandV1Schema/);
  assert.match(frontendGateway, /MailMessageFlagStatusRequestV1Schema/);
  assert.match(frontendController, /Provider mutation is pending/);
  assert.match(frontendController, /await input\.refreshProjection\(\)/);
  assert.match(frontendPresentation, /Mark unread/);
  assert.match(frontendPresentation, /Add star/);
  assert.match(frontendRoute, /useMailMessageFlags/);
  assert.match(frontendLayout, /mail\.message-flags\.command\.v1/);
  assert.match(frontendLayout, /mail\.message-flags\.query\.v1/);

  assert.doesNotMatch(
    `${proto}\n${validator}\n${wire}\n${persistence}\n${gmail}\n${imap}`,
    /makosh_communications|domains\/communications|communications::/i,
  );
});
