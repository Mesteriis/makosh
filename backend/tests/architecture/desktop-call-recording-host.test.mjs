import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import test from 'node:test';

const PROJECT_ROOT = resolve(import.meta.dirname, '../../..');
const read = (path) => readFile(resolve(PROJECT_ROOT, path), 'utf8');

test('desktop recording native host stays route-bound and outside business ownership', async () => {
  const [manifest, root, transport, capture, consent, shell, capability, telemost] = await Promise.all([
    read('frontend/src-tauri/Cargo.toml'),
    read('frontend/src-tauri/src/desktop_call_recording_host/mod.rs'),
    read('frontend/src-tauri/src/desktop_call_recording_host/transport.rs'),
    read('frontend/src-tauri/src/desktop_call_recording_host/capture.rs'),
    read('frontend/src-tauri/src/desktop_call_recording_host/consent.rs'),
    read('frontend/src-tauri/src/lib.rs'),
    read('frontend/src-tauri/capabilities/default.json'),
    read('frontend/src-tauri/src/yandex_telemost_companion.rs'),
  ]);

  assert.match(manifest, /desktop-call-recording-host = \[/);
  for (const featureDependency of [
    'dep:cpal',
    'dep:getrandom',
    'dep:makosh-desktop-call-recording-api',
    'dep:rfd',
  ]) {
    assert.match(manifest, new RegExp(`"${featureDependency}"`));
  }
  assert.match(manifest, /makosh-desktop-call-recording-api/);
  assert.match(root, /DesktopRecordingHostCommandClaimV1/);
  assert.match(root, /ConsentAuthorityV1/);
  assert.match(root, /SelectedInputV1::system_default/);
  assert.match(root, /consent\.request[\s\S]*input\.start/);
  assert.match(root, /host_has_no_capture_or_worker_before_explicit_connect/);
  assert.match(transport, /ManagedIntegrationHostBridgeConfigurationV1/);
  assert.match(transport, /route_binding_sha256/);
  assert.match(transport, /admitted_route_exists/);
  assert.match(capture, /OUTPUT_SAMPLE_RATE: u32 = 16_000/);
  assert.match(capture, /MAX_AUDIO_BYTES_V1/);
  assert.match(capture, /PermissionDenied => "os_permission_denied"/);
  assert.match(consent, /MessageButtons::OkCancelCustom/);
  assert.match(consent, /call transcription/);
  assert.match(shell, /cfg\(feature = "desktop-call-recording-host"\)/);
  assert.match(root, /admitted_route_exists[\s\S]*add_capability/);
  assert.doesNotMatch(capability, /desktop-call-recording-host/);

  for (const forbidden of [
    'makosh-communications',
    'makosh-call-transcription',
    'sqlx',
    'nats',
    'blob_reference',
    'durable event',
    'Command::new',
  ]) {
    assert.doesNotMatch(
      `${manifest}\n${root}\n${transport}\n${capture}\n${consent}`,
      new RegExp(forbidden),
    );
  }

  assert.doesNotMatch(telemost, /ffmpeg|consent_attested|recording_start|recording_stop/);
});

test('desktop recording bundle declares current macOS microphone authority', async () => {
  const [infoPlist, entitlements] = await Promise.all([
    read('frontend/src-tauri/Info.plist'),
    read('frontend/src-tauri/Entitlements.plist'),
  ]);
  assert.match(infoPlist, /NSMicrophoneUsageDescription/);
  assert.match(infoPlist, /explicitly approve/);
  assert.match(entitlements, /com\.apple\.security\.device\.audio-input/);
});
