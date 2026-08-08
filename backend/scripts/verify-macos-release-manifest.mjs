#!/usr/bin/env node

import { execFileSync, spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { lstatSync, readFileSync, statSync } from 'node:fs';
import { extname, join } from 'node:path';

const digestPattern = /^[a-f0-9]{64}$/;
const teamIdPattern = /^[A-Z0-9]{10}$/;

function fail(message) {
  process.stderr.write(`macos-release-manifest: ${message}\n`);
  process.exitCode = 1;
}

function isObject(value) {
  return value !== null && typeof value === 'object' && !Array.isArray(value);
}

function isAbsolutePath(value) {
  return typeof value === 'string' && value.startsWith('/');
}

function isExpectedBundleArtifactPath(bundlePath, actualPath, relativePath) {
  return actualPath === join(bundlePath, ...relativePath);
}

export function validateManifest(manifest) {
  if (!isObject(manifest)) return ['manifest must be an object'];
  const errors = [];
  if (manifest.schema_version !== 1) errors.push('schema_version must be 1');
  if (manifest.deployment_profile !== 'macos_tauri_embedded_v1') {
    errors.push('deployment_profile must be macos_tauri_embedded_v1');
  }
  if (manifest.runtime_lifecycle !== 'managed_child') {
    errors.push('runtime_lifecycle must be managed_child');
  }
  if (!isObject(manifest.signing) || !teamIdPattern.test(manifest.signing.team_id ?? '')) {
    errors.push('signing.team_id must be a 10-character Apple Team ID');
  }
  if (!isObject(manifest.artifacts)) {
    errors.push('artifacts must be an object');
    return errors;
  }
  const {
    tauri_bundle: bundle,
    kernel_sidecar: sidecar,
    release_trust_root: trustRoot,
    signed_distribution_manifest: signedManifest,
    distribution_root: distributionRoot,
  } = manifest.artifacts;
  if (!isObject(bundle) || !isAbsolutePath(bundle.path) || extname(bundle.path) !== '.app') {
    errors.push('artifacts.tauri_bundle.path must be an absolute .app bundle');
  }
  if (!isObject(sidecar) || !isAbsolutePath(sidecar.path) || !digestPattern.test(sidecar.sha256 ?? '')) {
    errors.push('artifacts.kernel_sidecar must have an absolute path and lowercase sha256 digest');
  }
  if (!isObject(trustRoot) || !isAbsolutePath(trustRoot.path) || !digestPattern.test(trustRoot.sha256 ?? '')) {
    errors.push('artifacts.release_trust_root must have an absolute path and lowercase sha256 digest');
  }
  if (!isObject(signedManifest) || !isAbsolutePath(signedManifest.path) || !digestPattern.test(signedManifest.sha256 ?? '')) {
    errors.push('artifacts.signed_distribution_manifest must have an absolute path and lowercase sha256 digest');
  }
  if (!isObject(distributionRoot) || !isAbsolutePath(distributionRoot.path)) {
    errors.push('artifacts.distribution_root.path must be absolute');
  }
  if (isObject(bundle) && isAbsolutePath(bundle.path)) {
    if (!isExpectedBundleArtifactPath(
      bundle.path,
      sidecar?.path,
      ['Contents', 'MacOS', 'makosh-kernel'],
    )) {
      errors.push('artifacts.kernel_sidecar must be the Tauri-normalized Kernel sidecar inside the bundle');
    }
    if (!isExpectedBundleArtifactPath(
      bundle.path,
      trustRoot?.path,
      ['Contents', 'Resources', 'makosh-kernel-release', 'makosh-release-trust-root.pb'],
    )) {
      errors.push('artifacts.release_trust_root must be inside the signed managed-launch resource directory');
    }
    if (!isExpectedBundleArtifactPath(
      bundle.path,
      signedManifest?.path,
      ['Contents', 'Resources', 'makosh-kernel-release', 'makosh-signed-distribution-manifest.pb'],
    )) {
      errors.push('artifacts.signed_distribution_manifest must be inside the signed managed-launch resource directory');
    }
    if (!isExpectedBundleArtifactPath(
      bundle.path,
      distributionRoot?.path,
      ['Contents', 'Resources', 'makosh-kernel-release', 'distribution'],
    )) {
      errors.push('artifacts.distribution_root must be inside the signed managed-launch resource directory');
    }
  }
  return errors;
}

function parseManifest(path) {
  try {
    return JSON.parse(readFileSync(path, 'utf8'));
  } catch (error) {
    fail(`cannot parse manifest: ${error.message}`);
    return null;
  }
}

function rejectSymlink(path, label) {
  if (lstatSync(path).isSymbolicLink()) throw new Error(`${label} must not be a symlink`);
}

function codesignDetails(path) {
  const result = spawnSync('/usr/bin/codesign', ['-dvv', path], { encoding: 'utf8' });
  if (result.error) throw result.error;
  if (result.status !== 0) {
    throw new Error(result.stderr || 'codesign could not read artifact identity');
  }
  // `codesign -dvv` reports signing metadata on stderr even on success.
  return result.stderr;
}

function verifyArtifact(path, expectedTeamId) {
  execFileSync('/usr/bin/codesign', ['--verify', '--strict', '--verbose=4', path], { stdio: 'inherit' });
  const details = codesignDetails(path);
  if (!details.includes(`TeamIdentifier=${expectedTeamId}`)) {
    throw new Error('signed artifact team identity does not match the release manifest');
  }
}

function verifyNotarizationStaple(bundlePath) {
  execFileSync('/usr/bin/xcrun', ['stapler', 'validate', bundlePath], { stdio: 'inherit' });
}

export function main(argv = process.argv.slice(2)) {
  if (argv.length !== 1) {
    fail('usage: verify-macos-release-manifest.mjs <manifest.json>');
    return;
  }
  if (process.platform !== 'darwin') {
    fail('macOS release verification must run on macOS');
    return;
  }
  const manifest = parseManifest(argv[0]);
  if (!manifest) return;
  const errors = validateManifest(manifest);
  if (errors.length > 0) {
    for (const error of errors) fail(error);
    return;
  }
  const {
    tauri_bundle: bundle,
    kernel_sidecar: sidecar,
    release_trust_root: trustRoot,
    signed_distribution_manifest: signedManifest,
    distribution_root: distributionRoot,
  } = manifest.artifacts;
  try {
    rejectSymlink(bundle.path, 'Tauri bundle');
    if (!statSync(bundle.path).isDirectory()) throw new Error('Tauri bundle must be a directory');
    rejectSymlink(sidecar.path, 'Kernel sidecar');
    if (!statSync(sidecar.path).isFile()) throw new Error('Kernel sidecar must be a regular file');
    verifyArtifact(bundle.path, manifest.signing.team_id);
    verifyNotarizationStaple(bundle.path);
    execFileSync('/usr/sbin/spctl', ['--assess', '--type', 'execute', '--verbose=4', bundle.path], { stdio: 'inherit' });
    verifyArtifact(sidecar.path, manifest.signing.team_id);
    const actualDigest = createHash('sha256').update(readFileSync(sidecar.path)).digest('hex');
    if (actualDigest !== sidecar.sha256) throw new Error('Kernel sidecar digest does not match release manifest');
    rejectSymlink(trustRoot.path, 'Release trust root');
    if (!statSync(trustRoot.path).isFile()) throw new Error('release trust root must be a regular file');
    const trustRootDigest = createHash('sha256').update(readFileSync(trustRoot.path)).digest('hex');
    if (trustRootDigest !== trustRoot.sha256) throw new Error('release trust root digest does not match release manifest');
    rejectSymlink(signedManifest.path, 'signed distribution manifest');
    if (!statSync(signedManifest.path).isFile()) throw new Error('signed distribution manifest must be a regular file');
    const signedManifestDigest = createHash('sha256').update(readFileSync(signedManifest.path)).digest('hex');
    if (signedManifestDigest !== signedManifest.sha256) throw new Error('signed distribution manifest digest does not match release manifest');
    rejectSymlink(distributionRoot.path, 'managed distribution bundle');
    if (!statSync(distributionRoot.path).isDirectory()) throw new Error('managed distribution bundle must be a directory');
  } catch (error) {
    fail(`artifact verification failed: ${error.message}`);
  }
}

if (import.meta.url === `file://${process.argv[1]}`) main();
