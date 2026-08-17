import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

const paths = {
  adr: new URL(
    'docs/adr/ADR-0299-mail-sync-run-history-and-provider-path-health.md',
    PROJECT_ROOT,
  ),
  inventory: new URL(
    'architecture/communications-settings-reconstruction.json',
    BACKEND_ROOT,
  ),
  proto: new URL(
    'src/mail-api/proto/makosh/mail/sync_health/v1/client.proto',
    BACKEND_ROOT,
  ),
  validator: new URL('src/mail-api/src/sync_health.rs', BACKEND_ROOT),
  wire: new URL('src/mail-api/src/sync_health_wire.rs', BACKEND_ROOT),
  contract: new URL('src/mail-api/src/client_contract.rs', BACKEND_ROOT),
  persistence: new URL('src/mail-persistence/src/sync_health.rs', BACKEND_ROOT),
  schema: new URL('src/mail-persistence/src/schema.rs', BACKEND_ROOT),
  runtime: new URL('src/mail-runtime/src/managed.rs', BACKEND_ROOT),
  runtimeRoot: new URL('src/mail-runtime/src/main.rs', BACKEND_ROOT),
  gmailWorker: new URL(
    'src/mail-runtime/src/gmail_sync_worker.rs',
    BACKEND_ROOT,
  ),
  imapAdapter: new URL('src/mail-imap/src/lib.rs', BACKEND_ROOT),
  clientPort: new URL('src/mail-runtime/src/client_port.rs', BACKEND_ROOT),
  admission: new URL('src/mail-runtime/src/admission.rs', BACKEND_ROOT),
  managedSetup: new URL(
    'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/mail_managed_setup.rs',
    BACKEND_ROOT,
  ),
  managedFlow: new URL(
    'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/mail_sync_health_flow.rs',
    BACKEND_ROOT,
  ),
  build: new URL('src/mail-api/build.rs', BACKEND_ROOT),
  api: new URL('src/mail-api/src/lib.rs', BACKEND_ROOT),
  frontendGenerator: new URL('frontend/scripts/generate-proto.mjs', PROJECT_ROOT),
  frontendGenerated: new URL(
    'frontend/src/gen/makosh/mail/sync_health/v1/client_pb.ts',
    PROJECT_ROOT,
  ),
  frontendClient: new URL(
    'frontend/src/integrations/mail/api/mailSyncHealthClient.ts',
    PROJECT_ROOT,
  ),
  frontendGateway: new URL(
    'frontend/src/integrations/mail/api/mailSyncHealthGateway.ts',
    PROJECT_ROOT,
  ),
  frontendConnections: new URL(
    'frontend/src/integrations/mail/queries/mailAccountConnections.ts',
    PROJECT_ROOT,
  ),
  frontendController: new URL(
    'frontend/src/integrations/mail/queries/useMailSyncHealth.ts',
    PROJECT_ROOT,
  ),
  frontendModel: new URL(
    'frontend/src/integrations/mail/presentation/mailSyncHealthModel.ts',
    PROJECT_ROOT,
  ),
  frontendPanel: new URL(
    'frontend/src/integrations/mail/presentation/MailSyncHealthPanel.vue',
    PROJECT_ROOT,
  ),
  frontendRoute: new URL(
    'frontend/src/integrations/mail/views/MailOperationalRoute.vue',
    PROJECT_ROOT,
  ),
  frontendApp: new URL('frontend/src/app/layout/AppLayoutRoot.vue', PROJECT_ROOT),
};

test('Mail sync health is exact, restart-safe and cut over through its generated client', async () => {
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
    runtimeRoot,
    gmailWorker,
    imapAdapter,
    clientPort,
    admission,
    managedSetup,
    managedFlow,
    build,
    api,
    frontendGenerator,
    frontendGenerated,
    frontendClient,
    frontendGateway,
    frontendConnections,
    frontendController,
    frontendModel,
    frontendPanel,
    frontendRoute,
    frontendApp,
  ] = await Promise.all(
    Object.values(paths).map((path) => readFile(path, 'utf8')),
  );
  const inventory = JSON.parse(inventorySource);
  const slice = inventory.slices.find(
    ({ gate }) => gate === 'mail_sync_health_v1',
  );

  assert.deepEqual(slice, {
    gate: 'mail_sync_health_v1',
    role: 'integration',
    owner: 'mail',
    state: 'implemented',
    dependsOn: ['mail_account_lifecycle_v1', 'mail.sync.v1'],
  });
  assert.match(adr, /Mail integration владеет/);
  assert.match(adr, /Mail не владеет/);
  assert.match(adr, /Mail runtime не запускает detached polling timer/);
  assert.match(adr, /First-party frontend cutover выполнен/);
  assert.match(adr, /Gate `mail_sync_health_v1` открыт как `implemented`/);

  assert.match(proto, /package makosh\.mail\.sync_health\.v1/);
  assert.match(
    proto,
    /oneof query[\s\S]*get_status[\s\S]*list_runs[\s\S]*get_run/,
  );
  assert.match(proto, /service MailSyncHealthQueryService/);
  assert.match(proto, /MAIL_SYNC_OUTCOME_INTERRUPTED/);
  assert.match(proto, /MAIL_SYNC_FAILURE_CODE_RUNTIME_RESTARTED/);
  assert.match(proto, /MAIL_SYNC_FAILURE_CODE_DEADLINE_EXCEEDED = 10/);
  assert.doesNotMatch(
    proto,
    /\b(?:password|secret|token|cookie|provider_cursor|checkpoint|host|username|message_body)\b/i,
  );
  assert.doesNotMatch(proto, /\bmap\s*</);

  assert.match(validator, /MAX_SYNC_HEALTH_PAGE_SIZE: u32 = 200/);
  assert.match(validator, /validate_sync_health_query/);
  assert.match(validator, /validate_sync_run/);
  assert.match(wire, /encode_sync_health_query/);
  assert.match(wire, /decode_sync_health_response/);
  assert.match(wire, /encode_sync_health_response\(&response\)\? != bytes/);
  assert.match(contract, /MAIL_CLIENT_CONTRACT_REVISION: u32 = 15/);
  assert.match(contract, /mail\.sync\.health\.query\.v1/);
  assert.match(
    contract,
    /\/makosh\.mail\.sync_health\.v1\.MailSyncHealthQueryService\/Query/,
  );

  assert.match(
    persistence,
    /CREATE TABLE IF NOT EXISTS makosh_data\.mail_sync_runs/,
  );
  assert.match(
    persistence,
    /CREATE TABLE IF NOT EXISTS makosh_data\.mail_sync_status/,
  );
  assert.match(persistence, /mail_sync_runs_one_current_per_connection_idx/);
  assert.match(persistence, /begin_sync_run/);
  assert.match(persistence, /complete_sync_run/);
  assert.match(persistence, /interrupt_stale_sync_runs/);
  assert.match(persistence, /decode_cursor\(connection_id, value\)/);
  assert.match(schema, /MAIL_STORAGE_BUNDLE_REVISION_V12/);
  assert.match(schema, /migration_id: "mail_sync_health"/);

  assert.match(runtime, /interrupt_stale_sync_runs/);
  assert.match(runtime, /begin_sync_run/);
  assert.match(runtime, /complete_sync_run/);
  assert.match(runtime, /MailSyncTriggerV1::Manual/);
  assert.match(runtime, /prepare_pending_gmail_sync/);
  assert.match(runtime, /finalize_imap_sync_provider_page/);
  assert.match(runtime, /finalize_gmail_sync_provider_page/);
  assert.match(runtime, /finalize_gmail_sync_provider_operation/);
  assert.match(
    runtimeRoot,
    /ActiveGmailSyncProviderOperationV1[\s\S]*mpsc::Receiver<GmailSyncProviderPageDeliveryV1>/,
  );
  assert.match(
    runtimeRoot,
    /gmail_oauth_provider_operations\s*=\s*BTreeMap::<[\s\S]*CompletedGmailOAuthProviderOperationV1/,
  );
  assert.match(
    runtimeRoot,
    /imap_sync_provider_operations\s*=\s*BTreeMap::<String, ActiveImapSyncProviderOperationV1>/,
  );
  assert.match(
    runtimeRoot,
    /gmail_sync_provider_operations\s*=\s*BTreeMap::<String, ActiveGmailSyncProviderOperationV1>/,
  );
  assert.doesNotMatch(
    runtimeRoot,
    /let mut (?:gmail_oauth|imap_sync|gmail_sync)_provider_operation\s*:\s*Option/,
  );
  assert.match(
    runtimeRoot,
    /MAX_SYNC_PAGES_PER_ACCOUNT_PER_TICK:\s*usize\s*=\s*1/,
  );
  assert.match(
    runtimeRoot,
    /tokio::sync::mpsc::channel\(1\)[\s\S]*execute_gmail_sync_provider_operation\([\s\S]*page_sender/,
  );
  assert.match(
    runtimeRoot,
    /std::sync::mpsc::sync_channel\(1\)[\s\S]*execute_imap_sync_provider_operation\([\s\S]*page_sender/,
  );
  assert.match(
    runtimeRoot,
    /configuration\.configuration_instance_id\.clone\(\),\s*selected_snapshot/,
  );
  assert.match(
    runtimeRoot,
    /instance\.configuration_instance_id\.clone\(\), snapshot/,
  );
  assert.doesNotMatch(
    runtimeRoot,
    /configuration_instance_id:\s*snapshot\.target_id/,
  );
  assert.doesNotMatch(runtimeRoot, /execute_pending_gmail_sync/);
  assert.match(gmailWorker, /list_messages/);
  assert.match(gmailWorker, /list_history/);
  assert.match(gmailWorker, /fetch_raw_message/);
  assert.match(gmailWorker, /Zeroizing<Vec<u8>>/);
  assert.match(gmailWorker, /mpsc::Sender<GmailSyncProviderPageDeliveryV1>/);
  assert.match(gmailWorker, /oneshot::channel\(\)/);
  assert.match(gmailWorker, /matches!\(committed\.await, Ok\(true\)\)/);
  assert.doesNotMatch(
    gmailWorker,
    /pages:\s*Vec<GmailSyncProviderPageV1>/,
  );
  assert.match(imapAdapter, /FnMut\(ImapSyncResult\) -> Result<\(\), \(\)>/);
  assert.match(imapAdapter, /fetch_uids\.chunks\(execution\.page_size\)/);
  assert.match(imapAdapter, /finalize_page\(ImapSyncResult/);
  assert.match(imapAdapter, /select_latest_uids/);
  assert.match(
    imapAdapter,
    /attempts < retry::MAX_SYNC_ATTEMPTS/,
  );
  assert.match(
    runtime,
    /record_operational_materializations\(&materializations,\s*observed_at_unix_seconds\)/,
  );
  assert.doesNotMatch(
    gmailWorker,
    /MailBootstrapError|crate::managed|MailDurablePersistence|ManagedControlChannel|BlobDataClient|communications_outbox|makosh_communications/,
  );
  assert.doesNotMatch(
    runtime,
    /\.list_messages\(|\.list_history\(|\.fetch_raw_message\(/,
  );
  assert.match(
    runtime,
    /sync_health_query_connection_id\(query\)[\s\S]*self\.account\.connection_id/,
  );
  assert.match(clientPort, /MailClientRequestV1::SyncHealthQuery/);
  assert.match(clientPort, /sync_health_wire::encode_sync_health_query/);
  assert.match(clientPort, /sync_health_wire::decode_sync_health_response/);
  assert.match(admission, /MailClientContractV1::SyncHealthQuery/);
  assert.match(managedSetup, /mail_runtime_storage_bundle_v1/);
  assert.match(managedSetup, /MailClientContractV1::SyncHealthQuery/);
  assert.match(managedFlow, /assert_mail_sync_replay_and_health/);
  assert.match(
    managedFlow,
    /MailClientContractV1::Sync[\s\S]*MailClientResponseV1::SyncInboxAccepted/,
  );
  assert.match(managedFlow, /assert_stale_generation_is_interrupted/);
  assert.match(managedFlow, /MailSyncOutcomeV1::Interrupted/);
  assert.match(managedFlow, /MailSyncFailureCodeV1::RuntimeRestarted/);
  assert.match(runtime, /MAIL_SYNC_OPERATION_DEADLINE_SECONDS: i64 = 300/);
  assert.match(
    runtime,
    /sync_operation_deadline[\s\S]*checked_add\(MAIL_SYNC_OPERATION_DEADLINE_SECONDS\)/,
  );
  assert.match(
    runtimeRoot,
    /expire_pending_sync_operations[\s\S]*expire_active_gmail_sync_operation[\s\S]*expire_active_imap_sync_operation/,
  );
  assert.match(runtimeRoot, /active\.completion\.abort\(\)/);
  assert.match(runtimeRoot, /active\.pages = None/);
  assert.match(
    build,
    /proto\/makosh\/mail\/sync_health\/v1\/client\.proto/,
  );
  assert.match(api, /pub mod sync_health;/);
  assert.match(
    frontendGenerator,
    /mail', 'sync_health', 'v1', 'client\.proto'/,
  );
  assert.match(frontendGenerated, /MailSyncHealthQueryService/);
  assert.match(frontendClient, /MailSyncHealthQueryService/);
  assert.match(frontendClient, /createBrowserGatewayConnectTransport/);
  assert.match(frontendGateway, /MailSyncHealthQueryV1Schema/);
  assert.match(frontendGateway, /GetMailSyncStatusQueryV1Schema/);
  assert.match(frontendGateway, /ListMailSyncRunsQueryV1Schema/);
  assert.match(frontendGateway, /GetMailSyncRunQueryV1Schema/);
  assert.match(frontendConnections, /mail\.account\.catalog\.query\.v1/);
  assert.match(
    frontendConnections,
    /syncReadiness[\s\S]*MailProviderPathReadinessV1\.MAIL_PROVIDER_PATH_READINESS_READY/,
  );
  assert.match(frontendConnections, /syncReady/);
  assert.match(frontendController, /getMailSyncStatus/);
  assert.match(frontendController, /listMailSyncRuns/);
  assert.match(frontendModel, /MailSyncFailureCodeV1/);
  assert.match(frontendPanel, /Sync health/);
  assert.match(frontendPanel, /Run history/);
  assert.match(frontendRoute, /useMailSyncHealth/);
  assert.match(frontendApp, /mail\.sync\.health\.query\.v1/);
  assert.doesNotMatch(
    [
      proto,
      validator,
      wire,
      persistence,
      frontendClient,
      frontendGateway,
      frontendConnections,
      frontendController,
      frontendModel,
      frontendPanel,
      frontendRoute,
    ].join('\n'),
    /makosh_communications|domains\/communications|makosh_scheduler|makosh_kernel/i,
  );
});
