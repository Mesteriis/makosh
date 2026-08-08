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
  operationalProto: new URL(
    'src/mail-api/proto/makosh/mail/operational/v1/client.proto',
    BACKEND_ROOT,
  ),
  flagsProto: new URL(
    'src/mail-api/proto/makosh/mail/message_flags/v1/client.proto',
    BACKEND_ROOT,
  ),
  operationalApi: new URL('src/mail-api/src/operational.rs', BACKEND_ROOT),
  flagsApi: new URL('src/mail-api/src/message_flags.rs', BACKEND_ROOT),
  locator: new URL('src/mail-persistence/src/provider_location.rs', BACKEND_ROOT),
  operationalPersistence: new URL(
    'src/mail-persistence/src/operational.rs',
    BACKEND_ROOT,
  ),
  schema: new URL('src/mail-persistence/src/schema.rs', BACKEND_ROOT),
  imap: new URL('src/mail-imap/src/lib.rs', BACKEND_ROOT),
  runtime: new URL('src/mail-runtime/src/managed.rs', BACKEND_ROOT),
  managedSetup: new URL(
    'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/mail_managed_setup.rs',
    BACKEND_ROOT,
  ),
  managedFlow: new URL(
    'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/mail_operational_flow.rs',
    BACKEND_ROOT,
  ),
  imapFixture: new URL(
    'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/mail_imap_fixture.rs',
    BACKEND_ROOT,
  ),
  operationalGenerated: new URL(
    'frontend/src/gen/makosh/mail/operational/v1/client_pb.ts',
    PROJECT_ROOT,
  ),
  flagsGenerated: new URL(
    'frontend/src/gen/makosh/mail/message_flags/v1/client_pb.ts',
    PROJECT_ROOT,
  ),
  operationalGateway: new URL(
    'frontend/src/integrations/mail/api/mailOperationalReadGateway.ts',
    PROJECT_ROOT,
  ),
  flagsGateway: new URL(
    'frontend/src/integrations/mail/api/mailMessageFlagsGateway.ts',
    PROJECT_ROOT,
  ),
};

test('Mail provider location identity is stable, private and live-conformant', async () => {
  const [
    adr,
    inventorySource,
    operationalProto,
    flagsProto,
    operationalApi,
    flagsApi,
    locator,
    operationalPersistence,
    schema,
    imap,
    runtime,
    managedSetup,
    managedFlow,
    imapFixture,
    operationalGenerated,
    flagsGenerated,
    operationalGateway,
    flagsGateway,
  ] = await Promise.all(
    Object.values(paths).map((path) => readFile(path, 'utf8')),
  );
  const inventory = JSON.parse(inventorySource);
  const identity = inventory.slices.find(
    ({ gate }) => gate === 'mail_provider_location_identity_v1',
  );
  const location = inventory.slices.find(
    ({ gate }) => gate === 'mail_message_location_command_v1',
  );

  assert.deepEqual(identity, {
    gate: 'mail_provider_location_identity_v1',
    role: 'integration',
    owner: 'mail',
    state: 'implemented',
    dependsOn: ['mail_operational_read_v1', 'mail_message_flags_command_v1'],
  });
  assert.equal(location.state, 'implemented');
  assert.match(adr, /реализовано полностью/);
  assert.match(adr, /V13\/V14/);
  assert.match(adr, /stale-UIDVALIDITY negative conformance/);
  assert.match(adr, /Gate `mail_message_location_command_v1`/);

  for (const source of [
    operationalProto,
    flagsProto,
    operationalApi,
    flagsApi,
    operationalGenerated,
    flagsGenerated,
    operationalGateway,
    flagsGateway,
  ]) {
    assert.doesNotMatch(source, /\bprovider_?message_?id\b/i);
  }
  assert.match(operationalProto, /string message_id = 2/);
  assert.match(flagsProto, /string message_id = 3/);

  assert.match(locator, /CREATE TABLE IF NOT EXISTS makosh_data\.mail_imap_message_locators/);
  assert.match(locator, /UNIQUE \(connection_id, mailbox_id, uid_validity, uid\)/);
  assert.match(locator, /pub fn initial_imap_message_id/);
  assert.match(locator, /format!\("imap:v1:\{\}"/);
  assert.match(locator, /pub async fn resolve_imap_message_id/);
  assert.match(locator, /pub async fn imap_message_locator/);
  assert.match(locator, /ON CONFLICT \(connection_id, message_id\) DO UPDATE/);
  assert.match(schema, /MAIL_STORAGE_BUNDLE_REVISION_V14: u32 = 14/);
  assert.match(schema, /mail_stable_message_identity_and_imap_locator/);
  assert.match(schema, /mail_stable_message_identity_indexes/);
  assert.match(operationalPersistence, /upsert_imap_message_locator/);
  assert.match(operationalPersistence, /transaction\s*\.commit\(\)/);

  assert.match(imap, /session\s*\.list\(None, Some\("\*"\)\)/);
  assert.match(imap, /MAX_DISCOVERED_MAILBOXES/);
  assert.match(imap, /uid_validity/);
  assert.match(imap, /imap mailbox UIDVALIDITY does not match the stored locator/);
  assert.match(runtime, /resolve_imap_message_id/);
  assert.match(runtime, /imap_message_locator/);
  assert.doesNotMatch(runtime, /split_once\(['"]:/);
  assert.doesNotMatch(runtime, /select\(["']INBOX["']\)/i);

  assert.match(managedSetup, /mail_runtime_storage_bundle_v1/);
  assert.match(managedFlow, /assert_opaque_imap_message_id/);
  assert.match(managedFlow, /assert_mail_identity_survives_restart/);
  assert.match(managedFlow, /MailMessageFlagOperationOutcomeV1::Rejected/);
  assert.match(managedFlow, /stale UIDVALIDITY must be rejected before UID STORE/);
  assert.match(imapFixture, /\\Archive/);
  assert.match(imapFixture, /\\Trash/);
  assert.match(imapFixture, /set_uid_validity/);
});
