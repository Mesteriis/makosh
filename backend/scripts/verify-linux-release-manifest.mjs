#!/usr/bin/env node

import { execFileSync } from 'node:child_process';
import {
  closeSync,
  fstatSync,
  lstatSync,
  mkdirSync,
  mkdtempSync,
  openSync,
  readFileSync,
  renameSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname } from 'node:path';

import { renderCompose } from './lib/linux-release-compose.mjs';

const digestPattern = /^[a-z0-9][a-z0-9._/-]*@sha256:[a-f0-9]{64}$/;
const requiredServiceNames = ['kernel', 'vault', 'telemetry', 'storage-control', 'postgres', 'pgbouncer', 'nats'];
const MAX_MANIFEST_BYTES = 1024 * 1024;
const requiredTrustOptions = [
  '--certificate-identity',
  '--certificate-oidc-issuer',
  '--manifest-signature',
  '--manifest-certificate',
];

function fail(message) {
  process.stderr.write(`linux-release-manifest: ${message}\n`);
  process.exitCode = 1;
}

function parseManifestBytes(bytes) {
  try {
    return JSON.parse(bytes.toString('utf8'));
  } catch (error) {
    fail(`cannot parse manifest: ${error.message}`);
    return null;
  }
}

function isObject(value) {
  return value !== null && typeof value === 'object' && !Array.isArray(value);
}

export function validateManifest(manifest) {
  if (!isObject(manifest)) return ['manifest must be an object'];
  const errors = [];
  if (manifest.schema_version !== 1) errors.push('schema_version must be 1');
  if (manifest.deployment_profile !== 'linux_docker_server_v1') {
    errors.push('deployment_profile must be linux_docker_server_v1');
  }
  if (manifest.runtime_lifecycle !== 'external_compose') {
    errors.push('runtime_lifecycle must be external_compose');
  }
  if (manifest.docker_socket_access !== false) {
    errors.push('docker_socket_access must be false');
  }
  if (manifest.service_contract !== 'makosh_platform_service_v1') {
    errors.push('service_contract must be makosh_platform_service_v1');
  }
  if (!isObject(manifest.cosign) || typeof manifest.cosign.certificate_identity !== 'string'
    || manifest.cosign.certificate_identity.length === 0
    || typeof manifest.cosign.oidc_issuer !== 'string'
    || manifest.cosign.oidc_issuer.length === 0) {
    errors.push('cosign certificate identity and OIDC issuer are required');
  }
  if (!isObject(manifest.services)) {
    errors.push('services must be an object');
    return errors;
  }
  const serviceNames = Object.keys(manifest.services).sort();
  if (serviceNames.join(',') !== [...requiredServiceNames].sort().join(',')) {
    errors.push('services must exactly declare the Linux platform runtime set');
  }
  for (const [name, service] of Object.entries(manifest.services)) {
    if (!isObject(service) || typeof service.image !== 'string' || !digestPattern.test(service.image)) {
      errors.push(`${name} must use an immutable sha256 image digest`);
    }
  }
  return errors;
}

export function validateManifestSignerTrust(manifest, trust) {
  if (!isObject(manifest?.cosign)
    || manifest.cosign.certificate_identity !== trust.certificateIdentity
    || manifest.cosign.oidc_issuer !== trust.oidcIssuer) {
    return ['manifest Cosign identity does not match the explicitly pinned release signer'];
  }
  return [];
}

function parseArguments(argv) {
  if (argv.length < 1) return null;
  const [manifestPath, ...argumentsList] = argv;
  const values = new Map();
  let composePath = null;
  for (let index = 0; index < argumentsList.length; index += 2) {
    const option = argumentsList[index];
    const value = argumentsList[index + 1];
    if (typeof value !== 'string' || values.has(option) || (option !== '--write-compose'
      && !requiredTrustOptions.includes(option))) {
      return null;
    }
    values.set(option, value);
  }
  if (values.has('--write-compose')) composePath = values.get('--write-compose');
  if (!requiredTrustOptions.every((option) => values.has(option)
    && values.get(option).length > 0)) return null;
  return {
    manifestPath,
    composePath,
    trust: {
      certificateIdentity: values.get('--certificate-identity'),
      oidcIssuer: values.get('--certificate-oidc-issuer'),
      signaturePath: values.get('--manifest-signature'),
      certificatePath: values.get('--manifest-certificate'),
    },
  };
}

function sameFile(left, right) {
  return left.dev === right.dev
    && left.ino === right.ino
    && left.size === right.size
    && left.mtimeMs === right.mtimeMs
    && left.ctimeMs === right.ctimeMs;
}

function readStableRegularFile(path, label, maximumBytes) {
  const before = lstatSync(path);
  if (before.isSymbolicLink() || !before.isFile() || before.size > maximumBytes) {
    throw new Error(`${label} must be a bounded regular non-symlink file`);
  }
  const descriptor = openSync(path, 'r');
  try {
    const opened = fstatSync(descriptor);
    if (!sameFile(before, opened)) throw new Error(`${label} changed while it was opened`);
    const bytes = readFileSync(descriptor);
    const after = fstatSync(descriptor);
    const pathAfter = lstatSync(path);
    if (!sameFile(opened, after) || !sameFile(opened, pathAfter)) {
      throw new Error(`${label} changed while it was read`);
    }
    return bytes;
  } finally {
    closeSync(descriptor);
  }
}

function requireRegularNonSymlinkFile(path, label) {
  const metadata = lstatSync(path);
  if (metadata.isSymbolicLink() || !metadata.isFile()) {
    throw new Error(`${label} must be a regular non-symlink file`);
  }
}

function verifySignedManifest(manifestPath, trust) {
  const manifestBytes = readStableRegularFile(
    manifestPath,
    'release manifest',
    MAX_MANIFEST_BYTES,
  );
  requireRegularNonSymlinkFile(trust.signaturePath, 'release manifest signature');
  requireRegularNonSymlinkFile(trust.certificatePath, 'release manifest certificate');
  const temporaryDirectory = mkdtempSync(`${tmpdir()}/makosh-release-manifest-`);
  const manifestCopy = `${temporaryDirectory}/release.json`;
  try {
    writeFileSync(manifestCopy, manifestBytes, { mode: 0o600 });
    execFileSync('cosign', [
      'verify-blob',
      '--certificate-identity', trust.certificateIdentity,
      '--certificate-oidc-issuer', trust.oidcIssuer,
      '--signature', trust.signaturePath,
      '--certificate', trust.certificatePath,
      manifestCopy,
    ], { stdio: 'inherit' });
    return parseManifestBytes(manifestBytes);
  } finally {
    rmSync(temporaryDirectory, { recursive: true, force: true });
  }
}

export function main(argv = process.argv.slice(2)) {
  const options = parseArguments(argv);
  if (!options) {
    fail('usage: verify-linux-release-manifest.mjs <manifest.json> --certificate-identity <identity> --certificate-oidc-issuer <issuer> --manifest-signature <signature> --manifest-certificate <certificate> [--write-compose <compose.yaml>]');
    return;
  }
  let manifest;
  try {
    manifest = verifySignedManifest(options.manifestPath, options.trust);
  } catch (error) {
    fail(`Cosign release-manifest verification failed: ${error.message}`);
    return;
  }
  if (!manifest) return;
  const errors = validateManifest(manifest);
  errors.push(...validateManifestSignerTrust(manifest, options.trust));
  if (errors.length > 0) {
    for (const error of errors) fail(error);
    return;
  }
  try {
    for (const service of Object.values(manifest.services)) {
      execFileSync('cosign', [
        'verify',
        '--certificate-identity', manifest.cosign.certificate_identity,
        '--certificate-oidc-issuer', options.trust.oidcIssuer,
        service.image,
      ], { stdio: 'inherit' });
    }
  } catch (error) {
    fail(`Cosign verification failed: ${error.message}`);
    return;
  }
  if (options.composePath) {
    const outputDirectory = dirname(options.composePath);
    const temporaryPath = `${options.composePath}.tmp-${process.pid}`;
    try {
      mkdirSync(outputDirectory, { recursive: true, mode: 0o700 });
      writeFileSync(temporaryPath, renderCompose(manifest, `${outputDirectory}/secrets`), { mode: 0o600 });
      renameSync(temporaryPath, options.composePath);
    } catch (error) {
      fail(`cannot write Compose descriptor: ${error.message}`);
    }
  }
}

if (import.meta.url === `file://${process.argv[1]}`) main();
