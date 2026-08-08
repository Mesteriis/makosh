import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, mkdtempSync, readFileSync, realpathSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const backendRoot = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const inputBuilder = join(backendRoot, 'scripts/build-local-platform-release-input.mjs');
const platformBinaries = [
  'makosh-blob-service',
  'makosh-events-authority-runtime',
  'makosh-scheduler-runtime',
  'makosh-storage-runtime',
  'makosh-telemetry-collector',
  'makosh-vault-runtime',
];

function temporaryDirectory(prefix) {
  return mkdtempSync(join(realpathSync(tmpdir()), prefix));
}

function decodeVarint(bytes, offset) {
  let value = 0n;
  let shift = 0n;
  let cursor = offset;
  while (cursor < bytes.length) {
    const byte = bytes[cursor];
    cursor += 1;
    value |= BigInt(byte & 0x7f) << shift;
    if ((byte & 0x80) === 0) return [value, cursor];
    shift += 7n;
  }
  throw new Error('truncated protobuf varint');
}

function decodeFields(bytes) {
  const fields = new Map();
  let offset = 0;
  while (offset < bytes.length) {
    const [tag, afterTag] = decodeVarint(bytes, offset);
    const field = Number(tag >> 3n);
    const wireType = Number(tag & 0x07n);
    offset = afterTag;
    if (wireType === 0) {
      const [value, afterValue] = decodeVarint(bytes, offset);
      fields.set(field, [...(fields.get(field) ?? []), value]);
      offset = afterValue;
      continue;
    }
    assert.equal(wireType, 2);
    const [length, afterLength] = decodeVarint(bytes, offset);
    const end = afterLength + Number(length);
    fields.set(field, [...(fields.get(field) ?? []), bytes.subarray(afterLength, end)]);
    offset = end;
  }
  return fields;
}

function fieldString(fields, number) {
  return fields.get(number)?.[0]?.toString('utf8');
}

function runBuilder(root, browserAssetsDirectory = null, generation = '7') {
  const artifactDirectory = join(root, 'artifacts');
  const browserBootstrap = join(root, 'bootstrap.html');
  const output = join(root, 'release-input.json');
  const descriptorDirectory = join(root, 'contracts');
  return {
    artifactDirectory,
    descriptorDirectory,
    output,
    result: spawnSync(process.execPath, [
      inputBuilder,
      '--target', 'aarch64-apple-darwin',
      '--artifact-dir', artifactDirectory,
      '--browser-bootstrap', browserBootstrap,
      '--output', output,
      '--descriptor-dir', descriptorDirectory,
      '--distribution-id', 'makosh-desktop',
      '--generation', generation,
      '--release-version', '0.1.0',
      '--build-id', 'local-platform-v1',
      '--source-commit', '1'.repeat(40),
      '--lockfile-sha256', '2'.repeat(64),
      '--sbom-sha256', '3'.repeat(64),
      '--toolchain-sha256', '4'.repeat(64),
      ...(browserAssetsDirectory === null ? [] : ['--browser-assets-dir', browserAssetsDirectory]),
    ], { cwd: backendRoot, encoding: 'utf8' }),
  };
}

function createFixture(root, missingBinary = null) {
  const artifactDirectory = join(root, 'artifacts');
  const browserBootstrap = join(root, 'bootstrap.html');
  mkdirSync(artifactDirectory, { mode: 0o700 });
  writeFileSync(browserBootstrap, '<!doctype html>');
  for (const binary of platformBinaries) {
    if (binary !== missingBinary) writeFileSync(join(artifactDirectory, binary), binary, { mode: 0o700 });
  }
}

test('creates exact local platform runtime contracts before compiling a distribution', () => {
  const root = temporaryDirectory('makosh-local-platform-release-');
  try {
    createFixture(root);
    const { descriptorDirectory, output, result } = runBuilder(root);
    assert.equal(result.status, 0, result.stderr);
    const input = JSON.parse(readFileSync(output, 'utf8'));
    assert.equal(input.source_commit, '1'.repeat(40));
    assert.equal(input.lockfile_sha256, '2'.repeat(64));
    assert.equal(input.sbom_sha256, '3'.repeat(64));
    assert.equal(input.toolchain_sha256, '4'.repeat(64));
    assert.equal(input.generation, 7);
    assert.deepEqual(input.artifacts.map((artifact) => artifact.artifact_id), [
      'browser.bootstrap',
      'platform.blob',
      'platform.events-authority',
      'platform.scheduler',
      'platform.storage',
      'platform.telemetry',
      'platform.vault',
    ]);
    const scheduler = input.artifacts.find((artifact) => artifact.artifact_id === 'platform.scheduler');
    assert.equal(scheduler.artifact_kind, 'module_runtime');
    assert.equal(scheduler.relative_path, 'bin/makosh-scheduler-runtime');
    assert.equal(readFileSync(scheduler.settings_schema.source_path).toString('hex'), '08011001');
    const descriptor = decodeFields(readFileSync(scheduler.descriptor.source_path));
    assert.equal(descriptor.get(2)?.[0], 2n);
    assert.equal(fieldString(descriptor, 3), 'scheduler');
    assert.equal(fieldString(descriptor, 4), 'scheduler');
    assert.equal(descriptor.get(5)?.[0], 5n);
    const schemaReference = decodeFields(descriptor.get(10)[0]);
    assert.equal(schemaReference.get(3)?.[0], 4n);
    assert.equal(schemaReference.get(4)?.[0].length, 32);
    assert.equal(existsSync(descriptorDirectory), true);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test('includes exact compiled browser assets in the signed distribution input', () => {
  const root = temporaryDirectory('makosh-local-platform-browser-assets-');
  try {
    createFixture(root);
    const browserAssetsDirectory = join(root, 'browser-assets');
    mkdirSync(browserAssetsDirectory, { recursive: true, mode: 0o700 });
    writeFileSync(join(root, 'bootstrap.html'), '<script src="/assets/app.js"></script>');
    writeFileSync(join(browserAssetsDirectory, 'app.js'), 'import "/assets/theme.css"');
    writeFileSync(join(browserAssetsDirectory, 'theme.css'), 'compiled browser styles');
    writeFileSync(join(browserAssetsDirectory, 'unused.png'), 'unreferenced source asset');

    const { output, result } = runBuilder(root, browserAssetsDirectory);
    assert.equal(result.status, 0, result.stderr);
    const input = JSON.parse(readFileSync(output, 'utf8'));
    assert.deepEqual(
      input.artifacts
        .filter((artifact) => artifact.artifact_kind === 'browser_client_asset')
        .map(({ artifact_id, relative_path, source_path, required }) => ({ artifact_id, relative_path, source_path, required })),
      [
        {
          artifact_id: 'browser.asset.app.js',
          relative_path: 'browser/assets/app.js',
          source_path: join(browserAssetsDirectory, 'app.js'),
          required: true,
        },
        {
          artifact_id: 'browser.asset.theme.css',
          relative_path: 'browser/assets/theme.css',
          source_path: join(browserAssetsDirectory, 'theme.css'),
          required: true,
        },
      ],
    );
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test('orders Vite asset identifiers by exact ASCII bytes across punctuation', () => {
  const root = temporaryDirectory('makosh-local-platform-browser-asset-order-');
  try {
    createFixture(root);
    const browserAssetsDirectory = join(root, 'browser-assets');
    mkdirSync(browserAssetsDirectory, { recursive: true, mode: 0o700 });
    writeFileSync(
      join(root, 'bootstrap.html'),
      '<link href="/assets/index--style.css"><script src="/assets/index-_runtime.js"></script>',
    );
    writeFileSync(join(browserAssetsDirectory, 'index--style.css'), 'compiled styles');
    writeFileSync(join(browserAssetsDirectory, 'index-_runtime.js'), 'compiled runtime');

    const { output, result } = runBuilder(root, browserAssetsDirectory);
    assert.equal(result.status, 0, result.stderr);
    const input = JSON.parse(readFileSync(output, 'utf8'));
    assert.deepEqual(
      input.artifacts
        .filter((artifact) => artifact.artifact_kind === 'browser_client_asset')
        .map((artifact) => artifact.artifact_id),
      ['browser.asset.index--style.css', 'browser.asset.index-_runtime.js'],
    );
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test('rejects a referenced browser asset outside the signed Gateway allowlist', () => {
  const root = temporaryDirectory('makosh-local-platform-browser-invalid-asset-');
  try {
    createFixture(root);
    const browserAssetsDirectory = join(root, 'browser-assets');
    mkdirSync(browserAssetsDirectory, { mode: 0o700 });
    writeFileSync(join(root, 'bootstrap.html'), '<script src="/assets/runtime.wasm"></script>');
    writeFileSync(join(browserAssetsDirectory, 'runtime.wasm'), 'unsigned execution format');

    const { output, result } = runBuilder(root, browserAssetsDirectory);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /browser asset reference is invalid/);
    assert.equal(existsSync(output), false);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test('rejects an incomplete platform inventory before creating contracts', () => {
  const root = temporaryDirectory('makosh-local-platform-release-missing-');
  try {
    createFixture(root, 'makosh-vault-runtime');
    const { descriptorDirectory, output, result } = runBuilder(root);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /platform\.vault binary is unavailable: ENOENT/);
    assert.equal(existsSync(output), false);
    assert.equal(existsSync(descriptorDirectory), false);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test('rejects a non-positive or unsafe distribution generation', () => {
  for (const generation of ['0', '-1', '9007199254740992']) {
    const root = temporaryDirectory('makosh-local-platform-generation-');
    try {
      createFixture(root);
      const { output, result } = runBuilder(root, null, generation);
      assert.equal(result.status, 1);
      assert.match(result.stderr, /--generation must be a positive safe integer/);
      assert.equal(existsSync(output), false);
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  }
});
