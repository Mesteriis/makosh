import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import test from 'node:test';

const PROJECT_ROOT = resolve(import.meta.dirname, '../../..');
const read = (path) => readFile(resolve(PROJECT_ROOT, path), 'utf8');

test('call transcription runtime is an event-only workflow with SSE metadata and Blob content', async () => {
  const [policySource, manifest, admission, managed, ingress, stt, realtime, client, proto, sessionAdr] =
    await Promise.all([
      read('backend/architecture/policy.json'),
      read('backend/src/call-transcription-runtime/Cargo.toml'),
      read('backend/src/call-transcription-runtime/src/admission.rs'),
      read('backend/src/call-transcription-runtime/src/managed_runtime.rs'),
      read('backend/src/call-transcription-runtime/src/ingress.rs'),
      read('backend/src/call-transcription-runtime/src/stt.rs'),
      read('backend/src/call-transcription-runtime/src/client_realtime.rs'),
      read('backend/src/call-transcription-runtime/src/client_port.rs'),
      read('backend/src/platform/runtime_protocol/proto/makosh/runtime/v1/module_client.proto'),
      read('docs/adr/ADR-0395-authenticated-client-session-binding-in-managed-module-transport.md'),
    ]);
  const policy = JSON.parse(policySource);
  const packages = new Map(
    policy.implementation.productionPackages.map((descriptor) => [descriptor.name, descriptor]),
  );

  assert.equal(policy.implementation.currentSlice, 'call_transcription_managed_conformance_v1');
  assert.deepEqual(packages.get('makosh-call-transcription-runtime'), {
    name: 'makosh-call-transcription-runtime',
    role: 'workflow',
    owner: 'call_transcription',
    surface: 'runtime',
  });
  assert.match(manifest, /\[\[bin\]\][\s\S]*makosh-call-transcription-runtime/);
  assert.match(manifest, /makosh-call-transcription-ingress/);
  assert.match(manifest, /makosh-speech-to-text-api/);
  assert.doesNotMatch(
    manifest,
    /makosh-communications|makosh-desktop-call-recording-(?:core|runtime)|makosh-speech-to-text-(?:core|runtime)|makosh-whisper/,
  );

  for (const required of [
    'RECORDING_READY_CAPABILITY_ID_V1',
    'RECORDING_REJECTED_CAPABILITY_ID_V1',
    'BLOB_CAPABILITY_ID_V1',
    'STT_DEPENDENCY_CAPABILITY_ID_V1',
    'ModuleKindV1::Workflow',
  ]) {
    assert.match(admission, new RegExp(required));
  }
  assert.match(managed, /request_managed_runtime_event_access_v2/);
  assert.match(managed, /bind_recording_subscriptions/);
  assert.match(managed, /dispatch_client_request_v1/);
  assert.match(ingress, /accept_recording_custody_v1[\s\S]*persist_recording_ingress/);
  assert.match(stt, /fresh_stt_source_proof_v1/);
  assert.match(stt, /speech_to_text_contract_reference_v1/);
  assert.match(realtime, /ManagedRuntimeClientRealtimePublishRequestV1/);
  assert.match(realtime, /realtime_after/);
  assert.doesNotMatch(realtime, /transcript_text|segment_text|audio_bytes/);
  assert.match(client, /client_session_sha256/);
  assert.match(client, /ModuleClientBlobAuthorizationV1/);
  assert.match(proto, /authenticated_client_session_id/);
  assert.match(sessionAdr, /actor\/session-bound/);

  for (const capability of [
    'call_transcription.blob.v1',
    'call_transcription.recording_ready.v1',
    'call_transcription.recording_rejected.v1',
    'call_transcription.stt.v1',
  ]) {
    assert(policy.implementation.ownerInventory.businessCapabilities.includes(capability));
  }
});
