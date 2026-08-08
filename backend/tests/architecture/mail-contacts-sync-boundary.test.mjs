import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../../', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

const files = {
  adr: new URL(
    'docs/adr/ADR-0379-mail-address-book-sync-and-contacts-command-boundary.md',
    PROJECT_ROOT,
  ),
  sourceAdr: new URL(
    'docs/adr/ADR-0381-contacts-target-bound-mail-sync-source-port.md',
    PROJECT_ROOT,
  ),
  providerAdr: new URL(
    'docs/adr/ADR-0382-mail-address-book-provider-execution-and-authority.md',
    PROJECT_ROOT,
  ),
  providerLinkAdr: new URL(
    'docs/adr/ADR-0383-contacts-provider-link-reconciliation-after-mail-write.md',
    PROJECT_ROOT,
  ),
  recoveryAdr: new URL(
    'docs/adr/ADR-0384-mail-contacts-sync-outage-recovery-and-revocation-fencing.md',
    PROJECT_ROOT,
  ),
  inventory: new URL('architecture/communications-settings-reconstruction.json', BACKEND_ROOT),
  policy: new URL('architecture/policy.json', BACKEND_ROOT),
  apiManifest: new URL('src/contacts-command-api/Cargo.toml', BACKEND_ROOT),
  coreManifest: new URL('src/contacts-core/Cargo.toml', BACKEND_ROOT),
  proto: new URL(
    'src/contacts-command-api/proto/makosh/contacts/command/v1/contacts_command.proto',
    BACKEND_ROOT,
  ),
  api: new URL('src/contacts-command-api/src/lib.rs', BACKEND_ROOT),
  envelope: new URL('src/contacts-command-api/src/envelope.rs', BACKEND_ROOT),
  sourceApiManifest: new URL('src/contacts-mail-sync-source-api/Cargo.toml', BACKEND_ROOT),
  sourceProto: new URL(
    'src/contacts-mail-sync-source-api/proto/makosh/contacts/mail_sync_source/v1/mail_sync_source.proto',
    BACKEND_ROOT,
  ),
  sourceApi: new URL('src/contacts-mail-sync-source-api/src/lib.rs', BACKEND_ROOT),
  sourceEnvelope: new URL(
    'src/contacts-mail-sync-source-api/src/envelope.rs',
    BACKEND_ROOT,
  ),
  core: new URL('src/contacts-core/src/lib.rs', BACKEND_ROOT),
  identity: new URL('src/contacts-core/src/identity.rs', BACKEND_ROOT),
  upsert: new URL('src/contacts-core/src/upsert.rs', BACKEND_ROOT),
  persistenceManifest: new URL('src/contacts-persistence/Cargo.toml', BACKEND_ROOT),
  persistence: new URL('src/contacts-persistence/src/repository.rs', BACKEND_ROOT),
  migration: new URL(
    'src/contacts-persistence/migrations/0001_contacts.sql',
    BACKEND_ROOT,
  ),
  sourceMigration: new URL(
    'src/contacts-persistence/migrations/0002_mail_sync_source.sql',
    BACKEND_ROOT,
  ),
  providerLinkMigration: new URL(
    'src/contacts-persistence/migrations/0003_mail_provider_link_command.sql',
    BACKEND_ROOT,
  ),
  providerLinkPersistence: new URL(
    'src/contacts-persistence/src/provider_link.rs',
    BACKEND_ROOT,
  ),
  runtimeManifest: new URL('src/contacts-runtime/Cargo.toml', BACKEND_ROOT),
  runtimeAdmission: new URL('src/contacts-runtime/src/admission.rs', BACKEND_ROOT),
  runtimeCommand: new URL('src/contacts-runtime/src/command.rs', BACKEND_ROOT),
  runtimeSource: new URL('src/contacts-runtime/src/source.rs', BACKEND_ROOT),
  runtimeProviderLink: new URL('src/contacts-runtime/src/provider_link.rs', BACKEND_ROOT),
  managedRuntime: new URL('src/contacts-runtime/src/managed_runtime.rs', BACKEND_ROOT),
  assemblyManifest: new URL('src/contacts-assembly/Cargo.toml', BACKEND_ROOT),
  assembly: new URL('src/contacts-assembly/src/lib.rs', BACKEND_ROOT),
  developmentRelease: new URL('scripts/materialize-dev-release.sh', BACKEND_ROOT),
  mailContractManifest: new URL('src/mail-address-book-contract/Cargo.toml', BACKEND_ROOT),
  mailContract: new URL(
    'src/mail-address-book-contract/proto/makosh/mail/address_book/v1/address_book.proto',
    BACKEND_ROOT,
  ),
  googlePeopleManifest: new URL('src/mail-google-people/Cargo.toml', BACKEND_ROOT),
  googlePeople: new URL('src/mail-google-people/src/lib.rs', BACKEND_ROOT),
  cardDavManifest: new URL('src/mail-carddav/Cargo.toml', BACKEND_ROOT),
  cardDav: new URL('src/mail-carddav/src/lib.rs', BACKEND_ROOT),
  mailPersistenceManifest: new URL(
    'src/mail-address-book-persistence/Cargo.toml',
    BACKEND_ROOT,
  ),
  mailPersistenceMigration: new URL(
    'src/mail-address-book-persistence/migrations/0001_address_book_upsert.sql',
    BACKEND_ROOT,
  ),
  mailPersistenceCustodyMigration: new URL(
    'src/mail-address-book-persistence/migrations/0002_snapshot_custody.sql',
    BACKEND_ROOT,
  ),
  mailPersistenceProviderPageMigration: new URL(
    'src/mail-address-book-persistence/migrations/0003_provider_page.sql',
    BACKEND_ROOT,
  ),
  mailPersistenceCustody: new URL(
    'src/mail-address-book-persistence/src/custody.rs',
    BACKEND_ROOT,
  ),
  mailPersistenceDelivery: new URL(
    'src/mail-address-book-persistence/src/delivery.rs',
    BACKEND_ROOT,
  ),
  mailPersistenceFetchDelivery: new URL(
    'src/mail-address-book-persistence/src/fetch_delivery.rs',
    BACKEND_ROOT,
  ),
  mailPersistenceSchema: new URL(
    'src/mail-address-book-persistence/src/schema.rs',
    BACKEND_ROOT,
  ),
  mailCredentialPersistence: new URL('src/mail-persistence/src/account.rs', BACKEND_ROOT),
  mailLifecyclePersistence: new URL('src/mail-persistence/src/lifecycle.rs', BACKEND_ROOT),
  mailCardDavCredentialMigration: new URL(
    'src/mail-persistence/migrations/0029_icloud_carddav_credential_bindings.sql',
    BACKEND_ROOT,
  ),
  mailRuntimeSettings: new URL('src/mail-runtime/src/settings.rs', BACKEND_ROOT),
  mailRuntimeStorageBundle: new URL('src/mail-runtime/src/storage_bundle.rs', BACKEND_ROOT),
  mailRuntimeManifest: new URL('src/mail-runtime/Cargo.toml', BACKEND_ROOT),
  mailRuntimeAdmission: new URL('src/mail-runtime/src/admission.rs', BACKEND_ROOT),
  mailRuntimeAddressBookConsumer: new URL(
    'src/mail-runtime/src/address_book_consumer.rs',
    BACKEND_ROOT,
  ),
  mailRuntimeAddressBookSnapshot: new URL(
    'src/mail-runtime/src/address_book_snapshot.rs',
    BACKEND_ROOT,
  ),
  mailRuntimeAddressBookWorker: new URL(
    'src/mail-runtime/src/address_book_worker.rs',
    BACKEND_ROOT,
  ),
  mailRuntimeAddressBookFetchWorker: new URL(
    'src/mail-runtime/src/address_book_fetch_worker.rs',
    BACKEND_ROOT,
  ),
  mailRuntimeAddressBookProvider: new URL(
    'src/mail-runtime/src/address_book_provider.rs',
    BACKEND_ROOT,
  ),
  mailRuntimeAddressBookOutbox: new URL(
    'src/mail-runtime/src/address_book_outbox.rs',
    BACKEND_ROOT,
  ),
  mailRuntimeManaged: new URL('src/mail-runtime/src/managed.rs', BACKEND_ROOT),
  mailRuntimeMain: new URL('src/mail-runtime/src/main.rs', BACKEND_ROOT),
  mailApi: new URL('src/mail-api/src/lib.rs', BACKEND_ROOT),
  managedMailSetup: new URL(
    'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/mail_managed_setup.rs',
    BACKEND_ROOT,
  ),
  managedProviderFlow: new URL(
    'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/mail_address_book_provider_flow.rs',
    BACKEND_ROOT,
  ),
  managedGoogleFixture: new URL(
    'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/mail_gmail_fixture.rs',
    BACKEND_ROOT,
  ),
  managedCardDavFixture: new URL(
    'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/mail_carddav_fixture.rs',
    BACKEND_ROOT,
  ),
  managedSyncSetup: new URL(
    'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/mail_contacts_sync_managed_setup.rs',
    BACKEND_ROOT,
  ),
  managedSyncFlow: new URL(
    'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/mail_contacts_sync_managed_flow.rs',
    BACKEND_ROOT,
  ),
  authenticatedStorageHarness: new URL(
    'scripts/test-authenticated-storage.mjs',
    BACKEND_ROOT,
  ),
  mailOAuthCore: new URL('src/mail-core/src/oauth.rs', BACKEND_ROOT),
  mailOAuthPersistence: new URL('src/mail-persistence/src/oauth.rs', BACKEND_ROOT),
  mailPortabilityProto: new URL(
    'src/mail-api/proto/makosh/mail/portability/v1/portability.proto',
    BACKEND_ROOT,
  ),
  workflowApiManifest: new URL('src/mail-contacts-sync-api/Cargo.toml', BACKEND_ROOT),
  workflowApi: new URL(
    'src/mail-contacts-sync-api/proto/makosh/mail_contacts_sync/v1/sync.proto',
    BACKEND_ROOT,
  ),
  workflowCoreManifest: new URL('src/mail-contacts-sync-core/Cargo.toml', BACKEND_ROOT),
  workflowCore: new URL('src/mail-contacts-sync-core/src/lib.rs', BACKEND_ROOT),
  workflowPersistenceManifest: new URL(
    'src/mail-contacts-sync-persistence/Cargo.toml',
    BACKEND_ROOT,
  ),
  workflowPersistence: new URL(
    'src/mail-contacts-sync-persistence/src/repository.rs',
    BACKEND_ROOT,
  ),
  workflowOrchestration: new URL(
    'src/mail-contacts-sync-persistence/src/orchestration.rs',
    BACKEND_ROOT,
  ),
  workflowRelay: new URL('src/mail-contacts-sync-persistence/src/relay.rs', BACKEND_ROOT),
  workflowRealtime: new URL(
    'src/mail-contacts-sync-persistence/src/realtime.rs',
    BACKEND_ROOT,
  ),
  workflowMigration: new URL(
    'src/mail-contacts-sync-persistence/migrations/0001_mail_contacts_sync.sql',
    BACKEND_ROOT,
  ),
  workflowOrchestrationMigration: new URL(
    'src/mail-contacts-sync-persistence/migrations/0002_mail_contacts_sync_orchestration.sql',
    BACKEND_ROOT,
  ),
  workflowReverseMigration: new URL(
    'src/mail-contacts-sync-persistence/migrations/0003_reverse_sync.sql',
    BACKEND_ROOT,
  ),
  workflowSchedulerCompletionMigration: new URL(
    'src/mail-contacts-sync-persistence/migrations/0004_scheduler_completion.sql',
    BACKEND_ROOT,
  ),
  workflowReverseOriginMigration: new URL(
    'src/mail-contacts-sync-persistence/migrations/0005_reverse_origin_run.sql',
    BACKEND_ROOT,
  ),
  workflowProviderLinkMigration: new URL(
    'src/mail-contacts-sync-persistence/migrations/0006_provider_link_reconciliation.sql',
    BACKEND_ROOT,
  ),
  workflowScheduledCompletionPersistence: new URL(
    'src/mail-contacts-sync-persistence/src/scheduled_completion.rs',
    BACKEND_ROOT,
  ),
  workflowReversePersistence: new URL(
    'src/mail-contacts-sync-persistence/src/reverse_sync.rs',
    BACKEND_ROOT,
  ),
  workflowRuntimeManifest: new URL('src/mail-contacts-sync-runtime/Cargo.toml', BACKEND_ROOT),
  workflowRuntimeLib: new URL('src/mail-contacts-sync-runtime/src/lib.rs', BACKEND_ROOT),
  workflowCommands: new URL('src/mail-contacts-sync-runtime/src/commands.rs', BACKEND_ROOT),
  workflowProviderEvents: new URL(
    'src/mail-contacts-sync-runtime/src/provider_events.rs',
    BACKEND_ROOT,
  ),
  workflowRunProgress: new URL(
    'src/mail-contacts-sync-runtime/src/run_progress.rs',
    BACKEND_ROOT,
  ),
  workflowRuntimeAdmission: new URL('src/mail-contacts-sync-runtime/src/admission.rs', BACKEND_ROOT),
  workflowManagedRuntime: new URL('src/mail-contacts-sync-runtime/src/managed_runtime.rs', BACKEND_ROOT),
  workflowRuntimeMain: new URL('src/mail-contacts-sync-runtime/src/main.rs', BACKEND_ROOT),
  workflowAssemblyManifest: new URL(
    'src/mail-contacts-sync-assembly/Cargo.toml',
    BACKEND_ROOT,
  ),
  workflowAssembly: new URL(
    'src/mail-contacts-sync-assembly/src/lib.rs',
    BACKEND_ROOT,
  ),
  workflowScheduler: new URL('src/mail-contacts-sync-runtime/src/scheduler_due.rs', BACKEND_ROOT),
  workflowSchedulerExecution: new URL(
    'src/mail-contacts-sync-runtime/src/scheduler_execution.rs',
    BACKEND_ROOT,
  ),
  workflowSchedulerCompletion: new URL(
    'src/mail-contacts-sync-runtime/src/scheduler_completion.rs',
    BACKEND_ROOT,
  ),
  workflowReverseChange: new URL(
    'src/mail-contacts-sync-runtime/src/reverse_change.rs',
    BACKEND_ROOT,
  ),
  workflowSourceResults: new URL(
    'src/mail-contacts-sync-runtime/src/source_results.rs',
    BACKEND_ROOT,
  ),
  workflowProviderWriteResults: new URL(
    'src/mail-contacts-sync-runtime/src/provider_write_results.rs',
    BACKEND_ROOT,
  ),
  workflowProviderLinkResults: new URL(
    'src/mail-contacts-sync-runtime/src/provider_link_results.rs',
    BACKEND_ROOT,
  ),
  postgresLive: new URL(
    'tests/support/mail-contacts-sync/tests/postgres_live.rs',
    BACKEND_ROOT,
  ),
};

test('mail contacts sync agreement keeps integration workflow and domain separate', async () => {
  const [adr, inventorySource, policySource] = await Promise.all([
    readFile(files.adr, 'utf8'),
    readFile(files.inventory, 'utf8'),
    readFile(files.policy, 'utf8'),
  ]);
  const inventory = JSON.parse(inventorySource);
  const policy = JSON.parse(policySource);
  const contactsGate = inventory.slices.find(
    ({ gate }) => gate === 'contacts_mail_identity_command_v1',
  );
  const workflowGate = inventory.slices.find(({ gate }) => gate === 'mail_contacts_sync_v1');

  assert.deepEqual(contactsGate, {
    gate: 'contacts_mail_identity_command_v1',
    role: 'domain',
    owner: 'contacts',
    state: 'implemented',
    dependsOn: ['client_gateway_v1'],
  });
  assert.deepEqual(workflowGate, {
    gate: 'mail_contacts_sync_v1',
    role: 'workflow',
    owner: 'mail_contacts_sync',
    state: 'implemented',
    dependsOn: ['mail_account_lifecycle_v1', 'contacts_mail_identity_command_v1'],
  });
  assert.match(adr, /Mail integration владеет Google People\/CardDAV protocol/);
  assert.match(adr, /Contacts domain владеет person/);
  assert.match(adr, /Workflow\s+владеет направлением sync, correlation, checkpoints, retry/);
  assert.match(adr, /periodic polling[\s\S]*forbidden/i);
  assert.equal(
    policy.implementation.currentSlice,
    'call_transcription_managed_conformance_v1',
  );
  assert(policy.implementation.ownerInventory.domains.includes('contacts'));
  assert(
    policy.implementation.ownerInventory.businessCapabilities.includes(
      'contacts.mail-identity.command.v1',
    ),
  );
  assert(policy.implementation.ownerInventory.workflows.includes('mail_contacts_sync'));
  assert(
    policy.implementation.ownerInventory.businessCapabilities.includes(
      'mail.address-book.provider.v1',
    ),
  );
});

test('managed sync runtime uses staged settings and exact event-only owner contracts', async () => {
  const [
    manifest,
    admission,
    runtime,
    main,
    scheduler,
    reverseChange,
    sourceResults,
    providerWriteResults,
    providerLinkResults,
  ] = await Promise.all([
    readFile(files.workflowRuntimeManifest, 'utf8'),
    readFile(files.workflowRuntimeAdmission, 'utf8'),
    readFile(files.workflowManagedRuntime, 'utf8'),
    readFile(files.workflowRuntimeMain, 'utf8'),
    readFile(files.workflowScheduler, 'utf8'),
    readFile(files.workflowReverseChange, 'utf8'),
    readFile(files.workflowSourceResults, 'utf8'),
    readFile(files.workflowProviderWriteResults, 'utf8'),
    readFile(files.workflowProviderLinkResults, 'utf8'),
  ]);
  assert.match(manifest, /role = "workflow"/);
  assert.match(manifest, /owner = "mail_contacts_sync"/);
  assert.match(manifest, /surface = "runtime"/);
  assert.doesNotMatch(manifest, /makosh-mail-(?:runtime|persistence)|makosh-contacts-(?:runtime|persistence)/);
  assert.match(admission, /SchedulerJobRequestV1/);
  assert.match(admission, /DurableEnvelopeKindV1::Ack/);
  assert.match(admission, /DurableEnvelopeKindV1::Result/);
  assert.match(runtime, /request_managed_runtime_event_access_v2/);
  assert.match(runtime, /StorageVaultLeaseAdapterV1/);
  assert.match(main, /configuration_instance_id/);
  assert.match(main, /settings_snapshot_bytes/);
  assert.match(runtime, /pump_client_realtime_once/);
  assert.doesNotMatch(runtime, /reqwest|provider_kind\s*==|SELECT .*contacts|SELECT .*mail/i);
  assert.match(scheduler, /JOB_EXECUTE_CAPABILITY_V1/);
  assert.match(scheduler, /configuration_instance_id/);
  assert.doesNotMatch(scheduler, /account_id|provider_kind|access_token|refresh_token/);
  assert.match(reverseChange, /consume_contact_changed_once_v1/);
  assert.match(reverseChange, /SyncDirectionV1::Bidirectional/);
  assert.match(reverseChange, /remote_write_enabled/);
  assert.match(sourceResults, /consume_source_prepared_once_v1/);
  assert.match(sourceResults, /build_upsert_mail_address_book_entry_command_v1/);
  assert.match(providerWriteResults, /consume_mail_entry_upserted_once_v1/);
  assert.match(providerWriteResults, /complete_mail_address_book_upsert/);
  assert.match(providerWriteResults, /MailContactsSyncProviderWriteOutcomeV1::OutcomeUnknown/);
  assert.match(providerWriteResults, /build_bind_mail_address_book_provider_link_command_outbox_record_v1/);
  assert.match(providerLinkResults, /consume_provider_link_bound_once_v1/);
  assert.match(providerLinkResults, /consume_provider_link_rejected_once_v1/);
  assert.match(providerLinkResults, /complete_contacts_provider_link/);
  assert.doesNotMatch(
    `${reverseChange}\n${sourceResults}\n${providerWriteResults}\n${providerLinkResults}`,
    /BlobDataClient|provider_kind\s*==|reqwest/,
  );
});

test('mail contacts sync assembly is a distinct unsigned workflow build unit', async () => {
  const [manifest, assembly, developmentRelease] = await Promise.all([
    readFile(files.workflowAssemblyManifest, 'utf8'),
    readFile(files.workflowAssembly, 'utf8'),
    readFile(files.developmentRelease, 'utf8'),
  ]);
  assert.match(manifest, /role = "workflow"/);
  assert.match(manifest, /owner = "mail_contacts_sync"/);
  assert.match(manifest, /surface = "assembly"/);
  assert.match(manifest, /makosh-mail-contacts-sync-persistence/);
  assert.match(manifest, /makosh-mail-contacts-sync-runtime/);
  assert.doesNotMatch(
    manifest,
    /makosh-mail-(?:runtime|persistence)|makosh-contacts-(?:runtime|persistence)/,
  );
  assert.match(assembly, /mail_contacts_sync_module_descriptor_v1/);
  assert.match(assembly, /mail_contacts_sync_settings_schema_v1/);
  assert.match(assembly, /mail_contacts_sync_storage_bundle_v1/);
  assert.match(assembly, /create_new\(true\)/);
  assert.doesNotMatch(
    assembly,
    /SigningKey|--signing-key|private[_-]?key|provider_kind|reqwest/i,
  );
  assert.match(developmentRelease, /--package makosh-mail-contacts-sync-assembly/);
  assert.match(
    developmentRelease,
    /mail_contacts_sync\.release-artifacts\.json/,
  );
});

test('Mail address-book providers are separate bounded integration adapters', async () => {
  const [adr, policySource, googleManifest, google, cardDavManifest, cardDav] =
    await Promise.all([
      readFile(files.providerAdr, 'utf8'),
      readFile(files.policy, 'utf8'),
      readFile(files.googlePeopleManifest, 'utf8'),
      readFile(files.googlePeople, 'utf8'),
      readFile(files.cardDavManifest, 'utf8'),
      readFile(files.cardDav, 'utf8'),
    ]);
  const policy = JSON.parse(policySource);
  const packages = new Map(
    policy.implementation.productionPackages.map((descriptor) => [descriptor.name, descriptor]),
  );

  assert.match(adr, /makosh-mail-google-people/);
  assert.match(adr, /makosh-mail-carddav/);
  assert.match(adr, /никогда не выводит provider из[\s\S]*hostname, email suffix/);
  assert.match(adr, /mail_icloud_carddav_password/);
  assert.deepEqual(packages.get('makosh-mail-google-people'), {
    name: 'makosh-mail-google-people',
    role: 'integration',
    owner: 'mail',
    surface: 'implementation',
  });
  assert.deepEqual(packages.get('makosh-mail-carddav'), {
    name: 'makosh-mail-carddav',
    role: 'integration',
    owner: 'mail',
    surface: 'implementation',
  });

  for (const manifest of [googleManifest, cardDavManifest]) {
    assert.match(manifest, /role = "integration"/);
    assert.match(manifest, /owner = "mail"/);
    assert.match(manifest, /surface = "implementation"/);
    assert.doesNotMatch(
      manifest,
      /makosh-(?:contacts|communications|mail-contacts-sync|events|storage|vault)/,
    );
  }
  assert.match(google, /GOOGLE_PEOPLE_API_HOST_V1: &str = "people.googleapis.com"/);
  assert.match(google, /GOOGLE_PEOPLE_CONTACTS_SCOPE_V1/);
  assert.match(google, /OutcomeUnknown/);
  assert.match(google, /expected_etag/);
  assert.match(google, /take\(\(MAX_RESPONSE_BYTES \+ 1\) as u64\)/);
  assert.doesNotMatch(google, /reqwest|sqlx|async_nats|makosh_contacts/i);

  assert.match(cardDav, /ICLOUD_CARDDAV_HOST_V1: &str = "contacts.icloud.com"/);
  assert.match(cardDav, /ICLOUD_CARDDAV_CREDENTIAL_PURPOSE_V1/);
  assert.match(cardDav, /ReadOnlyProvider/);
  assert.match(cardDav, /supports_remote_write/);
  assert.match(cardDav, /take\(\(MAX_RESPONSE_BYTES \+ 1\) as u64\)/);
  assert.doesNotMatch(cardDav, /reqwest|sqlx|async_nats|makosh_contacts/i);
});

test('Mail owns address-book persistence settings and credential authority without Contacts storage', async () => {
  const [
    manifest,
    migration,
    custodyMigration,
    providerPageMigration,
    custody,
    delivery,
    fetchDelivery,
    schema,
    credentialPersistence,
    lifecyclePersistence,
    cardDavCredentialMigration,
    settings,
    storageBundle,
    oauthCore,
    oauthPersistence,
    portability,
    policySource,
  ] = await Promise.all([
    readFile(files.mailPersistenceManifest, 'utf8'),
    readFile(files.mailPersistenceMigration, 'utf8'),
    readFile(files.mailPersistenceCustodyMigration, 'utf8'),
    readFile(files.mailPersistenceProviderPageMigration, 'utf8'),
    readFile(files.mailPersistenceCustody, 'utf8'),
    readFile(files.mailPersistenceDelivery, 'utf8'),
    readFile(files.mailPersistenceFetchDelivery, 'utf8'),
    readFile(files.mailPersistenceSchema, 'utf8'),
    readFile(files.mailCredentialPersistence, 'utf8'),
    readFile(files.mailLifecyclePersistence, 'utf8'),
    readFile(files.mailCardDavCredentialMigration, 'utf8'),
    readFile(files.mailRuntimeSettings, 'utf8'),
    readFile(files.mailRuntimeStorageBundle, 'utf8'),
    readFile(files.mailOAuthCore, 'utf8'),
    readFile(files.mailOAuthPersistence, 'utf8'),
    readFile(files.mailPortabilityProto, 'utf8'),
    readFile(files.policy, 'utf8'),
  ]);
  const policy = JSON.parse(policySource);
  const descriptor = policy.implementation.productionPackages.find(
    ({ name }) => name === 'makosh-mail-address-book-persistence',
  );

  assert.deepEqual(descriptor, {
    name: 'makosh-mail-address-book-persistence',
    role: 'integration',
    owner: 'mail',
    surface: 'persistence',
  });
  assert.match(manifest, /role = "integration"/);
  assert.match(manifest, /owner = "mail"/);
  assert.match(manifest, /surface = "persistence"/);
  assert.doesNotMatch(manifest, /makosh-contacts|makosh-mail-contacts-sync/);
  assert.match(schema, /MAIL_ADDRESS_BOOK_STORAGE_BUNDLE_REVISION_V1: u32 = 28/);
  assert.match(storageBundle, /append_mail_address_book_storage_v1/);
  assert.match(storageBundle, /append_mail_icloud_carddav_credential_storage_v1/);
  assert.match(cardDavCredentialMigration, /mail_icloud_carddav_credential_bindings/);
  assert.match(cardDavCredentialMigration, /mail_icloud_carddav_lifecycle_credentials/);
  assert.doesNotMatch(cardDavCredentialMigration, /DROP |ALTER TABLE/i);
  assert.match(credentialPersistence, /mail_icloud_carddav_credential_bindings/);
  assert.match(lifecyclePersistence, /mail_icloud_carddav_lifecycle_credentials/);
  assert.match(migration, /mail_address_book_upsert_inbox/);
  assert.match(migration, /mail_address_book_upsert_result_outbox/);
  assert.match(migration, /contacts_write_authorized BOOLEAN NOT NULL DEFAULT FALSE/);
  assert.match(custodyMigration, /target_contact_snapshot_reference_id/);
  assert.match(custodyMigration, /mail_address_book_target_snapshot_receipt_complete/);
  assert.match(custody, /record_target_snapshot_receipt/);
  assert.match(custody, /AlreadyRecorded/);
  assert.match(providerPageMigration, /mail_address_book_fetch_inbox/);
  assert.match(providerPageMigration, /mail_address_book_fetch_outbox/);
  assert.match(providerPageMigration, /UNIQUE \(command_message_id, command_id\)/);
  assert.match(fetchDelivery, /accept_fetch_command/);
  assert.match(fetchDelivery, /complete_fetch_command/);
  assert.match(fetchDelivery, /pending_fetch_events/);
  assert.match(fetchDelivery, /FOR UPDATE/);
  assert.doesNotMatch(migration, /CREATE TABLE makosh_data\.contacts_/);
  assert.doesNotMatch(custodyMigration, /makosh_data\.contacts_/);
  assert.doesNotMatch(providerPageMigration, /makosh_data\.contacts_/);
  assert.doesNotMatch(migration, /mail_contacts_sync_/);
  assert.match(delivery, /mark_dispatch_started/);
  assert.match(delivery, /uncertain_upserts/);
  assert.match(delivery, /exact_envelope_bytes/);
  assert.doesNotMatch(delivery, /SELECT[\s\S]*makosh_data\.contacts_/i);
  assert.match(settings, /mail\.address_book\.provider/);
  assert.match(settings, /MailAddressBookProviderV1::GooglePeople/);
  assert.match(settings, /MailAddressBookProviderV1::IcloudCardDav/);
  assert.doesNotMatch(settings, /ends_with\(|contains\("gmail|contains\("icloud/i);
  assert.match(oauthCore, /GOOGLE_CONTACTS_WRITE_SCOPE_V1/);
  assert.match(oauthCore, /gmail_oauth_scope_authorizes_contacts_write/);
  assert.match(oauthPersistence, /contacts_write_authorized/);
  assert.match(portability, /MailAddressBookProviderV1 address_book_provider/);
  assert.match(portability, /optional string carddav_username/);
});

test('Mail runtime executes reverse sync through exact event Blob and provider boundaries', async () => {
  const [
    manifest,
    admission,
    consumer,
    snapshot,
    worker,
    provider,
    outbox,
    managed,
    main,
    policySource,
  ] = await Promise.all([
    readFile(files.mailRuntimeManifest, 'utf8'),
    readFile(files.mailRuntimeAdmission, 'utf8'),
    readFile(files.mailRuntimeAddressBookConsumer, 'utf8'),
    readFile(files.mailRuntimeAddressBookSnapshot, 'utf8'),
    readFile(files.mailRuntimeAddressBookWorker, 'utf8'),
    readFile(files.mailRuntimeAddressBookProvider, 'utf8'),
    readFile(files.mailRuntimeAddressBookOutbox, 'utf8'),
    readFile(files.mailRuntimeManaged, 'utf8'),
    readFile(files.mailRuntimeMain, 'utf8'),
    readFile(files.policy, 'utf8'),
  ]);
  const policy = JSON.parse(policySource);

  assert.match(manifest, /makosh-mail-address-book-contract/);
  assert.match(manifest, /makosh-mail-address-book-persistence/);
  assert.match(manifest, /makosh-mail-google-people/);
  assert.match(manifest, /makosh-contacts-mail-sync-source-api/);
  assert.doesNotMatch(manifest, /makosh-contacts-(?:runtime|persistence|core)/);
  assert.match(admission, /MAIL_ADDRESS_BOOK_CAPABILITY_ID_V1/);
  assert.match(admission, /CONTACT_MAIL_SYNC_SOURCE_BLOB_TARGET_CAPABILITY_ID_V1/);
  assert.match(admission, /BlobQuotaOperationV1::CustodyTransfer/);
  assert.match(consumer, /receive_runtime_pull_delivery/);
  assert.match(consumer, /MAIL_ADDRESS_BOOK_COMMAND_SOURCE_MODULE_ID_V1/);
  assert.match(consumer, /accept_upsert_command/);
  assert.match(consumer, /acknowledge\(\)/);
  assert.doesNotMatch(consumer, /provider_kind|GooglePeopleClientV1|CardDav/);
  assert.match(snapshot, /request_managed_blob_custody_transfer_v2/);
  assert.match(snapshot, /request_managed_blob_session_v2/);
  assert.match(snapshot, /transfer_contact_snapshot_custody_v1/);
  assert.match(snapshot, /read_contact_snapshot_v1/);
  assert.match(snapshot, /ContactMailSyncSourceContentV1::decode/);
  assert.match(snapshot, /Sha256::digest/);
  assert.match(worker, /MailAddressBookProviderV1::GooglePeople|google_people_client_v1/);
  assert.match(worker, /MailAddressBookProviderV1::IcloudCardDav/);
  assert.match(worker, /MailAddressBookRejectCodeReadOnlyProvider/);
  assert.match(worker, /contacts_write_authorized/);
  assert.match(worker, /mark_dispatch_started/);
  assert.match(worker, /record_target_snapshot_receipt/);
  assert.match(worker, /MailAddressBookRejectCodeOutcomeUnknown/);
  assert.match(provider, /GooglePeopleClientV1::for_conformance_endpoint/);
  assert.match(provider, /GooglePeopleClientV1::new/);
  assert.doesNotMatch(`${consumer}\n${snapshot}\n${worker}`, /SELECT\s|INSERT\s|UPDATE\s|DELETE\s/i);
  assert.match(outbox, /publish_exact/);
  assert.match(outbox, /mark_result_published/);
  assert.match(managed, /address_book_upsert_subscribe_permit/);
  assert.match(main, /process_next_mail_address_book_upsert_v1/);
  assert.equal(
    policy.implementation.currentSlice,
    'call_transcription_managed_conformance_v1',
  );
  assert(
    policy.implementation.ownerInventory.businessCapabilities.includes(
      'mail.address-book.contact-source.blob.v1',
    ),
  );
});

test('Mail runtime paginates provider address books behind typed event-only commands', async () => {
  const [consumer, worker, provider, outbox, managed, main, admission] = await Promise.all([
    readFile(files.mailRuntimeAddressBookConsumer, 'utf8'),
    readFile(files.mailRuntimeAddressBookFetchWorker, 'utf8'),
    readFile(files.mailRuntimeAddressBookProvider, 'utf8'),
    readFile(files.mailRuntimeAddressBookOutbox, 'utf8'),
    readFile(files.mailRuntimeManaged, 'utf8'),
    readFile(files.mailRuntimeMain, 'utf8'),
    readFile(files.mailRuntimeAdmission, 'utf8'),
  ]);
  assert.match(consumer, /consume_next_mail_address_book_fetch_v1/);
  assert.match(consumer, /accept_fetch_command/);
  assert.match(worker, /google_people_client_v1/);
  assert.match(worker, /carddav_client_v1/);
  assert.match(provider, /GooglePeopleClientV1/);
  assert.match(provider, /CardDavClientV1/);
  assert.match(worker, /complete_fetch_command/);
  assert.match(worker, /GOOGLE_CURSOR_PREFIX/);
  assert.match(worker, /CARDDAV_CURSOR_PREFIX/);
  assert.doesNotMatch(worker, /makosh_contacts|mail_contacts_sync/);
  assert.match(outbox, /pending_fetch_events/);
  assert.match(managed, /address_book_fetch_subscribe_permit/);
  assert.match(main, /process_next_mail_address_book_fetch_v1/);
  assert.match(admission, /FetchPageCommand\.consume_request/);
  assert.match(admission, /EntryObserved\.publish_request/);
  assert.match(admission, /PageCompleted\.publish_request/);
  assert.match(admission, /PageRejected\.publish_request/);
});

test('managed Mail provider conformance keeps endpoints credentials and evidence owner-local', async () => {
  const [api, settings, worker, admission, setup, flow, googleFixture, cardDavFixture] =
    await Promise.all([
      readFile(files.mailApi, 'utf8'),
      readFile(files.mailRuntimeSettings, 'utf8'),
      readFile(files.mailRuntimeAddressBookFetchWorker, 'utf8'),
      readFile(files.mailRuntimeAdmission, 'utf8'),
      readFile(files.managedMailSetup, 'utf8'),
      readFile(files.managedProviderFlow, 'utf8'),
      readFile(files.managedGoogleFixture, 'utf8'),
      readFile(files.managedCardDavFixture, 'utf8'),
    ]);
  assert.match(api, /MailAddressBookTlsEndpointV1/);
  assert.match(api, /MailCardDavEndpointV1/);
  assert.match(api, /valid_google_people_endpoint_v1/);
  assert.match(api, /valid_carddav_endpoint_v1/);
  assert.match(settings, /mail\.address_book\.google_people_host/);
  assert.match(settings, /mail\.address_book\.carddav_host/);
  assert.match(worker, /google_people_client_v1\(endpoint\)/);
  assert.match(worker, /carddav_client_v1\(endpoint\)/);
  assert.match(admission, /provider_credential_request_v1\("mail_icloud_carddav_password"\)/);
  assert.match(setup, /GooglePeopleAddressBook/);
  assert.match(setup, /CardDavAddressBook/);
  assert.match(flow, /managed_mail_google_people_page_is_exact_restart_safe_and_private/);
  assert.match(flow, /managed_mail_carddav_page_uses_separate_credential_and_read_only_provider/);
  assert.match(flow, /MailAddressBookRejectCodeWriteScopeRequired/);
  assert.match(flow, /MailAddressBookRejectCodeReadOnlyProvider/);
  assert.match(flow, /accepted_people_writes\(\), 0/);
  assert.match(flow, /restart_mail_runtime_without_smtp/);
  assert.match(googleFixture, /\/v1\/people\/me\/connections/);
  assert.match(setup, /managed-mail-carddav-password/);
  assert.match(cardDavFixture, /authorization/);
  assert.match(cardDavFixture, /set_nonblocking\(false\)/);
  assert.doesNotMatch(cardDavFixture, /managed-mail-carddav-password/);
  assert.doesNotMatch(`${worker}\n${setup}`, /makosh_contacts_(?:runtime|persistence)/);
});

test('managed bidirectional and scheduled sync cross owners only through durable events', async () => {
  const [
    setup,
    flow,
    harness,
    mailManaged,
    schedulerExecution,
    schedulerCompletion,
    completionPersistence,
    completionMigration,
    runtimeMain,
  ] = await Promise.all([
    readFile(files.managedSyncSetup, 'utf8'),
    readFile(files.managedSyncFlow, 'utf8'),
    readFile(files.authenticatedStorageHarness, 'utf8'),
    readFile(files.mailRuntimeManaged, 'utf8'),
    readFile(files.workflowSchedulerExecution, 'utf8'),
    readFile(files.workflowSchedulerCompletion, 'utf8'),
    readFile(files.workflowScheduledCompletionPersistence, 'utf8'),
    readFile(files.workflowSchedulerCompletionMigration, 'utf8'),
    readFile(files.workflowRuntimeMain, 'utf8'),
  ]);
  assert.match(setup, /installed_mail_contacts_sync_ensemble_release_v1/);
  assert.match(setup, /mail_release_artifact/);
  assert.match(setup, /contacts_release_artifact_v1/);
  assert.match(setup, /mail_contacts_sync_release_artifact_v1/);
  assert.match(setup, /scheduler_release_artifact/);
  assert.match(setup, /start_reserved_workflow_with_settings/);
  assert.match(setup, /ClientRealtimePublishHandlerV1/);
  assert.match(flow, /managed_mail_contacts_sync_reaches_contacts_through_events/);
  assert.match(flow, /route_start/);
  assert.match(flow, /wait_for_completed/);
  assert.match(flow, /contacts_created, 1/);
  assert.match(flow, /provider\.accepted_people_reads\(\), 1/);
  assert.match(flow, /blob_launch::start_from_kernel/);
  assert.match(flow, /wait_for_people_write/);
  assert.match(flow, /write\.method, "PATCH"/);
  assert.match(flow, /managed-contact-1:updateContact/);
  assert.match(flow, /managed-etag-1/);
  assert.match(flow, /provider_entries_written, 1/);
  assert.match(flow, /wait_for_reverse_terminal/);
  assert.match(flow, /provider\.accepted_people_writes\(\), 4/);
  assert.match(flow, /scheduler_launch::start_from_reservation/);
  assert.match(flow, /ScheduleTriggerV1::FixedInterval/);
  assert.match(flow, /wait_for_scheduled_run_id/);
  assert.match(flow, /wait_for_scheduler_terminal/);
  assert.match(flow, /provider\.accepted_people_reads\(\), 2/);
  assert.match(flow, /duplicate\.run_id, accepted\.run_id/);
  assert.match(schedulerExecution, /if launch\.is_none\(\)/);
  assert.match(schedulerExecution, /lease_expires_at_unix_millis: due\.lease_expires_at_unix_millis/);
  assert.match(schedulerCompletion, /pending_scheduled_terminal/);
  assert.match(schedulerCompletion, /build_mail_contacts_sync_terminal_receipt_from_binding_v1/);
  assert.match(completionPersistence, /runs\.state IN \(6, 7\)/);
  assert.match(completionPersistence, /queue_scheduled_terminal/);
  assert.match(completionMigration, /mail_contacts_sync_scheduler_runs/);
  assert.match(completionMigration, /terminal_receipt_queued/);
  assert.match(runtimeMain, /queue_scheduler_terminal_once/);
  assert.match(harness, /makosh-mail-contacts-sync-runtime/);
  assert.match(harness, /managed_mail_contacts_sync_reaches_contacts_through_events/);
  assert.match(mailManaged, /address_book_upsert_subscribe_permit,[\s\S]*&self\.logical_human_owner_id/);
  assert.match(mailManaged, /address_book_fetch_subscribe_permit,[\s\S]*&self\.logical_human_owner_id/);
});

test('Mail provider contract and sync workflow foundation preserve owner boundaries', async () => {
  const [
    mailContractManifest,
    mailContract,
    workflowApiManifest,
    workflowApi,
    workflowCoreManifest,
    workflowCore,
  ] = await Promise.all([
    readFile(files.mailContractManifest, 'utf8'),
    readFile(files.mailContract, 'utf8'),
    readFile(files.workflowApiManifest, 'utf8'),
    readFile(files.workflowApi, 'utf8'),
    readFile(files.workflowCoreManifest, 'utf8'),
    readFile(files.workflowCore, 'utf8'),
  ]);

  assert.match(mailContractManifest, /role = "integration"/);
  assert.match(mailContractManifest, /owner = "mail"/);
  assert.doesNotMatch(mailContractManifest, /makosh-contacts|makosh-communications/);
  assert.match(mailContract, /FetchMailAddressBookPageCommandV1/);
  assert.match(mailContract, /MailAddressBookEntryObservedV1/);
  assert.match(mailContract, /UpsertMailAddressBookEntryCommandV1/);
  assert.match(mailContract, /contact_snapshot_reference_id/);
  assert.match(mailContract, /contact_snapshot_custody_source_proof/);
  const upsertCommand = mailContract
    .split('message UpsertMailAddressBookEntryCommandV1')[1]
    .split('message MailAddressBookEntryUpsertedV1')[0];
  assert.match(upsertCommand, /reserved 9, 10/);
  assert.doesNotMatch(
    upsertCommand,
    /provider_kind|provider_entry_id|expected_provider_etag|display_name|email|phone/,
  );
  assert.match(mailContract, /outcome_unknown/);
  assert.doesNotMatch(
    mailContract,
    /access_token|refresh_token|password|cookie|raw_json|raw_xml|map</,
  );

  for (const manifest of [workflowApiManifest, workflowCoreManifest]) {
    assert.match(manifest, /role = "workflow"/);
    assert.match(manifest, /owner = "mail_contacts_sync"/);
    assert.doesNotMatch(manifest, /makosh-mail-(?:runtime|persistence)|makosh-contacts-(?:runtime|persistence)/);
  }
  assert.match(workflowApi, /rpc Start/);
  assert.match(workflowApi, /rpc Get/);
  assert.match(workflowApi, /MailContactsSyncStatusChangedV1/);
  assert.doesNotMatch(workflowApi, /Poll|provider_entry_id|provider_etag|credential|map</);
  assert.match(workflowCore, /MailContactsSyncStateV1/);
  assert.match(workflowCore, /ReconcilingOutcome/);
  assert.match(workflowCore, /MAIL_CONTACTS_SYNC_MAX_CURSOR_BYTES_V1/);
  assert.doesNotMatch(workflowCore, /reqwest|sqlx|provider sdk|oauth|gateway|nats/i);
});

test('sync persistence owns atomic state relay, reverse operations and SSE replay without foreign storage', async () => {
  const [
    manifest,
    repository,
    orchestration,
    reversePersistence,
    relay,
    realtime,
    migration,
    orchestrationMigration,
    reverseMigration,
    schedulerCompletionMigration,
    reverseOriginMigration,
    providerLinkMigration,
    postgresLive,
  ] = await Promise.all([
    readFile(files.workflowPersistenceManifest, 'utf8'),
    readFile(files.workflowPersistence, 'utf8'),
    readFile(files.workflowOrchestration, 'utf8'),
    readFile(files.workflowReversePersistence, 'utf8'),
    readFile(files.workflowRelay, 'utf8'),
    readFile(files.workflowRealtime, 'utf8'),
    readFile(files.workflowMigration, 'utf8'),
    readFile(files.workflowOrchestrationMigration, 'utf8'),
    readFile(files.workflowReverseMigration, 'utf8'),
    readFile(files.workflowSchedulerCompletionMigration, 'utf8'),
    readFile(files.workflowReverseOriginMigration, 'utf8'),
    readFile(files.workflowProviderLinkMigration, 'utf8'),
    readFile(files.postgresLive, 'utf8'),
  ]);
  assert.match(manifest, /role = "workflow"/);
  assert.match(manifest, /owner = "mail_contacts_sync"/);
  assert.match(manifest, /surface = "persistence"/);
  assert.doesNotMatch(manifest, /makosh-mail-(?:runtime|persistence)|makosh-contacts-(?:runtime|persistence)/);
  assert.match(repository, /create_run/);
  assert.match(repository, /apply_transition/);
  assert.match(repository, /mail_contacts_sync_inbox/);
  assert.match(repository, /insert_outbox/);
  assert.match(repository, /insert_realtime/);
  assert.match(relay, /unpublished_commands/);
  assert.match(relay, /mark_command_published/);
  assert.match(realtime, /client_realtime_window/);
  assert.match(orchestration, /accept_provider_entry/);
  assert.match(orchestration, /accept_provider_page/);
  assert.match(orchestration, /accept_contact_outcome/);
  assert.match(orchestration, /account_pending_outcomes/);
  assert.match(reversePersistence, /accept_contact_changed_for_mail_sync/);
  assert.match(reversePersistence, /complete_contact_mail_sync_source/);
  assert.match(reversePersistence, /complete_mail_address_book_upsert/);
  assert.match(reversePersistence, /complete_contacts_provider_link/);
  assert.match(reversePersistence, /apply_provider_outcome_to_run/);
  assert.match(reversePersistence, /MailContactsSyncTransitionV1::ProviderWriteApplied/);
  for (const table of [
    'mail_contacts_sync_runs',
    'mail_contacts_sync_inbox',
    'mail_contacts_sync_outbox',
    'mail_contacts_sync_realtime',
  ]) {
    assert.match(migration, new RegExp(table));
  }
  assert.match(orchestrationMigration, /mail_contacts_sync_pages/);
  assert.match(orchestrationMigration, /mail_contacts_sync_entries/);
  assert.match(orchestrationMigration, /outcome_accounted/);
  assert.match(reverseMigration, /mail_contacts_sync_reverse_inbox/);
  assert.match(reverseMigration, /mail_contacts_sync_reverse_operations/);
  assert.match(reverseOriginMigration, /origin_run_id/);
  assert.match(reverseOriginMigration, /mail_contacts_sync_reverse_origin_run_idx/);
  assert.match(providerLinkMigration, /mail_contacts_sync_provider_link_reconciliation/);
  assert.match(providerLinkMigration, /contacts_command_message_id/);
  assert.match(schedulerCompletionMigration, /mail_contacts_sync_scheduler_runs/);
  assert.match(postgresLive, /reverse_provider_result_is_atomic_restart_safe_and_replayable/);
  assert.match(postgresLive, /commit provider result after restart/);
  assert.match(postgresLive, /terminalize late provider link without rewriting completed run/);
  assert.match(postgresLive, /MailContactsSyncStateV1::WritingProvider/);
  assert.match(postgresLive, /MailContactsSyncStateV1::Completed/);
  assert.doesNotMatch(
    `${migration}\n${orchestrationMigration}\n${reverseMigration}\n${schedulerCompletionMigration}\n${reverseOriginMigration}\n${providerLinkMigration}`,
    /makosh_data\.(?:contacts_state|contacts_provider_links|mail_accounts|communications_)/,
  );
  assert.doesNotMatch(
    `${repository}\n${orchestration}\n${reversePersistence}\n${relay}\n${realtime}`,
    /reqwest|oauth|provider sdk/i,
  );
});

test('Mail Contacts Sync failure isolation is durable and revoke-fenced', async () => {
  const [
    adr,
    flow,
    providerFixture,
    fetchWorker,
    runtimeLib,
    commands,
    providerEvents,
    runProgress,
  ] =
    await Promise.all([
    readFile(files.recoveryAdr, 'utf8'),
    readFile(files.managedSyncFlow, 'utf8'),
    readFile(files.managedGoogleFixture, 'utf8'),
    readFile(files.mailRuntimeAddressBookFetchWorker, 'utf8'),
    readFile(files.workflowRuntimeLib, 'utf8'),
    readFile(files.workflowCommands, 'utf8'),
    readFile(files.workflowProviderEvents, 'utf8'),
    readFile(files.workflowRunProgress, 'utf8'),
  ]);

  assert.match(adr, /OUTCOME_UNKNOWN/);
  assert.match(adr, /не выпускает второй[\s\S]*command/);
  assert.match(adr, /Recovery выполняется observation-first/);
  assert.match(adr, /grant epoch[\s\S]*отклоняются до provider IO/);
  assert.match(adr, /deadline 300[\s\S]*секунд/);
  assert.match(adr, /page_completed[\s\S]*PendingPrerequisites/);
  assert.match(adr, /межsubjectная задержка[\s\S]*не завершает process/);
  assert.match(adr, /canonical mutation[\s\S]*provider provenance refresh/);
  assert.match(adr, /не повышают `contact_revision`[\s\S]*feedback write/);
  assert.match(adr, /entry_digest[\s\S]*не используется как ordering revision/);
  assert.match(adr, /source_revision[\s\S]*Mail-owned observed Unix time/);
  assert.match(fetchWorker, /source_revision\(now_unix_seconds\)\?/);
  assert.doesNotMatch(fetchWorker, /source_revision\(&digest\)/);
  assert.match(providerEvents, /PendingPrerequisites/);
  assert.match(runtimeLib, /MAIL_CONTACTS_SYNC_COMMAND_DEADLINE_SECONDS_V1: i64 = 300/);
  for (const source of [commands, providerEvents, runProgress]) {
    assert.match(source, /MAIL_CONTACTS_SYNC_COMMAND_DEADLINE_SECONDS_V1/);
    assert.doesNotMatch(source, /COMMAND_DEADLINE_SECONDS_V1: i64 = 30/);
  }
  assert.match(flow, /set_authenticated_nats_container_running\(false\)/);
  assert.match(flow, /wait_for_workflow_pending_outbox/);
  assert.match(flow, /provider\.accepted_people_reads\(\), 0/);
  assert.match(flow, /provider\.accepted_people_writes\(\), 0/);
  assert.match(flow, /set_authenticated_nats_container_running\(true\)/);
  assert.match(flow, /drop_next_people_write_response/);
  assert.match(flow, /assert_latest_mail_write_is_outcome_unknown/);
  assert.match(flow, /ambiguous provider mutation must never be retried automatically/);
  assert.match(flow, /created-etag-3/);
  assert.match(flow, /transition_registration\(/);
  assert.match(flow, /ModuleRegistrationState::Revoked/);
  assert.match(flow, /assert_revoked_start_route_is_rejected/);
  assert.match(flow, /is_active\(&mail\.registration_id\)/);
  assert.match(flow, /is_active\(&contacts\.registration_id\)/);
  assert.match(providerFixture, /drop_next_people_write_response\s*\.\s*swap\(false/);
  assert.match(providerFixture, /ambiguous_people_write_committed\s*\.\s*store\(true/);
  assert.match(providerFixture, /"etag": "created-etag-3"/);
});

test('staged Contacts slice keeps six functional build units isolated', async () => {
  const [
    providerLinkAdr,
    sourceAdr,
    apiManifest,
    sourceApiManifest,
    coreManifest,
    proto,
    sourceProto,
    api,
    envelope,
    sourceApi,
    sourceEnvelope,
    core,
    identity,
    upsert,
    persistenceManifest,
    persistence,
    migration,
    sourceMigration,
    providerLinkMigration,
    providerLinkPersistence,
    runtimeManifest,
    runtimeAdmission,
    runtimeCommand,
    runtimeSource,
    runtimeProviderLink,
    managedRuntime,
    assemblyManifest,
    assembly,
    developmentRelease,
  ] =
    await Promise.all([
      readFile(files.providerLinkAdr, 'utf8'),
      readFile(files.sourceAdr, 'utf8'),
      readFile(files.apiManifest, 'utf8'),
      readFile(files.sourceApiManifest, 'utf8'),
      readFile(files.coreManifest, 'utf8'),
      readFile(files.proto, 'utf8'),
      readFile(files.sourceProto, 'utf8'),
      readFile(files.api, 'utf8'),
      readFile(files.envelope, 'utf8'),
      readFile(files.sourceApi, 'utf8'),
      readFile(files.sourceEnvelope, 'utf8'),
      readFile(files.core, 'utf8'),
      readFile(files.identity, 'utf8'),
      readFile(files.upsert, 'utf8'),
      readFile(files.persistenceManifest, 'utf8'),
      readFile(files.persistence, 'utf8'),
      readFile(files.migration, 'utf8'),
      readFile(files.sourceMigration, 'utf8'),
      readFile(files.providerLinkMigration, 'utf8'),
      readFile(files.providerLinkPersistence, 'utf8'),
      readFile(files.runtimeManifest, 'utf8'),
      readFile(files.runtimeAdmission, 'utf8'),
      readFile(files.runtimeCommand, 'utf8'),
      readFile(files.runtimeSource, 'utf8'),
      readFile(files.runtimeProviderLink, 'utf8'),
      readFile(files.managedRuntime, 'utf8'),
      readFile(files.assemblyManifest, 'utf8'),
      readFile(files.assembly, 'utf8'),
      readFile(files.developmentRelease, 'utf8'),
    ]);

  for (const manifest of [apiManifest, sourceApiManifest, coreManifest]) {
    assert.match(manifest, /role = "domain"/);
    assert.match(manifest, /owner = "contacts"/);
    assert.doesNotMatch(manifest, /makosh-mail|makosh-communications|sqlx|reqwest/);
  }
  assert.match(proto, /message UpsertContactFromMailAddressBookEntryCommandV1/);
  assert.match(proto, /message ContactUpsertedFromMailAddressBookEntryV1/);
  assert.match(proto, /message ContactUpsertFromMailAddressBookEntryRejectedV1/);
  assert.match(proto, /message BindMailAddressBookProviderLinkCommandV1/);
  assert.match(proto, /message MailAddressBookProviderLinkBoundV1/);
  assert.match(proto, /message BindMailAddressBookProviderLinkRejectedV1/);
  assert.doesNotMatch(proto, /map<|bytes payload|token|password|cookie/);
  assert.match(api, /contacts\.mail-identity\.command\.v1/);
  assert.match(envelope, /validate_envelope_v1/);
  assert.match(sourceAdr, /шестая Contacts-owned unit/);
  assert.match(sourceProto, /message ContactChangedForMailSyncV1/);
  assert.match(sourceProto, /message PrepareContactMailSyncSourceCommandV1/);
  assert.match(sourceProto, /message ContactMailSyncSourceContentV1/);
  const changedEvent = sourceProto
    .split('message ContactChangedForMailSyncV1')[1]
    .split('message PrepareContactMailSyncSourceCommandV1')[0];
  assert.doesNotMatch(changedEvent, /display_name|email|phone|provider_kind|provider_entry_id|etag/);
  assert.match(sourceApi, /CONTACT_MAIL_SYNC_SOURCE_BLOB_TARGET_OWNER_ID_V1: &str = "mail"/);
  assert.match(sourceApi, /CONTACT_MAIL_SYNC_SOURCE_BLOB_TARGET_MODULE_ID_V1: &str = "makosh-mail-runtime"/);
  assert.match(sourceApi, /mail\.address-book\.contact-source\.blob\.v1/);
  assert.match(sourceEnvelope, /Semantics::Event\(EventMetadataV1/);
  assert.match(sourceEnvelope, /Semantics::Command\(CommandMetadataV1/);
  assert.match(core, /mod identity;[\s\S]*mod model;[\s\S]*mod upsert;/);
  assert.match(identity, /normalize_email_v1/);
  assert.match(identity, /normalize_phone_v1/);
  assert.match(upsert, /IdentityAmbiguous/);
  assert.match(upsert, /ProviderLinkConflict/);
  assert.match(upsert, /refreshed\.provenance = normalized\.provenance/);
  assert.match(upsert, /provider_fence_refresh_does_not_change_canonical_contact_revision/);
  assert.doesNotMatch(
    `${core}\n${identity}\n${upsert}`,
    /provider sdk|oauth|postgres|gateway|nats|communications/i,
  );
  assert.match(persistenceManifest, /role = "domain"/);
  assert.match(persistenceManifest, /owner = "contacts"/);
  assert.match(persistenceManifest, /surface = "persistence"/);
  assert.doesNotMatch(persistenceManifest, /makosh-mail|makosh-communications/);
  assert.match(persistence, /reserve_inbox/);
  assert.match(persistence, /reserve_contact_mail_sync_source/);
  assert.match(persistence, /persist_contact_mail_sync_source_result/);
  assert.match(persistence, /persist_contact/);
  assert.match(persistence, /Provider provenance has an independent freshness lifecycle/);
  assert.match(persistence, /persist_provider_link\(&mut transaction, &contact\)\.await\?/);
  assert.match(persistence, /insert_outbox/);
  assert.match(migration, /contacts_mail_entry_inbox/);
  assert.match(migration, /contacts_provider_links/);
  assert.match(migration, /contacts_outbox/);
  assert.match(sourceMigration, /contacts_mail_sync_source_inbox/);
  assert.match(providerLinkMigration, /contacts_mail_provider_link_inbox/);
  assert.match(providerLinkPersistence, /bind_mail_provider_link/);
  assert.match(providerLinkPersistence, /ON CONFLICT[\s\S]*DO UPDATE SET provider_etag/);
  assert.doesNotMatch(migration, /mail_credential|communications_|tasks_|review_/);
  for (const manifest of [runtimeManifest, assemblyManifest]) {
    assert.match(manifest, /role = "domain"/);
    assert.match(manifest, /owner = "contacts"/);
    assert.doesNotMatch(manifest, /makosh-mail|makosh-communications/);
  }
  assert.match(runtimeManifest, /surface = "runtime"/);
  assert.match(assemblyManifest, /surface = "assembly"/);
  assert.match(runtimeAdmission, /ModuleKindV1::Domain/);
  assert.match(runtimeAdmission, /ProvidedSurfaceKindV1::DurableConsumer/);
  assert.match(runtimeAdmission, /ProvidedSurfaceKindV1::DurablePublisher/);
  assert.match(runtimeAdmission, /StorageNamespaceRequestV1/);
  assert.doesNotMatch(runtimeAdmission, /ClientRpc|RequestRpc|QueryRpc/);
  assert.match(runtimeCommand, /consume_contacts_command_once_v1/);
  assert.match(runtimeCommand, /reject_mail_entry/);
  assert.match(runtimeCommand, /delivery\.acknowledge\(\)\.await/);
  assert.match(runtimeSource, /consume_contact_mail_sync_source_once_v1/);
  assert.ok(
    runtimeSource.indexOf('reserve_contact_mail_sync_source')
      < runtimeSource.indexOf('contact_mail_sync_source_snapshot'),
  );
  assert.match(runtimeSource, /request_managed_blob_session_v2/);
  assert.match(runtimeSource, /BlobDataOperationV1::BlobDataOperationWriteV1/);
  assert.match(runtimeSource, /CONTACT_MAIL_SYNC_SOURCE_BLOB_TARGET_CAPABILITY_ID_V1/);
  assert.match(runtimeProviderLink, /consume_bind_mail_provider_link_once_v1/);
  assert.match(runtimeProviderLink, /bind_mail_provider_link/);
  assert.match(runtimeProviderLink, /delivery\.acknowledge\(\)\.await/);
  assert.match(providerLinkAdr, /event-only/i);
  assert.match(managedRuntime, /StorageVaultLeaseAdapterV1/);
  assert.match(managedRuntime, /connect_runtime_with_jwt/);
  assert.match(managedRuntime, /signal_ready/);
  assert.doesNotMatch(managedRuntime, /makosh_mail|makosh_communications/);
  assert.match(assembly, /Unsigned Contacts release assembly/);
  assert.match(assembly, /contacts_storage_bundle_v1/);
  assert.match(assembly, /materialize_contacts_release_assembly_v1/);
  assert.doesNotMatch(assembly, /sign_release|launch_managed|KernelReleaseAuthorityV1/);
  assert.match(developmentRelease, /--package makosh-contacts-runtime/);
  assert.match(developmentRelease, /--package makosh-contacts-assembly/);
  assert.match(
    developmentRelease,
    /--artifact-fragment "\$contacts_assembly\/contacts\.release-artifacts\.json"/,
  );
});

test('Mail Contacts Sync frontend is app-composed and uses generated Start/Get with shared SSE', async () => {
  const [composition, panel, query, api, commandClient, queryClient, generated] =
    await Promise.all([
      readFile(new URL('frontend/src/app/settings/MailSettingsComposition.vue', PROJECT_ROOT), 'utf8'),
      readFile(new URL('frontend/src/workflows/mail-contacts-sync/presentation/MailContactsSyncSettingsPanel.vue', PROJECT_ROOT), 'utf8'),
      readFile(new URL('frontend/src/workflows/mail-contacts-sync/queries/useMailContactsSyncSettings.ts', PROJECT_ROOT), 'utf8'),
      readFile(new URL('frontend/src/workflows/mail-contacts-sync/api/mailContactsSync.ts', PROJECT_ROOT), 'utf8'),
      readFile(new URL('frontend/src/platform/connect/mailContactsSyncCommandClient.ts', PROJECT_ROOT), 'utf8'),
      readFile(new URL('frontend/src/platform/connect/mailContactsSyncQueryClient.ts', PROJECT_ROOT), 'utf8'),
      readFile(new URL('frontend/src/gen/makosh/mail_contacts_sync/v1/sync_pb.ts', PROJECT_ROOT), 'utf8'),
    ]);

  assert.match(composition, /MailSettingsPanel/);
  assert.match(composition, /MailContactsSyncSettingsPanel/);
  assert.doesNotMatch(composition, /setInterval|setTimeout/);
  assert.match(panel, /Mail account/);
  assert.match(panel, /Apply configuration/);
  assert.match(panel, /Sync now/);
  assert.match(query, /await realtime\.ready/);
  assert.ok(query.indexOf('await realtime.ready') < query.indexOf('await startMailContactsSync'));
  assert.match(query, /realtime\.attachRun\(runId\)/);
  assert.match(query, /status\.value = await getMailContactsSync\(runId\)/);
  assert.doesNotMatch(query, /setInterval|setTimeout|poll/i);
  assert.match(api, /getBrowserGatewayRealtimeHub/);
  assert.match(api, /mail_contacts_sync_realtime/);
  assert.match(api, /mail\.contacts-sync\.status-changed\.v1/);
  assert.match(commandClient, /MailContactsSyncCommandService/);
  assert.match(queryClient, /MailContactsSyncQueryService/);
  assert.match(generated, /MailContactsSyncCommandService/);
  assert.match(generated, /MailContactsSyncQueryService/);
});
