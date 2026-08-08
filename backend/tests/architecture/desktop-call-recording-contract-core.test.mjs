import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import test from 'node:test';

const PROJECT_ROOT = resolve(import.meta.dirname, '../../..');
const read = (path) => readFile(resolve(PROJECT_ROOT, path), 'utf8');

test('desktop recording contract core and target ingress are isolated build units', async () => {
  const [policySource, api, core, ingress] = await Promise.all([
    read('backend/architecture/policy.json'),
    read('backend/src/desktop-call-recording-api/Cargo.toml'),
    read('backend/src/desktop-call-recording-core/Cargo.toml'),
    read('backend/src/call-transcription-ingress/Cargo.toml'),
  ]);
  const packages = new Map(
    JSON.parse(policySource).implementation.productionPackages.map((item) => [item.name, item]),
  );

  assert.deepEqual(packages.get('makosh-desktop-call-recording-api'), {
    name: 'makosh-desktop-call-recording-api',
    role: 'integration',
    owner: 'desktop_call_recording',
    surface: 'contract',
  });
  assert.deepEqual(packages.get('makosh-desktop-call-recording-core'), {
    name: 'makosh-desktop-call-recording-core',
    role: 'integration',
    owner: 'desktop_call_recording',
    surface: 'implementation',
  });
  assert.deepEqual(packages.get('makosh-call-transcription-ingress'), {
    name: 'makosh-call-transcription-ingress',
    role: 'workflow',
    owner: 'call_transcription',
    surface: 'contract',
  });
  assert.doesNotMatch(`${api}\n${core}`, /makosh-communications|call-transcription/);
  assert.doesNotMatch(ingress, /desktop-call-recording|communications/);
});

test('public recording surface is metadata-only while private host completion is bounded audio', async () => {
  const [proto, core] = await Promise.all([
    read('backend/src/desktop-call-recording-api/proto/makosh/desktop_call_recording/v1/recording.proto'),
    read('backend/src/desktop-call-recording-core/src/lib.rs'),
  ]);
  const publicSurface = proto.slice(0, proto.indexOf('message DesktopRecordingHostHandshakeV1'));

  assert.doesNotMatch(publicSurface, /audio|blob|custody|path|device|consent_attested/);
  assert.match(proto, /DesktopCaptureCompletedV1[\s\S]*bytes canonical_wav_bytes/);
  assert.match(core, /WAV_BYTES_PER_SECOND_V1: u64 = 32_000/);
  assert.match(core, /&bytes\[0\.\.4\] != b"RIFF"/);
  assert.match(core, /RecordingStateV1::Ready[\s\S]*InvalidTransition/);
  assert.doesNotMatch(core.split('#[cfg(test)]')[0], /std::process|Command::new|filesystem|telemost/i);
});

test('recording ready event is target-owned and keeps audio outside durable envelopes', async () => {
  const proto = await read(
    'backend/src/call-transcription-ingress/proto/makosh/call_transcription/ingress/v1/recording.proto',
  );
  assert.match(proto, /message RecordingReadyV1/);
  for (const required of [
    'consent_receipt_id',
    'target_blob_reference_id',
    'custody_transfer_source_proof',
    'audio_sha256',
  ]) {
    assert.match(proto, new RegExp(`\\b${required}\\b`));
  }
  assert.doesNotMatch(proto, /audio_bytes|filesystem_path|provider_id|device_id|participant/);
});

test('recording persistence owns lifecycle leases and exact outbox without private capture bytes', async () => {
  const [policySource, manifest, migration, repository] = await Promise.all([
    read('backend/architecture/policy.json'),
    read('backend/src/desktop-call-recording-persistence/Cargo.toml'),
    read('backend/src/desktop-call-recording-persistence/migrations/0001_desktop_call_recording.sql'),
    read('backend/src/desktop-call-recording-persistence/src/repository.rs'),
  ]);
  const policy = JSON.parse(policySource);
  assert.deepEqual(
    policy.implementation.productionPackages.find(
      ({ name }) => name === 'makosh-desktop-call-recording-persistence',
    ),
    {
      name: 'makosh-desktop-call-recording-persistence',
      role: 'integration',
      owner: 'desktop_call_recording',
      surface: 'persistence',
    },
  );
  assert.match(manifest, /makosh-desktop-call-recording-core/);
  assert.doesNotMatch(manifest, /communications|call-transcription/);
  for (const required of [
    'makosh_data.desktop_call_recording_runs',
    'makosh_data.desktop_call_recording_host_commands',
    'makosh_data.desktop_call_recording_outbox',
    'makosh_data.desktop_call_recording_realtime',
  ]) {
    assert.match(migration, new RegExp(required.replaceAll('.', '\\.')));
  }
  for (const source of [migration, repository]) {
    assert.doesNotMatch(
      source,
      /audio_bytes|canonical_wav|filesystem_path|audio_input_label|consent_body|custody_transfer_source_proof/,
    );
  }
  assert.match(repository, /FOR UPDATE SKIP LOCKED/);
  assert.match(repository, /exact_envelope_bytes/);
  assert.match(repository, /accept_or_replay/);
  assert.match(repository, /recording_revision=\$1[\s\S]*run_state=4/);
});

test('desktop recording runtime is an isolated managed integration with private host audio and shared realtime', async () => {
  const [policySource, manifest, main, hostPort, hostTransport, realtime] = await Promise.all([
    read('backend/architecture/policy.json'),
    read('backend/src/desktop-call-recording-runtime/Cargo.toml'),
    read('backend/src/desktop-call-recording-runtime/src/main.rs'),
    read('backend/src/desktop-call-recording-runtime/src/host_port.rs'),
    read('backend/src/desktop-call-recording-runtime/src/host_transport.rs'),
    read('backend/src/desktop-call-recording-runtime/src/client_realtime.rs'),
  ]);
  const policy = JSON.parse(policySource);
  assert.deepEqual(
    policy.implementation.productionPackages.find(
      ({ name }) => name === 'makosh-desktop-call-recording-runtime',
    ),
    {
      name: 'makosh-desktop-call-recording-runtime',
      role: 'integration',
      owner: 'desktop_call_recording',
      surface: 'runtime',
    },
  );
  assert.match(manifest, /makosh-call-transcription-ingress/);
  assert.doesNotMatch(manifest, /makosh-communications|call-transcription-(?:core|runtime|persistence)/);
  assert.match(main, /serve-inherited/);
  assert.doesNotMatch(main, /consent_attested|filesystem_path|ffmpeg/i);
  assert.match(main, /ManagedIntegrationHostBridgeConfigurationV1/);
  assert.match(hostTransport, /MAX_FRAME_BYTES_V1/);
  assert.match(hostPort, /validate_canonical_wav_v1/);
  assert.match(hostPort, /build_recording_ready_outbox_record_v1/);
  assert.match(realtime, /validate_managed_client_realtime_publish_request_v1/);
  assert.doesNotMatch(`${hostPort}\n${realtime}`, /setInterval|polling|audio.*realtime/i);
});

test('desktop recording assembly emits only unsigned runtime and owner storage inputs', async () => {
  const [policySource, manifest, library, main, release] = await Promise.all([
    read('backend/architecture/policy.json'),
    read('backend/src/desktop-call-recording-assembly/Cargo.toml'),
    read('backend/src/desktop-call-recording-assembly/src/lib.rs'),
    read('backend/src/desktop-call-recording-assembly/src/main.rs'),
    read('backend/scripts/materialize-dev-release.sh'),
  ]);
  const policy = JSON.parse(policySource);
  assert.deepEqual(
    policy.implementation.productionPackages.find(
      ({ name }) => name === 'makosh-desktop-call-recording-assembly',
    ),
    {
      name: 'makosh-desktop-call-recording-assembly',
      role: 'integration',
      owner: 'desktop_call_recording',
      surface: 'assembly',
    },
  );
  assert.match(manifest, /makosh-desktop-call-recording-runtime/);
  assert.match(manifest, /makosh-desktop-call-recording-persistence/);
  assert.doesNotMatch(manifest, /communications|call-transcription/);
  assert.match(library, /artifact_kind: "module_runtime"/);
  assert.match(library, /artifact_kind: "storage_bundle"/);
  assert.doesNotMatch(
    `${library}\n${main}`,
    /sign(?:ing|ature|_release)|launch|serve-inherited|private key/i,
  );
  assert.match(release, /--package makosh-desktop-call-recording-assembly/);
  assert.match(release, /debug\/makosh-desktop-call-recording-assembly/);
  assert.match(
    release,
    /desktop-call-recording\.release-artifacts\.json/,
  );
});
