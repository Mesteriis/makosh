import assert from 'node:assert/strict';
import { createHash, sign } from 'node:crypto';
import { mkdtemp, rm, stat } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';
import {
  kernelCommand,
  openOwnerSession,
  openRuntimeSession,
  ownerApprove,
  ownerBindExternal,
  ownerUpdateSettings,
  registerModule,
  runtimeAcknowledge,
  runtimeAuthorize,
  runtimeBegin,
  runtimeComplete,
  runtimeKeyPair,
  runtimeProof,
  runtimeSubmitSchema,
  startKernel,
  stopKernel,
} from './support/production-kernel-ipc.mjs';
import {
  decode,
  errorCode,
  field,
  request,
  stringValue,
  text,
  uint64,
} from './support/protobuf-ipc.mjs';

function schemaArtifact() {
  const flags = Buffer.from([0x18, 0x06, 0x20, 0x01, 0x28, 0x01, 0x30, 0x01, 0x38, 0x01]);
  const definition = Buffer.concat([
    text(1, 'poll_interval'), text(2, 'capability.read'), flags, text(10, 'Poll interval'),
  ]);
  return Buffer.concat([uint64(1, 1), uint64(2, 1), field(3, definition)]);
}

function moduleDescriptor(schema) {
  const capability = Buffer.concat([
    text(1, 'capability.read'), uint64(2, 1), uint64(3, 1), text(9, 'poll_interval'),
  ]);
  const schemaRef = Buffer.concat([
    uint64(1, 1), uint64(2, 1), uint64(3, schema.length),
    field(4, createHash('sha256').update(schema).digest()),
  ]);
  return Buffer.concat([
    uint64(1, 1), uint64(2, 1), text(3, 'module-runtime-session'),
    text(4, 'owner-runtime-session'), uint64(5, 2), text(6, '1'),
    text(7, 'build-1'), field(9, capability), field(10, schemaRef),
  ]);
}

function settingsSnapshot(registrationId) {
  const entry = Buffer.concat([
    text(1, 'poll_interval'), field(2, Buffer.from([0x30, 0xe8, 0x07])),
  ]);
  return Buffer.concat([text(1, registrationId), uint64(2, 1), field(3, entry)]);
}

function runtimeOptions(socketPath, registrationId, pair, digest) {
  return {
    socketPath,
    registrationId,
    runtimeId: 'runtime-session',
    generation: 1,
    distributionSha256: digest,
    pair,
  };
}

function routeVaultCiphertext(sessionId) {
  return field(6, Buffer.concat([text(1, sessionId), field(2, Buffer.alloc(0))]));
}

async function proveInvalidInputs(socketPath, registrationId, pair, digest) {
  assert.equal(errorCode(await request(
    socketPath, runtimeBegin(registrationId, 'runtime-session', 1, Buffer.alloc(0)),
  )), 'invalid_request');
  const first = decode(await request(
    socketPath, runtimeBegin(registrationId, 'runtime-session', 1, digest),
  )).get(1);
  const challengeId = stringValue(decode(first), 1);
  assert.match(challengeId, /^[0-9a-f]{64}$/);
  assert.equal(errorCode(await request(
    socketPath, runtimeComplete(challengeId, Buffer.alloc(64)),
  )), 'runtime_proof_invalid');
  assert.equal(errorCode(await request(
    socketPath, runtimeComplete(challengeId, Buffer.alloc(64)),
  )), 'runtime_session_unavailable');
  await proveDigestBinding(socketPath, registrationId, pair, digest);
}

async function proveDigestBinding(socketPath, registrationId, pair, digest) {
  const begun = decode(await request(
    socketPath, runtimeBegin(registrationId, 'runtime-session', 1, digest),
  )).get(1);
  const fields = decode(begun);
  const challenge = {
    challengeId: stringValue(fields, 1),
    bytes: fields.get(2),
    kernelInstanceId: stringValue(fields, 3),
    grantEpoch: fields.get(4),
    registrationId,
    runtimeId: 'runtime-session',
    runtimeGeneration: 1,
    distributionSha256: Buffer.alloc(32, 9),
  };
  const signature = sign('sha256', runtimeProof(challenge), {
    key: pair.privateKey,
    dsaEncoding: 'ieee-p1363',
  });
  assert.equal(errorCode(await request(
    socketPath, runtimeComplete(challenge.challengeId, signature),
  )), 'runtime_proof_invalid');
}

async function exerciseAuthorizedRoutes(context) {
  const { socketPaths, registrationId, ownerSessionId, pair, digest, descriptor, schema } = context;
  const options = runtimeOptions(socketPaths.runtime, registrationId, pair, digest);
  const sessionId = await openRuntimeSession(options);
  assert.equal(errorCode(await request(
    socketPaths.runtime, routeVaultCiphertext(sessionId),
  )), 'runtime_session_denied');
  assert.equal(errorCode(await request(
    socketPaths.runtime, runtimeAuthorize(sessionId, 'kernel.control'),
  )), 'runtime_session_denied');
  const authorized = decode(await request(
    socketPaths.runtime, runtimeAuthorize(sessionId, 'capability.read'),
  )).get(3);
  assert.equal(decode(authorized).get(1), 3);
  const admitted = decode(await request(
    socketPaths.runtime, runtimeSubmitSchema(sessionId, descriptor, schema),
  )).get(4);
  assert.equal(stringValue(decode(admitted), 1), registrationId);
  await request(
    socketPaths.owner,
    ownerUpdateSettings(registrationId, 0, settingsSnapshot(registrationId), ownerSessionId),
  );
  const lifecycleSession = await openRuntimeSession(options);
  const acknowledged = decode(await request(
    socketPaths.runtime,
    runtimeAcknowledge(lifecycleSession, 1, 'validation_accepted'),
  )).get(5);
  assert.equal(stringValue(decode(acknowledged), 1), registrationId);
}

test('external runtime session IPC requires a bound key proof and fences stale routes', async () => {
  const root = await mkdtemp(join(tmpdir(), 'makosh-kernel-runtime-session-'));
  const dataDir = join(root, 'data');
  let server;
  try {
    const schema = schemaArtifact();
    const descriptor = moduleDescriptor(schema);
    const digest = createHash('sha256').update('runtime artifact').digest();
    const enrolled = kernelCommand(
      dataDir, 'initial-owner-enroll', '--owner-id', 'owner-runtime-session',
      '--device-id', 'device-runtime-session',
    );
    assert.equal(enrolled.status, 0, enrolled.stderr);
    const started = await startKernel(dataDir);
    server = started.server;
    const { socketPaths } = started;
    const registrationId = await registerModule(socketPaths.registration, descriptor);
    const ownerSessionId = await openOwnerSession(socketPaths.owner, dataDir);
    await request(socketPaths.owner, ownerApprove(registrationId, ownerSessionId));
    const { pair, publicKey } = runtimeKeyPair();
    await request(socketPaths.owner, ownerBindExternal(registrationId, publicKey, ownerSessionId));
    assert.equal((await stat(socketPaths.runtime)).mode & 0o777, 0o600);
    await proveInvalidInputs(socketPaths.runtime, registrationId, pair, digest);
    await exerciseAuthorizedRoutes({
      socketPaths, registrationId, ownerSessionId, pair, digest, descriptor, schema,
    });
  } finally {
    if (server) await stopKernel(server);
    await rm(root, { recursive: true, force: true });
  }
});
