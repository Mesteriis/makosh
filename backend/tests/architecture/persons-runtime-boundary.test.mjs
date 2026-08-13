import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import test from 'node:test';

const PROJECT_ROOT = resolve(import.meta.dirname, '../../..');
const read = (path) => readFile(resolve(PROJECT_ROOT, path), 'utf8');

test('Task 6 admits the exact Persons production package family', async () => {
  const policy = JSON.parse(await read('backend/architecture/policy.json'));
  assert.deepEqual(
    policy.implementation.productionPackages.filter(({ owner }) => owner === 'persons'),
    [
      { name: 'makosh-persons-api', role: 'domain', owner: 'persons', surface: 'contract' },
      { name: 'makosh-persons-core', role: 'domain', owner: 'persons', surface: 'implementation' },
      { name: 'makosh-persons-persistence', role: 'domain', owner: 'persons', surface: 'persistence' },
      { name: 'makosh-persons-runtime', role: 'domain', owner: 'persons', surface: 'runtime' },
      { name: 'makosh-persons-assembly', role: 'domain', owner: 'persons', surface: 'assembly' },
    ],
  );
  assert.equal(policy.implementation.productionPackages.length, 283);
  assert.equal(policy.implementation.currentSlice, 'speech_to_text_whisper_admission_v1');
  assert.equal(policy.implementation.ownerInventory.domains.includes('persons'), true);
});

test('Persons transport uses one bounded discriminated command and sanitized event unions', async () => {
  const proto = await read('backend/src/persons-api/proto/makosh/persons/v1/persons.proto');
  const command = proto.split('message PersonsCommandV1').at(1)?.split('message ReadPersonDirectoryRequestV1').at(0) ?? '';
  assert.match(command, /oneof command/);
  for (const name of [
    'ManualCreatePersonCommandV1', 'UpdatePersonOwnerProfileCommandV1',
    'ObserveProviderSourceContactCommandV1', 'UpdateProviderSourceContactCommandV1',
    'RemoveProviderSourceContactCommandV1', 'ConfirmAttachPersonSourceCommandV1',
    'ConfirmDetachPersonSourceCommandV1', 'ConfirmMergePersonsCommandV1',
    'ConfirmSplitPersonCommandV1',
  ]) assert.match(command, new RegExp(name));
  assert.match(proto, /message PersonsOwnerEventV1[\s\S]*oneof event/);
  assert.match(proto, /message PersonCommandSucceededV1/);
  assert.match(proto, /message PersonCommandRejectedV1/);
  for (const event of [
    'PersonChangedEventV1',
    'PersonProfileChangedEventV1',
    'PersonSourceLinkChangedEventV1',
    'PersonLineageChangedEventV1',
    'PersonReviewCandidateRaisedEventV1',
  ]) {
    const body = proto.split(`message ${event}`).at(1)?.split(/\nmessage |\nenum /).at(0) ?? '';
    assert.match(body, /uint64 resulting_owner_revision = \d+;/, `${event} owner revision`);
  }
  assert.doesNotMatch(proto, /google\.protobuf\.Any|map<|json|error_detail/i);
});

test('Persons runtime is one managed owner process with exact command and publish routes', async () => {
  const [manifest, admission, command, relay, persistence, managed, binary, runner, kernelContour, eventHub] = await Promise.all([
    read('backend/src/persons-runtime/Cargo.toml'),
    read('backend/src/persons-runtime/src/admission.rs'),
    read('backend/src/persons-runtime/src/command.rs'),
    read('backend/src/persons-runtime/src/event_outbox.rs'),
    read('backend/src/persons-persistence/src/repository.rs'),
    read('backend/src/persons-runtime/src/managed_runtime.rs'),
    read('backend/src/persons-runtime/src/main.rs'),
    read('backend/scripts/test-authenticated-storage.mjs'),
    read('backend/tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/persons_managed_flow.rs'),
    read('backend/src/platform/events/jetstream/src/connection/event_hub.rs'),
  ]);
  assert.match(manifest, /role = "domain"[\s\S]*owner = "persons"[\s\S]*surface = "runtime"/);
  assert.doesNotMatch(manifest, /contacts|mail-|provider-|blob/);
  assert.match(admission, /ProvidedSurfaceKindV1::DurableConsumer/);
  assert.match(admission, /ProvidedSurfaceKindV1::DurablePublisher/);
  assert.match(admission, /StorageNamespaceRequestV1/);
  assert.match(admission, /client_rpc_route:\s*Some/);
  assert.doesNotMatch(admission, /BlobQuota|client_blob_route:\s*Some/);
  assert.match(command, /ManualCreate[\s\S]*OwnerProfileUpdate[\s\S]*SourceObserve[\s\S]*SourceUpdate[\s\S]*SourceRemove[\s\S]*ConfirmedAttach[\s\S]*ConfirmedDetach[\s\S]*ConfirmedMerge[\s\S]*ConfirmedSplit/);
  assert.match(relay, /mark_outbox_published[\s\S]*envelope_sha256/);
  assert.match(persistence, /ORDER BY resulting_owner_revision, created_at_unix_millis, command_message_id, semantic_order_key/);
  assert.match(persistence, /outbox_ordinal/);
  assert.match(managed, /pump_control_once/);
  assert.match(managed, /tokio::select!/);
  assert.ok(managed.indexOf('open_pull_consumer') < managed.indexOf('signal_ready'));
  assert.match(managed, /peer_closed_preserving_frames/);
  assert.match(binary, /serve-inherited/);
  assert.doesNotMatch(binary, /PersonsManagedRuntimeErrorV1::Unavailable[\s\S]{0,120}=>\s*Ok\(\(\)\)/);
  assert.match(runner, /makosh-persons-runtime/);
  assert.match(runner, /MAKOSH_PERSONS_RUNTIME_BIN/);
  assert.match(runner, /managed_persons_command_is_atomic_replayable_restart_and_control_close_safe/);
  assert.match(runner, /managed_persons_bootstrap_is_control_responsive_and_requires_exact_consumer/);
  assert.match(kernelContour, /StopVaultAfterConfiguration/);
  assert.match(kernelContour, /UnavailableStoragePort/);
  assert.match(kernelContour, /set_authenticated_nats_container_running\(false\)/);
  assert.match(kernelContour, /missing consumer topology/);
  assert.match(kernelContour, /PersonsConsumerDriftV1::DeliverPolicyNew/);
  assert.match(kernelContour, /PersonsConsumerDriftV1::RetryBackoff/);
  assert.match(kernelContour, /PersonsConsumerDriftV1::HeadersOnly/);
  assert.match(kernelContour, /assert_topology_never_ready_before_stop_v1/);
  assert.match(eventHub, /actual == &expected\.into_consumer_config\(\)/);
  assert.match(kernelContour, /stop_if_active/);
  assert.match(kernelContour, /Duration::from_secs\(2\)/);
  assert.match(kernelContour, /persons_command_inbox/);
  assert.match(kernelContour, /persons_outbox/);
});

test('Persons assembly is unsigned private deterministic and overwrite-safe', async () => {
  const [manifest, assembly] = await Promise.all([
    read('backend/src/persons-assembly/Cargo.toml'),
    read('backend/src/persons-assembly/src/lib.rs'),
  ]);
  assert.match(manifest, /role = "domain"[\s\S]*owner = "persons"[\s\S]*surface = "assembly"/);
  assert.match(assembly, /create_new\(true\)/);
  assert.match(assembly, /mode\(0o700\)/);
  assert.match(assembly, /mode\(0o600\)/);
  assert.match(assembly, /symlink_metadata/);
  assert.match(assembly, /remove_dir_all/);
  assert.doesNotMatch(assembly, /\bsign(?:ed|ing)?\s*\(|signed_release|production_release/i);
});
