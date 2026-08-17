import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const REPOSITORY_ROOT = new URL('../', BACKEND_ROOT);

async function backendSource(path) {
  return readFile(new URL(path, BACKEND_ROOT), 'utf8');
}

test('runtime artifact binding is one owner-neutral private bootstrap contract', async () => {
  const [binding, integration, workflow, engine, adr] = await Promise.all([
    backendSource(
      'src/platform/runtime_protocol/proto/makosh/runtime/v1/managed_runtime_artifact.proto',
    ),
    backendSource(
      'src/platform/runtime_protocol/proto/makosh/runtime/v1/managed_integration_runtime.proto',
    ),
    backendSource(
      'src/platform/runtime_protocol/proto/makosh/runtime/v1/managed_workflow_runtime.proto',
    ),
    backendSource(
      'src/platform/runtime_protocol/proto/makosh/runtime/v1/managed_engine_runtime.proto',
    ),
    readFile(
      new URL(
        'docs/adr/ADR-0372-kernel-staged-runtime-resources-for-managed-workflows.md',
        REPOSITORY_ROOT,
      ),
      'utf8',
    ),
  ]);

  assert.match(binding, /message ManagedRuntimeArtifactBindingV1/);
  assert.match(binding, /RuntimeArtifactUseV1 use = 2/);
  assert.doesNotMatch(integration, /message ManagedRuntimeArtifactBindingV1/);
  for (const configuration of [integration, workflow, engine]) {
    assert.match(configuration, /import "makosh\/runtime\/v1\/managed_runtime_artifact.proto"/);
    assert.match(configuration, /repeated ManagedRuntimeArtifactBindingV1 runtime_artifacts/);
  }
  assert.match(adr, /Gateway, Event Hub, Settings Registry, client API, health и telemetry этот\n+binding не видят/);
});

test('runtime resource types are exact and domains cannot request them', async () => {
  const [recovery, distribution, descriptor, validator] = await Promise.all([
    backendSource('src/platform/runtime_protocol/proto/makosh/runtime/v1/recovery.proto'),
    backendSource('src/platform/runtime_protocol/proto/makosh/runtime/v1/distribution.proto'),
    backendSource('src/platform/runtime_protocol/src/validation/descriptor.rs'),
    backendSource(
      'src/platform/runtime_protocol/src/validation/managed_runtime_artifact.rs',
    ),
  ]);

  assert.match(recovery, /RUNTIME_ARTIFACT_USE_V1_NATIVE_DYNAMIC_LIBRARY = 1/);
  assert.match(recovery, /RUNTIME_ARTIFACT_USE_V1_NATIVE_EXECUTABLE = 2/);
  assert.match(recovery, /RUNTIME_ARTIFACT_USE_V1_READ_ONLY_DATA = 3/);
  assert.match(distribution, /MODULE_RUNTIME_NATIVE_EXECUTABLE = 7/);
  assert.match(distribution, /MODULE_RUNTIME_READ_ONLY_DATA = 8/);
  assert.match(
    descriptor,
    /ModuleKindV1::Integration \| ModuleKindV1::Workflow \| ModuleKindV1::Engine/,
  );
  assert.match(descriptor, /workflow\.module_kind = ModuleKindV1::Domain/);
  assert.match(validator, /paths\.insert\(artifact\.staged_path\.as_str\(\)\)/);
  assert.match(validator, /artifact\.sha256\.iter\(\)\.any/);
});

test('Kernel stages granted workflow and engine resources without owner semantics', async () => {
  const [selector, nativeLaunch, managedLaunch, ownerControl, stagedArtifact] =
    await Promise.all([
      backendSource('src/kernel/src/distribution/runtime_dependencies.rs'),
      backendSource('src/kernel/src/platform/macos/native_launch.rs'),
      backendSource('src/kernel/src/platform/macos/managed_launch.rs'),
      backendSource('src/kernel/src/identity/owner_control/dispatch.rs'),
      backendSource('src/kernel/src/distribution/staged_artifact.rs'),
    ]);

  assert.match(selector, /ModuleKindV1::Integration \| ModuleKindV1::Workflow \| ModuleKindV1::Engine/);
  assert.match(selector, /artifact\.artifact_kind != distribution_kind\(use_kind\) as i32/);
  assert.match(selector, /artifact\.bound_module_id != descriptor\.module_id/);
  assert.match(selector, /managed runtime artifact use is ambiguous/);
  assert.match(nativeLaunch, /prepare_bound_managed_runtime_with_artifacts/);
  assert.match(nativeLaunch, /r#use: request\.use_kind\(\) as i32/);
  assert.match(nativeLaunch, /cleanup_staged_runtime_artifacts/);
  assert.match(nativeLaunch, /remove_dir_all\(artifact_directory\)/);
  assert.match(managedLaunch, /fn prepare_runtime_with_artifacts/);
  assert.match(managedLaunch, /fn managed_launch_directory/);
  assert.match(
    managedLaunch,
    /reservation\.runtime_generation\(\),\s*reservation\.runtime_instance_id\(\)/,
  );
  assert.doesNotMatch(
    managedLaunch,
    /join\(format!\("launch-\{\}", reservation\.runtime_generation\(\)\)\)/,
  );
  assert.match(managedLaunch, /configuration\.runtime_artifacts = prepared\.runtime_artifact_bindings/);
  assert.match(ownerControl, /effective_granted_capability_ids/);
  assert.match(stagedArtifact, /StagedArtifactAccessV1::ReadOnly => 0o400/);
  assert.match(stagedArtifact, /StagedArtifactAccessV1::ReadExecute => 0o500/);
  const selectorProduction = selector.slice(0, selector.indexOf('#[cfg(test)]'));
  assert.doesNotMatch(
    `${selectorProduction}\n${nativeLaunch}\n${managedLaunch}`,
    /tesseract|ocr\.eng|ocr\.rus/i,
  );
});
