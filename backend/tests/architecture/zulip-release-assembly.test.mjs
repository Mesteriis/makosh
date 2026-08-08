import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);

test('Zulip storage and release assembly remain separate owner-local units', async () => {
  const [
    workspace,
    persistenceManifest,
    persistenceSchema,
    runtimeManifest,
    assemblyManifest,
    assemblySource,
  ] = await Promise.all([
    readFile(new URL('Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL('src/zulip-persistence/Cargo.toml', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/zulip-persistence/src/schema.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(new URL('src/zulip-runtime/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/zulip-assembly/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/zulip-assembly/src/lib.rs', BACKEND_ROOT), 'utf8'),
  ]);

  assert.match(workspace, /"src\/zulip-assembly"/);
  assert.match(persistenceManifest, /surface = "persistence"/);
  assert.match(persistenceManifest, /makosh-storage-protocol/);
  assert.match(persistenceSchema, /owner_id: "zulip"\.to_owned\(\)/);
  assert.match(
    persistenceSchema,
    /forward_sql_utf8: ZULIP_SCHEMA_V1\.as_bytes\(\)\.to_vec\(\)/,
  );
  assert.match(
    persistenceSchema,
    /sha256: Sha256::digest\(ZULIP_SCHEMA_V1\.as_bytes\(\)\)\.to_vec\(\)/,
  );

  assert.match(assemblyManifest, /role = "integration"/);
  assert.match(assemblyManifest, /owner = "zulip"/);
  assert.match(assemblyManifest, /surface = "assembly"/);
  for (const dependency of [
    'makosh-zulip-runtime',
    'makosh-zulip-persistence',
    'makosh-runtime-protocol',
    'makosh-storage-protocol',
  ]) {
    assert.match(assemblyManifest, new RegExp(dependency));
  }
  for (const forbiddenDependency of [
    'makosh-kernel',
    'makosh-gateway',
    'makosh-communications',
    'ring',
    'sha2',
  ]) {
    assert.doesNotMatch(assemblyManifest, new RegExp(forbiddenDependency));
  }
  assert.doesNotMatch(runtimeManifest, /makosh-zulip-assembly/);
  assert.doesNotMatch(persistenceManifest, /makosh-zulip-assembly/);

  assert.match(
    assemblySource,
    /zulip_module_descriptor_v1\(build_id\)/,
  );
  assert.match(assemblySource, /zulip_settings_schema_v3\(\)/);
  assert.match(assemblySource, /zulip_storage_bundle_v1\(\)/);
  assert.match(assemblySource, /"module_runtime"\.to_owned\(\)/);
  assert.match(assemblySource, /"storage_bundle"\.to_owned\(\)/);
  assert.doesNotMatch(assemblySource, /NativeDependency/);
});
