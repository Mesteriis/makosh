import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import test from 'node:test';

const PROJECT_ROOT = resolve(import.meta.dirname, '../../..');
const read = (path) => readFile(resolve(PROJECT_ROOT, path), 'utf8');

test('Whisper transcript artifact is a private bounded engine contract', async () => {
  const [manifest, proto, validation, policy] = await Promise.all([
    read('backend/src/speech-transcript-artifact/Cargo.toml'),
    read('backend/src/speech-transcript-artifact/proto/makosh/speech_transcript/v1/transcript.proto'),
    read('backend/src/speech-transcript-artifact/src/lib.rs'),
    read('backend/architecture/policy.json').then(JSON.parse),
  ]);

  assert.match(manifest, /role = "engine"/);
  assert.match(manifest, /owner = "speech_to_text"/);
  assert.match(manifest, /surface = "contract"/);
  assert.match(proto, /Private Blob document/);
  assert.match(proto, /repeated SpeechTranscriptSegmentV1 segments/);
  assert.match(proto, /bytes content_utf8/);
  for (const forbidden of [
    'provider_name',
    'model_name',
    'filesystem_path',
    'custody_proof',
    'map<',
  ]) {
    assert.ok(!proto.includes(forbidden), `forbidden artifact token ${forbidden}`);
  }
  assert.match(validation, /document\.encoded_len\(\) > encoded_limit/);
  assert.match(validation, /segment\.start_millis < previous_end/);
  assert.match(validation, /std::str::from_utf8/);
  assert.ok(
    policy.dependencies.integrationEngineContractPackages.includes(
      'makosh-speech-transcript-artifact',
    ),
  );
});

test('Whisper core and native process are separate integration units', async () => {
  const [coreManifest, core, processManifest, process, adr] = await Promise.all([
    read('backend/src/whisper-stt-core/Cargo.toml'),
    read('backend/src/whisper-stt-core/src/lib.rs'),
    read('backend/src/whisper-stt-process/Cargo.toml'),
    read('backend/src/whisper-stt-process/src/lib.rs'),
    read('docs/adr/ADR-0391-whisper-stt-provider-integration.md'),
  ]);

  for (const manifest of [coreManifest, processManifest]) {
    assert.match(manifest, /role = "integration"/);
    assert.match(manifest, /owner = "whisper_stt"/);
    assert.doesNotMatch(manifest, /communications|call-transcription/);
  }
  assert.doesNotMatch(core, /std::process|Command::new|filesystem_path|provider_name/);
  const production = process.split('#[cfg(test)]')[0];
  assert.match(production, /Command::new\(&configuration\.executable\)/);
  assert.match(production, /\.env_clear\(\)/);
  assert.match(production, /\.stdin\(Stdio::null\(\)\)/);
  assert.match(production, /\.stdout\(Stdio::null\(\)\)/);
  assert.match(production, /\.stderr\(Stdio::null\(\)\)/);
  assert.match(production, /--output-json/);
  assert.match(production, /child\.kill\(\)/);
  assert.doesNotMatch(production, /Command::new\("(?:sh|bash|zsh)"\)|\.arg\("-c"\)/);
  assert.doesNotMatch(production, /std::env|provider_name|communications|call_transcription/);
  assert.match(adr, /`whisper_stt` является отдельной bundled integration/);
  assert.match(adr, /System executable\/model fallback/);
});

test('Whisper persistence is owner-local and stores no private source or transcript content', async () => {
  const [manifest, model, repository, schema, migration] = await Promise.all([
    read('backend/src/whisper-stt-persistence/Cargo.toml'),
    read('backend/src/whisper-stt-persistence/src/model.rs'),
    read('backend/src/whisper-stt-persistence/src/repository.rs'),
    read('backend/src/whisper-stt-persistence/src/schema.rs'),
    read('backend/src/whisper-stt-persistence/migrations/0001_whisper_stt_runs.sql'),
  ]);

  assert.match(manifest, /role = "integration"/);
  assert.match(manifest, /owner = "whisper_stt"/);
  assert.match(manifest, /surface = "persistence"/);
  assert.match(model, /WhisperSttRunStateV1::Uncertain/);
  assert.match(model, /current\.identity != transition\.next\.identity/);
  assert.match(repository, /ON CONFLICT \(logical_owner_id, request_id\) DO NOTHING/);
  assert.match(repository, /SELECT_RUN_FOR_UPDATE/);
  assert.match(schema, /owner_id: "whisper_stt"/);
  assert.match(migration, /model_revision_sha256/);
  assert.match(migration, /transcript_sha256/);
  assert.match(migration, /run_state BETWEEN 1 AND 5/);
  assert.doesNotMatch(
    `${repository}\n${migration}`,
    /audio_bytes|transcript_text|segment_text|custody_proof|filesystem_path|stdout|stderr|communications_/i,
  );
});

test('Whisper managed runtime binds exact native resources and provider request routing', async () => {
  const [manifest, admission, resources, runtime, worker, blob, processRoot] = await Promise.all([
    read('backend/src/whisper-stt-runtime/Cargo.toml'),
    read('backend/src/whisper-stt-runtime/src/admission.rs'),
    read('backend/src/whisper-stt-runtime/src/resources.rs'),
    read('backend/src/whisper-stt-runtime/src/managed_runtime.rs'),
    read('backend/src/whisper-stt-runtime/src/worker.rs'),
    read('backend/src/whisper-stt-runtime/src/blob.rs'),
    read('backend/src/whisper-stt-runtime/src/main.rs'),
  ]);

  assert.match(manifest, /role = "integration"/);
  assert.match(manifest, /owner = "whisper_stt"/);
  assert.match(manifest, /surface = "runtime"/);
  assert.match(admission, /ModuleKindV1::Integration/);
  assert.match(admission, /speech_to_text_provider_contract_reference_v1/);
  assert.match(admission, /RuntimeArtifactUseV1::ReadOnlyData/);
  assert.match(admission, /RuntimeArtifactUseV1::NativeExecutable/);
  assert.match(resources, /path_before\.file_type\(\)\.is_symlink\(\)/);
  assert.match(resources, /same_file\(&opened, &path_after\)/);
  assert.match(resources, /observed\.as_slice\(\) != binding\.sha256\.as_slice\(\)/);
  assert.match(runtime, /Operation::DeliverModuleRequest/);
  assert.match(runtime, /delivery\.logical_owner_id == self\.logical_human_owner_id/);
  assert.match(runtime, /response_blob_target_owner_id/);
  assert.match(worker, /WhisperSttRunStateV1::Uncertain/);
  assert.match(worker, /require_ready_match/);
  assert.match(blob, /BlobDataOperationV1::BlobDataOperationReadRangeV1/);
  assert.match(blob, /BlobDataOperationV1::BlobDataOperationWriteV1/);
  assert.match(blob, /custody_target: Some\(ManagedBlobCustodyTargetV1/);
  assert.match(processRoot, /serve-inherited/);
  assert.match(processRoot, /prepare_whisper_stt_resources_v1/);
  assert.doesNotMatch(
    `${admission}\n${resources}\n${runtime}\n${worker}\n${blob}\n${processRoot}`,
    /makosh_communications|communications_|automatic.*download|Command::new\("(?:sh|bash|zsh)"\)/i,
  );
});

test('Whisper assembly emits unsigned runtime storage runner and model inputs only', async () => {
  const [manifest, assembly, cli] = await Promise.all([
    read('backend/src/whisper-stt-assembly/Cargo.toml'),
    read('backend/src/whisper-stt-assembly/src/lib.rs'),
    read('backend/src/whisper-stt-assembly/src/main.rs'),
  ]);

  assert.match(manifest, /surface = "assembly"/);
  assert.match(manifest, /makosh-whisper-stt-runtime/);
  assert.match(assembly, /whisper_stt_module_descriptor_v1/);
  assert.match(assembly, /whisper_stt_settings_schema_v1/);
  assert.match(assembly, /whisper_stt_storage_bundle_v1/);
  assert.match(assembly, /module_runtime_native_executable/);
  assert.match(assembly, /module_runtime_read_only_data/);
  assert.match(assembly, /create_new\(true\)/);
  assert.match(assembly, /mode\(0o600\)/);
  assert.match(cli, /--runner/);
  assert.match(cli, /--model/);
  assert.doesNotMatch(`${assembly}\n${cli}`, /signing|launch|communications_/i);
});

test('Whisper native release build pins source model toolchain and reproducibility', async () => {
  const build = await read('backend/scripts/build-whisper-stt-macos.sh');

  assert.match(build, /23ee03506a91ac3d3f0071b40e66a430eebdfa1d/);
  assert.match(build, /5359861c739e955e79d9a303bcbc70fb988958b1/);
  assert.match(build, /60ed5bc3dd14eea856493d334349b405782ddcaf0028d4b5df4088345fba2efe/);
  assert.match(build, /readonly MODEL_SIZE_BYTES="147951465"/);
  assert.match(build, /800fc86838e913fff969b499886c80baeb4ccfd00f0e39906b34aa334f39ab6c/);
  assert.match(build, /readonly XCODE_VERSION="26\.6"/);
  assert.match(build, /export PATH="\/usr\/bin:\/bin:\/usr\/sbin:\/sbin"/);
  assert.match(build, /-DCMAKE_FIND_USE_SYSTEM_ENVIRONMENT_PATH=OFF/);
  assert.match(build, /-DCMAKE_IGNORE_PREFIX_PATH="\/opt\/homebrew;\/usr\/local"/);
  assert.match(build, /-DBUILD_SHARED_LIBS=OFF/);
  assert.match(build, /-DGGML_STATIC=ON/);
  assert.match(build, /-DGGML_NATIVE=OFF/);
  assert.match(build, /-DGGML_BLAS=OFF/);
  assert.match(build, /-DGGML_METAL=OFF/);
  assert.match(build, /-DWHISPER_CURL=OFF/);
  assert.match(build, /otool -L/);
  assert.match(build, /--verify-reproducibility/);
  assert.match(build, /cmp -s/);
  assert.match(build, /"release_eligible": \$\{reproducibility_verified\}/);
  assert.doesNotMatch(
    build,
    /(?:^|\s)brew(?:\s|$)|Command::new|submodule update --remote|--branch\s/m,
  );
});

test('development release composes the exact Speech-to-Text and Whisper assembly fragments', async () => {
  const release = await read('backend/scripts/materialize-dev-release.sh');

  assert.match(release, /--package makosh-speech-to-text-runtime/);
  assert.match(release, /--package makosh-speech-to-text-assembly/);
  assert.match(release, /debug\/makosh-speech-to-text-assembly/);
  assert.match(
    release,
    /--artifact-fragment "\$speech_to_text_assembly\/speech-to-text\.release-artifacts\.json"/,
  );
  assert.match(release, /MAKOSH_DEV_WHISPER_STT_ROOT/);
  assert.match(release, /build-whisper-stt-macos\.sh/);
  assert.match(release, /--package makosh-whisper-stt-runtime/);
  assert.match(release, /--package makosh-whisper-stt-assembly/);
  assert.match(release, /debug\/makosh-whisper-stt-assembly/);
  assert.match(release, /--runner "\$whisper_stt_runner"/);
  assert.match(release, /--model "\$whisper_stt_model"/);
  assert.match(
    release,
    /--artifact-fragment "\$whisper_stt_assembly\/whisper-stt\.release-artifacts\.json"/,
  );
});

test('Speech-to-Text routes managed Whisper through authenticated custody boundaries', async () => {
  const [root, engineSetup, providerSetup, flow, blob, runner] = await Promise.all([
    read('backend/tests/support/kernel-recovery/src/tests/managed_storage_vault_docker.rs'),
    read('backend/tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/speech_to_text_managed_setup.rs'),
    read('backend/tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/whisper_stt_managed_setup.rs'),
    read('backend/tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/whisper_stt_managed_flow.rs'),
    read('backend/tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/whisper_stt_blob_fixture.rs'),
    read('backend/scripts/test-authenticated-storage.mjs'),
  ]);

  assert.match(root, /mod whisper_stt_managed_flow/);
  assert.match(root, /mod speech_to_text_managed_setup/);
  assert.match(engineSetup, /ModuleRequestRouteHandlerV1/);
  assert.match(engineSetup, /start_reserved_engine/);
  assert.match(engineSetup, /restart_speech_to_text_runtime_v1/);
  assert.match(providerSetup, /install_with_runtime_resources/);
  assert.match(providerSetup, /SignedRuntimeResource::native_executable/);
  assert.match(providerSetup, /SignedRuntimeResource::read_only_data/);
  assert.match(providerSetup, /issue_managed/);
  assert.match(providerSetup, /restart_whisper_stt_runtime_v1/);
  assert.match(flow, /start_from_kernel/);
  assert.match(flow, /SpeechTranscriptDocumentV1::decode/);
  assert.match(flow, /validate_speech_transcript_document_v1/);
  assert.match(flow, /restart_whisper_stt_runtime_v1/);
  assert.match(flow, /restart_speech_to_text_runtime_v1/);
  assert.match(flow, /speech_to_text_contract_reference_v1/);
  assert.doesNotMatch(flow, /speech_to_text_provider_contract_reference_v1/);
  assert.match(flow, /wrong_owner/);
  assert.match(flow, /conflicting/);
  assert.match(blob, /custody_target_owner_id: target\.owner_id/);
  assert.match(blob, /custody_source_proof: blob\.custody_transfer_source_proof/);
  assert.match(runner, /managed_speech_to_text_routes_whisper_private_blob_and_replays_after_restart/);
  assert.match(runner, /MAKOSH_SPEECH_TO_TEXT_RUNTIME_BIN/);
  assert.match(runner, /MAKOSH_WHISPER_STT_RUNNER/);
  assert.match(runner, /MAKOSH_WHISPER_STT_MODEL/);
  assert.match(runner, /MAKOSH_WHISPER_STT_TEST_WAV/);
});

test('Whisper provider and Speech-to-Text engine gates require completed managed conformance', async () => {
  const inventory = JSON.parse(
    await read('backend/architecture/communications-settings-reconstruction.json'),
  );
  const provider = inventory.slices.find((slice) => slice.gate === 'whisper_stt_provider_v1');
  const engine = inventory.slices.find((slice) => slice.gate === 'speech_to_text_engine_v1');
  assert.equal(provider.role, 'integration');
  assert.equal(provider.owner, 'whisper_stt');
  assert.equal(provider.state, 'implemented');
  assert.equal(engine.role, 'engine');
  assert.equal(engine.owner, 'speech_to_text');
  assert.equal(engine.state, 'implemented');
});
