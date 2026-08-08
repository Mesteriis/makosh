import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import test from 'node:test';

const PROJECT_ROOT = resolve(import.meta.dirname, '../../..');

const read = (path) => readFile(resolve(PROJECT_ROOT, path), 'utf8');

test('Speech-to-Text contract and core are separate engine build units', async () => {
  const [policySource, apiManifest, coreManifest] = await Promise.all([
    read('backend/architecture/policy.json'),
    read('backend/src/speech-to-text-api/Cargo.toml'),
    read('backend/src/speech-to-text-core/Cargo.toml'),
  ]);
  const policy = JSON.parse(policySource);
  const packages = new Map(
    policy.implementation.productionPackages.map((descriptor) => [descriptor.name, descriptor]),
  );

  assert.deepEqual(packages.get('makosh-speech-to-text-api'), {
    name: 'makosh-speech-to-text-api',
    role: 'engine',
    owner: 'speech_to_text',
    surface: 'contract',
  });
  assert.deepEqual(packages.get('makosh-speech-to-text-core'), {
    name: 'makosh-speech-to-text-core',
    role: 'engine',
    owner: 'speech_to_text',
    surface: 'implementation',
  });
  assert.match(apiManifest, /owner = "speech_to_text"/);
  assert.match(apiManifest, /makosh-runtime-protocol/);
  assert.doesNotMatch(apiManifest, /communications|call-transcription|whisper|sqlx/);
  assert.match(coreManifest, /owner = "speech_to_text"/);
  assert.match(coreManifest, /\[dependencies\]\s*$/);
});

test('Speech-to-Text wire contract carries receipts and never private text or provider choice', async () => {
  const [proto, apiSource, validationSource, coreSource, policySource] = await Promise.all([
    read('backend/src/speech-to-text-api/proto/makosh/speech_to_text/v1/speech_to_text.proto'),
    read('backend/src/speech-to-text-api/src/lib.rs'),
    read('backend/src/speech-to-text-api/src/validation.rs'),
    read('backend/src/speech-to-text-core/src/lib.rs'),
    read('backend/architecture/policy.json'),
  ]);

  for (const required of [
    'SpeechAudioSourceReceiptV1',
    'SpeechTranscriptArtifactReceiptV1',
    'consent_receipt_id',
    'consent_policy_revision',
    'request_digest',
    'custody_transfer_source_proof',
  ]) {
    assert.match(proto, new RegExp(`\\b${required}\\b`));
  }
  for (const forbidden of [
    'transcript_text',
    'segment_text',
    'summary',
    'sender',
    'subject',
    'body_utf8',
    'provider_name',
    'model_name',
    'filesystem_path',
    'map<',
  ]) {
    assert.ok(!proto.includes(forbidden), `forbidden wire token ${forbidden}`);
  }
  assert.match(apiSource, /speech_to_text\.transcribe\.v1/);
  assert.match(apiSource, /speech_to_text\.provider_transcribe/);
  assert.match(validationSource, /custody_transfer_source_proof\.clear\(\)/);
  assert.match(validationSource, /makosh\.speech-to-text\.request\.v1/);
  assert.match(coreSource, /RequestResultMismatch/);
  assert.match(coreSource, /InvalidConsent/);
  assert.doesNotMatch(coreSource, /makosh_communications|makosh_call_transcription|whisper/);
  const policy = JSON.parse(policySource);
  assert.ok(
    policy.dependencies.integrationEngineContractPackages.includes('makosh-speech-to-text-api'),
  );
});

test('Speech-to-Text persistence is an owner-local engine unit without content or custody secrets', async () => {
  const [policySource, manifest, migration, repository] = await Promise.all([
    read('backend/architecture/policy.json'),
    read('backend/src/speech-to-text-persistence/Cargo.toml'),
    read('backend/src/speech-to-text-persistence/migrations/0001_speech_to_text.sql'),
    read('backend/src/speech-to-text-persistence/src/repository.rs'),
  ]);
  const policy = JSON.parse(policySource);
  const descriptor = policy.implementation.productionPackages.find(
    (candidate) => candidate.name === 'makosh-speech-to-text-persistence',
  );

  assert.deepEqual(descriptor, {
    name: 'makosh-speech-to-text-persistence',
    role: 'engine',
    owner: 'speech_to_text',
    surface: 'persistence',
  });
  assert.match(manifest, /makosh-speech-to-text-core/);
  assert.match(manifest, /makosh-storage-protocol/);
  assert.doesNotMatch(manifest, /communications|call-transcription|whisper/);
  for (const source of [migration, repository.split('#[cfg(test)]')[0]]) {
    for (const forbidden of [
      'audio_bytes',
      'transcript_text',
      'segment_text',
      'custody_proof',
      'provider_name',
      'model_name',
      'filesystem_path',
    ]) {
      assert.ok(!source.includes(forbidden), `forbidden persistence token ${forbidden}`);
    }
  }
  assert.match(repository, /load_recoverable_runs/);
  assert.match(repository, /state_revision/);
});

test('Speech-to-Text runtime and assembly are isolated provider-neutral engine build units', async () => {
  const [
    runtimeManifest,
    runtimeAdmission,
    runtimeWorker,
    managedPorts,
    assemblyManifest,
    assembly,
  ] = await Promise.all([
    read('backend/src/speech-to-text-runtime/Cargo.toml'),
    read('backend/src/speech-to-text-runtime/src/admission.rs'),
    read('backend/src/speech-to-text-runtime/src/worker.rs'),
    read('backend/src/speech-to-text-runtime/src/managed_ports.rs'),
    read('backend/src/speech-to-text-assembly/Cargo.toml'),
    read('backend/src/speech-to-text-assembly/src/lib.rs'),
  ]);

  for (const manifest of [runtimeManifest, assemblyManifest]) {
    assert.match(manifest, /role = "engine"/);
    assert.match(manifest, /owner = "speech_to_text"/);
    assert.doesNotMatch(manifest, /makosh-communications|call-transcription|whisper/);
  }
  assert.match(runtimeAdmission, /ModuleKindV1::Engine/);
  assert.match(runtimeAdmission, /speech_to_text_provider_contract_reference_v1/);
  assert.doesNotMatch(runtimeAdmission, /provider_name|whisper/);
  assert.match(managedPorts, /request_managed_blob_resolved_provider_custody_delegation_v1/);
  assert.match(managedPorts, /request_managed_blob_custody_delegation_v2/);
  assert.match(managedPorts, /response_blob_capability_id:\s*SPEECH_TO_TEXT_BLOB_CAPABILITY_ID_V1/);
  assert.match(runtimeWorker, /require_persisted_ready_match/);
  assert.doesNotMatch(runtimeWorker, /transcript_text|audio_bytes|provider_name|filesystem_path/);
  assert.match(assembly, /Unsigned Speech-to-Text engine release assembly/);
  assert.match(assembly, /speech_to_text_storage_bundle_v1/);
  assert.doesNotMatch(assembly.split('#[cfg(test)]')[0], /Command::new|std::process|whisper/);
});

test('reconstruction keeps recording engine provider and workflow as four independent gates', async () => {
  const [inventorySource, adr] = await Promise.all([
    read('backend/architecture/communications-settings-reconstruction.json'),
    read('docs/adr/ADR-0390-call-recording-custody-and-speech-to-text-boundary.md'),
  ]);
  const inventory = JSON.parse(inventorySource);
  const gates = new Map(inventory.slices.map((slice) => [slice.gate, slice]));

  assert.equal(gates.get('desktop_call_recording_v1').role, 'integration');
  assert.equal(gates.get('speech_to_text_engine_v1').role, 'engine');
  assert.equal(gates.get('whisper_stt_provider_v1').role, 'integration');
  assert.deepEqual(gates.get('call_transcription_v1').dependsOn, [
    'communications_call_evidence_v1',
    'desktop_call_recording_v1',
    'speech_to_text_engine_v1',
    'whisper_stt_provider_v1',
    'blob_v1',
    'client_gateway_v1',
  ]);
  assert.equal(gates.get('desktop_call_recording_v1').state, 'implemented');
  assert.equal(gates.get('speech_to_text_engine_v1').state, 'implemented');
  assert.equal(gates.get('whisper_stt_provider_v1').state, 'implemented');
  assert.equal(gates.get('call_transcription_v1').state, 'implemented');
  assert.match(adr, /Generic `ai\.inference`, Ollama text generation/);
  assert.match(adr, /fixture provider\. Production admission/);
});
