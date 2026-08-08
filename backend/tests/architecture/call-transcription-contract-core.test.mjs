import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import test from 'node:test';

const PROJECT_ROOT = resolve(import.meta.dirname, '../../..');
const read = (path) => readFile(resolve(PROJECT_ROOT, path), 'utf8');

test('call transcription client contract and pure core are separate workflow units', async () => {
  const [policySource, apiManifest, coreManifest, proto, api, core] = await Promise.all([
    read('backend/architecture/policy.json'),
    read('backend/src/call-transcription-api/Cargo.toml'),
    read('backend/src/call-transcription-core/Cargo.toml'),
    read('backend/src/call-transcription-api/proto/makosh/call_transcription/v1/transcription.proto'),
    read('backend/src/call-transcription-api/src/lib.rs'),
    read('backend/src/call-transcription-core/src/lib.rs'),
  ]);
  const policy = JSON.parse(policySource);
  const packages = new Map(
    policy.implementation.productionPackages.map((descriptor) => [descriptor.name, descriptor]),
  );

  assert.deepEqual(packages.get('makosh-call-transcription-api'), {
    name: 'makosh-call-transcription-api',
    role: 'workflow',
    owner: 'call_transcription',
    surface: 'contract',
  });
  assert.deepEqual(packages.get('makosh-call-transcription-core'), {
    name: 'makosh-call-transcription-core',
    role: 'workflow',
    owner: 'call_transcription',
    surface: 'implementation',
  });
  assert.match(apiManifest, /makosh-runtime-protocol/);
  assert.match(coreManifest, /makosh-call-transcription-api/);
  assert.match(proto, /StartCallTranscriptionRequestV1/);
  assert.match(proto, /IssueCallTranscriptReadRequestV1/);
  assert.match(proto, /recording_evidence_id/);
  assert.match(proto, /consent_receipt_id/);
  assert.match(api, /CALL_TRANSCRIPTION_SCHEMA_SHA256/);
  assert.match(core, /CallTranscriptionStateV1/);
  assert.match(core, /request_fingerprint_v1/);
  assert.match(core, /SourceMismatch/);
  assert.match(core, /InvalidTransition/);

  for (const forbidden of [
    'transcript_text',
    'segment_text',
    'summary_utf8',
    'source_message_id',
    'provider_id',
    'model_id',
    'filesystem_path',
    'custody_proof',
    'map<',
  ]) {
    assert.doesNotMatch(proto, new RegExp(forbidden));
  }
  for (const forbiddenDependency of [
    'makosh-communications',
    'makosh-desktop-call-recording',
    'makosh-speech-to-text',
    'makosh-whisper',
    'sqlx',
    'async-nats',
  ]) {
    assert.doesNotMatch(`${apiManifest}\n${coreManifest}`, new RegExp(forbiddenDependency));
  }
});
