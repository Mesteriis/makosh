import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

const paths = {
  adr: new URL(
    'docs/adr/ADR-0305-mail-owned-composition-drafts-templates-and-signatures.md',
    PROJECT_ROOT,
  ),
  inventory: new URL(
    'architecture/communications-settings-reconstruction.json',
    BACKEND_ROOT,
  ),
  proto: new URL(
    'src/mail-api/proto/makosh/mail/composition/v1/client.proto',
    BACKEND_ROOT,
  ),
  validator: new URL('src/mail-api/src/composition.rs', BACKEND_ROOT),
  wire: new URL('src/mail-api/src/composition_wire.rs', BACKEND_ROOT),
  contract: new URL('src/mail-api/src/client_contract.rs', BACKEND_ROOT),
  persistence: new URL('src/mail-persistence/src/composition.rs', BACKEND_ROOT),
  schema: new URL('src/mail-persistence/src/schema.rs', BACKEND_ROOT),
  runtime: new URL('src/mail-runtime/src/managed.rs', BACKEND_ROOT),
  clientPort: new URL('src/mail-runtime/src/client_port.rs', BACKEND_ROOT),
  admission: new URL('src/mail-runtime/src/admission.rs', BACKEND_ROOT),
  managedSetup: new URL(
    'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/mail_managed_setup.rs',
    BACKEND_ROOT,
  ),
  managedFlow: new URL(
    'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/mail_composition_flow.rs',
    BACKEND_ROOT,
  ),
  build: new URL('src/mail-api/build.rs', BACKEND_ROOT),
  frontendGenerator: new URL('frontend/scripts/generate-proto.mjs', PROJECT_ROOT),
  frontendGenerated: new URL(
    'frontend/src/gen/makosh/mail/composition/v1/client_pb.ts',
    PROJECT_ROOT,
  ),
  frontendCommandClient: new URL(
    'frontend/src/integrations/mail/api/mailCompositionCommandClient.ts',
    PROJECT_ROOT,
  ),
  frontendQueryClient: new URL(
    'frontend/src/integrations/mail/api/mailCompositionQueryClient.ts',
    PROJECT_ROOT,
  ),
  frontendGateway: new URL(
    'frontend/src/integrations/mail/api/mailCompositionGateway.ts',
    PROJECT_ROOT,
  ),
  frontendConnections: new URL(
    'frontend/src/integrations/mail/queries/mailAccountConnections.ts',
    PROJECT_ROOT,
  ),
  frontendWorkspace: new URL(
    'frontend/src/integrations/mail/queries/useMailComposition.ts',
    PROJECT_ROOT,
  ),
  frontendDrafts: new URL(
    'frontend/src/integrations/mail/queries/useMailDrafts.ts',
    PROJECT_ROOT,
  ),
  frontendTemplates: new URL(
    'frontend/src/integrations/mail/queries/useMailTemplates.ts',
    PROJECT_ROOT,
  ),
  frontendSignatures: new URL(
    'frontend/src/integrations/mail/queries/useMailSignatures.ts',
    PROJECT_ROOT,
  ),
  frontendPanel: new URL(
    'frontend/src/integrations/mail/presentation/MailCompositionPanel.vue',
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

test('Mail composition is owner-local, independently admitted and cut over through generated clients', async () => {
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
    runtime,
    clientPort,
    admission,
    managedSetup,
    managedFlow,
    build,
    frontendGenerator,
    frontendGenerated,
    frontendCommandClient,
    frontendQueryClient,
    frontendGateway,
    frontendConnections,
    frontendWorkspace,
    frontendDrafts,
    frontendTemplates,
    frontendSignatures,
    frontendPanel,
    frontendRoute,
    frontendLayout,
  ] = sources;

  const inventory = JSON.parse(inventorySource);
  const slice = inventory.slices.find(({ gate }) => gate === 'mail_composition_v1');
  assert.deepEqual(slice, {
    gate: 'mail_composition_v1',
    role: 'integration',
    owner: 'mail',
    state: 'implemented',
    dependsOn: ['client_gateway_v1', 'mail.delivery.v1'],
  });

  assert.match(adr, /Состояние реализации: реализовано полностью/);
  assert.match(adr, /Mail является integration owner/);
  assert.match(adr, /mail\.composition\.command\.v1/);
  assert.match(adr, /mail\.composition\.query\.v1/);
  assert.match(adr, /Сохранение draft не отправляет сообщение/);
  assert.match(adr, /Runtime не является assembly/);
  assert.match(adr, /live managed Gateway conformance подтверждён/);

  assert.match(proto, /package makosh\.mail\.composition\.v1/);
  assert.match(proto, /service MailCompositionCommandService/);
  assert.match(proto, /service MailCompositionQueryService/);
  assert.match(
    proto,
    /oneof command[\s\S]*upsert_draft[\s\S]*delete_draft[\s\S]*upsert_template[\s\S]*delete_template[\s\S]*upsert_signature[\s\S]*delete_signature/,
  );
  assert.match(
    proto,
    /oneof query[\s\S]*list_drafts[\s\S]*get_draft[\s\S]*list_templates[\s\S]*preview_template[\s\S]*list_signatures/,
  );
  assert.doesNotMatch(proto, /\bmap\s*</);
  assert.doesNotMatch(proto, /\bgoogle\.protobuf\.(?:Any|Struct|Value)\b/);
  assert.doesNotMatch(proto, /\b(?:json|password|secret|token|cookie)\b/i);

  assert.match(validator, /validate_composition_command/);
  assert.match(validator, /validate_composition_query/);
  assert.match(validator, /render_mail_template_preview/);
  assert.match(validator, /MAX_COMPOSITION_PAGE_SIZE: u32 = 100/);
  assert.match(wire, /encode_composition_command/);
  assert.match(wire, /decode_composition_query_response/);
  assert.match(wire, /encode_composition_command\(&command\)\?\s*!= bytes/);
  assert.match(contract, /MAIL_CLIENT_CONTRACT_REVISION: u32 = 14/);
  assert.match(contract, /mail\.composition\.command\.v1/);
  assert.match(contract, /mail\.composition\.query\.v1/);

  assert.match(persistence, /CREATE TABLE IF NOT EXISTS makosh_data\.mail_composition_commands/);
  assert.match(persistence, /CREATE TABLE IF NOT EXISTS makosh_data\.mail_drafts/);
  assert.match(persistence, /CREATE TABLE IF NOT EXISTS makosh_data\.mail_templates/);
  assert.match(persistence, /CREATE TABLE IF NOT EXISTS makosh_data\.mail_signatures/);
  assert.match(persistence, /execute_composition_command/);
  assert.match(persistence, /expected_revision/);
  assert.match(persistence, /mail-composition-cursor-v1/);
  assert.doesNotMatch(persistence, /makosh_data\.communications|REFERENCES\s+makosh_data\./i);
  assert.match(schema, /MAIL_STORAGE_BUNDLE_REVISION_V12/);
  assert.match(schema, /migration_id: "mail_composition"/);

  assert.match(runtime, /execute_composition_command/);
  assert.match(runtime, /execute_composition_query/);
  assert.match(
    runtime,
    /composition_command_connection_id\(command\) != self\.account\.connection_id/,
  );
  assert.match(clientPort, /MailClientRequestV1::CompositionCommand/);
  assert.match(clientPort, /MailClientRequestV1::CompositionQuery/);
  assert.match(admission, /MailClientContractV1::CompositionCommand/);
  assert.match(admission, /MailClientContractV1::CompositionQuery/);
  assert.match(managedSetup, /mail_runtime_storage_bundle_v1/);
  assert.match(managedFlow, /assert_wrong_scope_cursor_is_rejected/);
  assert.match(managedFlow, /assert_cross_account_query_is_rejected/);
  assert.match(managedFlow, /assert_conflicting_operation_is_rejected/);
  assert.match(managedFlow, /assert_stale_revision_is_rejected/);
  assert.match(managedFlow, /assert_default_signature_switch_is_atomic/);
  assert.match(managedFlow, /assert_mail_composition_survives_restart/);
  assert.match(build, /proto\/makosh\/mail\/composition\/v1\/client\.proto/);

  assert.match(frontendGenerator, /mail', 'composition', 'v1', 'client\.proto'/);
  assert.match(frontendGenerated, /MailCompositionCommandService/);
  assert.match(frontendGenerated, /MailCompositionQueryService/);
  assert.match(frontendCommandClient, /MailCompositionCommandService/);
  assert.match(frontendQueryClient, /MailCompositionQueryService/);
  assert.match(frontendGateway, /MailCompositionCommandV1Schema/);
  assert.match(frontendGateway, /MailCompositionQueryV1Schema/);
  assert.match(frontendConnections, /mail\.account\.catalog\.query\.v1/);
  assert.match(frontendConnections, /deliveryReady/);
  assert.match(frontendWorkspace, /MailAccountConnection/);
  assert.match(frontendWorkspace, /useMailDrafts/);
  assert.match(frontendWorkspace, /useMailTemplates/);
  assert.match(frontendWorkspace, /useMailSignatures/);
  assert.match(frontendDrafts, /upsertMailDraft/);
  assert.match(frontendTemplates, /previewMailTemplate/);
  assert.match(frontendSignatures, /upsertMailSignature/);
  assert.match(frontendPanel, /Drafts, templates/);
  assert.match(frontendRoute, /useMailComposition/);
  assert.match(frontendLayout, /mail\.composition\.command\.v1/);
  assert.match(frontendLayout, /mail\.composition\.query\.v1/);

  const compositionOwnedSources = [
    proto,
    validator,
    wire,
    persistence,
    managedFlow,
    frontendGenerated,
    frontendCommandClient,
    frontendQueryClient,
    frontendGateway,
    frontendConnections,
    frontendWorkspace,
    frontendDrafts,
    frontendTemplates,
    frontendSignatures,
    frontendPanel,
  ];
  assert.doesNotMatch(
    compositionOwnedSources.join('\n'),
    /domains\/communications|integrations\/(?:telegram|whatsapp|zulip)|makosh_communications/,
  );
});
