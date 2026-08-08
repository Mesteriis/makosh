import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const REPOSITORY_ROOT = new URL('../../../', import.meta.url);

async function backendSource(path) {
  return readFile(new URL(path, BACKEND_ROOT), 'utf8');
}

test('Review attention contract and core are separate owner units', async () => {
  const [apiManifest, coreManifest, api, proto, core] = await Promise.all([
    backendSource('src/review-attention-api/Cargo.toml'),
    backendSource('src/review-attention-core/Cargo.toml'),
    backendSource('src/review-attention-api/src/lib.rs'),
    backendSource(
      'src/review-attention-api/proto/makosh/review/attention/client/v1/client.proto',
    ),
    backendSource('src/review-attention-core/src/lib.rs'),
  ]);

  for (const manifest of [apiManifest, coreManifest]) {
    assert.match(manifest, /role = "domain"/);
    assert.match(manifest, /owner = "review"/);
    assert.doesNotMatch(
      manifest,
      /communications-|mail-|telegram-|whatsapp-|zulip-|sqlx|kernel|gateway/,
    );
  }
  assert.match(apiManifest, /surface = "contract"/);
  assert.match(coreManifest, /surface = "implementation"/);
  assert.match(api, /review\.communication-attention\.command\.v1/);
  assert.match(api, /review\.communication-attention\.query\.v1/);
  assert.match(api, /review\.communication-attention\.realtime\.v1/);
  assert.match(proto, /oneof operation/);
  assert.match(proto, /bytes source_evidence_id = 3/);
  assert.match(proto, /uint64 expected_revision = 4/);
  const coreProduction = core.replace(/#\[cfg\(test\)\][\s\S]*$/u, '');
  assert.doesNotMatch(
    `${proto}\n${coreProduction}`,
    /provider_call|provider_account|message_body|email_address|phone_number|google\.protobuf\.Any|map</,
  );
});

test('Review attention core owns optimistic revision and bounded snooze invariants', async () => {
  const core = await backendSource('src/review-attention-core/src/lib.rs');
  assert.match(core, /current\.revision != request\.expected_revision/);
  assert.match(core, /current\.source_evidence_id != request\.source_evidence_id/);
  assert.match(core, /MAX_SNOOZE_SECONDS_V1/);
  assert.match(core, /DismissedAttention/);
  assert.match(core, /attention\.pinned = false/);
  assert.match(core, /attention\.snoozed_until = None/);
  assert.match(core, /if changed \{/);
  assert.doesNotMatch(core.replace(/#\[cfg\(test\)\][\s\S]*$/u, ''), /serde_json|sqlx|tokio|prost/);
});

test('Review attention persistence is owner-local atomic and operation-idempotent', async () => {
  const [manifest, repository, query, realtime, schema, migration, realtimeMigration] = await Promise.all([
    backendSource('src/review-attention-persistence/Cargo.toml'),
    backendSource('src/review-attention-persistence/src/repository.rs'),
    backendSource('src/review-attention-persistence/src/query.rs'),
    backendSource('src/review-attention-persistence/src/realtime.rs'),
    backendSource('src/review-attention-persistence/src/schema.rs'),
    backendSource(
      'src/review-attention-persistence/migrations/0001_review_attention.sql',
    ),
    backendSource(
      'src/review-attention-persistence/migrations/0002_review_attention_realtime.sql',
    ),
  ]);

  assert.match(manifest, /role = "domain"/);
  assert.match(manifest, /owner = "review"/);
  assert.match(manifest, /surface = "persistence"/);
  assert.match(manifest, /makosh-review-attention-core/);
  assert.match(manifest, /makosh-storage-protocol/);
  assert.doesNotMatch(manifest, /communications-|mail-|telegram-|whatsapp-|zulip-/);
  assert.match(repository, /\.pool\s*\.begin\(\)/);
  assert.match(repository, /ON CONFLICT \(logical_owner_id, operation_id\) DO NOTHING/);
  assert.match(repository, /request_sha256/);
  assert.match(repository, /stored_sha256\.as_slice\(\) != request_sha256/);
  assert.match(repository, /FOR UPDATE/);
  assert.match(repository, /outcome\.changed/);
  assert.match(repository, /insert_realtime_transition/);
  assert.match(repository, /transaction\.commit\(\)/);
  assert.match(query, /ORDER BY attention_id ASC/);
  assert.match(query, /REVIEW_ATTENTION_MAX_PAGE_SIZE_V1: u16 = 100/);
  assert.match(realtime, /realtime_sequence > \$2/);
  assert.match(realtime, /\.bind\(logical_owner_id\)/);
  assert.match(realtime, /REVIEW_ATTENTION_REALTIME_REPLAY_LIMIT_V1: u16 = 256/);
  assert.match(schema, /owner_id: "review"/);
  assert.match(migration, /makosh_data\.review_attention_state/);
  assert.match(migration, /makosh_data\.review_attention_operations/);
  assert.match(migration, /expected_revision BIGINT NOT NULL/);
  assert.match(realtimeMigration, /makosh_data\.review_attention_realtime/);
  assert.match(realtimeMigration, /UNIQUE \(logical_owner_id, attention_id, state_revision\)/);
  assert.doesNotMatch(
    `${repository}\n${query}\n${realtime}\n${migration}\n${realtimeMigration}`,
    /communications_|mail_|telegram_|provider_|message_body|subject|email_address|phone_number/,
  );
});

test('Review managed runtime owns exact client dispatch and shared realtime replay', async () => {
  const [manifest, admission, clientPort, managedRuntime, realtime, main] = await Promise.all([
    backendSource('src/review-attention-runtime/Cargo.toml'),
    backendSource('src/review-attention-runtime/src/admission.rs'),
    backendSource('src/review-attention-runtime/src/client_port.rs'),
    backendSource('src/review-attention-runtime/src/managed_runtime.rs'),
    backendSource('src/review-attention-runtime/src/realtime.rs'),
    backendSource('src/review-attention-runtime/src/main.rs'),
  ]);

  assert.match(manifest, /role = "domain"/);
  assert.match(manifest, /owner = "review"/);
  assert.match(manifest, /surface = "runtime"/);
  assert.doesNotMatch(
    manifest,
    /communications-|mail-|telegram-|whatsapp-|zulip-|events-jetstream/,
  );
  assert.match(admission, /ModuleKindV1::Domain/);
  assert.match(admission, /REVIEW_ATTENTION_COMMAND_CAPABILITY_ID_V1/);
  assert.match(admission, /REVIEW_ATTENTION_QUERY_CAPABILITY_ID_V1/);
  assert.match(admission, /REVIEW_ATTENTION_REALTIME_CAPABILITY_ID_V1/);
  assert.match(admission, /REVIEW_ATTENTION_STORAGE_CAPABILITY_ID_V1/);
  assert.match(clientPort, /command_payload_v1/);
  assert.match(clientPort, /query_payload_v1/);
  assert.match(managedRuntime, /logical_human_owner_id/);
  assert.match(managedRuntime, /logical_human_owner_id == admission\.logical_owner_id/);
  assert.match(managedRuntime, /StorageVaultLeaseAdapterV1/);
  assert.match(managedRuntime, /request\.logical_owner_id == admission\.logical_human_owner_id/);
  assert.match(managedRuntime, /ReviewAttentionNestedRequestDispatcherV1/);
  assert.match(managedRuntime, /MAX_NESTED_REALTIME_PASSES_V1: u8 = 8/);
  assert.match(realtime, /request_next_with_dispatch/);
  assert.match(realtime, /review-attention\/\{\}/);
  assert.match(managedRuntime, /\.receive_request\(\)/);
  assert.doesNotMatch(managedRuntime, /try_receive_request|set_nonblocking/);
  assert.doesNotMatch(main, /sleep|pump_client_realtime_once/);
  assert.match(main, /ManagedDomainRuntimeConfigurationV1/);
  assert.doesNotMatch(
    `${admission}\n${clientPort}\n${managedRuntime}\n${realtime}\n${main}`,
    /makosh_communications|communication_observed|Event Hub|event_hub_endpoint|JetStream|provider_account/,
  );
});

test('Review release assembly is a separate unsigned domain build unit', async () => {
  const [manifest, assembly, main] = await Promise.all([
    backendSource('src/review-attention-assembly/Cargo.toml'),
    backendSource('src/review-attention-assembly/src/lib.rs'),
    backendSource('src/review-attention-assembly/src/main.rs'),
  ]);

  assert.match(manifest, /role = "domain"/);
  assert.match(manifest, /owner = "review"/);
  assert.match(manifest, /surface = "assembly"/);
  assert.match(manifest, /makosh-review-attention-persistence/);
  assert.match(manifest, /makosh-review-attention-runtime/);
  assert.doesNotMatch(manifest, /kernel|gateway|communications-|mail-|telegram-|whatsapp-|zulip-/);
  assert.match(assembly, /review_attention_module_descriptor_v1/);
  assert.match(assembly, /review_attention_settings_schema_v1/);
  assert.match(assembly, /review_attention_storage_bundle_v1/);
  assert.match(assembly, /create_new\(true\)/);
  assert.match(assembly, /"module_runtime"/);
  assert.match(assembly, /"storage_bundle"/);
  assert.match(main, /materialize_review_attention_release_assembly_v1/);
  assert.doesNotMatch(
    `${assembly}\n${main}`,
    /signing_key|private_key|DistributionManifestV1|ManagedControlChannel|serve-inherited/,
  );
});

test('Review owner is admitted through signed Kernel Gateway and shared SSE conformance', async () => {
  const [
    adr,
    eventlessAdr,
    inventorySource,
    policySource,
    domainValidation,
    ownerControl,
    liveSetup,
    liveFlow,
  ] = await Promise.all([
    readFile(
      new URL(
        'docs/adr/ADR-0351-review-communications-attention-owner-admission.md',
        REPOSITORY_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'docs/adr/ADR-0352-capability-scoped-domain-event-hub-launch-configuration.md',
        REPOSITORY_ROOT,
      ),
      'utf8',
    ),
    backendSource('architecture/communications-settings-reconstruction.json'),
    backendSource('architecture/policy.json'),
    backendSource(
      'src/platform/runtime_protocol/src/validation/managed_domain_runtime.rs',
    ),
    backendSource('src/kernel/src/identity/owner_control/dispatch.rs'),
    backendSource(
      'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/review_attention_managed_setup.rs',
    ),
    backendSource(
      'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/review_attention_managed_flow.rs',
    ),
  ]);
  const inventory = JSON.parse(inventorySource);
  const policy = JSON.parse(policySource);
  const gate = inventory.slices.find(
    ({ gate: name }) => name === 'review_communications_attention_v1',
  );

  assert.deepEqual(gate, {
    gate: 'review_communications_attention_v1',
    role: 'domain',
    owner: 'review',
    state: 'implemented',
    dependsOn: ['communications_canonical_read_v2'],
  });
  assert.equal(
    policy.implementation.currentSlice,
    'call_transcription_managed_conformance_v1',
  );
  assert.deepEqual(policy.implementation.ownerInventory.domains, [
    'communications',
    'contacts',
    'knowledge',
    'review',
    'tasks',
  ]);
  assert.deepEqual(
    policy.implementation.ownerInventory.businessCapabilities.filter(
      (capability) => capability.startsWith('review.communication-attention.'),
    ),
    [
      'review.communication-attention.command.v1',
      'review.communication-attention.query.v1',
      'review.communication-attention.realtime.v1',
      'review.communication-attention.storage.v1',
    ],
  );
  assert.match(adr, /Review packages не зависят от Communications packages/);
  assert.match(adr, /`review_communications_attention_v1` открыт как implemented/);
  assert.match(adr, /operation ID вместе с exact request hash/);
  assert.match(adr, /live managed proof through Gateway and shared SSE/);
  assert.match(eventlessAdr, /eventless: endpoint == "" and credential_revision == 0/);
  assert.match(domainValidation, /valid_event_hub_configuration/);
  assert.match(domainValidation, /endpoint\.is_empty\(\) && credential_revision == 0/);
  assert.match(ownerControl, /domain_event_hub_configuration/);
  assert.match(ownerControl, /module_event_route_requests/);
  assert.match(liveSetup, /event_hub_endpoint: String::new\(\)/);
  assert.match(liveSetup, /event_credential_revision: 0/);
  assert.match(liveFlow, /stale_response\.error_code, "stale_revision"/);
  assert.match(liveFlow, /windows\(source_evidence_id\.len\(\)\)/);
  assert.match(liveFlow, /assert_eq!\(replayed\.cursor, first_cursor\)/);
});
