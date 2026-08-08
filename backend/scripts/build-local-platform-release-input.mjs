#!/usr/bin/env node

import { createHash } from 'node:crypto';
import { lstatSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { isAbsolute, join } from 'node:path';

const REQUIRED_OPTIONS = [
  '--target', '--artifact-dir', '--browser-bootstrap', '--output', '--descriptor-dir',
  '--distribution-id', '--generation', '--release-version', '--build-id',
  '--source-commit', '--lockfile-sha256', '--sbom-sha256', '--toolchain-sha256',
];

const PLATFORM_ARTIFACTS = [
  ['platform.blob', 'blob', 'blob', 'makosh-blob-service'],
  ['platform.events-authority', 'events', 'events', 'makosh-events-authority-runtime'],
  ['platform.scheduler', 'scheduler', 'scheduler', 'makosh-scheduler-runtime'],
  ['platform.storage', 'storage', 'storage', 'makosh-storage-runtime'],
  ['platform.telemetry', 'telemetry', 'telemetry', 'makosh-telemetry-collector'],
  ['platform.vault', 'vault', 'vault', 'makosh-vault-runtime'],
];

function fail(message) {
  process.stderr.write(`local-platform-release-input: ${message}\n`);
  process.exitCode = 1;
}

function parseArguments(argv) {
  if (argv.length < REQUIRED_OPTIONS.length * 2 || argv.length > (REQUIRED_OPTIONS.length + 1) * 2 || argv.length % 2 !== 0) return null;
  const options = new Map();
  for (let index = 0; index < argv.length; index += 2) {
    const [name, value] = [argv[index], argv[index + 1]];
    if (![...REQUIRED_OPTIONS, '--browser-assets-dir'].includes(name) || typeof value !== 'string' || value.length === 0 || options.has(name)) {
      return null;
    }
    options.set(name, value);
  }
  return REQUIRED_OPTIONS.every((name) => options.has(name)) ? options : null;
}

const BROWSER_ASSET_REFERENCE = /\/assets\/[A-Za-z0-9][A-Za-z0-9._/-]*/g;
const BROWSER_ASSET_EXTENSIONS = new Set(['.css', '.js', '.png', '.svg', '.webp']);
const TEXT_BROWSER_ASSET_EXTENSIONS = new Set(['.css', '.js']);

function compareAscii(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

function browserAssetReferences(bytes) {
  return [...bytes.toString('utf8').matchAll(BROWSER_ASSET_REFERENCE)]
    .map((match) => match[0]);
}

function validateBrowserAssetName(name) {
  const parts = name.split('/');
  const extensionIndex = name.lastIndexOf('.');
  const extension = extensionIndex < 0 ? '' : name.slice(extensionIndex).toLowerCase();
  if (
    name.length === 0
    || parts.some((part) => part.length === 0 || part === '.' || part === '..')
    || !BROWSER_ASSET_EXTENSIONS.has(extension)
  ) {
    throw new Error(`browser asset reference is invalid: /assets/${name}`);
  }
  return extension;
}

function requireBrowserAsset(directory, name) {
  let current = directory;
  for (const part of name.split('/')) {
    current = join(current, part);
    const metadata = lstatSync(current);
    if (metadata.isSymbolicLink()) {
      throw new Error(`browser asset must not traverse symlinks: /assets/${name}`);
    }
  }
  const metadata = lstatSync(current);
  if (!metadata.isFile()) {
    throw new Error(`browser asset must be a regular file: /assets/${name}`);
  }
  return current;
}

function browserAssetArtifacts(directory, browserBootstrap) {
  if (!directory) return [];
  if (!isAbsolute(directory) || !lstatSync(directory).isDirectory() || lstatSync(directory).isSymbolicLink()) {
    throw new Error('browser assets directory must be an absolute non-symlink directory');
  }

  const pending = browserAssetReferences(readFileSync(browserBootstrap));
  const discovered = new Map();
  while (pending.length > 0) {
    const reference = pending.pop();
    const name = reference.slice('/assets/'.length);
    const extension = validateBrowserAssetName(name);
    if (discovered.has(name)) continue;
    const path = requireBrowserAsset(directory, name);
    discovered.set(name, path);
    if (TEXT_BROWSER_ASSET_EXTENSIONS.has(extension)) {
      pending.push(...browserAssetReferences(readFileSync(path)));
    }
  }

  return [...discovered.entries()].sort(([left], [right]) => compareAscii(left, right)).map(([name, path]) => {
    return {
      artifact_kind: 'browser_client_asset', artifact_id: `browser.asset.${name}`,
      relative_path: `browser/assets/${name}`, source_path: path, required: true,
    };
  });
}

function encodeVarint(value) {
  const bytes = [];
  let remaining = BigInt(value);
  do {
    let byte = Number(remaining & 0x7fn);
    remaining >>= 7n;
    if (remaining !== 0n) byte |= 0x80;
    bytes.push(byte);
  } while (remaining !== 0n);
  return Buffer.from(bytes);
}

function fieldBytes(number, value) {
  const bytes = Buffer.from(value);
  return Buffer.concat([encodeVarint((number << 3) | 2), encodeVarint(bytes.length), bytes]);
}

function fieldString(number, value) {
  return fieldBytes(number, Buffer.from(value, 'utf8'));
}

function fieldVarint(number, value) {
  return Buffer.concat([encodeVarint(number << 3), encodeVarint(value)]);
}

function emptySettingsSchema() {
  return Buffer.concat([fieldVarint(1, 1), fieldVarint(2, 1)]);
}

function moduleDescriptor({
  moduleId, ownerId, releaseVersion, buildId, schema, descriptorRevision,
}) {
  const schemaReference = Buffer.concat([
    fieldVarint(1, 1),
    fieldVarint(2, 1),
    fieldVarint(3, schema.length),
    fieldBytes(4, createHash('sha256').update(schema).digest()),
  ]);
  return Buffer.concat([
    fieldVarint(1, 1),
    fieldVarint(2, descriptorRevision),
    fieldString(3, moduleId),
    fieldString(4, ownerId),
    fieldVarint(5, 5),
    fieldString(6, releaseVersion),
    fieldString(7, buildId),
    fieldBytes(10, schemaReference),
  ]);
}

function requireRegularFile(path, label) {
  if (!isAbsolute(path)) throw new Error(`${label} must be absolute`);
  let metadata;
  try {
    metadata = lstatSync(path);
  } catch (error) {
    throw new Error(`${label} is unavailable: ${error.code ?? error.message}`);
  }
  if (metadata.isSymbolicLink() || !metadata.isFile()) {
    throw new Error(`${label} must be a regular non-symlink file`);
  }
}

function createModuleArtifact({ artifactId, moduleId, ownerId, binaryName }, options) {
  const schema = emptySettingsSchema();
  const descriptor = moduleDescriptor({
    moduleId,
    ownerId,
    releaseVersion: options.get('--release-version'),
    buildId: options.get('--build-id'),
    schema,
    descriptorRevision: artifactId === 'platform.scheduler' ? 2 : 1,
  });
  const descriptorPath = join(options.get('--descriptor-dir'), `${artifactId}.descriptor.pb`);
  const schemaPath = join(options.get('--descriptor-dir'), `${artifactId}.settings.pb`);
  writeFileSync(descriptorPath, descriptor, { mode: 0o600, flag: 'wx' });
  writeFileSync(schemaPath, schema, { mode: 0o600, flag: 'wx' });
  return {
    artifact_kind: 'module_runtime', artifact_id: artifactId, relative_path: `bin/${binaryName}`,
    source_path: join(options.get('--artifact-dir'), binaryName), required: true,
    descriptor: { relative_path: `contracts/${artifactId}.descriptor.pb`, source_path: descriptorPath },
    settings_schema: { relative_path: `contracts/${artifactId}.settings.pb`, source_path: schemaPath },
  };
}

function buildInput(options) {
  const browserBootstrap = options.get('--browser-bootstrap');
  requireRegularFile(browserBootstrap, 'browser bootstrap');
  for (const [artifactId, , , binaryName] of PLATFORM_ARTIFACTS) {
    requireRegularFile(join(options.get('--artifact-dir'), binaryName), `${artifactId} binary`);
  }
  mkdirSync(options.get('--descriptor-dir'), { recursive: true, mode: 0o700 });
  const artifacts = PLATFORM_ARTIFACTS
    .map(([artifactId, moduleId, ownerId, binaryName]) => createModuleArtifact(
      { artifactId, moduleId, ownerId, binaryName }, options,
    ));
  artifacts.unshift({
    artifact_kind: 'browser_bootstrap_bundle', artifact_id: 'browser.bootstrap',
    relative_path: 'browser/bootstrap.html', source_path: browserBootstrap, required: true,
  });
  artifacts.push(...browserAssetArtifacts(options.get('--browser-assets-dir'), browserBootstrap));
  artifacts.sort((left, right) => compareAscii(left.artifact_id, right.artifact_id));
  return {
    verification_key_id: 'local-release-2026', trust_root_revision: 1, revision: 1,
    distribution_id: options.get('--distribution-id'), release_version: options.get('--release-version'),
    build_id: options.get('--build-id'), target_triple: options.get('--target'),
    generation: Number(options.get('--generation')),
    source_commit: options.get('--source-commit'),
    lockfile_sha256: options.get('--lockfile-sha256'),
    sbom_sha256: options.get('--sbom-sha256'),
    toolchain_sha256: options.get('--toolchain-sha256'),
    additional_verification_keys: [], artifacts,
  };
}

export function main(argv = process.argv.slice(2)) {
  const options = parseArguments(argv);
  if (!options) {
    fail('usage: build-local-platform-release-input.mjs --target <triple> --artifact-dir <dir> --browser-bootstrap <html> [--browser-assets-dir <dir>] --output <json> --descriptor-dir <dir> --distribution-id <id> --generation <positive-safe-integer> --release-version <version> --build-id <id> --source-commit <hex> --lockfile-sha256 <hex> --sbom-sha256 <hex> --toolchain-sha256 <hex>');
    return;
  }
  try {
    for (const option of ['--artifact-dir', '--descriptor-dir', '--output']) {
      if (!isAbsolute(options.get(option))) throw new Error(`${option} must be absolute`);
    }
    if (!/^[a-f0-9]{40}$|^[a-f0-9]{64}$/.test(options.get('--source-commit'))) {
      throw new Error('--source-commit must be a 40-byte or 64-byte lowercase hex digest');
    }
    for (const option of ['--lockfile-sha256', '--sbom-sha256', '--toolchain-sha256']) {
      if (!/^[a-f0-9]{64}$/.test(options.get(option))) {
        throw new Error(`${option} must be a lowercase SHA-256 digest`);
      }
    }
    const generation = Number(options.get('--generation'));
    if (!Number.isSafeInteger(generation) || generation <= 0) {
      throw new Error('--generation must be a positive safe integer');
    }
    writeFileSync(options.get('--output'), `${JSON.stringify(buildInput(options), null, 2)}\n`, { mode: 0o600, flag: 'wx' });
  } catch (error) {
    fail(error.message);
  }
}

if (import.meta.url === `file://${process.argv[1]}`) main();
