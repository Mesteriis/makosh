import {
  createHash,
  createPrivateKey,
  createPublicKey,
  generateKeyPairSync,
  sign,
} from 'node:crypto';
import {
  closeSync,
  copyFileSync,
  createReadStream,
  fstatSync,
  lstatSync,
  mkdtempSync,
  mkdirSync,
  openSync,
  readFileSync,
  renameSync,
  rmSync,
  unlinkSync,
  writeFileSync,
} from 'node:fs';
import { dirname, isAbsolute, join } from 'node:path';

const MAX_ARTIFACT_BYTES = 2 * 1024 * 1024 * 1024;
const MAX_CONTRACT_BYTES = 256 * 1024;
const MAX_RELEASE_INPUT_BYTES = 256 * 1024;
const MAX_RELEASE_SIGNING_KEY_BYTES = 64 * 1024;
const MAX_IDENTIFIER_BYTES = 128;
const artifactKinds = new Map([
  ['module_runtime', 1],
  ['infrastructure_executable', 2],
  ['storage_bundle', 3],
  ['browser_bootstrap_bundle', 4],
  ['browser_client_asset', 5],
  ['module_runtime_native_dependency', 6],
  ['module_runtime_native_executable', 7],
  ['module_runtime_read_only_data', 8],
]);
const boundRuntimeArtifactKinds = new Set([6, 7, 8]);

function exactKeys(value, keys) {
  return value !== null
    && typeof value === 'object'
    && !Array.isArray(value)
    && Object.keys(value).length === keys.length
    && Object.keys(value).every((key) => keys.includes(key));
}

function validIdentifier(value) {
  return typeof value === 'string' && value.length > 0 && value.length <= MAX_IDENTIFIER_BYTES
    && /^[\x00-\x7f]+$/.test(value);
}

function validModuleIdentifier(value) {
  return typeof value === 'string'
    && value.length > 0
    && value.length <= MAX_IDENTIFIER_BYTES
    && /^[a-z0-9_.-]+$/.test(value);
}

function validTarget(value) {
  return validIdentifier(value) && value.includes('-') && !/[\\/\s]/.test(value);
}

function validSha256(value) {
  return typeof value === 'string' && /^[a-f0-9]{64}$/.test(value);
}

function validSourceCommit(value) {
  return typeof value === 'string' && /^(?:[a-f0-9]{40}|[a-f0-9]{64})$/.test(value);
}

function compareAscii(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

function validRelativePath(value) {
  return typeof value === 'string'
    && value.length > 0
    && value.length <= 1024
    && !value.startsWith('/')
    && !value.includes('\\')
    && value.split('/').every((part) => part.length > 0 && part !== '.' && part !== '..');
}

function encodeVarint(value) {
  let remaining = BigInt(value);
  const bytes = [];
  do {
    let byte = Number(remaining & 0x7fn);
    remaining >>= 7n;
    if (remaining !== 0n) byte |= 0x80;
    bytes.push(byte);
  } while (remaining !== 0n);
  return Buffer.from(bytes);
}

function fieldVarint(number, value) {
  return Buffer.concat([encodeVarint(number << 3), encodeVarint(value)]);
}

function fieldBytes(number, bytes) {
  const value = Buffer.from(bytes);
  return Buffer.concat([encodeVarint((number << 3) | 2), encodeVarint(value.length), value]);
}

function fieldString(number, value) {
  return fieldBytes(number, Buffer.from(value, 'utf8'));
}

function encodeArtifact(artifact) {
  const fields = [
    fieldVarint(1, artifact.kind),
    fieldString(2, artifact.artifactId),
    fieldString(3, artifact.relativePath),
    fieldVarint(4, artifact.sizeBytes),
    fieldBytes(5, artifact.sha256),
  ];
  if (artifact.descriptor) {
    fields.push(
      fieldBytes(6, artifact.descriptor.sha256),
      fieldString(9, artifact.descriptor.relativePath),
      fieldVarint(10, artifact.descriptor.sizeBytes),
    );
  }
  if (artifact.settingsSchema) {
    fields.push(
      fieldBytes(7, artifact.settingsSchema.sha256),
      fieldString(11, artifact.settingsSchema.relativePath),
      fieldVarint(12, artifact.settingsSchema.sizeBytes),
    );
  }
  if (artifact.required) fields.push(fieldVarint(8, 1));
  if (artifact.boundModuleId) fields.push(fieldString(13, artifact.boundModuleId));
  return Buffer.concat(fields);
}

function encodeManifest(input, artifacts) {
  return Buffer.concat([
    fieldVarint(1, 1),
    fieldVarint(2, input.revision),
    fieldString(3, input.distribution_id),
    fieldString(4, input.release_version),
    fieldString(5, input.build_id),
    fieldString(6, input.target_triple),
    fieldVarint(7, input.generation),
    ...artifacts.map((artifact) => fieldBytes(8, encodeArtifact(artifact))),
    fieldString(9, input.source_commit),
    fieldString(10, input.lockfile_sha256),
    fieldString(11, input.sbom_sha256),
    fieldString(12, input.toolchain_sha256),
  ]);
}

function encodeTrustRoot(revision, verificationKeys) {
  const keys = verificationKeys.map(({ keyId, publicKeySec1 }) => fieldBytes(3, Buffer.concat([
    fieldString(1, keyId),
    fieldBytes(2, publicKeySec1),
  ])));
  return Buffer.concat([
    fieldVarint(1, 1),
    fieldVarint(2, revision),
    ...keys,
  ]);
}

function encodeSignedManifest(keyId, rawManifest, signature) {
  return Buffer.concat([
    fieldString(1, keyId),
    fieldBytes(2, rawManifest),
    fieldBytes(3, signature),
  ]);
}

function sameFile(left, right) {
  return left.dev === right.dev
    && left.ino === right.ino
    && left.size === right.size
    && left.mtimeMs === right.mtimeMs
    && left.ctimeMs === right.ctimeMs;
}

function assertRegularNonSymlinkPath(path, label) {
  if (!isAbsolute(path)) throw new Error(`${label} path must be absolute`);
  let current = '/';
  for (const part of path.split('/').filter(Boolean)) {
    current = `${current}${current === '/' ? '' : '/'}${part}`;
    if (lstatSync(current).isSymbolicLink()) throw new Error(`${label} must not traverse a symlink`);
  }
  const metadata = lstatSync(path);
  if (metadata.isSymbolicLink() || !metadata.isFile()) {
    throw new Error(`${label} must be a regular non-symlink file`);
  }
  return metadata;
}

function assertNonSymlinkDirectory(path, label) {
  if (!isAbsolute(path)) throw new Error(`${label} path must be absolute`);
  let current = '/';
  for (const part of path.split('/').filter(Boolean)) {
    current = `${current}${current === '/' ? '' : '/'}${part}`;
    if (lstatSync(current).isSymbolicLink()) throw new Error(`${label} must not traverse a symlink`);
  }
  const metadata = lstatSync(path);
  if (metadata.isSymbolicLink() || !metadata.isDirectory()) {
    throw new Error(`${label} must be a non-symlink directory`);
  }
}

function readStableRegularFile(path, label, maximumBytes) {
  const before = assertRegularNonSymlinkPath(path, label);
  if (before.size === 0 || before.size > maximumBytes) throw new Error(`${label} size is invalid`);
  const descriptor = openSync(path, 'r');
  try {
    const opened = fstatSync(descriptor);
    if (!sameFile(before, opened)) throw new Error(`${label} changed while it was opened`);
    const bytes = readFileSync(descriptor);
    const after = fstatSync(descriptor);
    const pathAfter = assertRegularNonSymlinkPath(path, label);
    if (!sameFile(opened, after) || !sameFile(opened, pathAfter)) {
      throw new Error(`${label} changed while it was read`);
    }
    return bytes;
  } finally {
    closeSync(descriptor);
  }
}

async function digestSource(path, label, maximumBytes) {
  const before = assertRegularNonSymlinkPath(path, label);
  if (before.size === 0 || before.size > maximumBytes) throw new Error(`${label} size is invalid`);
  const hash = createHash('sha256');
  const stream = createReadStream(path, { autoClose: true });
  const opened = await new Promise((resolve, reject) => {
    stream.once('open', (descriptor) => resolve(fstatSync(descriptor)));
    stream.once('error', reject);
  });
  if (!sameFile(before, opened)) throw new Error(`${label} changed while it was opened`);
  for await (const chunk of stream) hash.update(chunk);
  const after = assertRegularNonSymlinkPath(path, label);
  if (!sameFile(opened, after)) throw new Error(`${label} changed while it was read`);
  return { sizeBytes: BigInt(opened.size), sha256: hash.digest() };
}

function validateInput(input) {
  const requiredKeys = [
    'verification_key_id',
    'trust_root_revision',
    'revision',
    'distribution_id',
    'release_version',
    'build_id',
    'target_triple',
    'generation',
    'source_commit',
    'lockfile_sha256',
    'sbom_sha256',
    'toolchain_sha256',
    'additional_verification_keys',
    'artifacts',
  ];
  if (!exactKeys(input, requiredKeys)
    || !validIdentifier(input.verification_key_id)
    || !Number.isSafeInteger(input.trust_root_revision) || input.trust_root_revision < 1
    || !Number.isSafeInteger(input.revision) || input.revision < 1
    || !validIdentifier(input.distribution_id)
    || !validIdentifier(input.release_version)
    || !validIdentifier(input.build_id)
    || !validTarget(input.target_triple)
    || !Number.isSafeInteger(input.generation) || input.generation < 1
    || !validSourceCommit(input.source_commit)
    || !validSha256(input.lockfile_sha256)
    || !validSha256(input.sbom_sha256)
    || !validSha256(input.toolchain_sha256)
    || !Array.isArray(input.additional_verification_keys)
    || input.additional_verification_keys.length > 255
    || !Array.isArray(input.artifacts) || input.artifacts.length === 0 || input.artifacts.length > 256) {
    throw new Error('release compiler input is invalid');
  }
}

function validateAdditionalVerificationKeys(keys, activeKeyId) {
  let previousKeyId = '';
  return keys.map((key) => {
    if (!exactKeys(key, ['key_id', 'public_key_path'])
      || !validIdentifier(key.key_id)
      || key.key_id <= previousKeyId
      || key.key_id === activeKeyId
      || !isAbsolute(key.public_key_path)) {
      throw new Error('release compiler additional verification key is invalid');
    }
    previousKeyId = key.key_id;
    return key;
  });
}

function validateArtifactInput(artifact, previousArtifactId) {
  const commonKeys = ['artifact_kind', 'artifact_id', 'relative_path', 'source_path', 'required'];
  const moduleKeys = [...commonKeys, 'descriptor', 'settings_schema'];
  const nativeDependencyKeys = [...commonKeys, 'bound_module_id'];
  const kind = artifactKinds.get(artifact?.artifact_kind);
  const keys = kind === 1
    ? moduleKeys
    : boundRuntimeArtifactKinds.has(kind) ? nativeDependencyKeys : commonKeys;
  if (!kind || !exactKeys(artifact, keys)
    || !validIdentifier(artifact.artifact_id)
    || artifact.artifact_id <= previousArtifactId
    || !validRelativePath(artifact.relative_path)
    || !isAbsolute(artifact.source_path)
    || typeof artifact.required !== 'boolean') {
    const artifactId = validIdentifier(artifact?.artifact_id)
      ? artifact.artifact_id
      : '<invalid-artifact-id>';
    throw new Error(`release compiler artifact is invalid: ${artifactId}`);
  }
  if (kind === 1) {
    validateContractInput(artifact.descriptor, 'descriptor');
    if (artifact.settings_schema !== null) validateContractInput(artifact.settings_schema, 'settings schema');
  }
  if (boundRuntimeArtifactKinds.has(kind) && !validModuleIdentifier(artifact.bound_module_id)) {
    throw new Error('release compiler runtime artifact binding is invalid');
  }
  return kind;
}

function claimOutputPath(occupiedPaths, relativePath) {
  for (const occupied of occupiedPaths) {
    if (occupied === relativePath
      || occupied.startsWith(`${relativePath}/`)
      || relativePath.startsWith(`${occupied}/`)) {
      throw new Error('release compiler output paths must not overlap');
    }
  }
  occupiedPaths.add(relativePath);
}

function validateContractInput(contract, label) {
  if (!exactKeys(contract, ['relative_path', 'source_path'])
    || !validRelativePath(contract.relative_path)
    || !isAbsolute(contract.source_path)) {
    throw new Error(`release compiler ${label} is invalid`);
  }
}

function loadSigningKey(path) {
  const metadata = assertRegularNonSymlinkPath(path, 'release signing key');
  if ((metadata.mode & 0o077) !== 0) {
    throw new Error('release signing key must not grant group or other access');
  }
  const key = createPrivateKey(readStableRegularFile(
    path,
    'release signing key',
    MAX_RELEASE_SIGNING_KEY_BYTES,
  ));
  if (key.asymmetricKeyType !== 'ec' || key.asymmetricKeyDetails?.namedCurve !== 'prime256v1') {
    throw new Error('release signing key must be P-256');
  }
  return key;
}

function publicKeySec1(key, label) {
  const publicKey = key.type === 'public' ? key : createPublicKey(key);
  if (publicKey.asymmetricKeyType !== 'ec'
    || publicKey.asymmetricKeyDetails?.namedCurve !== 'prime256v1') {
    throw new Error(`${label} must be P-256`);
  }
  const jwk = publicKey.export({ format: 'jwk' });
  const x = Buffer.from(jwk.x, 'base64url');
  const y = Buffer.from(jwk.y, 'base64url');
  if (x.length !== 32 || y.length !== 32) throw new Error(`${label} is invalid`);
  return Buffer.concat([Buffer.from([4]), x, y]);
}

function loadAdditionalVerificationKey(path) {
  const bytes = readStableRegularFile(
    path,
    'release verification key',
    MAX_RELEASE_SIGNING_KEY_BYTES,
  );
  try {
    createPrivateKey(bytes);
  } catch {
    return publicKeySec1(createPublicKey(bytes), 'release verification key');
  }
  throw new Error('release verification key must contain only public key material');
}

export async function compileReleaseDistribution(input, privateKey) {
  const content = await compileUnsignedReleaseContent(input);
  const additionalVerificationKeys = validateAdditionalVerificationKeys(
    input.additional_verification_keys,
    input.verification_key_id,
  );
  const signature = sign('sha256', content.rawManifest, { key: privateKey, dsaEncoding: 'ieee-p1363' });
  if (signature.length !== 64) throw new Error('release manifest signature is invalid');
  const verificationKeys = [
    {
      keyId: input.verification_key_id,
      publicKeySec1: publicKeySec1(privateKey, 'release signing public key'),
    },
    ...additionalVerificationKeys.map((key) => ({
      keyId: key.key_id,
      publicKeySec1: loadAdditionalVerificationKey(key.public_key_path),
    })),
  ].sort((left, right) => compareAscii(left.keyId, right.keyId));
  return {
    ...content,
    signedManifest: encodeSignedManifest(input.verification_key_id, content.rawManifest, signature),
    trustRoot: encodeTrustRoot(input.trust_root_revision, verificationKeys),
  };
}

// This deliberately excludes trust-root material and the ECDSA signature. It is
// the deterministic content boundary that two isolated builds must agree on
// before a release signer is allowed to see either output.
export async function compileUnsignedReleaseContent(input) {
  validateInput(input);
  const artifacts = [];
  const occupiedPaths = new Set();
  let previousArtifactId = '';
  for (const artifact of input.artifacts) {
    const kind = validateArtifactInput(artifact, previousArtifactId);
    claimOutputPath(occupiedPaths, artifact.relative_path);
    if (kind === 1) {
      claimOutputPath(occupiedPaths, artifact.descriptor.relative_path);
      if (artifact.settings_schema !== null) {
        claimOutputPath(occupiedPaths, artifact.settings_schema.relative_path);
      }
    }
    const executable = await digestSource(artifact.source_path, 'release artifact', MAX_ARTIFACT_BYTES);
    const descriptor = kind === 1
      ? await digestSource(artifact.descriptor.source_path, 'release descriptor', MAX_CONTRACT_BYTES)
      : null;
    const settingsSchema = kind === 1 && artifact.settings_schema !== null
      ? await digestSource(artifact.settings_schema.source_path, 'release settings schema', MAX_CONTRACT_BYTES)
      : null;
    artifacts.push({
      kind,
      artifactId: artifact.artifact_id,
      relativePath: artifact.relative_path,
      sourcePath: artifact.source_path,
      sizeBytes: executable.sizeBytes,
      sha256: executable.sha256,
      required: artifact.required,
      descriptor: descriptor && {
        ...descriptor,
        relativePath: artifact.descriptor.relative_path,
        sourcePath: artifact.descriptor.source_path,
      },
      settingsSchema: settingsSchema && {
        ...settingsSchema,
        relativePath: artifact.settings_schema.relative_path,
        sourcePath: artifact.settings_schema.source_path,
      },
      boundModuleId: boundRuntimeArtifactKinds.has(kind) ? artifact.bound_module_id : null,
    });
    previousArtifactId = artifact.artifact_id;
  }
  return { artifacts, rawManifest: encodeManifest(input, artifacts) };
}

export function composeReleaseCompilerInput(input, fragments) {
  validateInput(input);
  validateArtifactSet(input.artifacts);
  if (!Array.isArray(fragments) || fragments.length === 0 || fragments.length > 64) {
    throw new Error('release compiler artifact fragments are invalid');
  }
  const moduleIds = new Set();
  const fragmentArtifacts = [];
  for (const fragment of fragments) {
    if (!exactKeys(fragment, ['version', 'owner_id', 'module_id', 'artifacts'])
      || fragment.version !== 1
      || !validModuleIdentifier(fragment.owner_id)
      || !validModuleIdentifier(fragment.module_id)
      || moduleIds.has(fragment.module_id)
      || !Array.isArray(fragment.artifacts)
      || fragment.artifacts.length === 0
      || fragment.artifacts.length > 64) {
      throw new Error('release compiler artifact fragment is invalid');
    }
    moduleIds.add(fragment.module_id);
    validateArtifactSet(fragment.artifacts);
    const moduleRuntimeCount = fragment.artifacts.filter(
      (artifact) => artifact.artifact_kind === 'module_runtime',
    ).length;
    if (moduleRuntimeCount !== 1
      || fragment.artifacts.some((artifact) => (
        [
          'module_runtime_native_dependency',
          'module_runtime_native_executable',
          'module_runtime_read_only_data',
        ].includes(artifact.artifact_kind)
        && artifact.bound_module_id !== fragment.module_id
      ))) {
      throw new Error('release compiler artifact fragment binding is invalid');
    }
    fragmentArtifacts.push(...fragment.artifacts);
  }
  const artifacts = [...input.artifacts, ...fragmentArtifacts]
    .sort((left, right) => compareAscii(left.artifact_id, right.artifact_id));
  const composed = { ...input, artifacts };
  validateInput(composed);
  validateArtifactSet(artifacts);
  return composed;
}

function validateArtifactSet(artifacts) {
  let previousArtifactId = '';
  for (const artifact of artifacts) {
    validateArtifactInput(artifact, previousArtifactId);
    previousArtifactId = artifact.artifact_id;
  }
}

export function readReleaseCompilerInput(path) {
  const bytes = readStableRegularFile(path, 'release compiler input', MAX_RELEASE_INPUT_BYTES);
  try {
    return JSON.parse(bytes.toString('utf8'));
  } catch {
    throw new Error('release compiler input is not valid JSON');
  }
}

export function readReleaseArtifactFragment(path) {
  const bytes = readStableRegularFile(
    path,
    'release artifact fragment',
    MAX_RELEASE_INPUT_BYTES,
  );
  try {
    return JSON.parse(bytes.toString('utf8'));
  } catch {
    throw new Error('release artifact fragment is not valid JSON');
  }
}

export function loadReleaseSigningKey(path) {
  return loadSigningKey(path);
}

export function writeReleaseArtifact(path, bytes) {
  assertReleaseArtifactAbsent(path);
  writeFileSync(path, bytes, { mode: 0o644, flag: 'wx' });
}

export async function materializeReleaseDistribution(artifacts, destination) {
  if (!isAbsolute(destination)) throw new Error('release distribution output path must be absolute');
  const parent = dirname(destination);
  assertNonSymlinkDirectory(parent, 'release distribution output directory');
  assertReleaseDistributionAbsent(destination);
  const staging = mkdtempSync(join(parent, '.makosh-distribution-'));
  try {
    for (const artifact of artifacts) {
      await copyVerifiedReleaseFile(staging, artifact.relativePath, artifact.sourcePath, artifact);
      if (artifact.descriptor) {
        await copyVerifiedReleaseFile(
          staging,
          artifact.descriptor.relativePath,
          artifact.descriptor.sourcePath,
          artifact.descriptor,
        );
      }
      if (artifact.settingsSchema) {
        await copyVerifiedReleaseFile(
          staging,
          artifact.settingsSchema.relativePath,
          artifact.settingsSchema.sourcePath,
          artifact.settingsSchema,
        );
      }
    }
    renameSync(staging, destination);
  } catch (error) {
    rmSync(staging, { recursive: true, force: true });
    throw error;
  }
}

async function copyVerifiedReleaseFile(root, relativePath, sourcePath, expected) {
  const destination = join(root, ...relativePath.split('/'));
  mkdirSync(dirname(destination), { recursive: true, mode: 0o700 });
  copyFileSync(sourcePath, destination, 0);
  const copied = await digestSource(destination, 'materialized release artifact', MAX_ARTIFACT_BYTES);
  if (copied.sizeBytes !== expected.sizeBytes || !copied.sha256.equals(expected.sha256)) {
    throw new Error('release artifact changed while distribution was materialized');
  }
}

export function assertReleaseArtifactAbsent(path) {
  if (!isAbsolute(path)) throw new Error('release output path must be absolute');
  assertNonSymlinkDirectory(dirname(path), 'release output directory');
  try {
    lstatSync(path);
  } catch (error) {
    if (error?.code === 'ENOENT') return;
    throw error;
  }
  throw new Error('release output path already exists');
}

export function assertReleaseDistributionAbsent(path) {
  if (!isAbsolute(path)) throw new Error('release distribution output path must be absolute');
  assertNonSymlinkDirectory(dirname(path), 'release distribution output directory');
  try {
    lstatSync(path);
  } catch (error) {
    if (error?.code === 'ENOENT') return;
    throw error;
  }
  throw new Error('release distribution output path already exists');
}

export function generateReleaseSigningKey(path) {
  if (!isAbsolute(path)) throw new Error('release signing key output path must be absolute');
  assertNonSymlinkDirectory(dirname(path), 'release signing key output directory');
  const keyPair = generateKeyPairSync('ec', {
    namedCurve: 'prime256v1',
    privateKeyEncoding: { type: 'pkcs8', format: 'pem' },
  });
  writeFileSync(path, keyPair.privateKey, { mode: 0o600, flag: 'wx' });
}

export function removeReleaseArtifact(path) {
  try {
    unlinkSync(path);
  } catch {
    // A failed release write has no reusable output to clean up.
  }
}

export function removeReleaseDistribution(path) {
  try {
    rmSync(path, { recursive: true, force: true });
  } catch {
    // A failed release materialization has no reusable output to keep.
  }
}
