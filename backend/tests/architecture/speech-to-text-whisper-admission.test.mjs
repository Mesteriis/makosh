import assert from 'node:assert/strict';
import { globSync, readFileSync } from 'node:fs';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import test from 'node:test';

const PROJECT_ROOT = resolve(import.meta.dirname, '../../..');
const absolute = (path) => resolve(PROJECT_ROOT, path);
const read = (path) => readFile(absolute(path), 'utf8');

const exactSpeechToTextPackages = [
  'makosh-speech-to-text-api:contract',
  'makosh-speech-to-text-core:implementation',
  'makosh-speech-to-text-persistence:persistence',
  'makosh-speech-to-text-runtime:runtime',
  'makosh-speech-to-text-assembly:assembly',
  'makosh-speech-transcript-artifact:contract',
];

const exactWhisperPackages = [
  'makosh-whisper-stt-core:implementation',
  'makosh-whisper-stt-assembly:assembly',
  'makosh-whisper-stt-persistence:persistence',
  'makosh-whisper-stt-process:implementation',
  'makosh-whisper-stt-runtime:runtime',
];

const exactSpeechWhisperCapabilities = [
  'speech_to_text.blob.v1',
  'speech_to_text.provider.v1',
  'speech_to_text.storage.v1',
  'speech_to_text.transcribe.v1',
  'whisper_stt.blob.v1',
  'whisper_stt.native.v1',
  'whisper_stt.provider.v1',
  'whisper_stt.storage.v1',
];

test('Task 9 atomically admits the exact Speech-to-Text and Whisper inventory', async () => {
  const policy = JSON.parse(await read('backend/architecture/policy.json'));
  const implementation = policy.implementation;
  const packagesFor = (owner) => implementation.productionPackages
    .filter((descriptor) => descriptor.owner === owner)
    .map(({ name, surface }) => `${name}:${surface}`);

  assert.equal(implementation.currentSlice, 'speech_to_text_whisper_admission_v1');
  assert.equal(implementation.productionPackages.length, 283);
  assert.equal(implementation.ownerInventory.businessCapabilities.length, 253);
  assert.equal(implementation.ownerInventory.engines.filter((owner) => owner === 'speech_to_text').length, 1);
  assert.equal(implementation.ownerInventory.integrations.filter((owner) => owner === 'whisper_stt').length, 1);
  assert.deepEqual(packagesFor('speech_to_text'), exactSpeechToTextPackages);
  assert.deepEqual(packagesFor('whisper_stt'), exactWhisperPackages);
  assert.deepEqual(
    implementation.ownerInventory.businessCapabilities.filter((capability) =>
      exactSpeechWhisperCapabilities.includes(capability)),
    exactSpeechWhisperCapabilities,
  );
});

test('Task 9 keeps exact workspace counts and adds no direct client surface', async () => {
  const workspace = await read('backend/Cargo.toml');
  const members = [...workspace.matchAll(/^\s*"([^"]+)",?$/gm)].map((match) => match[1]);
  const manifests = globSync('**/Cargo.toml', {
    cwd: absolute('backend'),
    exclude: ['target/**'],
  });
  const directGeneratedClients = [
    ...globSync('src/gen/**/speech_to_text/**/*', { cwd: absolute('frontend') }),
    ...globSync('src/gen/**/whisper_stt/**/*', { cwd: absolute('frontend') }),
  ];
  const directCompatibilityRoutes = globSync('src/**/*.{ts,vue}', {
    cwd: absolute('frontend'),
  }).flatMap((path) => readFileSync(absolute(`frontend/${path}`), 'utf8')
    .match(/\/api\/v1\/(?:speech(?:-to-text|_to_text)?|stt|whisper)[^'"\s]*/g) ?? []);

  assert.equal(members.length, 420);
  assert.equal(manifests.length, 421);
  assert.deepEqual(directGeneratedClients, []);
  assert.deepEqual(directCompatibilityRoutes, []);
});

test('Task 9 requires immutable FORCE RLS revision 2 for both owner stores', async () => {
  const [speechSchema, whisperSchema] = await Promise.all([
    read('backend/src/speech-to-text-persistence/src/schema.rs'),
    read('backend/src/whisper-stt-persistence/src/schema.rs'),
  ]);

  assert.match(speechSchema, /SPEECH_TO_TEXT_STORAGE_BUNDLE_REVISION_V1: u32 = 2/);
  assert.match(speechSchema, /0002_speech_to_text_owner_rls\.sql/);
  assert.match(whisperSchema, /WHISPER_STT_STORAGE_BUNDLE_REVISION_V1: u32 = 2/);
  assert.match(whisperSchema, /0002_whisper_stt_owner_rls\.sql/);
  for (const migration of [
    await read('backend/src/speech-to-text-persistence/migrations/0002_speech_to_text_owner_rls.sql'),
    await read('backend/src/whisper-stt-persistence/migrations/0002_whisper_stt_owner_rls.sql'),
  ]) {
    assert.match(migration, /ENABLE ROW LEVEL SECURITY/);
    assert.match(migration, /FORCE ROW LEVEL SECURITY/);
    assert.match(migration, /CREATE POLICY/);
    assert.match(migration, /current_setting/);
  }
});

test('Task 9 release and development assembly contain exactly both managed units', async () => {
  const [release, developmentAssembly] = await Promise.all([
    read('backend/scripts/materialize-dev-release.sh'),
    read('backend/development/assembly/src/main.rs'),
  ]);

  assert.match(release, /speech-to-text\.release-artifacts\.json/);
  assert.match(release, /whisper-stt\.release-artifacts\.json/);
  for (const artifact of [
    'speech_to_text.runtime.v1',
    'speech_to_text.storage.v1',
    'whisper_stt.runtime.v1',
    'whisper_stt.storage.v1',
  ]) {
    assert.equal(developmentAssembly.includes(artifact), true, artifact);
  }
  assert.match(developmentAssembly, /const MODULE_PLAN: \[ModulePlanV1; 41\]/);
});

test('Task 9 admission is backed by real media, storage isolation, bootstrap and privacy evidence', async () => {
  const [runner, speechFlow, callFlow] = await Promise.all([
    read('backend/scripts/test-authenticated-storage.mjs'),
    read('backend/tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/whisper_stt_managed_flow.rs'),
    read('backend/tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/call_transcription_managed_flow.rs'),
  ]);

  assert.match(runner, /managed_speech_to_text_routes_whisper_private_blob_and_replays_after_restart/);
  assert.match(runner, /managed_call_transcription_reaches_recording_stt_gateway_blob_and_restarts/);
  assert.match(runner, /managed_speech_to_text_whisper_bootstrap_fails_closed_and_stops_promptly/);
  assert.match(speechFlow, /runtime_storage_credential_for_registration_v1/);
  assert.match(speechFlow, /assert_supervised_speech_to_text_child_output_is_private_v1/);
  assert.match(speechFlow, /assert_supervised_whisper_stt_child_output_is_private_v1/);
  assert.match(callFlow, /SpeechTranscriptDocumentV1::decode/);
  assert.match(callFlow, /contains\("makosh"\)/);
  assert.match(callFlow, /restart_call_transcription_runtime_v1/);
});
