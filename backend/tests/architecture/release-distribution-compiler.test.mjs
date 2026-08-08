import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { generateKeyPairSync, verify } from 'node:crypto';
import {
  chmodSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  realpathSync,
  rmSync,
  statSync,
  writeFileSync,
} from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

import {
  compileUnsignedReleaseContent,
  compileReleaseDistribution,
  composeReleaseCompilerInput,
  generateReleaseSigningKey,
  loadReleaseSigningKey,
  writeReleaseArtifact,
} from '../../scripts/lib/release-distribution-compiler.mjs';

const browserBootstrapSource = resolve(
  dirname(fileURLToPath(import.meta.url)),
  '../../../frontend/browser-bootstrap/index.html',
);
const releaseProvenance = {
  source_commit: 'a'.repeat(40),
  lockfile_sha256: 'b'.repeat(64),
  sbom_sha256: 'c'.repeat(64),
  toolchain_sha256: 'd'.repeat(64),
};

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

function canonicalTemporaryDirectory(prefix) {
  return mkdtempSync(join(realpathSync(tmpdir()), prefix));
}

test('compiles a signed P-256 distribution manifest and matching trust root', async () => {
  const root = canonicalTemporaryDirectory('makosh-release-compiler-');
  try {
    const runtime = join(root, 'runtime');
    const descriptor = join(root, 'descriptor.pb');
    const privateKeyPath = join(root, 'release-key.pem');
    writeFileSync(runtime, 'runtime bytes', { mode: 0o700 });
    writeFileSync(descriptor, 'descriptor bytes', { mode: 0o600 });
    const keyPair = generateKeyPairSync('ec', { namedCurve: 'prime256v1' });
    writeFileSync(privateKeyPath, keyPair.privateKey.export({ type: 'pkcs8', format: 'pem' }), {
      mode: 0o600,
    });
    const artifacts = await compileReleaseDistribution({
      verification_key_id: 'release-2026',
      trust_root_revision: 1,
      revision: 1,
      distribution_id: 'makosh-desktop',
      release_version: '1.0.0',
      build_id: 'build-1',
      target_triple: 'aarch64-apple-darwin',
      generation: 1,
      ...releaseProvenance,
      additional_verification_keys: [],
      artifacts: [{
        artifact_kind: 'module_runtime',
        artifact_id: 'runtime.mail',
        relative_path: 'bin/mail',
        source_path: runtime,
        required: true,
        descriptor: {
          relative_path: 'contracts/mail.pb',
          source_path: descriptor,
        },
        settings_schema: null,
      }],
    }, loadReleaseSigningKey(privateKeyPath));

    const signed = decodeFields(artifacts.signedManifest);
    const rawManifest = signed.get(2)?.[0];
    assert.equal(fieldString(signed, 1), 'release-2026');
    assert.equal(signed.get(3)?.[0].length, 64);
    assert.ok(verify(
      'sha256',
      rawManifest,
      { key: keyPair.publicKey, dsaEncoding: 'ieee-p1363' },
      signed.get(3)[0],
    ));
    const manifest = decodeFields(rawManifest);
    assert.equal(fieldString(manifest, 3), 'makosh-desktop');
    assert.equal(manifest.get(8)?.length, 1);
    const artifact = decodeFields(manifest.get(8)[0]);
    assert.equal(fieldString(artifact, 2), 'runtime.mail');
    assert.equal(artifact.get(5)[0].length, 32);
    assert.equal(artifact.get(6)[0].length, 32);

    const trustRoot = decodeFields(artifacts.trustRoot);
    assert.equal(trustRoot.get(1)[0], 1n);
    const trustKey = decodeFields(trustRoot.get(3)[0]);
    assert.equal(fieldString(trustKey, 1), 'release-2026');
    assert.equal(trustKey.get(2)[0].length, 65);
    assert.equal(trustKey.get(2)[0][0], 4);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test('produces an identical unsigned content manifest from independent build inputs', async () => {
  const root = canonicalTemporaryDirectory('makosh-release-reproducibility-');
  try {
    const firstRoot = join(root, 'first');
    const secondRoot = join(root, 'second');
    const firstRuntime = join(firstRoot, 'runtime');
    const secondRuntime = join(secondRoot, 'runtime');
    const firstDescriptor = join(firstRoot, 'descriptor.pb');
    const secondDescriptor = join(secondRoot, 'descriptor.pb');
    mkdirSync(firstRoot, { mode: 0o700 });
    mkdirSync(secondRoot, { mode: 0o700 });
    writeFileSync(firstRuntime, 'runtime bytes', { mode: 0o700 });
    writeFileSync(secondRuntime, 'runtime bytes', { mode: 0o700 });
    writeFileSync(firstDescriptor, 'descriptor bytes', { mode: 0o600 });
    writeFileSync(secondDescriptor, 'descriptor bytes', { mode: 0o600 });
    const first = await compileUnsignedReleaseContent(releaseInput(firstRuntime, firstDescriptor, browserBootstrapSource));
    const second = await compileUnsignedReleaseContent(releaseInput(secondRuntime, secondDescriptor, browserBootstrapSource));
    assert.deepEqual(first.rawManifest, second.rawManifest);
    writeFileSync(secondRuntime, 'different runtime bytes', { mode: 0o700 });
    const changed = await compileUnsignedReleaseContent(releaseInput(secondRuntime, secondDescriptor, browserBootstrapSource));
    assert.notDeepEqual(first.rawManifest, changed.rawManifest);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test('accepts exact ASCII artifact order when Vite hashes introduce punctuation', async () => {
  const root = canonicalTemporaryDirectory('makosh-release-ascii-artifact-order-');
  try {
    const css = join(root, 'index--style.css');
    const javascript = join(root, 'index-_runtime.js');
    writeFileSync(css, 'compiled styles');
    writeFileSync(javascript, 'compiled runtime');
    await assert.doesNotReject(compileUnsignedReleaseContent({
      verification_key_id: 'release-2026',
      trust_root_revision: 1,
      revision: 1,
      distribution_id: 'makosh-desktop',
      release_version: '1.0.0',
      build_id: 'build-browser-assets',
      target_triple: 'aarch64-apple-darwin',
      generation: 1,
      ...releaseProvenance,
      additional_verification_keys: [],
      artifacts: [
        {
          artifact_kind: 'browser_client_asset',
          artifact_id: 'browser.asset.index--style.css',
          relative_path: 'browser/assets/index--style.css',
          source_path: css,
          required: true,
        },
        {
          artifact_kind: 'browser_client_asset',
          artifact_id: 'browser.asset.index-_runtime.js',
          relative_path: 'browser/assets/index-_runtime.js',
          source_path: javascript,
          required: true,
        },
      ],
    }));
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test('release reproducibility CLI refuses divergent unsigned content before signing', () => {
  const root = canonicalTemporaryDirectory('makosh-release-reproducibility-cli-');
  try {
    const firstRoot = join(root, 'first');
    const secondRoot = join(root, 'second');
    mkdirSync(firstRoot, { mode: 0o700 });
    mkdirSync(secondRoot, { mode: 0o700 });
    const firstRuntime = join(firstRoot, 'runtime');
    const secondRuntime = join(secondRoot, 'runtime');
    const firstDescriptor = join(firstRoot, 'descriptor.pb');
    const secondDescriptor = join(secondRoot, 'descriptor.pb');
    writeFileSync(firstRuntime, 'runtime bytes', { mode: 0o700 });
    writeFileSync(secondRuntime, 'runtime bytes', { mode: 0o700 });
    writeFileSync(firstDescriptor, 'descriptor bytes', { mode: 0o600 });
    writeFileSync(secondDescriptor, 'descriptor bytes', { mode: 0o600 });
    const firstInput = join(firstRoot, 'release.json');
    const secondInput = join(secondRoot, 'release.json');
    writeFileSync(firstInput, JSON.stringify(releaseInput(firstRuntime, firstDescriptor, browserBootstrapSource)), { mode: 0o600 });
    writeFileSync(secondInput, JSON.stringify(releaseInput(secondRuntime, secondDescriptor, browserBootstrapSource)), { mode: 0o600 });
    const command = [
      'scripts/verify-release-reproducibility.mjs', '--first-input', firstInput,
      '--second-input', secondInput,
    ];
    execFileSync(process.execPath, command, { cwd: process.cwd(), stdio: 'pipe' });
    writeFileSync(secondRuntime, 'different runtime bytes', { mode: 0o700 });
    assert.throws(
      () => execFileSync(process.execPath, command, { cwd: process.cwd(), stdio: 'pipe' }),
      /unsigned content manifests differ/,
    );
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test('binds a browser bootstrap document as a non-module signed release artifact', async () => {
  const root = canonicalTemporaryDirectory('makosh-browser-bootstrap-release-');
  try {
    const privateKeyPath = join(root, 'release-key.pem');
    assert.match(readFileSync(browserBootstrapSource, 'utf8'), /navigator\.credentials\.create/);
    const keyPair = generateKeyPairSync('ec', { namedCurve: 'prime256v1' });
    writeFileSync(privateKeyPath, keyPair.privateKey.export({ type: 'pkcs8', format: 'pem' }), {
      mode: 0o600,
    });
    const release = await compileReleaseDistribution({
      verification_key_id: 'release-2026', trust_root_revision: 1, revision: 1,
      distribution_id: 'makosh-desktop', release_version: '1.0.0', build_id: 'build-browser',
      target_triple: 'aarch64-apple-darwin', generation: 1, ...releaseProvenance, additional_verification_keys: [],
      artifacts: [{
        artifact_kind: 'browser_bootstrap_bundle', artifact_id: 'browser.bootstrap',
        relative_path: 'browser/bootstrap.html', source_path: browserBootstrapSource, required: true,
      }],
    }, loadReleaseSigningKey(privateKeyPath));
    const signed = decodeFields(release.signedManifest);
    const manifest = decodeFields(signed.get(2)[0]);
    const artifact = decodeFields(manifest.get(8)[0]);
    assert.equal(artifact.get(1)[0], 4n);
    assert.equal(fieldString(artifact, 2), 'browser.bootstrap');
    assert.equal(artifact.get(5)[0].length, 32);
    assert.equal(artifact.get(6), undefined);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test('binds a native runtime dependency to one exact managed module', async () => {
  const root = canonicalTemporaryDirectory('makosh-native-dependency-release-');
  try {
    const library = join(root, 'libtdjson.dylib');
    writeFileSync(library, 'native library bytes', { mode: 0o700 });
    const release = await compileUnsignedReleaseContent({
      verification_key_id: 'release-2026',
      trust_root_revision: 1,
      revision: 1,
      distribution_id: 'makosh-desktop',
      release_version: '1.0.0',
      build_id: 'build-native-dependency',
      target_triple: 'aarch64-apple-darwin',
      generation: 1,
      ...releaseProvenance,
      additional_verification_keys: [],
      artifacts: [{
        artifact_kind: 'module_runtime_native_dependency',
        artifact_id: 'telegram.tdjson.v1',
        relative_path: 'lib/libtdjson.dylib',
        source_path: library,
        required: true,
        bound_module_id: 'makosh-telegram-runtime',
      }],
    });
    const manifest = decodeFields(release.rawManifest);
    const artifact = decodeFields(manifest.get(8)[0]);
    assert.equal(artifact.get(1)[0], 6n);
    assert.equal(fieldString(artifact, 2), 'telegram.tdjson.v1');
    assert.equal(fieldString(artifact, 13), 'makosh-telegram-runtime');
    assert.equal(artifact.get(6), undefined);

    const invalid = {
      verification_key_id: 'release-2026',
      trust_root_revision: 1,
      revision: 1,
      distribution_id: 'makosh-desktop',
      release_version: '1.0.0',
      build_id: 'build-invalid-native-dependency',
      target_triple: 'aarch64-apple-darwin',
      generation: 1,
      ...releaseProvenance,
      additional_verification_keys: [],
      artifacts: [{
        artifact_kind: 'module_runtime_native_dependency',
        artifact_id: 'telegram.tdjson.v1',
        relative_path: 'lib/libtdjson.dylib',
        source_path: library,
        required: true,
        bound_module_id: 'Telegram Runtime',
      }],
    };
    await assert.rejects(
      compileUnsignedReleaseContent(invalid),
      /runtime artifact binding is invalid/,
    );
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test('binds native executables and read-only data to one exact managed module', async () => {
  const root = canonicalTemporaryDirectory('makosh-runtime-resources-release-');
  try {
    const runner = join(root, 'tesseract-runner');
    const model = join(root, 'eng.traineddata');
    writeFileSync(runner, 'runner bytes', { mode: 0o500 });
    writeFileSync(model, 'model bytes', { mode: 0o400 });
    const release = await compileUnsignedReleaseContent({
      verification_key_id: 'release-2026',
      trust_root_revision: 1,
      revision: 1,
      distribution_id: 'makosh-desktop',
      release_version: '1.0.0',
      build_id: 'build-runtime-resources',
      target_triple: 'aarch64-apple-darwin',
      generation: 1,
      ...releaseProvenance,
      additional_verification_keys: [],
      artifacts: [
        {
          artifact_kind: 'module_runtime_read_only_data',
          artifact_id: 'attachment_text_extraction.ocr.eng.v1',
          relative_path: 'runtime-resources/eng.traineddata',
          source_path: model,
          required: true,
          bound_module_id: 'makosh-attachment-text-extraction-runtime',
        },
        {
          artifact_kind: 'module_runtime_native_executable',
          artifact_id: 'attachment_text_extraction.ocr.runner.v1',
          relative_path: 'runtime-resources/tesseract-runner',
          source_path: runner,
          required: true,
          bound_module_id: 'makosh-attachment-text-extraction-runtime',
        },
      ],
    });
    const manifest = decodeFields(release.rawManifest);
    const artifacts = manifest.get(8).map(decodeFields);
    assert.deepEqual(artifacts.map((artifact) => artifact.get(1)[0]), [8n, 7n]);
    assert.ok(artifacts.every(
      (artifact) => fieldString(artifact, 13) === 'makosh-attachment-text-extraction-runtime',
    ));
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test('composes an exact Telegram artifact fragment before signing the full distribution', async () => {
  const root = canonicalTemporaryDirectory('makosh-telegram-release-fragment-');
  try {
    const runtime = join(root, 'makosh-telegram-runtime');
    const descriptor = join(root, 'telegram.runtime.descriptor.pb');
    const settings = join(root, 'telegram.runtime.settings.pb');
    const storage = join(root, 'telegram.storage.bundle.pb');
    const tdjson = join(root, 'libtdjson.dylib');
    const privateKeyPath = join(root, 'release-key.pem');
    for (const [path, bytes] of [
      [runtime, 'telegram runtime'],
      [descriptor, 'telegram descriptor'],
      [settings, 'telegram settings'],
      [storage, 'telegram storage'],
      [tdjson, 'telegram native dependency'],
    ]) {
      writeFileSync(path, bytes, { mode: 0o700 });
    }
    const keyPair = generateKeyPairSync('ec', { namedCurve: 'prime256v1' });
    writeFileSync(
      privateKeyPath,
      keyPair.privateKey.export({ type: 'pkcs8', format: 'pem' }),
      { mode: 0o600 },
    );
    const baseInput = {
      verification_key_id: 'release-2026',
      trust_root_revision: 1,
      revision: 1,
      distribution_id: 'makosh-desktop',
      release_version: '1.0.0',
      build_id: 'build-telegram',
      target_triple: 'aarch64-apple-darwin',
      generation: 1,
      ...releaseProvenance,
      additional_verification_keys: [],
      artifacts: [{
        artifact_kind: 'browser_bootstrap_bundle',
        artifact_id: 'browser.bootstrap',
        relative_path: 'browser/bootstrap.html',
        source_path: browserBootstrapSource,
        required: true,
      }],
    };
    const fragment = {
      version: 1,
      owner_id: 'telegram',
      module_id: 'makosh-telegram-runtime',
      artifacts: [
        {
          artifact_kind: 'module_runtime',
          artifact_id: 'telegram.runtime.v1',
          relative_path: 'bin/makosh-telegram-runtime',
          source_path: runtime,
          required: true,
          descriptor: {
            relative_path: 'contracts/telegram.runtime.descriptor.pb',
            source_path: descriptor,
          },
          settings_schema: {
            relative_path: 'contracts/telegram.runtime.settings.pb',
            source_path: settings,
          },
        },
        {
          artifact_kind: 'storage_bundle',
          artifact_id: 'telegram.storage.v1',
          relative_path: 'storage/telegram.storage.bundle.pb',
          source_path: storage,
          required: true,
        },
        {
          artifact_kind: 'module_runtime_native_dependency',
          artifact_id: 'telegram.tdjson.v1',
          relative_path: 'lib/libtdjson.dylib',
          source_path: tdjson,
          required: true,
          bound_module_id: 'makosh-telegram-runtime',
        },
      ],
    };

    const input = composeReleaseCompilerInput(baseInput, [fragment]);
    const release = await compileReleaseDistribution(
      input,
      loadReleaseSigningKey(privateKeyPath),
    );
    const signed = decodeFields(release.signedManifest);
    const manifest = decodeFields(signed.get(2)[0]);
    const artifacts = manifest.get(8).map(decodeFields);
    assert.deepEqual(
      artifacts.map((artifact) => [fieldString(artifact, 2), artifact.get(1)[0]]),
      [
        ['browser.bootstrap', 4n],
        ['telegram.runtime.v1', 1n],
        ['telegram.storage.v1', 3n],
        ['telegram.tdjson.v1', 6n],
      ],
    );
    assert.equal(artifacts[1].get(6)[0].length, 32);
    assert.equal(artifacts[1].get(7)[0].length, 32);
    assert.equal(fieldString(artifacts[3], 13), 'makosh-telegram-runtime');

    const wrongBinding = structuredClone(fragment);
    wrongBinding.artifacts[2].bound_module_id = 'makosh-other-runtime';
    assert.throws(
      () => composeReleaseCompilerInput(baseInput, [wrongBinding]),
      /artifact fragment binding is invalid/,
    );
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test('signs the exact Mail runtime and Storage entries emitted by Mail assembly', async () => {
  const root = canonicalTemporaryDirectory('makosh-mail-release-fragment-');
  try {
    const runtime = join(root, 'makosh-mail-runtime');
    const assemblyOutput = join(root, 'assembly');
    const privateKeyPath = join(root, 'release-key.pem');
    writeFileSync(runtime, 'mail runtime bytes', { mode: 0o700 });
    const keyPair = generateKeyPairSync('ec', { namedCurve: 'prime256v1' });
    writeFileSync(
      privateKeyPath,
      keyPair.privateKey.export({ type: 'pkcs8', format: 'pem' }),
      { mode: 0o600 },
    );
    execFileSync('cargo', [
      'run',
      '--quiet',
      '-p',
      'makosh-mail-assembly',
      '--',
      '--build-id',
      'build-mail',
      '--output-dir',
      assemblyOutput,
      '--runtime',
      runtime,
    ], { cwd: process.cwd(), stdio: 'pipe' });
    const fragment = JSON.parse(readFileSync(
      join(assemblyOutput, 'mail.release-artifacts.json'),
      'utf8',
    ));
    assert.deepEqual(
      fragment.artifacts.map(({ artifact_id, artifact_kind }) => [artifact_id, artifact_kind]),
      [
        ['mail.runtime.v1', 'module_runtime'],
        ['mail.storage.v1', 'storage_bundle'],
      ],
    );
    assert.equal(fragment.owner_id, 'mail');
    assert.equal(fragment.module_id, 'makosh-mail-runtime');

    const input = composeReleaseCompilerInput({
      verification_key_id: 'release-2026',
      trust_root_revision: 1,
      revision: 1,
      distribution_id: 'makosh-desktop',
      release_version: '1.0.0',
      build_id: 'build-mail',
      target_triple: 'aarch64-apple-darwin',
      generation: 1,
      ...releaseProvenance,
      additional_verification_keys: [],
      artifacts: [{
        artifact_kind: 'browser_bootstrap_bundle',
        artifact_id: 'browser.bootstrap',
        relative_path: 'browser/bootstrap.html',
        source_path: browserBootstrapSource,
        required: true,
      }],
    }, [fragment]);
    const release = await compileReleaseDistribution(
      input,
      loadReleaseSigningKey(privateKeyPath),
    );
    const signed = decodeFields(release.signedManifest);
    const manifest = decodeFields(signed.get(2)[0]);
    const artifacts = manifest.get(8).map(decodeFields);

    assert.deepEqual(
      artifacts.map((artifact) => [fieldString(artifact, 2), artifact.get(1)[0]]),
      [
        ['browser.bootstrap', 4n],
        ['mail.runtime.v1', 1n],
        ['mail.storage.v1', 3n],
      ],
    );
    assert.equal(artifacts[1].get(6)[0].length, 32);
    assert.equal(artifacts[1].get(7)[0].length, 32);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test('signs the exact Zulip runtime and Storage entries emitted by Zulip assembly', async () => {
  const root = canonicalTemporaryDirectory('makosh-zulip-release-fragment-');
  try {
    const runtime = join(root, 'makosh-zulip-runtime');
    const assemblyOutput = join(root, 'assembly');
    const privateKeyPath = join(root, 'release-key.pem');
    writeFileSync(runtime, 'zulip runtime bytes', { mode: 0o700 });
    const keyPair = generateKeyPairSync('ec', { namedCurve: 'prime256v1' });
    writeFileSync(
      privateKeyPath,
      keyPair.privateKey.export({ type: 'pkcs8', format: 'pem' }),
      { mode: 0o600 },
    );
    execFileSync('cargo', [
      'run',
      '--quiet',
      '-p',
      'makosh-zulip-assembly',
      '--',
      '--build-id',
      'build-zulip',
      '--output-dir',
      assemblyOutput,
      '--runtime',
      runtime,
    ], { cwd: process.cwd(), stdio: 'pipe' });
    const fragment = JSON.parse(readFileSync(
      join(assemblyOutput, 'zulip.release-artifacts.json'),
      'utf8',
    ));
    assert.deepEqual(
      fragment.artifacts.map(({ artifact_id, artifact_kind }) => [artifact_id, artifact_kind]),
      [
        ['zulip.runtime.v1', 'module_runtime'],
        ['zulip.storage.v1', 'storage_bundle'],
      ],
    );
    assert.equal(fragment.owner_id, 'zulip');
    assert.equal(fragment.module_id, 'makosh-zulip-runtime');

    const input = composeReleaseCompilerInput({
      verification_key_id: 'release-2026',
      trust_root_revision: 1,
      revision: 1,
      distribution_id: 'makosh-desktop',
      release_version: '1.0.0',
      build_id: 'build-zulip',
      target_triple: 'aarch64-apple-darwin',
      generation: 1,
      ...releaseProvenance,
      additional_verification_keys: [],
      artifacts: [{
        artifact_kind: 'browser_bootstrap_bundle',
        artifact_id: 'browser.bootstrap',
        relative_path: 'browser/bootstrap.html',
        source_path: browserBootstrapSource,
        required: true,
      }],
    }, [fragment]);
    const release = await compileReleaseDistribution(
      input,
      loadReleaseSigningKey(privateKeyPath),
    );
    const signed = decodeFields(release.signedManifest);
    const manifest = decodeFields(signed.get(2)[0]);
    const artifacts = manifest.get(8).map(decodeFields);

    assert.deepEqual(
      artifacts.map((artifact) => [fieldString(artifact, 2), artifact.get(1)[0]]),
      [
        ['browser.bootstrap', 4n],
        ['zulip.runtime.v1', 1n],
        ['zulip.storage.v1', 3n],
      ],
    );
    assert.equal(artifacts[1].get(6)[0].length, 32);
    assert.equal(artifacts[1].get(7)[0].length, 32);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test('signs the exact WhatsApp runtime and Storage entries emitted by WhatsApp assembly', async () => {
  const root = canonicalTemporaryDirectory('makosh-whatsapp-release-fragment-');
  try {
    const runtime = join(root, 'makosh-whatsapp-runtime');
    const assemblyOutput = join(root, 'assembly');
    const privateKeyPath = join(root, 'release-key.pem');
    writeFileSync(runtime, 'whatsapp runtime bytes', { mode: 0o700 });
    const keyPair = generateKeyPairSync('ec', { namedCurve: 'prime256v1' });
    writeFileSync(
      privateKeyPath,
      keyPair.privateKey.export({ type: 'pkcs8', format: 'pem' }),
      { mode: 0o600 },
    );
    execFileSync('cargo', [
      'run',
      '--quiet',
      '-p',
      'makosh-whatsapp-assembly',
      '--',
      '--build-id',
      'build-whatsapp',
      '--output-dir',
      assemblyOutput,
      '--runtime',
      runtime,
    ], { cwd: process.cwd(), stdio: 'pipe' });
    const fragment = JSON.parse(readFileSync(
      join(assemblyOutput, 'whatsapp.release-artifacts.json'),
      'utf8',
    ));
    assert.deepEqual(
      fragment.artifacts.map(({ artifact_id, artifact_kind }) => [artifact_id, artifact_kind]),
      [
        ['whatsapp.runtime.v1', 'module_runtime'],
        ['whatsapp.storage.v1', 'storage_bundle'],
      ],
    );
    assert.equal(fragment.owner_id, 'whatsapp');
    assert.equal(fragment.module_id, 'makosh-whatsapp-runtime');

    const input = composeReleaseCompilerInput({
      verification_key_id: 'release-2026',
      trust_root_revision: 1,
      revision: 1,
      distribution_id: 'makosh-desktop',
      release_version: '1.0.0',
      build_id: 'build-whatsapp',
      target_triple: 'aarch64-apple-darwin',
      generation: 1,
      ...releaseProvenance,
      additional_verification_keys: [],
      artifacts: [{
        artifact_kind: 'browser_bootstrap_bundle',
        artifact_id: 'browser.bootstrap',
        relative_path: 'browser/bootstrap.html',
        source_path: browserBootstrapSource,
        required: true,
      }],
    }, [fragment]);
    const release = await compileReleaseDistribution(
      input,
      loadReleaseSigningKey(privateKeyPath),
    );
    const signed = decodeFields(release.signedManifest);
    const manifest = decodeFields(signed.get(2)[0]);
    const artifacts = manifest.get(8).map(decodeFields);

    assert.deepEqual(
      artifacts.map((artifact) => [fieldString(artifact, 2), artifact.get(1)[0]]),
      [
        ['browser.bootstrap', 4n],
        ['whatsapp.runtime.v1', 1n],
        ['whatsapp.storage.v1', 3n],
      ],
    );
    assert.equal(artifacts[1].get(6)[0].length, 32);
    assert.equal(artifacts[1].get(7)[0].length, 32);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test('signs the exact Attachment Security runtime and Storage entries emitted by its assembly', async () => {
  const root = canonicalTemporaryDirectory('makosh-attachment-security-release-fragment-');
  try {
    const runtime = join(root, 'makosh-attachment-security-runtime');
    const assemblyOutput = join(root, 'assembly');
    const privateKeyPath = join(root, 'release-key.pem');
    writeFileSync(runtime, 'attachment security runtime bytes', { mode: 0o700 });
    const keyPair = generateKeyPairSync('ec', { namedCurve: 'prime256v1' });
    writeFileSync(
      privateKeyPath,
      keyPair.privateKey.export({ type: 'pkcs8', format: 'pem' }),
      { mode: 0o600 },
    );
    execFileSync('cargo', [
      'run',
      '--quiet',
      '-p',
      'makosh-attachment-security-assembly',
      '--',
      '--build-id',
      'build-attachment-security',
      '--output-dir',
      assemblyOutput,
      '--runtime',
      runtime,
    ], { cwd: process.cwd(), stdio: 'pipe' });
    const fragment = JSON.parse(readFileSync(
      join(assemblyOutput, 'attachment-security.release-artifacts.json'),
      'utf8',
    ));
    assert.deepEqual(
      fragment.artifacts.map(({ artifact_id, artifact_kind }) => [artifact_id, artifact_kind]),
      [
        ['attachment_security.runtime.v1', 'module_runtime'],
        ['attachment_security.storage.v1', 'storage_bundle'],
      ],
    );
    assert.equal(fragment.owner_id, 'attachment_security');
    assert.equal(fragment.module_id, 'makosh-attachment-security-runtime');

    const input = composeReleaseCompilerInput({
      verification_key_id: 'release-2026',
      trust_root_revision: 1,
      revision: 1,
      distribution_id: 'makosh-desktop',
      release_version: '1.0.0',
      build_id: 'build-attachment-security',
      target_triple: 'aarch64-apple-darwin',
      generation: 1,
      ...releaseProvenance,
      additional_verification_keys: [],
      artifacts: [{
        artifact_kind: 'browser_bootstrap_bundle',
        artifact_id: 'browser.bootstrap',
        relative_path: 'browser/bootstrap.html',
        source_path: browserBootstrapSource,
        required: true,
      }],
    }, [fragment]);
    const release = await compileReleaseDistribution(
      input,
      loadReleaseSigningKey(privateKeyPath),
    );
    const signed = decodeFields(release.signedManifest);
    const manifest = decodeFields(signed.get(2)[0]);
    const artifacts = manifest.get(8).map(decodeFields);

    assert.deepEqual(
      artifacts.map((artifact) => [fieldString(artifact, 2), artifact.get(1)[0]]),
      [
        ['attachment_security.runtime.v1', 1n],
        ['attachment_security.storage.v1', 3n],
        ['browser.bootstrap', 4n],
      ],
    );
    assert.equal(artifacts[0].get(6)[0].length, 32);
    assert.equal(artifacts[0].get(7)[0].length, 32);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test('signs the exact Preview runtime and Storage entries emitted by its assembly', async () => {
  const root = canonicalTemporaryDirectory('makosh-attachment-preview-release-fragment-');
  try {
    const runtime = join(root, 'makosh-attachment-preview-runtime');
    const assemblyOutput = join(root, 'assembly');
    const privateKeyPath = join(root, 'release-key.pem');
    writeFileSync(runtime, 'attachment preview runtime bytes', { mode: 0o700 });
    const keyPair = generateKeyPairSync('ec', { namedCurve: 'prime256v1' });
    writeFileSync(
      privateKeyPath,
      keyPair.privateKey.export({ type: 'pkcs8', format: 'pem' }),
      { mode: 0o600 },
    );
    execFileSync('cargo', [
      'run',
      '--quiet',
      '-p',
      'makosh-attachment-preview-assembly',
      '--',
      '--build-id',
      'build-attachment-preview',
      '--output-dir',
      assemblyOutput,
      '--runtime',
      runtime,
    ], { cwd: process.cwd(), stdio: 'pipe' });
    const fragment = JSON.parse(readFileSync(
      join(assemblyOutput, 'attachment-preview.release-artifacts.json'),
      'utf8',
    ));
    assert.deepEqual(
      fragment.artifacts.map(({ artifact_id, artifact_kind }) => [artifact_id, artifact_kind]),
      [
        ['attachment_preview.runtime.v1', 'module_runtime'],
        ['attachment_preview.storage.v1', 'storage_bundle'],
      ],
    );
    assert.equal(fragment.owner_id, 'attachment_preview');
    assert.equal(fragment.module_id, 'makosh-attachment-preview-runtime');

    const input = composeReleaseCompilerInput({
      verification_key_id: 'release-2026',
      trust_root_revision: 1,
      revision: 1,
      distribution_id: 'makosh-desktop',
      release_version: '1.0.0',
      build_id: 'build-attachment-preview',
      target_triple: 'aarch64-apple-darwin',
      generation: 1,
      ...releaseProvenance,
      additional_verification_keys: [],
      artifacts: [{
        artifact_kind: 'browser_bootstrap_bundle',
        artifact_id: 'browser.bootstrap',
        relative_path: 'browser/bootstrap.html',
        source_path: browserBootstrapSource,
        required: true,
      }],
    }, [fragment]);
    const release = await compileReleaseDistribution(
      input,
      loadReleaseSigningKey(privateKeyPath),
    );
    const signed = decodeFields(release.signedManifest);
    const manifest = decodeFields(signed.get(2)[0]);
    const artifacts = manifest.get(8).map(decodeFields);

    assert.deepEqual(
      artifacts.map((artifact) => [fieldString(artifact, 2), artifact.get(1)[0]]),
      [
        ['attachment_preview.runtime.v1', 1n],
        ['attachment_preview.storage.v1', 3n],
        ['browser.bootstrap', 4n],
      ],
    );
    assert.equal(artifacts[0].get(6)[0].length, 32);
    assert.equal(artifacts[0].get(7)[0].length, 32);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test('signs the exact text extraction runtime Storage and OCR entries emitted by its assembly', async () => {
  const root = canonicalTemporaryDirectory('makosh-text-extraction-release-fragment-');
  try {
    const runtime = join(root, 'makosh-attachment-text-extraction-runtime');
    const runner = join(root, 'tesseract-runner');
    const english = join(root, 'eng.traineddata');
    const russian = join(root, 'rus.traineddata');
    const assemblyOutput = join(root, 'assembly');
    const privateKeyPath = join(root, 'release-key.pem');
    writeFileSync(runtime, 'text extraction runtime bytes', { mode: 0o700 });
    writeFileSync(runner, 'tesseract runner bytes', { mode: 0o500 });
    writeFileSync(english, 'english model bytes', { mode: 0o400 });
    writeFileSync(russian, 'russian model bytes', { mode: 0o400 });
    const keyPair = generateKeyPairSync('ec', { namedCurve: 'prime256v1' });
    writeFileSync(
      privateKeyPath,
      keyPair.privateKey.export({ type: 'pkcs8', format: 'pem' }),
      { mode: 0o600 },
    );
    execFileSync('cargo', [
      'run',
      '--quiet',
      '-p',
      'makosh-attachment-text-extraction-assembly',
      '--',
      '--build-id',
      'build-text-extraction',
      '--output-dir',
      assemblyOutput,
      '--runtime',
      runtime,
      '--ocr-runner',
      runner,
      '--ocr-eng',
      english,
      '--ocr-rus',
      russian,
    ], { cwd: process.cwd(), stdio: 'pipe' });
    const fragment = JSON.parse(readFileSync(
      join(assemblyOutput, 'attachment_text_extraction.release-artifacts.json'),
      'utf8',
    ));
    assert.deepEqual(
      fragment.artifacts.map(({ artifact_id, artifact_kind }) => [artifact_id, artifact_kind]),
      [
        ['attachment_text_extraction.ocr.eng.v1', 'module_runtime_read_only_data'],
        ['attachment_text_extraction.ocr.runner.v1', 'module_runtime_native_executable'],
        ['attachment_text_extraction.ocr.rus.v1', 'module_runtime_read_only_data'],
        ['attachment_text_extraction.runtime.v1', 'module_runtime'],
        ['attachment_text_extraction.storage.v1', 'storage_bundle'],
      ],
    );
    assert.equal(fragment.owner_id, 'attachment_text_extraction');
    assert.equal(fragment.module_id, 'makosh-attachment-text-extraction-runtime');

    const input = composeReleaseCompilerInput({
      verification_key_id: 'release-2026',
      trust_root_revision: 1,
      revision: 1,
      distribution_id: 'makosh-desktop',
      release_version: '1.0.0',
      build_id: 'build-text-extraction',
      target_triple: 'aarch64-apple-darwin',
      generation: 1,
      ...releaseProvenance,
      additional_verification_keys: [],
      artifacts: [{
        artifact_kind: 'browser_bootstrap_bundle',
        artifact_id: 'browser.bootstrap',
        relative_path: 'browser/bootstrap.html',
        source_path: browserBootstrapSource,
        required: true,
      }],
    }, [fragment]);
    const release = await compileReleaseDistribution(
      input,
      loadReleaseSigningKey(privateKeyPath),
    );
    const signed = decodeFields(release.signedManifest);
    const manifest = decodeFields(signed.get(2)[0]);
    const artifacts = manifest.get(8).map(decodeFields);
    assert.deepEqual(
      artifacts.map((artifact) => [fieldString(artifact, 2), artifact.get(1)[0]]),
      [
        ['attachment_text_extraction.ocr.eng.v1', 8n],
        ['attachment_text_extraction.ocr.runner.v1', 7n],
        ['attachment_text_extraction.ocr.rus.v1', 8n],
        ['attachment_text_extraction.runtime.v1', 1n],
        ['attachment_text_extraction.storage.v1', 3n],
        ['browser.bootstrap', 4n],
      ],
    );
    assert.ok(artifacts.slice(0, 3).every(
      (artifact) => fieldString(artifact, 13) === 'makosh-attachment-text-extraction-runtime',
    ));
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test('signs the exact Review attention runtime and Storage entries emitted by its assembly', async () => {
  const root = canonicalTemporaryDirectory('makosh-review-attention-release-fragment-');
  try {
    const runtime = join(root, 'makosh-review-attention-runtime');
    const assemblyOutput = join(root, 'assembly');
    const privateKeyPath = join(root, 'release-key.pem');
    writeFileSync(runtime, 'review attention runtime bytes', { mode: 0o700 });
    const keyPair = generateKeyPairSync('ec', { namedCurve: 'prime256v1' });
    writeFileSync(
      privateKeyPath,
      keyPair.privateKey.export({ type: 'pkcs8', format: 'pem' }),
      { mode: 0o600 },
    );
    execFileSync('cargo', [
      'run',
      '--quiet',
      '-p',
      'makosh-review-attention-assembly',
      '--',
      '--build-id',
      'build-review-attention',
      '--output-dir',
      assemblyOutput,
      '--runtime',
      runtime,
    ], { cwd: process.cwd(), stdio: 'pipe' });
    const fragment = JSON.parse(readFileSync(
      join(assemblyOutput, 'review-attention.release-artifacts.json'),
      'utf8',
    ));
    assert.deepEqual(
      fragment.artifacts.map(({ artifact_id, artifact_kind }) => [artifact_id, artifact_kind]),
      [
        ['review_attention.runtime.v1', 'module_runtime'],
        ['review_attention.storage.v1', 'storage_bundle'],
      ],
    );
    assert.equal(fragment.owner_id, 'review');
    assert.equal(fragment.module_id, 'makosh-review-runtime');

    const input = composeReleaseCompilerInput({
      verification_key_id: 'release-2026',
      trust_root_revision: 1,
      revision: 1,
      distribution_id: 'makosh-desktop',
      release_version: '1.0.0',
      build_id: 'build-review-attention',
      target_triple: 'aarch64-apple-darwin',
      generation: 1,
      ...releaseProvenance,
      additional_verification_keys: [],
      artifacts: [{
        artifact_kind: 'browser_bootstrap_bundle',
        artifact_id: 'browser.bootstrap',
        relative_path: 'browser/bootstrap.html',
        source_path: browserBootstrapSource,
        required: true,
      }],
    }, [fragment]);
    const release = await compileReleaseDistribution(
      input,
      loadReleaseSigningKey(privateKeyPath),
    );
    const signed = decodeFields(release.signedManifest);
    const manifest = decodeFields(signed.get(2)[0]);
    const artifacts = manifest.get(8).map(decodeFields);

    assert.deepEqual(
      artifacts.map((artifact) => [fieldString(artifact, 2), artifact.get(1)[0]]),
      [
        ['browser.bootstrap', 4n],
        ['review_attention.runtime.v1', 1n],
        ['review_attention.storage.v1', 3n],
      ],
    );
    assert.equal(artifacts[1].get(6)[0].length, 32);
    assert.equal(artifacts[1].get(7)[0].length, 32);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test('signs distinct task note Contacts and mail sync runtime and Storage entries', async () => {
  const root = canonicalTemporaryDirectory('makosh-task-candidate-release-fragments-');
  try {
    const privateKeyPath = join(root, 'release-key.pem');
    const keyPair = generateKeyPairSync('ec', { namedCurve: 'prime256v1' });
    writeFileSync(
      privateKeyPath,
      keyPair.privateKey.export({ type: 'pkcs8', format: 'pem' }),
      { mode: 0o600 },
    );
    const units = [
      {
        package: 'makosh-communication-task-candidate-assembly',
        runtimeName: 'makosh-communication-task-candidate-runtime',
        fragmentName: 'communication_task_candidate.release-artifacts.json',
        owner: 'communication_task_candidate_extraction',
        ids: [
          'communication_task_candidate_extraction.runtime.v1',
          'communication_task_candidate_extraction.storage.v1',
        ],
      },
      {
        package: 'makosh-communication-note-candidate-assembly',
        runtimeName: 'makosh-communication-note-candidate-runtime',
        fragmentName: 'communication_note_candidate.release-artifacts.json',
        owner: 'communication_note_candidate_extraction',
        ids: [
          'communication_note_candidate_extraction.runtime.v1',
          'communication_note_candidate_extraction.storage.v1',
        ],
      },
      {
        package: 'makosh-review-task-candidate-assembly',
        runtimeName: 'makosh-review-task-candidate-runtime',
        fragmentName: 'review-task-candidate.release-artifacts.json',
        owner: 'review',
        ids: ['review.task-candidate.runtime.v1', 'review.task-candidate.storage.v1'],
      },
      {
        package: 'makosh-reviewed-task-candidate-promotion-assembly',
        runtimeName: 'makosh-reviewed-task-candidate-promotion-runtime',
        fragmentName: 'reviewed_task_candidate_promotion.release-artifacts.json',
        owner: 'reviewed_task_candidate_promotion',
        ids: [
          'reviewed_task_candidate_promotion.runtime.v1',
          'reviewed_task_candidate_promotion.storage.v1',
        ],
      },
      {
        package: 'makosh-tasks-assembly',
        runtimeName: 'makosh-tasks-runtime',
        fragmentName: 'tasks.release-artifacts.json',
        owner: 'tasks',
        ids: ['tasks.runtime.v1', 'tasks.storage.v1'],
      },
      {
        package: 'makosh-contacts-assembly',
        runtimeName: 'makosh-contacts-runtime',
        fragmentName: 'contacts.release-artifacts.json',
        owner: 'contacts',
        ids: ['contacts.runtime.v1', 'contacts.storage.v1'],
      },
      {
        package: 'makosh-mail-contacts-sync-assembly',
        runtimeName: 'makosh-mail-contacts-sync-runtime',
        fragmentName: 'mail_contacts_sync.release-artifacts.json',
        owner: 'mail_contacts_sync',
        ids: ['mail_contacts_sync.runtime.v1', 'mail_contacts_sync.storage.v1'],
      },
    ];
    const fragments = [];
    for (const [index, unit] of units.entries()) {
      const runtime = join(root, unit.runtimeName);
      const output = join(root, `assembly-${index}`);
      writeFileSync(runtime, `${unit.runtimeName} bytes`, { mode: 0o700 });
      execFileSync('cargo', [
        'run',
        '--quiet',
        '-p',
        unit.package,
        '--',
        '--build-id',
        'build-task-candidate-chain',
        '--output-dir',
        output,
        '--runtime',
        runtime,
      ], { cwd: process.cwd(), stdio: 'pipe' });
      const fragment = JSON.parse(readFileSync(join(output, unit.fragmentName), 'utf8'));
      assert.equal(fragment.owner_id, unit.owner);
      assert.equal(fragment.module_id, unit.runtimeName);
      assert.deepEqual(fragment.artifacts.map(({ artifact_id }) => artifact_id), unit.ids);
      fragments.push(fragment);
    }

    const input = composeReleaseCompilerInput({
      verification_key_id: 'release-2026',
      trust_root_revision: 1,
      revision: 1,
      distribution_id: 'makosh-desktop',
      release_version: '1.0.0',
      build_id: 'build-task-candidate-chain',
      target_triple: 'aarch64-apple-darwin',
      generation: 1,
      ...releaseProvenance,
      additional_verification_keys: [],
      artifacts: [{
        artifact_kind: 'browser_bootstrap_bundle',
        artifact_id: 'browser.bootstrap',
        relative_path: 'browser/bootstrap.html',
        source_path: browserBootstrapSource,
        required: true,
      }],
    }, fragments);
    const release = await compileReleaseDistribution(input, loadReleaseSigningKey(privateKeyPath));
    const signed = decodeFields(release.signedManifest);
    const manifest = decodeFields(signed.get(2)[0]);
    const artifacts = manifest.get(8).map(decodeFields);
    assert.deepEqual(
      artifacts.map((artifact) => [fieldString(artifact, 2), artifact.get(1)[0]]),
      [
        ['browser.bootstrap', 4n],
        ['communication_note_candidate_extraction.runtime.v1', 1n],
        ['communication_note_candidate_extraction.storage.v1', 3n],
        ['communication_task_candidate_extraction.runtime.v1', 1n],
        ['communication_task_candidate_extraction.storage.v1', 3n],
        ['contacts.runtime.v1', 1n],
        ['contacts.storage.v1', 3n],
        ['mail_contacts_sync.runtime.v1', 1n],
        ['mail_contacts_sync.storage.v1', 3n],
        ['review.task-candidate.runtime.v1', 1n],
        ['review.task-candidate.storage.v1', 3n],
        ['reviewed_task_candidate_promotion.runtime.v1', 1n],
        ['reviewed_task_candidate_promotion.storage.v1', 3n],
        ['tasks.runtime.v1', 1n],
        ['tasks.storage.v1', 3n],
      ],
    );
    for (const artifact of artifacts.filter((value) => value.get(1)[0] === 1n)) {
      assert.equal(artifact.get(6)[0].length, 32);
      assert.equal(artifact.get(7)[0].length, 32);
    }
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test('signs the exact Reply Suggestion runtime and Storage entries emitted by its assembly', async () => {
  const root = canonicalTemporaryDirectory('makosh-reply-suggestion-release-fragment-');
  try {
    const runtime = join(root, 'makosh-communication-reply-suggestion-runtime');
    const assemblyOutput = join(root, 'assembly');
    const privateKeyPath = join(root, 'release-key.pem');
    writeFileSync(runtime, 'reply suggestion runtime bytes', { mode: 0o700 });
    const keyPair = generateKeyPairSync('ec', { namedCurve: 'prime256v1' });
    writeFileSync(
      privateKeyPath,
      keyPair.privateKey.export({ type: 'pkcs8', format: 'pem' }),
      { mode: 0o600 },
    );
    execFileSync('cargo', [
      'run',
      '--quiet',
      '-p',
      'makosh-communication-reply-suggestion-assembly',
      '--',
      '--build-id',
      'build-reply-suggestion',
      '--output-dir',
      assemblyOutput,
      '--runtime',
      runtime,
    ], { cwd: process.cwd(), stdio: 'pipe' });
    const fragment = JSON.parse(readFileSync(
      join(assemblyOutput, 'communication_reply_suggestion.release-artifacts.json'),
      'utf8',
    ));
    assert.deepEqual(
      fragment.artifacts.map(({ artifact_id, artifact_kind }) => [artifact_id, artifact_kind]),
      [
        ['communication_reply_suggestion.runtime.v1', 'module_runtime'],
        ['communication_reply_suggestion.storage.v1', 'storage_bundle'],
      ],
    );
    assert.equal(fragment.owner_id, 'communication_reply_suggestion');
    assert.equal(fragment.module_id, 'makosh-communication-reply-suggestion-runtime');

    const input = composeReleaseCompilerInput({
      verification_key_id: 'release-2026',
      trust_root_revision: 1,
      revision: 1,
      distribution_id: 'makosh-desktop',
      release_version: '1.0.0',
      build_id: 'build-reply-suggestion',
      target_triple: 'aarch64-apple-darwin',
      generation: 1,
      ...releaseProvenance,
      additional_verification_keys: [],
      artifacts: [{
        artifact_kind: 'browser_bootstrap_bundle',
        artifact_id: 'browser.bootstrap',
        relative_path: 'browser/bootstrap.html',
        source_path: browserBootstrapSource,
        required: true,
      }],
    }, [fragment]);
    const release = await compileReleaseDistribution(
      input,
      loadReleaseSigningKey(privateKeyPath),
    );
    const signed = decodeFields(release.signedManifest);
    const manifest = decodeFields(signed.get(2)[0]);
    const artifacts = manifest.get(8).map(decodeFields);

    assert.deepEqual(
      artifacts.map((artifact) => [fieldString(artifact, 2), artifact.get(1)[0]]),
      [
        ['browser.bootstrap', 4n],
        ['communication_reply_suggestion.runtime.v1', 1n],
        ['communication_reply_suggestion.storage.v1', 3n],
      ],
    );
    assert.equal(artifacts[1].get(6)[0].length, 32);
    assert.equal(artifacts[1].get(7)[0].length, 32);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test('signs the exact Communication Summary runtime and Storage entries emitted by its assembly', async () => {
  const root = canonicalTemporaryDirectory('makosh-communication-summary-release-fragment-');
  try {
    const runtime = join(root, 'makosh-communication-summary-runtime');
    const assemblyOutput = join(root, 'assembly');
    const privateKeyPath = join(root, 'release-key.pem');
    writeFileSync(runtime, 'communication summary runtime bytes', { mode: 0o700 });
    const keyPair = generateKeyPairSync('ec', { namedCurve: 'prime256v1' });
    writeFileSync(
      privateKeyPath,
      keyPair.privateKey.export({ type: 'pkcs8', format: 'pem' }),
      { mode: 0o600 },
    );
    execFileSync('cargo', [
      'run',
      '--quiet',
      '-p',
      'makosh-communication-summary-assembly',
      '--',
      '--build-id',
      'build-communication-summary',
      '--output-dir',
      assemblyOutput,
      '--runtime',
      runtime,
    ], { cwd: process.cwd(), stdio: 'pipe' });
    const fragment = JSON.parse(readFileSync(
      join(assemblyOutput, 'communication_summary.release-artifacts.json'),
      'utf8',
    ));
    assert.deepEqual(
      fragment.artifacts.map(({ artifact_id, artifact_kind }) => [artifact_id, artifact_kind]),
      [
        ['communication_summary.runtime.v1', 'module_runtime'],
        ['communication_summary.storage.v1', 'storage_bundle'],
      ],
    );
    assert.equal(fragment.owner_id, 'communication_summary');
    assert.equal(fragment.module_id, 'makosh-communication-summary-runtime');

    const input = composeReleaseCompilerInput({
      verification_key_id: 'release-2026',
      trust_root_revision: 1,
      revision: 1,
      distribution_id: 'makosh-desktop',
      release_version: '1.0.0',
      build_id: 'build-communication-summary',
      target_triple: 'aarch64-apple-darwin',
      generation: 1,
      ...releaseProvenance,
      additional_verification_keys: [],
      artifacts: [{
        artifact_kind: 'browser_bootstrap_bundle',
        artifact_id: 'browser.bootstrap',
        relative_path: 'browser/bootstrap.html',
        source_path: browserBootstrapSource,
        required: true,
      }],
    }, [fragment]);
    const release = await compileReleaseDistribution(
      input,
      loadReleaseSigningKey(privateKeyPath),
    );
    const signed = decodeFields(release.signedManifest);
    const manifest = decodeFields(signed.get(2)[0]);
    const artifacts = manifest.get(8).map(decodeFields);

    assert.deepEqual(
      artifacts.map((artifact) => [fieldString(artifact, 2), artifact.get(1)[0]]),
      [
        ['browser.bootstrap', 4n],
        ['communication_summary.runtime.v1', 1n],
        ['communication_summary.storage.v1', 3n],
      ],
    );
    assert.equal(artifacts[1].get(6)[0].length, 32);
    assert.equal(artifacts[1].get(7)[0].length, 32);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test('signs the exact Communication Translation runtime and Storage entries emitted by its assembly', async () => {
  const root = canonicalTemporaryDirectory('makosh-communication-translation-release-fragment-');
  try {
    const runtime = join(root, 'makosh-communication-translation-runtime');
    const assemblyOutput = join(root, 'assembly');
    const privateKeyPath = join(root, 'release-key.pem');
    writeFileSync(runtime, 'communication translation runtime bytes', { mode: 0o700 });
    const keyPair = generateKeyPairSync('ec', { namedCurve: 'prime256v1' });
    writeFileSync(
      privateKeyPath,
      keyPair.privateKey.export({ type: 'pkcs8', format: 'pem' }),
      { mode: 0o600 },
    );
    execFileSync('cargo', [
      'run',
      '--quiet',
      '-p',
      'makosh-communication-translation-assembly',
      '--',
      '--build-id',
      'build-communication-translation',
      '--output-dir',
      assemblyOutput,
      '--runtime',
      runtime,
    ], { cwd: process.cwd(), stdio: 'pipe' });
    const fragment = JSON.parse(readFileSync(
      join(assemblyOutput, 'communication_translation.release-artifacts.json'),
      'utf8',
    ));
    assert.deepEqual(
      fragment.artifacts.map(({ artifact_id, artifact_kind }) => [artifact_id, artifact_kind]),
      [
        ['communication_translation.runtime.v1', 'module_runtime'],
        ['communication_translation.storage.v1', 'storage_bundle'],
      ],
    );
    assert.equal(fragment.owner_id, 'communication_translation');
    assert.equal(fragment.module_id, 'makosh-communication-translation-runtime');

    const input = composeReleaseCompilerInput({
      verification_key_id: 'release-2026',
      trust_root_revision: 1,
      revision: 1,
      distribution_id: 'makosh-desktop',
      release_version: '1.0.0',
      build_id: 'build-communication-translation',
      target_triple: 'aarch64-apple-darwin',
      generation: 1,
      ...releaseProvenance,
      additional_verification_keys: [],
      artifacts: [{
        artifact_kind: 'browser_bootstrap_bundle',
        artifact_id: 'browser.bootstrap',
        relative_path: 'browser/bootstrap.html',
        source_path: browserBootstrapSource,
        required: true,
      }],
    }, [fragment]);
    const release = await compileReleaseDistribution(
      input,
      loadReleaseSigningKey(privateKeyPath),
    );
    const signed = decodeFields(release.signedManifest);
    const manifest = decodeFields(signed.get(2)[0]);
    const artifacts = manifest.get(8).map(decodeFields);

    assert.deepEqual(
      artifacts.map((artifact) => [fieldString(artifact, 2), artifact.get(1)[0]]),
      [
        ['browser.bootstrap', 4n],
        ['communication_translation.runtime.v1', 1n],
        ['communication_translation.storage.v1', 3n],
      ],
    );
    assert.equal(artifacts[1].get(6)[0].length, 32);
    assert.equal(artifacts[1].get(7)[0].length, 32);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test('signs the exact Communication Explanation runtime and Storage entries emitted by its assembly', async () => {
  const root = canonicalTemporaryDirectory('makosh-communication-explanation-release-fragment-');
  try {
    const runtime = join(root, 'makosh-communication-explanation-runtime');
    const assemblyOutput = join(root, 'assembly');
    const privateKeyPath = join(root, 'release-key.pem');
    writeFileSync(runtime, 'communication explanation runtime bytes', { mode: 0o700 });
    const keyPair = generateKeyPairSync('ec', { namedCurve: 'prime256v1' });
    writeFileSync(
      privateKeyPath,
      keyPair.privateKey.export({ type: 'pkcs8', format: 'pem' }),
      { mode: 0o600 },
    );
    execFileSync('cargo', [
      'run',
      '--quiet',
      '-p',
      'makosh-communication-explanation-assembly',
      '--',
      '--build-id',
      'build-communication-explanation',
      '--output-dir',
      assemblyOutput,
      '--runtime',
      runtime,
    ], { cwd: process.cwd(), stdio: 'pipe' });
    const fragment = JSON.parse(readFileSync(
      join(assemblyOutput, 'communication_explanation.release-artifacts.json'),
      'utf8',
    ));
    assert.deepEqual(
      fragment.artifacts.map(({ artifact_id, artifact_kind }) => [artifact_id, artifact_kind]),
      [
        ['communication_explanation.runtime.v1', 'module_runtime'],
        ['communication_explanation.storage.v1', 'storage_bundle'],
      ],
    );
    assert.equal(fragment.owner_id, 'communication_explanation');
    assert.equal(fragment.module_id, 'makosh-communication-explanation-runtime');

    const input = composeReleaseCompilerInput({
      verification_key_id: 'release-2026',
      trust_root_revision: 1,
      revision: 1,
      distribution_id: 'makosh-desktop',
      release_version: '1.0.0',
      build_id: 'build-communication-explanation',
      target_triple: 'aarch64-apple-darwin',
      generation: 1,
      ...releaseProvenance,
      additional_verification_keys: [],
      artifacts: [{
        artifact_kind: 'browser_bootstrap_bundle',
        artifact_id: 'browser.bootstrap',
        relative_path: 'browser/bootstrap.html',
        source_path: browserBootstrapSource,
        required: true,
      }],
    }, [fragment]);
    const release = await compileReleaseDistribution(
      input,
      loadReleaseSigningKey(privateKeyPath),
    );
    const signed = decodeFields(release.signedManifest);
    const manifest = decodeFields(signed.get(2)[0]);
    const artifacts = manifest.get(8).map(decodeFields);

    assert.deepEqual(
      artifacts.map((artifact) => [fieldString(artifact, 2), artifact.get(1)[0]]),
      [
        ['browser.bootstrap', 4n],
        ['communication_explanation.runtime.v1', 1n],
        ['communication_explanation.storage.v1', 3n],
      ],
    );
    assert.equal(artifacts[1].get(6)[0].length, 32);
    assert.equal(artifacts[1].get(7)[0].length, 32);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test('signs the exact Recipient Suggestion runtime and Storage entries emitted by its assembly', async () => {
  const root = canonicalTemporaryDirectory('makosh-communication-recipient-suggestion-release-fragment-');
  try {
    const runtime = join(root, 'makosh-communication-recipient-suggestion-runtime');
    const assemblyOutput = join(root, 'assembly');
    const privateKeyPath = join(root, 'release-key.pem');
    writeFileSync(runtime, 'communication recipient suggestion runtime bytes', { mode: 0o700 });
    const keyPair = generateKeyPairSync('ec', { namedCurve: 'prime256v1' });
    writeFileSync(
      privateKeyPath,
      keyPair.privateKey.export({ type: 'pkcs8', format: 'pem' }),
      { mode: 0o600 },
    );
    execFileSync('cargo', [
      'run',
      '--quiet',
      '-p',
      'makosh-communication-recipient-suggestion-assembly',
      '--',
      '--build-id',
      'build-communication-recipient-suggestion',
      '--output-dir',
      assemblyOutput,
      '--runtime',
      runtime,
    ], { cwd: process.cwd(), stdio: 'pipe' });
    const fragment = JSON.parse(readFileSync(
      join(assemblyOutput, 'communication_recipient_suggestion.release-artifacts.json'),
      'utf8',
    ));
    assert.deepEqual(
      fragment.artifacts.map(({ artifact_id, artifact_kind }) => [artifact_id, artifact_kind]),
      [
        ['communication_recipient_suggestion.runtime.v1', 'module_runtime'],
        ['communication_recipient_suggestion.storage.v1', 'storage_bundle'],
      ],
    );
    assert.equal(fragment.owner_id, 'communication_recipient_suggestion');
    assert.equal(fragment.module_id, 'makosh-communication-recipient-suggestion-runtime');

    const input = composeReleaseCompilerInput({
      verification_key_id: 'release-2026',
      trust_root_revision: 1,
      revision: 1,
      distribution_id: 'makosh-desktop',
      release_version: '1.0.0',
      build_id: 'build-communication-recipient-suggestion',
      target_triple: 'aarch64-apple-darwin',
      generation: 1,
      ...releaseProvenance,
      additional_verification_keys: [],
      artifacts: [{
        artifact_kind: 'browser_bootstrap_bundle',
        artifact_id: 'browser.bootstrap',
        relative_path: 'browser/bootstrap.html',
        source_path: browserBootstrapSource,
        required: true,
      }],
    }, [fragment]);
    const release = await compileReleaseDistribution(
      input,
      loadReleaseSigningKey(privateKeyPath),
    );
    const signed = decodeFields(release.signedManifest);
    const manifest = decodeFields(signed.get(2)[0]);
    const artifacts = manifest.get(8).map(decodeFields);

    assert.deepEqual(
      artifacts.map((artifact) => [fieldString(artifact, 2), artifact.get(1)[0]]),
      [
        ['browser.bootstrap', 4n],
        ['communication_recipient_suggestion.runtime.v1', 1n],
        ['communication_recipient_suggestion.storage.v1', 3n],
      ],
    );
    assert.equal(artifacts[1].get(6)[0].length, 32);
    assert.equal(artifacts[1].get(7)[0].length, 32);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});


test('signs the exact Ollama AI runtime and Storage entries emitted by its assembly', async () => {
  const root = canonicalTemporaryDirectory('makosh-ollama-ai-release-fragment-');
  try {
    const runtime = join(root, 'makosh-ollama-ai-runtime');
    const assemblyOutput = join(root, 'assembly');
    const privateKeyPath = join(root, 'release-key.pem');
    writeFileSync(runtime, 'ollama ai runtime bytes', { mode: 0o700 });
    const keyPair = generateKeyPairSync('ec', { namedCurve: 'prime256v1' });
    writeFileSync(
      privateKeyPath,
      keyPair.privateKey.export({ type: 'pkcs8', format: 'pem' }),
      { mode: 0o600 },
    );
    execFileSync('cargo', [
      'run',
      '--quiet',
      '-p',
      'makosh-ollama-ai-assembly',
      '--',
      '--build-id',
      'build-ollama-ai',
      '--output-dir',
      assemblyOutput,
      '--runtime',
      runtime,
    ], { cwd: process.cwd(), stdio: 'pipe' });
    const fragment = JSON.parse(readFileSync(
      join(assemblyOutput, 'ollama-ai.release-artifacts.json'),
      'utf8',
    ));
    assert.deepEqual(
      fragment.artifacts.map(({ artifact_id, artifact_kind }) => [artifact_id, artifact_kind]),
      [
        ['ollama_ai.runtime.v1', 'module_runtime'],
        ['ollama_ai.storage.v1', 'storage_bundle'],
      ],
    );
    assert.equal(fragment.owner_id, 'ollama');
    assert.equal(fragment.module_id, 'makosh-ollama-ai-runtime');

    const input = composeReleaseCompilerInput({
      verification_key_id: 'release-2026',
      trust_root_revision: 1,
      revision: 1,
      distribution_id: 'makosh-desktop',
      release_version: '1.0.0',
      build_id: 'build-ollama-ai',
      target_triple: 'aarch64-apple-darwin',
      generation: 1,
      ...releaseProvenance,
      additional_verification_keys: [],
      artifacts: [{
        artifact_kind: 'browser_bootstrap_bundle',
        artifact_id: 'browser.bootstrap',
        relative_path: 'browser/bootstrap.html',
        source_path: browserBootstrapSource,
        required: true,
      }],
    }, [fragment]);
    const release = await compileReleaseDistribution(
      input,
      loadReleaseSigningKey(privateKeyPath),
    );
    const signed = decodeFields(release.signedManifest);
    const manifest = decodeFields(signed.get(2)[0]);
    const artifacts = manifest.get(8).map(decodeFields);

    assert.deepEqual(
      artifacts.map((artifact) => [fieldString(artifact, 2), artifact.get(1)[0]]),
      [
        ['browser.bootstrap', 4n],
        ['ollama_ai.runtime.v1', 1n],
        ['ollama_ai.storage.v1', 3n],
      ],
    );
    assert.equal(artifacts[1].get(6)[0].length, 32);
    assert.equal(artifacts[1].get(7)[0].length, 32);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test('signs the exact Speech-to-Text engine runtime and Storage entries emitted by its assembly', async () => {
  const root = canonicalTemporaryDirectory('makosh-speech-to-text-release-fragment-');
  try {
    const runtime = join(root, 'makosh-speech-to-text-runtime');
    const assemblyOutput = join(root, 'assembly');
    const privateKeyPath = join(root, 'release-key.pem');
    writeFileSync(runtime, 'speech engine runtime bytes', { mode: 0o700 });
    const keyPair = generateKeyPairSync('ec', { namedCurve: 'prime256v1' });
    writeFileSync(
      privateKeyPath,
      keyPair.privateKey.export({ type: 'pkcs8', format: 'pem' }),
      { mode: 0o600 },
    );
    execFileSync('cargo', [
      'run',
      '--quiet',
      '-p',
      'makosh-speech-to-text-assembly',
      '--',
      '--output-dir',
      assemblyOutput,
      '--build-id',
      'build-speech-to-text',
      '--runtime',
      runtime,
    ], { cwd: process.cwd(), stdio: 'pipe' });
    const fragment = JSON.parse(readFileSync(
      join(assemblyOutput, 'speech-to-text.release-artifacts.json'),
      'utf8',
    ));
    assert.deepEqual(
      fragment.artifacts.map(({ artifact_id, artifact_kind }) => [artifact_id, artifact_kind]),
      [
        ['speech_to_text.runtime.v1', 'module_runtime'],
        ['speech_to_text.storage.v1', 'storage_bundle'],
      ],
    );
    assert.equal(fragment.owner_id, 'speech_to_text');
    assert.equal(fragment.module_id, 'makosh-speech-to-text-runtime');

    const input = composeReleaseCompilerInput({
      verification_key_id: 'release-2026',
      trust_root_revision: 1,
      revision: 1,
      distribution_id: 'makosh-desktop',
      release_version: '1.0.0',
      build_id: 'build-speech-to-text',
      target_triple: 'aarch64-apple-darwin',
      generation: 1,
      ...releaseProvenance,
      additional_verification_keys: [],
      artifacts: [{
        artifact_kind: 'browser_bootstrap_bundle',
        artifact_id: 'browser.bootstrap',
        relative_path: 'browser/bootstrap.html',
        source_path: browserBootstrapSource,
        required: true,
      }],
    }, [fragment]);
    const release = await compileReleaseDistribution(
      input,
      loadReleaseSigningKey(privateKeyPath),
    );
    const signed = decodeFields(release.signedManifest);
    const manifest = decodeFields(signed.get(2)[0]);
    const artifacts = manifest.get(8).map(decodeFields);

    assert.deepEqual(
      artifacts.map((artifact) => [fieldString(artifact, 2), artifact.get(1)[0]]),
      [
        ['browser.bootstrap', 4n],
        ['speech_to_text.runtime.v1', 1n],
        ['speech_to_text.storage.v1', 3n],
      ],
    );
    assert.equal(artifacts[1].get(6)[0].length, 32);
    assert.equal(artifacts[1].get(7)[0].length, 32);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test('signs the exact Whisper runtime Storage runner and model entries emitted by its assembly', async () => {
  const root = canonicalTemporaryDirectory('makosh-whisper-stt-release-fragment-');
  try {
    const runtime = join(root, 'makosh-whisper-stt-runtime');
    const runner = join(root, 'whisper-cli');
    const model = join(root, 'ggml-base.bin');
    const assemblyOutput = join(root, 'assembly');
    const privateKeyPath = join(root, 'release-key.pem');
    writeFileSync(runtime, 'whisper runtime bytes', { mode: 0o700 });
    writeFileSync(runner, 'whisper runner bytes', { mode: 0o500 });
    writeFileSync(model, 'whisper model bytes', { mode: 0o400 });
    const keyPair = generateKeyPairSync('ec', { namedCurve: 'prime256v1' });
    writeFileSync(
      privateKeyPath,
      keyPair.privateKey.export({ type: 'pkcs8', format: 'pem' }),
      { mode: 0o600 },
    );
    execFileSync('cargo', [
      'run',
      '--quiet',
      '-p',
      'makosh-whisper-stt-assembly',
      '--',
      '--output',
      assemblyOutput,
      '--build-id',
      'build-whisper-stt',
      '--runtime',
      runtime,
      '--runner',
      runner,
      '--model',
      model,
    ], { cwd: process.cwd(), stdio: 'pipe' });
    const fragment = JSON.parse(readFileSync(
      join(assemblyOutput, 'whisper-stt.release-artifacts.json'),
      'utf8',
    ));
    assert.deepEqual(
      fragment.artifacts.map(({ artifact_id, artifact_kind }) => [artifact_id, artifact_kind]),
      [
        ['whisper_stt.model.v1', 'module_runtime_read_only_data'],
        ['whisper_stt.runner.v1', 'module_runtime_native_executable'],
        ['whisper_stt.runtime.v1', 'module_runtime'],
        ['whisper_stt.storage.v1', 'storage_bundle'],
      ],
    );
    assert.equal(fragment.owner_id, 'whisper_stt');
    assert.equal(fragment.module_id, 'makosh-whisper-stt-runtime');

    const input = composeReleaseCompilerInput({
      verification_key_id: 'release-2026',
      trust_root_revision: 1,
      revision: 1,
      distribution_id: 'makosh-desktop',
      release_version: '1.0.0',
      build_id: 'build-whisper-stt',
      target_triple: 'aarch64-apple-darwin',
      generation: 1,
      ...releaseProvenance,
      additional_verification_keys: [],
      artifacts: [{
        artifact_kind: 'browser_bootstrap_bundle',
        artifact_id: 'browser.bootstrap',
        relative_path: 'browser/bootstrap.html',
        source_path: browserBootstrapSource,
        required: true,
      }],
    }, [fragment]);
    const release = await compileReleaseDistribution(
      input,
      loadReleaseSigningKey(privateKeyPath),
    );
    const signed = decodeFields(release.signedManifest);
    const manifest = decodeFields(signed.get(2)[0]);
    const artifacts = manifest.get(8).map(decodeFields);

    assert.deepEqual(
      artifacts.map((artifact) => [fieldString(artifact, 2), artifact.get(1)[0]]),
      [
        ['browser.bootstrap', 4n],
        ['whisper_stt.model.v1', 8n],
        ['whisper_stt.runner.v1', 7n],
        ['whisper_stt.runtime.v1', 1n],
        ['whisper_stt.storage.v1', 3n],
      ],
    );
    assert.equal(fieldString(artifacts[1], 13), 'makosh-whisper-stt-runtime');
    assert.equal(fieldString(artifacts[2], 13), 'makosh-whisper-stt-runtime');
    assert.equal(artifacts[3].get(6)[0].length, 32);
    assert.equal(artifacts[3].get(7)[0].length, 32);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test('rejects unordered artifacts and an exposed release signing key', async () => {
  const root = canonicalTemporaryDirectory('makosh-release-compiler-invalid-');
  try {
    const runtime = join(root, 'runtime');
    const descriptor = join(root, 'descriptor.pb');
    const privateKeyPath = join(root, 'release-key.pem');
    writeFileSync(runtime, 'runtime bytes');
    writeFileSync(descriptor, 'descriptor bytes');
    const keyPair = generateKeyPairSync('ec', { namedCurve: 'prime256v1' });
    writeFileSync(privateKeyPath, keyPair.privateKey.export({ type: 'pkcs8', format: 'pem' }), {
      mode: 0o600,
    });
    const input = {
      verification_key_id: 'release-2026',
      trust_root_revision: 1,
      revision: 1,
      distribution_id: 'makosh-desktop',
      release_version: '1.0.0',
      build_id: 'build-1',
      target_triple: 'aarch64-apple-darwin',
      generation: 1,
      ...releaseProvenance,
      additional_verification_keys: [],
      artifacts: [
        {
          artifact_kind: 'module_runtime', artifact_id: 'runtime.z', relative_path: 'bin/z',
          source_path: runtime, required: true,
          descriptor: { relative_path: 'contracts/z.pb', source_path: descriptor }, settings_schema: null,
        },
        {
          artifact_kind: 'module_runtime', artifact_id: 'runtime.a', relative_path: 'bin/a',
          source_path: runtime, required: true,
          descriptor: { relative_path: 'contracts/a.pb', source_path: descriptor }, settings_schema: null,
        },
      ],
    };
    await assert.rejects(
      compileReleaseDistribution(input, loadReleaseSigningKey(privateKeyPath)),
      /artifact is invalid/,
    );
    chmodSync(privateKeyPath, 0o644);
    assert.throws(() => loadReleaseSigningKey(privateKeyPath), /group or other access/);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test('adds sorted public P-256 rotation keys to the release trust root', async () => {
  const root = canonicalTemporaryDirectory('makosh-release-rotation-');
  try {
    const runtime = join(root, 'runtime');
    const descriptor = join(root, 'descriptor.pb');
    const activeKeyPath = join(root, 'active-release-key.pem');
    const nextKeyPath = join(root, 'next-release-key.pem');
    writeFileSync(runtime, 'runtime bytes', { mode: 0o700 });
    writeFileSync(descriptor, 'descriptor bytes', { mode: 0o600 });
    const active = generateKeyPairSync('ec', { namedCurve: 'prime256v1' });
    const next = generateKeyPairSync('ec', { namedCurve: 'prime256v1' });
    writeFileSync(activeKeyPath, active.privateKey.export({ type: 'pkcs8', format: 'pem' }), {
      mode: 0o600,
    });
    writeFileSync(nextKeyPath, next.publicKey.export({ type: 'spki', format: 'pem' }), {
      mode: 0o644,
    });
    const artifacts = await compileReleaseDistribution({
      verification_key_id: 'release-2026',
      trust_root_revision: 2,
      revision: 1,
      distribution_id: 'makosh-desktop',
      release_version: '1.0.0',
      build_id: 'build-rotation',
      target_triple: 'aarch64-apple-darwin',
      generation: 1,
      ...releaseProvenance,
      additional_verification_keys: [{
        key_id: 'release-2027',
        public_key_path: nextKeyPath,
      }],
      artifacts: [{
        artifact_kind: 'module_runtime',
        artifact_id: 'runtime.mail',
        relative_path: 'bin/mail',
        source_path: runtime,
        required: true,
        descriptor: { relative_path: 'contracts/mail.pb', source_path: descriptor },
        settings_schema: null,
      }],
    }, loadReleaseSigningKey(activeKeyPath));
    const trustRoot = decodeFields(artifacts.trustRoot);
    assert.equal(trustRoot.get(2)[0], 2n);
    assert.deepEqual(
      trustRoot.get(3).map((key) => fieldString(decodeFields(key), 1)),
      ['release-2026', 'release-2027'],
    );
    assert.ok(verify(
      'sha256',
      decodeFields(artifacts.signedManifest).get(2)[0],
      { key: active.publicKey, dsaEncoding: 'ieee-p1363' },
      decodeFields(artifacts.signedManifest).get(3)[0],
    ));
    await assert.rejects(
      compileReleaseDistribution({
        verification_key_id: 'release-2026',
        trust_root_revision: 2,
        revision: 1,
        distribution_id: 'makosh-desktop',
        release_version: '1.0.0',
        build_id: 'build-reject-private-key',
        target_triple: 'aarch64-apple-darwin',
        generation: 1,
        ...releaseProvenance,
        additional_verification_keys: [{
          key_id: 'release-2027',
          public_key_path: activeKeyPath,
        }],
        artifacts: [{
          artifact_kind: 'module_runtime',
          artifact_id: 'runtime.mail',
          relative_path: 'bin/mail',
          source_path: runtime,
          required: true,
          descriptor: { relative_path: 'contracts/mail.pb', source_path: descriptor },
          settings_schema: null,
        }],
      }, loadReleaseSigningKey(activeKeyPath)),
      /only public key material/,
    );
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test('never overwrites a release artifact output', () => {
  const root = canonicalTemporaryDirectory('makosh-release-output-');
  try {
    const output = join(root, 'trust-root.pb');
    writeReleaseArtifact(output, Buffer.from('first'));
    assert.throws(() => writeReleaseArtifact(output, Buffer.from('second')));
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test('generates an owner-private P-256 release key without overwriting an existing file', () => {
  const root = canonicalTemporaryDirectory('makosh-release-key-');
  try {
    const output = join(root, 'release-key.pem');
    generateReleaseSigningKey(output);
    assert.equal(loadReleaseSigningKey(output).asymmetricKeyType, 'ec');
    assert.equal(statSync(output).mode & 0o077, 0);
    assert.throws(() => generateReleaseSigningKey(output));
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test('release build CLI materializes every signed artifact and preflights all outputs', () => {
  const root = canonicalTemporaryDirectory('makosh-release-cli-');
  try {
    const runtime = join(root, 'runtime');
    const descriptor = join(root, 'descriptor.pb');
    const inputPath = join(root, 'release.json');
    const keyPath = join(root, 'release-key.pem');
    const trustRootPath = join(root, 'trust-root.pb');
    const signedManifestPath = join(root, 'signed-manifest.pb');
    const distributionRoot = join(root, 'distribution');
    writeFileSync(runtime, 'runtime bytes', { mode: 0o700 });
    writeFileSync(descriptor, 'descriptor bytes', { mode: 0o600 });
    writeFileSync(inputPath, JSON.stringify(releaseInput(runtime, descriptor, browserBootstrapSource)), { mode: 0o600 });
    execFileSync(process.execPath, [
      'scripts/generate-release-signing-key.mjs', '--output', keyPath,
    ], { cwd: process.cwd(), stdio: 'pipe' });

    writeFileSync(signedManifestPath, 'stale signed manifest', { mode: 0o600 });
    assert.throws(() => execFileSync(process.execPath, [
      'scripts/build-distribution-release.mjs', '--input', inputPath,
      '--signing-key', keyPath, '--trust-root', trustRootPath,
      '--signed-manifest', signedManifestPath, '--distribution-root', distributionRoot,
    ], { cwd: process.cwd(), stdio: 'pipe' }));
    assert.equal(existsSync(trustRootPath), false);
    assert.equal(existsSync(distributionRoot), false);
    assert.equal(readFileSync(signedManifestPath, 'utf8'), 'stale signed manifest');

    rmSync(signedManifestPath);
    execFileSync(process.execPath, [
      'scripts/build-distribution-release.mjs', '--input', inputPath,
      '--signing-key', keyPath, '--trust-root', trustRootPath,
      '--signed-manifest', signedManifestPath, '--distribution-root', distributionRoot,
    ], { cwd: process.cwd(), stdio: 'pipe' });
    assert.ok(readFileSync(trustRootPath).length > 0);
    assert.ok(readFileSync(signedManifestPath).length > 0);
    assert.equal(readFileSync(join(distributionRoot, 'bin/mail'), 'utf8'), 'runtime bytes');
    assert.equal(readFileSync(join(distributionRoot, 'contracts/mail.pb'), 'utf8'), 'descriptor bytes');
    assert.equal(
      readFileSync(join(distributionRoot, 'browser/bootstrap.html'), 'utf8'),
      readFileSync(browserBootstrapSource, 'utf8'),
    );
    assert.equal(statSync(distributionRoot).mode & 0o077, 0);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

function releaseInput(runtime, descriptor, browserBootstrap) {
  return {
    verification_key_id: 'release-2026',
    trust_root_revision: 1,
    revision: 1,
    distribution_id: 'makosh-desktop',
    release_version: '1.0.0',
    build_id: 'build-cli',
    target_triple: 'aarch64-apple-darwin',
    generation: 1,
    ...releaseProvenance,
    additional_verification_keys: [],
    artifacts: [
      {
        artifact_kind: 'browser_bootstrap_bundle',
        artifact_id: 'browser.bootstrap',
        relative_path: 'browser/bootstrap.html',
        source_path: browserBootstrap,
        required: true,
      },
      {
        artifact_kind: 'module_runtime',
        artifact_id: 'runtime.mail',
        relative_path: 'bin/mail',
        source_path: runtime,
        required: true,
        descriptor: { relative_path: 'contracts/mail.pb', source_path: descriptor },
        settings_schema: null,
      },
    ],
  };
}
