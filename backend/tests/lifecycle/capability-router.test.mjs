import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';
import {
  kernelCommand,
  openOwnerSession,
  openRuntimeSession,
  ownerApprove,
  ownerBindExternal,
  ownerTransition,
  registerModule,
  runtimeAuthorize,
  runtimeKeyPair,
  startKernel,
  stopKernel,
} from './support/production-kernel-ipc.mjs';
import {
  decode,
  errorCode,
  field,
  request,
  text,
  uint64,
} from './support/protobuf-ipc.mjs';

function capability(capabilityId) {
  return Buffer.concat([text(1, capabilityId), uint64(2, 1), uint64(3, 1)]);
}

function moduleDescriptor() {
  return Buffer.concat([
    uint64(1, 1), uint64(2, 1), text(3, 'module-router'),
    text(4, 'owner-router'), uint64(5, 2), text(6, '1'), text(7, 'build-1'),
    field(9, capability('capability.read')),
    field(9, capability('kernel.control')),
  ]);
}

function runtimeOptions(socketPath, registrationId, pair, digest, generation) {
  return {
    socketPath,
    registrationId,
    runtimeId: 'runtime-router',
    generation,
    distributionSha256: digest,
    pair,
  };
}

async function approveAndBind(socketPaths, dataDir, registrationId, publicKey) {
  const sessionId = await openOwnerSession(socketPaths.owner, dataDir);
  const reserved = await request(
    socketPaths.owner,
    ownerApprove(registrationId, sessionId, ['kernel.control']),
  );
  assert.equal(errorCode(reserved), 'operation_denied');
  const approved = decode(await request(
    socketPaths.owner,
    ownerApprove(registrationId, sessionId),
  )).get(2);
  assert.equal(decode(approved).get(2), 2);
  const bindingSession = await openOwnerSession(socketPaths.owner, dataDir);
  const bound = decode(await request(
    socketPaths.owner,
    ownerBindExternal(registrationId, publicKey, bindingSession),
  )).get(7);
  assert.equal(decode(bound).get(2), 3);
}

async function suspendAndReapprove(socketPaths, dataDir, registrationId) {
  const suspendSession = await openOwnerSession(socketPaths.owner, dataDir);
  await request(
    socketPaths.owner,
    ownerTransition(registrationId, 'suspended', suspendSession),
  );
  const reapprovalSession = await openOwnerSession(socketPaths.owner, dataDir);
  const reapproved = decode(await request(
    socketPaths.owner,
    ownerApprove(registrationId, reapprovalSession),
  )).get(2);
  assert.equal(decode(reapproved).get(2), 5);
}

test('capability router requires the current approved external runtime generation', async () => {
  const root = await mkdtemp(join(tmpdir(), 'makosh-kernel-capability-router-'));
  const dataDir = join(root, 'data');
  let server;
  try {
    const enrolled = kernelCommand(
      dataDir, 'initial-owner-enroll', '--owner-id', 'owner-router',
      '--device-id', 'device-router',
    );
    assert.equal(enrolled.status, 0, enrolled.stderr);
    const started = await startKernel(dataDir);
    server = started.server;
    const { socketPaths } = started;
    const registrationId = await registerModule(socketPaths.registration, moduleDescriptor());
    const { pair, publicKey } = runtimeKeyPair();
    await approveAndBind(socketPaths, dataDir, registrationId, publicKey);
    const digest = createHash('sha256').update('external runtime artifact').digest();
    const first = runtimeOptions(socketPaths.runtime, registrationId, pair, digest, 1);
    const sessionId = await openRuntimeSession(first);
    const current = decode(await request(
      socketPaths.runtime, runtimeAuthorize(sessionId, 'capability.read'),
    )).get(3);
    assert.equal(decode(current).get(1), 3);
    assert.equal(errorCode(await request(
      socketPaths.runtime, runtimeAuthorize(sessionId, 'kernel.control'),
    )), 'runtime_session_denied');
    await suspendAndReapprove(socketPaths, dataDir, registrationId);
    assert.equal(errorCode(await request(
      socketPaths.runtime, runtimeAuthorize(sessionId, 'capability.read'),
    )), 'runtime_session_stale');
    await assert.rejects(openRuntimeSession(first));
    const renewed = await openRuntimeSession(
      runtimeOptions(socketPaths.runtime, registrationId, pair, digest, 2),
    );
    const authorized = decode(await request(
      socketPaths.runtime, runtimeAuthorize(renewed, 'capability.read'),
    )).get(3);
    assert.equal(decode(authorized).get(1), 5);
  } finally {
    if (server) await stopKernel(server);
    await rm(root, { recursive: true, force: true });
  }
});
