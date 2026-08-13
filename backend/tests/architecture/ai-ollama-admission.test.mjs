import assert from 'node:assert/strict';
import { globSync, readFileSync } from 'node:fs';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import test from 'node:test';

const PROJECT_ROOT = resolve(import.meta.dirname, '../../..');
const absolute = (path) => resolve(PROJECT_ROOT, path);
const read = (path) => readFile(absolute(path), 'utf8');

const exactAiPackages = [
  'makosh-ai-contracts:contract',
  'makosh-ai-inference-core:implementation',
  'makosh-ai-inference-persistence:persistence',
  'makosh-ai-inference-runtime:runtime',
  'makosh-ai-inference-assembly:assembly',
];

const exactOllamaPackages = [
  'makosh-ollama-ai-api:contract',
  'makosh-ollama-ai-assembly:assembly',
  'makosh-ollama-ai-core:implementation',
  'makosh-ollama-ai-http:implementation',
  'makosh-ollama-ai-persistence:persistence',
  'makosh-ollama-ai-runtime:runtime',
];

const exactAiOllamaCapabilities = [
  'ai.attachment-translation.request.v1',
  'ai.explanation.request.v1',
  'ai.inference.blob.v1',
  'ai.inference.request.v1',
  'ai.inference.storage.v1',
  'ai.provider.explain.v1',
  'ai.provider.generate.v1',
  'ai.provider.summarize.v1',
  'ai.provider.translate.v1',
  'ai.summary.request.v1',
  'ai.translation.request.v1',
  'ollama.ai.storage.v1',
];

test('Task 8 atomically admits exact AI runtime and Ollama owner inventory', async () => {
  const policy = JSON.parse(await read('backend/architecture/policy.json'));
  const implementation = policy.implementation;
  const packagesFor = (owner) => implementation.productionPackages
    .filter((descriptor) => descriptor.owner === owner)
    .map(({ name, surface }) => `${name}:${surface}`);

  assert.equal(implementation.currentSlice, 'speech_to_text_whisper_admission_v1');
  assert.equal(implementation.productionPackages.length, 283);
  assert.equal(implementation.ownerInventory.integrations.length, 4);
  assert.equal(implementation.ownerInventory.integrations.includes('ollama'), true);
  assert.equal(implementation.ownerInventory.engines.filter((owner) => owner === 'ai').length, 1);
  assert.equal(implementation.ownerInventory.businessCapabilities.length, 253);
  assert.deepEqual(packagesFor('ai'), exactAiPackages);
  assert.deepEqual(packagesFor('ollama'), exactOllamaPackages);
  assert.deepEqual(
    implementation.ownerInventory.businessCapabilities.filter((capability) =>
      exactAiOllamaCapabilities.includes(capability)),
    exactAiOllamaCapabilities,
  );
});

test('Task 8 keeps exact workspace counts and adds no package or compatibility client', async () => {
  const workspace = await read('backend/Cargo.toml');
  const members = [...workspace.matchAll(/^\s*"([^"]+)",?$/gm)].map((match) => match[1]);
  const manifests = globSync('**/Cargo.toml', {
    cwd: absolute('backend'),
    exclude: ['target/**'],
  });
  const generated = [
    ...globSync('src/gen/makosh/ai/**/*', { cwd: absolute('frontend') }),
    ...globSync('src/gen/makosh/ollama/**/*', { cwd: absolute('frontend') }),
    ...globSync('src/gen/hermes/ai/**/*', { cwd: absolute('frontend') }),
    ...globSync('src/gen/hermes/ollama/**/*', { cwd: absolute('frontend') }),
  ];
  const compatibility = globSync('src/**/*.{ts,vue}', { cwd: absolute('frontend') })
    .flatMap((path) => readFileSync(absolute(`frontend/${path}`), 'utf8')
      .match(/\/api\/v1\/(?:ai|ollama)[^'"\s]*/g) ?? []);

  assert.equal(members.length, 420);
  assert.equal(manifests.length, 421);
  assert.deepEqual(generated, []);
  assert.deepEqual(compatibility, [
    '/api/v1/ai/status',
    '/api/v1/ai/agents',
    '/api/v1/ai/runs?${params.toString()}`,' ,
    '/api/v1/ai/runs/${encodeURIComponent(runId)}`,' ,
    '/api/v1/ai/answers',
    '/api/v1/ai/meeting-prep',
    '/api/v1/ai/task-candidates/refresh',
  ]);
  const adapters = await read('frontend/src/app/client-surfaces/compiledClientSurfaceAdapters.ts');
  assert.doesNotMatch(adapters, /ai-engine|ollama-integration|agents-workspace/);
});

test('Task 8 signs and develops exactly AI inference plus Ollama runtime and storage', async () => {
  const [release, developmentAssembly] = await Promise.all([
    read('backend/scripts/materialize-dev-release.sh'),
    read('backend/development/assembly/src/main.rs'),
  ]);
  for (const fragment of [
    'ai-inference.release-artifacts.json',
    'ollama-ai.release-artifacts.json',
  ]) {
    assert.equal(release.includes(fragment), true, fragment);
  }
  for (const artifact of [
    'ai_inference.runtime.v1',
    'ai_inference.storage.v1',
    'ollama_ai.runtime.v1',
    'ollama_ai.storage.v1',
  ]) {
    assert.equal(developmentAssembly.includes(artifact), true, artifact);
  }
});

test('Task 8 exposes only exact owner-managed loopback settings and no direct client RPC', async () => {
  const [settings, http, aiAdmission, ollamaAdmission, settingsClient] = await Promise.all([
    read('backend/src/ollama-ai-api/src/settings.rs'),
    read('backend/src/ollama-ai-http/src/wire.rs'),
    read('backend/src/ai-inference-runtime/src/admission.rs'),
    read('backend/src/ollama-ai-runtime/src/admission.rs'),
    read('frontend/src/platform/settings/ownerModuleSettingsClient.ts'),
  ]);

  assert.match(settings, /ollama\.chat_model/);
  assert.match(settings, /ollama\.port/);
  assert.match(settings, /ollama\.timeout_millis/);
  assert.match(settings, /SettingMutationAuthorityV1::OperatorManaged/);
  assert.match(settings, /fresh_owner_proof_required: true/);
  assert.doesNotMatch(
    settings,
    /"ollama\.(?:password|token|secret|credential|endpoint|host)"/i,
  );
  assert.match(http, /127\.0\.0\.1/);
  assert.match(http, /\/api\/tags/);
  assert.match(http, /\/api\/chat/);
  assert.doesNotMatch(`${aiAdmission}\n${ollamaAdmission}`, /client_rpc_route: Some/);
  assert.match(settingsClient, /deviceProof\.sign\(prepared\.challengeBytes\)/);
  assert.match(settingsClient, /prepared\.expiresAtUnixMillis <= BigInt\(Date\.now\(\)\)/);
});

test('Task 8 requires FORCE RLS and fresh default plus live-provider managed evidence', async () => {
  const [aiSchema, ollamaSchema, runner, execution, sharedFixture, aiFlow, ollamaFlow] = await Promise.all([
    read('backend/src/ai-inference-persistence/src/schema.rs'),
    read('backend/src/ollama-ai-persistence/src/schema.rs'),
    read('backend/scripts/test-authenticated-storage.mjs'),
    read('backend/src/kernel/src/runtime/managed/execution.rs'),
    read('backend/tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/shared_fixture.rs'),
    read('backend/tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/ai_inference_managed_flow.rs'),
    read('backend/tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/ollama_ai_managed_flow.rs'),
  ]);

  assert.match(aiSchema, /AI_INFERENCE_STORAGE_BUNDLE_REVISION_V1: u32 = 6/);
  assert.match(ollamaSchema, /OLLAMA_AI_STORAGE_BUNDLE_REVISION_V1: u32 = 5/);
  assert.match(aiSchema, /0006_ai_inference_owner_rls\.sql/);
  assert.match(ollamaSchema, /0005_ollama_ai_owner_rls\.sql/);
  assert.match(runner, /MAKOSH_STORAGE_MANAGED_TEST_FILTER/);
  assert.match(execution, /MANAGED_CHILD_TEST_STDIO_CAPTURE_DIRECTORY_ENV/);
  assert.match(execution, /#\[cfg\(test\)\]/);
  assert.match(execution, /stdout\(Stdio::null\(\)\)/);
  assert.match(sharedFixture, /runtime_storage_credential_for_registration_v1/);
  assert.match(sharedFixture, /issue_runtime_credential/);
  assert.match(sharedFixture, /resolve_runtime_credential/);
  assert.doesNotMatch(runner, /makosh-vault-runtime\/conformance-test-support/);
  assert.match(aiFlow, /managed_ai_inference_completes_real_provider_generation/);
  assert.match(aiFlow, /managed_ai_inference_bootstrap_fails_closed_and_stops_promptly/);
  assert.match(aiFlow, /assert_ai_runtime_bootstrap_active_until_requested_stop_v1/);
  assert.match(aiFlow, /AI_PRIVATE_INPUT_SENTINEL_V1/);
  assert.match(aiFlow, /assert_supervised_ai_child_output_is_private_v1/);
  assert.match(aiFlow, /runtime_storage_credential_for_registration_v1/);
  assert.match(aiFlow, /required\("MAKOSH_OLLAMA_LIVE_PORT"\)/);
  assert.match(ollamaFlow, /managed_ollama_ai_runtime_completes_real_provider_generation/);
  assert.match(ollamaFlow, /managed_ollama_ai_bootstrap_fails_closed_and_stops_promptly/);
  assert.match(ollamaFlow, /assert_ollama_runtime_bootstrap_active_until_requested_stop_v1/);
  assert.match(ollamaFlow, /OLLAMA_RAW_PROVIDER_SENTINEL_V1/);
  assert.match(ollamaFlow, /runtime_storage_credential_for_registration_v1/);
  assert.doesNotMatch(ollamaFlow, /MAKOSH_STORAGE_AUTHENTICATED_PGBOUNCER_PASSWORD_FILE/);
  assert.doesNotMatch(ollamaFlow, /std::process::Command::new\(executable\)/);
  assert.match(ollamaFlow, /required\("MAKOSH_OLLAMA_LIVE_PORT"\)/);
});
