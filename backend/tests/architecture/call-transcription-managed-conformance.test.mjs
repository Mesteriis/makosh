import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import test from 'node:test';

const PROJECT_ROOT = resolve(import.meta.dirname, '../../..');
const read = (path) => readFile(resolve(PROJECT_ROOT, path), 'utf8');

test('call transcription managed conformance proves the exact clean-room contour', async () => {
  const [
    policySource,
    authorityProto,
    recordingClient,
    blob,
    admission,
    managedFlow,
    managedSetup,
    gatewayFixture,
    conformanceScript,
    frontendClient,
    authorityAdr,
  ] = await Promise.all([
    read('backend/architecture/policy.json'),
    read(
      'backend/src/desktop-call-recording-api/proto/makosh/desktop_call_recording/v1/recording.proto',
    ),
    read('backend/src/desktop-call-recording-runtime/src/client_port.rs'),
    read('backend/src/call-transcription-runtime/src/blob.rs'),
    read('backend/src/call-transcription-runtime/src/admission.rs'),
    read(
      'backend/tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/call_transcription_managed_flow.rs',
    ),
    read(
      'backend/tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/call_transcription_managed_setup.rs',
    ),
    read(
      'backend/tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/call_transcription_gateway_fixture.rs',
    ),
    read('backend/scripts/test-authenticated-storage.mjs'),
    read('frontend/src/workflows/call-transcription/api/callTranscription.ts'),
    read('docs/adr/ADR-0396-recording-to-transcription-client-authority-handoff.md'),
  ]);
  const policy = JSON.parse(policySource);

  assert.equal(policy.implementation.currentSlice, 'call_transcription_managed_conformance_v1');
  assert.match(authorityProto, /message RecordingTranscriptionAuthorityV1/);
  assert.match(authorityProto, /optional RecordingTranscriptionAuthorityV1 transcription_authority/);
  assert.match(recordingClient, /RecordingStateV1::Ready/);
  assert.match(recordingClient, /fn transcription_authority/);
  assert.match(recordingClient, /Some\(RecordingTranscriptionAuthorityV1/);
  assert.match(blob, /replayable_source_reference_id/);
  assert.match(blob, /fresh_source_cleanup_proof_v1/);
  assert.match(blob, /BlobDataOperationV1::BlobDataOperationWriteV1/);
  assert.match(admission, /BlobQuotaOperationV1::Write/);
  assert.doesNotMatch(blob, /sqlx|Postgres|custody_proof.*INSERT/i);

  for (const evidence of [
    'set_authenticated_nats_container_running(false)',
    'wait_for_authenticated_nats_reconnect',
    'SpeechTranscriptDocumentV1::decode',
    'authenticate_secondary_gateway_router',
    'StatusCode::SERVICE_UNAVAILABLE',
    'restart_call_transcription_runtime_v1',
  ]) {
    assert.match(managedFlow, new RegExp(evidence.replace(/[()]/g, '\\$&')));
  }
  assert.match(managedSetup, /installed_call_transcription_ensemble_release_v1/);
  assert.match(gatewayFixture, /\/api\/realtime\/v1\/events/);
  assert.match(conformanceScript, /managed_call_transcription_reaches_recording_stt_gateway_blob_and_restarts/);
  assert.match(conformanceScript, /canonicalize_whisper_test_wav/);
  assert.match(frontendClient, /fromBinary\(SpeechTranscriptDocumentV1Schema, bytes\)/);
  assert.doesNotMatch(frontendClient, /decoder\.decode\(bytes\)/);
  assert.match(authorityAdr, /managed\/browser\s+conformance реализованы/);
});
