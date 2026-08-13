import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);

async function source(path) {
  return readFile(new URL(path, BACKEND_ROOT), 'utf8');
}

test('Telegram Calls contract, core and persistence are separate integration build units', async () => {
  const [apiManifest, coreManifest, persistenceManifest, runtimeManifest] = await Promise.all([
    source('src/telegram-calls-api/Cargo.toml'),
    source('src/telegram-calls-core/Cargo.toml'),
    source('src/telegram-calls-persistence/Cargo.toml'),
    source('src/telegram-runtime/Cargo.toml'),
  ]);

  for (const manifest of [apiManifest, coreManifest, persistenceManifest]) {
    assert.match(manifest, /role = "integration"/);
    assert.match(manifest, /owner = "telegram"/);
    assert.doesNotMatch(manifest, /communications-domain|kernel|gateway/);
  }
  assert.doesNotMatch(apiManifest, /telegram-calls-core|sqlx|telegram-runtime/);
  assert.doesNotMatch(coreManifest, /sqlx|prost|telegram-runtime|telegram-tdlib/);
  assert.match(persistenceManifest, /makosh-telegram-calls-core/);
  assert.match(coreManifest, /makosh-scheduler-protocol/);
  assert.match(persistenceManifest, /makosh-scheduler-protocol/);
  assert.match(persistenceManifest, /makosh-events-protocol/);
  assert.doesNotMatch(persistenceManifest, /makosh-telegram-calls-api|telegram-tdlib/);
  assert.match(runtimeManifest, /makosh-telegram-calls-api/);
  assert.match(runtimeManifest, /makosh-telegram-calls-core/);
  assert.match(runtimeManifest, /makosh-telegram-calls-persistence/);
});

test('Telegram Calls admits exact query command and replay routes after signaling conformance', async () => {
  const [admission, runtimePort, process, operations, assembly, fixture] = await Promise.all([
    source('src/telegram-runtime/src/admission.rs'),
    source('src/telegram-runtime/src/calls_client_port.rs'),
    source('src/telegram-runtime/src/process.rs'),
    source('src/telegram-calls-persistence/src/operations.rs'),
    source('src/telegram-assembly/src/lib.rs'),
    source('tests/fixtures/telegram-tdjson/tdjson.c'),
  ]);

  assert.match(
    admission,
    /telegram_calls_client_capability_v1\(TelegramCallsContractV1::Command\)/,
  );
  assert.match(admission, /telegram_calls_client_capability_v1\(TelegramCallsContractV1::Query\)/);
  assert.match(
    admission,
    /telegram_calls_client_capability_v1\(TelegramCallsContractV1::Realtime\)/,
  );
  assert.match(runtimePort, /TelegramCallsContractV1::Command => TelegramCallsRoute::Command/);
  assert.doesNotMatch(runtimePort, /makosh_communications|makosh_telegram_tdlib/);
  assert.match(process, /execute_due_call_operations/);
  assert.match(operations, /reconcile_stale_call_operations/);
  assert.match(operations, /TelegramCallFailureCategory::Unknown/);
  assert.match(assembly, /telegram_storage_bundle_with_calls_backfill_v6/);
  assert.match(assembly, /telegram_calls_storage_migration_v1/);
  assert.match(assembly, /telegram_calls_storage_migration_v4/);
  assert.match(fixture, /createCall/);
  assert.match(fixture, /discardCall/);
  assert.match(fixture, /updateCall/);
  assert.match(fixture, /callStateDiscarded/);
});

test('Telegram Calls upgrade job is owner-local, DDL-only and cursor preserving', async () => {
  const [
    schedulerProto,
    schedulerManifest,
    core,
    persistenceManifest,
    persistence,
    runtime,
    schema,
    communicationsManifest,
  ] = await Promise.all([
    source('src/platform/scheduler/protocol/proto/makosh/scheduler/v1/job_command.proto'),
    source('src/platform/scheduler/implementation/Cargo.toml'),
    source('src/telegram-calls-core/src/backfill.rs'),
    source('src/telegram-calls-persistence/Cargo.toml'),
    source('src/telegram-calls-persistence/src/backfill/executor.rs'),
    source('src/telegram-runtime/src/calls_backfill.rs'),
    source('src/telegram-calls-persistence/src/schema.rs'),
    source('src/communications-runtime/Cargo.toml'),
  ]);

  const ownerCommand = schedulerProto.match(/message OwnerJobCommandV1 \{(?<body>.*?)\n\}/s);
  assert.ok(ownerCommand?.groups?.body);
  assert.match(schedulerProto, /OWNER_JOB_TRIGGER_KIND_V1_UPGRADE_RECONCILIATION/);
  assert.doesNotMatch(ownerCommand.groups.body, /schedule_id|schedule_revision/);
  assert.match(core, /calls_realtime_backfill/);
  assert.match(core, /BATCH_SIZE_V1: u32 = 256/);
  assert.match(persistenceManifest, /makosh-events-protocol/);
  assert.match(persistenceManifest, /makosh-scheduler-protocol/);
  assert.doesNotMatch(persistenceManifest, /communications-runtime|makosh-kernel/);
  assert.match(persistence, /telegram_call_realtime_replay_order/);
  assert.match(persistence, /telegram_call_realtime_replay_cursor/);
  assert.match(persistence, /StaleLease/);
  assert.match(runtime, /DurableEnvelopeV1/);
  assert.match(runtime, /complete_calls_realtime_backfill_v1/);
  assert.match(runtime, /ExecutionPolicyExhausted/);
  assert.doesNotMatch(schedulerManifest, /telegram/);
  assert.doesNotMatch(communicationsManifest, /telegram-calls/);

  const migration = schema.match(
    /pub const TELEGRAM_CALLS_SCHEMA_V4: &str = r#"(?<body>.*?)"#;/s,
  );
  assert.ok(migration?.groups?.body);
  assert.match(migration.groups.body, /telegram_call_realtime_replay_order/);
  assert.match(migration.groups.body, /telegram_call_realtime_backfill_jobs/);
  assert.doesNotMatch(migration.groups.body, /\b(?:INSERT|UPDATE|DELETE)\b/i);
});

test('Telegram Calls contracts are typed and do not expose media secrets', async () => {
  const [contract, proto, schema] = await Promise.all([
    source('src/telegram-calls-api/src/contract.rs'),
    source('src/telegram-calls-api/proto/makosh/telegram/calls/v1/calls.proto'),
    source('src/telegram-calls-persistence/src/schema.rs'),
  ]);

  for (const identity of [
    'telegram.calls.query.v1',
    'telegram.calls.command.v1',
    'telegram.calls.realtime.v1',
  ]) {
    assert.match(contract, new RegExp(identity.replaceAll('.', '\\.')));
  }
  assert.match(proto, /service TelegramCallsQueryService/);
  assert.match(proto, /service TelegramCallsCommandService/);
  assert.match(proto, /service TelegramCallsRealtimeService/);
  assert.doesNotMatch(proto, /\bgoogle\.protobuf\.Any\b|\bmap\s*</);
  for (const privateField of [
    'encryption_key',
    'custom_parameters',
    'raw_json',
    'audio_bytes',
    'debug_log',
  ]) {
    assert.doesNotMatch(proto, new RegExp(privateField));
    assert.doesNotMatch(schema, new RegExp(privateField));
  }
  assert.doesNotMatch(schema, /communications_/);
});

test('Telegram Calls history keeps volatile TDLib identity scoped to runtime generation', async () => {
  const [core, schema] = await Promise.all([
    source('src/telegram-calls-core/src/lib.rs'),
    source('src/telegram-calls-persistence/src/schema.rs'),
  ]);

  assert.match(core, /runtime_generation/);
  assert.match(core, /tdlib_call_id/);
  assert.match(core, /provider_call_unique_id/);
  assert.match(schema, /UNIQUE \(account_id, runtime_generation, tdlib_call_id\)/);
  assert.match(schema, /UNIQUE \(account_id, provider_call_unique_id\)/);
  assert.match(schema, /telegram_call_state_history/);
  assert.match(schema, /telegram_call_realtime_frames/);
});

test('Telegram call media contract and tgcalls adapter remain separate integration units', async () => {
  const [contractManifest, adapterManifest, contract, adapter] = await Promise.all([
    source('src/telegram-call-media-contract/Cargo.toml'),
    source('src/telegram-call-media-tgcalls/Cargo.toml'),
    source('src/telegram-call-media-contract/src/lib.rs'),
    source('src/telegram-call-media-tgcalls/src/lib.rs'),
  ]);

  for (const manifest of [contractManifest, adapterManifest]) {
    assert.match(manifest, /role = "integration"/);
    assert.match(manifest, /owner = "telegram"/);
    assert.doesNotMatch(manifest, /communications-domain|kernel|gateway|sqlx|prost/);
  }
  assert.doesNotMatch(contractManifest, /libloading|telegram-runtime|telegram-tdlib/);
  assert.match(adapterManifest, /makosh-telegram-call-media-contract/);
  assert.match(adapterManifest, /libloading/);
  assert.doesNotMatch(
    adapterManifest,
    /telegram-runtime|telegram-tdlib|telegram-persistence|telegram-assembly/,
  );
  assert.match(contract, /TelegramCallReadyPlanV1/);
  assert.match(contract, /TelegramCallSecretBytesV1/);
  assert.match(contract, /TelegramCallMediaEventV1/);
  assert.match(adapter, /TgCallsMediaAdapter/);
  assert.match(adapter, /load_exact/);
  assert.doesNotMatch(adapter, /Library::new\(["'][^"']+["']\)/);
});

test('Telegram tgcalls native build is pinned, system-audio backed and secret-negative', async () => {
  const [adapter, bridge, bridgeBuild, audioProbe, patch, buildScript, readme] =
    await Promise.all([
      source('src/telegram-call-media-tgcalls/src/lib.rs'),
      source('src/telegram-call-media-tgcalls/native/bridge.cpp'),
      source('src/telegram-call-media-tgcalls/native/BUILD.bazel'),
      source('src/telegram-call-media-tgcalls/native/audio_device_conformance.cpp'),
      source(
        'src/telegram-call-media-tgcalls/native/patches/telegram-ios-macos-audio-device.patch',
      ),
      source('scripts/build-telegram-tgcalls-bridge-macos.sh'),
      source('src/telegram-call-media-tgcalls/README.md'),
    ]);

  assert.match(bridgeBuild, /\/\/submodules\/TgVoipWebrtc:tgcalls_core/);
  assert.match(bridgeBuild, /\/\/third-party\/webrtc:makosh_macos_audio_device/);
  assert.doesNotMatch(bridgeBuild, /FakeAudioDeviceModule|SineRecorder|NoOpRenderer/);
  assert.match(bridge, /createAudioDeviceModule = \{\}/);
  assert.match(bridge, /SetLoggingFunction/);
  assert.match(bridge, /std::fill\(typed->key->begin\(\), typed->key->end\(\), 0\)/);
  assert.match(bridge, /StopCompletion/);
  assert.match(bridge, /wait_for\(lock, kStopTimeout/);
  assert.doesNotMatch(bridge, /fprintf|std::cout|std::cerr|printf\(/);
  assert.match(adapter, /library_guard: Arc<NativeApi>/);
  assert.match(adapter, /abandon_active_native_session/);
  assert.match(adapter, /poisoned: bool/);
  assert.match(patch, /audio_device_impl\.cc/);
  assert.match(patch, /audio_device_mac\.cc/);
  assert.match(buildScript, /6ad963e5b62d354da79040f388ae2b9132fb17b8/);
  assert.match(buildScript, /e3069322a3d1e16ecb11a5e302242e59ddd7f09e/);
  assert.match(buildScript, /3817e906cb6c22ec9cc62023b073e1a668d9cb33/);
  assert.match(buildScript, /45e9388abf21d1107e146ea366ad080eb93cb6a5f3a4a3b048f78de0bc3faffa/);
  assert.match(buildScript, /da7eabb7bafdf7d3ae5e9f223aa5bdc1eece45ac569dc21b3b037520b4464768/);
  assert.match(buildScript, /readonly XCODE_VERSION="26\.2"/);
  assert.match(buildScript, /--audio-conformance/);
  assert.match(buildScript, /--development-audio-conformance/);
  assert.match(buildScript, /"release_eligible": \$\{release_eligible\}/);
  assert.doesNotMatch(buildScript, /submodule update --remote|--branch\s/);

  const productionTarget = bridgeBuild.match(
    /cc_binary\(\s*name = "libmakosh_tgcalls_bridge\.dylib"[\s\S]*?\n\)/,
  )?.[0];
  assert.ok(productionTarget);
  assert.doesNotMatch(productionTarget, /audio_device_conformance/);
  assert.match(bridgeBuild, /name = "makosh_tgcalls_audio_device_conformance"/);
  assert.match(
    buildScript,
    /"\$\{native_directory\}\/audio_device_conformance\.cpp"/,
  );
  assert.match(bridgeBuild, /testonly = True/);
  assert.match(audioProbe, /AudioDeviceModule::Create/);
  assert.match(audioProbe, /StartPlayout/);
  assert.match(audioProbe, /StartRecording/);
  assert.match(audioProbe, /class MicrophoneMuteGuard/);
  assert.match(audioProbe, /microphone_mute_guard\.restore\(\)/);
  assert.match(audioProbe, /--allow-microphone-and-speaker-access/);
  assert.match(audioProbe, /std::memset\(audio_samples, 0/);
  assert.doesNotMatch(audioProbe, /\b(?:fwrite|ofstream|open|write)\s*\(/);
  assert.match(readme, /is not\s+referenced by Telegram assembly/);
});

test('Telegram release and runtime bind the exact staged tgcalls artifact', async () => {
  const [assembly, assemblyCli, admission, bindings, runtimeMain, runtimeCore, fixture] =
    await Promise.all([
      source('src/telegram-assembly/src/lib.rs'),
      source('src/telegram-assembly/src/main.rs'),
      source('src/telegram-runtime/src/admission.rs'),
      source('src/telegram-runtime/src/runtime_bindings.rs'),
      source('src/telegram-runtime/src/main.rs'),
      source('src/telegram-runtime/src/lib.rs'),
      source('tests/fixtures/telegram-tgcalls/bridge.c'),
    ]);

  for (const value of [assembly, admission]) {
    assert.match(value, /telegram\.tgcalls\.v1/);
  }
  assert.match(assembly, /lib\/libmakosh_tgcalls_bridge\.dylib/);
  assert.match(assemblyCli, /--tgcalls/);
  assert.match(admission, /RuntimeArtifactUseV1::NativeDynamicLibrary/);
  assert.match(bindings, /tgcalls_artifact_path/);
  assert.match(bindings, /TELEGRAM_TGCALLS_ARTIFACT_ID/);
  assert.match(bindings, /MAX_TGCALLS_ARTIFACT_BYTES/);
  assert.match(runtimeMain, /TgCallsMediaAdapter::load_exact/);
  assert.match(runtimeCore, /install_call_media_port/);
  assert.doesNotMatch(runtimeMain, /MAKOSH_TGCALLS_BRIDGE_PATH/);
  assert.match(fixture, /Test-only ABI fixture/);
  assert.match(fixture, /no\s*\n\s*\* audio device, network transport or production media behavior/);
});
