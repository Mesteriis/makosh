import assert from 'node:assert/strict';
import {
  createECDH,
  createPrivateKey,
  generateKeyPairSync,
  sign,
} from 'node:crypto';
import { readFile } from 'node:fs/promises';
import { spawn, spawnSync } from 'node:child_process';
import { join } from 'node:path';
import {
  decode,
  errorCode,
  field,
  request,
  stringValue,
  text,
  uint64,
} from './protobuf-ipc.mjs';

const backendRoot = new URL('../../../', import.meta.url);
const ownerProofDomain = Buffer.from('makosh.owner-control-session.v1\0', 'ascii');
const runtimeProofDomain = Buffer.from('makosh.external-runtime-session.v1\0', 'ascii');
const cargoEnvironment = { ...process.env, RUSTC_WRAPPER: '' };

export function kernelCommand(dataDir, ...args) {
  return spawnSync(
    'cargo',
    ['run', '-q', '-p', 'makosh-kernel', '--', '--data-dir', dataDir, ...args],
    { cwd: backendRoot, encoding: 'utf8', env: cargoEnvironment },
  );
}

export async function startKernel(dataDir) {
  const server = spawn(
    'cargo',
    ['run', '-q', '-p', 'makosh-kernel', '--', '--data-dir', dataDir, 'serve'],
    { cwd: backendRoot, stdio: ['ignore', 'pipe', 'pipe'], env: cargoEnvironment },
  );
  return { server, socketPaths: await collectSocketPaths(server) };
}

export async function stopKernel(server) {
  if (server.exitCode !== null || server.signalCode !== null) return;
  server.kill('SIGTERM');
  await new Promise((resolve) => server.once('exit', resolve));
}

async function collectSocketPaths(server) {
  return new Promise((resolve, reject) => {
    const timeout = setTimeout(() => reject(new Error('Kernel IPC sockets did not start')), 15_000);
    let stdout = '';
    let stderr = '';
    server.stderr.on('data', (chunk) => { stderr += chunk.toString('utf8'); });
    server.stdout.on('data', (chunk) => {
      stdout += chunk.toString('utf8');
      const registration = stdout.match(/module_registration_socket=(.+)\n/)?.[1];
      const owner = stdout.match(/owner_control_socket=(.+)\n/)?.[1];
      const runtime = stdout.match(/external_runtime_session_socket=(.+)\n/)?.[1];
      if (!registration || !owner || !runtime) return;
      clearTimeout(timeout);
      resolve({ registration, owner, runtime });
    });
    server.once('exit', (code) => {
      clearTimeout(timeout);
      reject(new Error(`Kernel exited before IPC readiness: ${code}: ${stderr}`));
    });
  });
}

export async function registerModule(socketPath, descriptor) {
  const begun = decode(await request(socketPath, field(2, Buffer.alloc(0)))).get(2);
  const sessionId = stringValue(decode(begun), 1);
  assert.match(sessionId, /^[0-9a-f]{32}$/);
  const described = decode(await request(
    socketPath,
    field(3, Buffer.concat([text(1, sessionId), field(2, descriptor)])),
  )).get(3);
  const registrationId = stringValue(decode(described), 1);
  assert.match(registrationId, /^[0-9a-f]{32}$/);
  return registrationId;
}

export function ownerBegin() {
  return field(4, Buffer.alloc(0));
}

export function ownerComplete(challengeId, signature) {
  return field(5, Buffer.concat([text(1, challengeId), field(2, signature)]));
}

export async function openOwnerSession(socketPath, dataDir) {
  const challengeFields = decode(decode(await request(socketPath, ownerBegin())).get(4));
  const challenge = ownerChallenge(challengeFields);
  const rawKey = await readFile(join(dataDir, 'device-es256.key'));
  const signature = sign('sha256', ownerProof(challenge), {
    key: ownerSigningKey(rawKey),
    dsaEncoding: 'ieee-p1363',
  });
  const completed = decode(await request(
    socketPath,
    ownerComplete(challenge.challengeId, signature),
  )).get(5);
  const sessionId = stringValue(decode(completed), 1);
  assert.match(sessionId, /^[0-9a-f]{64}$/);
  return sessionId;
}

function ownerChallenge(fields) {
  return {
    challengeId: stringValue(fields, 1),
    bytes: fields.get(2),
    kernelInstanceId: stringValue(fields, 3),
    ownerId: stringValue(fields, 4),
    deviceId: stringValue(fields, 5),
    generation: fields.get(6),
  };
}

function ownerProof(challenge) {
  const names = [challenge.kernelInstanceId, challenge.ownerId, challenge.deviceId]
    .map(lengthPrefixedText);
  const generation = Buffer.alloc(8);
  generation.writeBigUInt64BE(BigInt(challenge.generation));
  return Buffer.concat([ownerProofDomain, ...names, generation, challenge.bytes]);
}

function ownerSigningKey(rawKey) {
  const ecdh = createECDH('prime256v1');
  ecdh.setPrivateKey(rawKey);
  const publicKey = ecdh.getPublicKey(undefined, 'uncompressed');
  return createPrivateKey({
    key: {
      kty: 'EC',
      crv: 'P-256',
      d: rawKey.toString('base64url'),
      x: publicKey.subarray(1, 33).toString('base64url'),
      y: publicKey.subarray(33).toString('base64url'),
    },
    format: 'jwk',
  });
}

export function ownerApprove(registrationId, sessionId, capabilities = ['capability.read']) {
  return field(2, Buffer.concat([
    text(1, registrationId),
    ...capabilities.map((capability) => text(2, capability)),
    text(3, sessionId),
  ]));
}

export function ownerTransition(registrationId, state, sessionId) {
  return field(3, Buffer.concat([
    text(1, registrationId), text(2, state), text(3, sessionId),
  ]));
}

export function ownerStatus(registrationId) {
  return field(1, text(1, registrationId));
}

export function ownerBindExternal(registrationId, publicKey, sessionId) {
  return field(7, Buffer.concat([
    text(1, registrationId), field(2, publicKey), text(3, sessionId),
  ]));
}

export function ownerUpdateSettings(registrationId, expectedRevision, snapshot, sessionId) {
  return field(6, Buffer.concat([
    text(1, registrationId),
    uint64(2, expectedRevision),
    field(3, snapshot),
    text(4, sessionId),
  ]));
}

export function ownerBeginBrowserPairing(sessionId) {
  return field(35, text(1, sessionId));
}

export function ownerTelemetryDiagnostics(sessionId) {
  return field(14, text(1, sessionId));
}

export function runtimeKeyPair() {
  const pair = generateKeyPairSync('ec', { namedCurve: 'prime256v1' });
  const publicKey = pair.publicKey.export({ type: 'spki', format: 'der' }).subarray(-65);
  assert.equal(publicKey[0], 0x04);
  return { pair, publicKey };
}

export function runtimeBegin(registrationId, runtimeId, generation, distributionSha256) {
  return field(1, Buffer.concat([
    text(1, registrationId),
    text(2, runtimeId),
    uint64(3, generation),
    field(4, distributionSha256),
  ]));
}

export function runtimeComplete(challengeId, signature) {
  return field(2, Buffer.concat([text(1, challengeId), field(2, signature)]));
}

export async function openRuntimeSession(options) {
  const { socketPath, registrationId, runtimeId, generation, distributionSha256, pair } = options;
  const beginResponse = await request(
    socketPath,
    runtimeBegin(registrationId, runtimeId, generation, distributionSha256),
  );
  const begun = decode(beginResponse).get(1);
  if (!begun) throw new Error(`runtime begin failed: ${errorCode(beginResponse)}`);
  const challenge = runtimeChallenge(
    decode(begun), registrationId, runtimeId, generation, distributionSha256,
  );
  const signature = sign('sha256', runtimeProof(challenge), {
    key: pair.privateKey,
    dsaEncoding: 'ieee-p1363',
  });
  const completeResponse = await request(
    socketPath,
    runtimeComplete(challenge.challengeId, signature),
  );
  const completed = decode(completeResponse).get(2);
  if (!completed) throw new Error(`runtime proof failed: ${errorCode(completeResponse)}`);
  const sessionId = stringValue(decode(completed), 1);
  assert.match(sessionId, /^[0-9a-f]{64}$/);
  return sessionId;
}

function runtimeChallenge(fields, registrationId, runtimeId, generation, distributionSha256) {
  return {
    challengeId: stringValue(fields, 1),
    bytes: fields.get(2),
    kernelInstanceId: stringValue(fields, 3),
    grantEpoch: fields.get(4),
    registrationId,
    runtimeId,
    runtimeGeneration: generation,
    distributionSha256,
  };
}

export function runtimeProof(challenge) {
  const names = [challenge.kernelInstanceId, challenge.registrationId, challenge.runtimeId]
    .map(lengthPrefixedText);
  const generation = Buffer.alloc(8);
  generation.writeBigUInt64BE(BigInt(challenge.runtimeGeneration));
  const epoch = Buffer.alloc(8);
  epoch.writeBigUInt64BE(BigInt(challenge.grantEpoch));
  return Buffer.concat([
    runtimeProofDomain,
    ...names,
    generation,
    epoch,
    challenge.distributionSha256,
    challenge.bytes,
  ]);
}

function lengthPrefixedText(value) {
  const bytes = Buffer.from(value, 'ascii');
  const length = Buffer.alloc(2);
  length.writeUInt16BE(bytes.length);
  return Buffer.concat([length, bytes]);
}

export function runtimeAuthorize(sessionId, capabilityId) {
  return field(3, Buffer.concat([text(1, sessionId), text(2, capabilityId)]));
}

export function runtimeSubmitSchema(sessionId, descriptor, schema) {
  return field(4, Buffer.concat([
    text(1, sessionId), field(2, descriptor), field(3, schema),
  ]));
}

export function runtimeAcknowledge(sessionId, revision, acknowledgement, reasonCode = '') {
  return field(5, Buffer.concat([
    text(1, sessionId),
    uint64(2, revision),
    text(3, acknowledgement),
    ...(reasonCode ? [text(4, reasonCode)] : []),
  ]));
}
