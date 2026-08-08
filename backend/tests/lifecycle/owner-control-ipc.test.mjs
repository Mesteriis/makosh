import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdtemp, rm, stat } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';
import {
  kernelCommand,
  openOwnerSession,
  openRuntimeSession,
  ownerApprove,
  ownerBeginBrowserPairing,
  ownerBegin,
  ownerBindExternal,
  ownerComplete,
  ownerStatus,
  ownerTelemetryDiagnostics,
  ownerTransition,
  ownerUpdateSettings,
  registerModule,
  runtimeKeyPair,
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
    uint64(1, 1), uint64(2, 1), text(3, 'module-ipc'), text(4, 'owner-ipc'),
    uint64(5, 2), text(6, '1'), text(7, 'build-1'),
    field(9, capability), field(10, schemaRef),
  ]);
}

function settingsSnapshot(registrationId) {
  const entry = Buffer.concat([
    text(1, 'poll_interval'), field(2, Buffer.from([0x30, 0xe8, 0x07])),
  ]);
  return Buffer.concat([text(1, registrationId), uint64(2, 1), field(3, entry)]);
}

function bindBundledManagedRelease(registrationId, artifactId, sessionId) {
  return field(8, Buffer.concat([
    text(1, registrationId), text(2, artifactId), text(3, sessionId),
  ]));
}

function startBundledManagedRuntime(registrationId, sessionId) {
  return field(9, Buffer.concat([text(1, registrationId), text(2, sessionId)]));
}

function bindPlatformVaultRelease(sessionId) {
  return field(10, text(1, sessionId));
}

function startPlatformVaultRuntime(sessionId) {
  return field(11, text(1, sessionId));
}

function bindPlatformBlobRelease(sessionId) {
  return field(23, text(1, sessionId));
}

function startPlatformBlobRuntime(sessionId) {
  return field(24, text(1, sessionId));
}

function configurePlatformEventHubTopology(sessionId) {
  const budgets = [1, 2, 3, 4, 5].map((kind) => field(2, Buffer.concat([
    uint64(1, kind), uint64(2, 1_048_576), uint64(3, 3_600_000), uint64(4, 1),
  ])));
  return field(28, Buffer.concat([
    text(1, sessionId), ...budgets, text(3, 'nats://127.0.0.1:4222'),
    text(4, 'event_hub'), uint64(5, 1),
  ]));
}

function configurePlatformStorageTopology(sessionId, generation, instanceId) {
  return field(17, Buffer.concat([
    text(1, sessionId), uint64(2, generation), text(3, instanceId),
    text(4, 'makosh'), uint64(5, 1), field(6, Buffer.alloc(32, 3)),
    field(7, Buffer.alloc(32, 4)),
    text(8, '127.0.0.1'), uint64(9, 5432), text(10, '127.0.0.1'), uint64(11, 6432),
    text(12, 'postgres'), uint64(13, 5432),
  ]));
}

function issueManagedStorageBinding(sessionId, registrationId) {
  return field(18, Buffer.concat([
    text(1, sessionId), text(2, registrationId), text(3, 'storage.access'),
    text(4, 'runtime-managed'), uint64(5, 1), uint64(6, 1), uint64(7, 1),
    uint64(8, 1), field(9, Buffer.alloc(32, 7)),
  ]));
}

function issueExternalStorageBinding(sessionId, registrationId) {
  return field(21, Buffer.concat([
    text(1, sessionId), text(2, registrationId), text(3, 'storage.access'),
    text(4, 'runtime-owner-ipc'), uint64(5, 1), uint64(6, 1), uint64(7, 1),
    uint64(8, 1), field(9, Buffer.alloc(32, 7)),
  ]));
}

function admitStorageBundle(sessionId) {
  return field(19, Buffer.concat([text(1, sessionId), field(2, Buffer.from([0x08, 0x01]))]));
}

function beginManagedStorageBindingRevocation(sessionId, registrationId) {
  return field(20, Buffer.concat([
    text(1, sessionId), text(2, registrationId), text(3, 'storage.access'), uint64(4, 1),
  ]));
}

function restartSchedulerRuntime(sessionId, registrationId) {
  return field(34, Buffer.concat([
    text(1, registrationId), text(2, 'storage.access'), text(3, sessionId),
    uint64(4, 1), uint64(5, 1), uint64(6, 1), field(7, Buffer.alloc(32, 7)),
  ]));
}

async function proveOwnerSessionEnforcement(socketPath, registrationId) {
  assert.equal(errorCode(await request(
    socketPath, ownerApprove(registrationId, 'missing'),
  )), 'operation_denied');
  const begun = decode(await request(socketPath, ownerBegin())).get(4);
  const challengeId = stringValue(decode(begun), 1);
  assert.match(challengeId, /^[0-9a-f]{64}$/);
  assert.equal(errorCode(await request(
    socketPath, ownerComplete(challengeId, Buffer.alloc(64)),
  )), 'operation_denied');
  assert.equal(errorCode(await request(
    socketPath, ownerComplete(challengeId, Buffer.alloc(64)),
  )), 'operation_denied');
  assert.equal(errorCode(await request(
    socketPath, bindPlatformVaultRelease('missing'),
  )), 'operation_denied');
  assert.equal(errorCode(await request(
    socketPath, startPlatformVaultRuntime('missing'),
  )), 'operation_denied');
  assert.equal(errorCode(await request(
    socketPath, bindPlatformBlobRelease('missing'),
  )), 'operation_denied');
  assert.equal(errorCode(await request(
    socketPath, startPlatformBlobRuntime('missing'),
  )), 'operation_denied');
  assert.equal(errorCode(await request(
    socketPath, ownerTelemetryDiagnostics('missing'),
  )), 'operation_denied');
  assert.equal(errorCode(await request(
    socketPath, configurePlatformStorageTopology('missing', 1, 'storage-1'),
  )), 'operation_denied');
  assert.equal(errorCode(await request(
    socketPath, issueManagedStorageBinding('missing', registrationId),
  )), 'operation_denied');
  assert.equal(errorCode(await request(
    socketPath, issueExternalStorageBinding('missing', registrationId),
  )), 'operation_denied');
  assert.equal(errorCode(await request(
    socketPath, admitStorageBundle('missing'),
  )), 'operation_denied');
  assert.equal(errorCode(await request(
    socketPath, beginManagedStorageBindingRevocation('missing', registrationId),
  )), 'operation_denied');
  assert.equal(errorCode(await request(
    socketPath, restartSchedulerRuntime('missing', registrationId),
  )), 'operation_denied');
  assert.equal(errorCode(await request(
    socketPath, ownerBeginBrowserPairing('missing'),
  )), 'operation_denied');
  assert.equal(errorCode(await request(
    socketPath, configurePlatformEventHubTopology('missing'),
  )), 'operation_denied');
}

async function exerciseOwnerMutations(context, dataDir, descriptor, schema) {
  const { socketPaths, registrationId } = context;
  const ownerSessionId = await openOwnerSession(socketPaths.owner, dataDir);
  const pairSession = runtimeKeyPair();
  await assertModuleBindRestrictions(
    socketPaths.owner,
    registrationId,
    ownerSessionId,
    dataDir,
    pairSession,
  );
  await admitSchema(context, pairSession, descriptor, schema);
  await assertTopologyAndSchedulerGuards(socketPaths.owner, registrationId, ownerSessionId);
  return ownerSessionId;
}

async function assertModuleBindRestrictions(ownerSocket, registrationId, ownerSessionId, dataDir, pairSession) {
  const { publicKey } = pairSession;
  const approval = await request(ownerSocket, ownerApprove(registrationId, ownerSessionId));
  assert.ok(
    decode(approval).get(2),
    `owner approval failed: ${errorCode(approval)}`,
  );
  assert.equal(errorCode(await request(
    ownerSocket,
    bindBundledManagedRelease(registrationId, 'runtime.mail', ownerSessionId),
  )), 'operation_denied');
  assert.equal(errorCode(await request(
    ownerSocket,
    startBundledManagedRuntime(registrationId, ownerSessionId),
  )), 'operation_denied');
  const bindingSession = await openOwnerSession(ownerSocket, dataDir);
  const binding = await request(
    ownerSocket,
    ownerBindExternal(registrationId, publicKey, bindingSession),
  );
  const bound = decode(binding).get(7);
  assert.ok(bound, `external runtime bind failed: ${errorCode(binding)}`);
  assert.equal(decode(bound).get(2), 3);
  assert.equal(errorCode(await request(
    ownerSocket,
    ownerBindExternal(registrationId, Buffer.alloc(65, 4), 'missing'),
  )), 'operation_denied');
}

async function admitSchema(context, pairSession, descriptor, schema) {
  const { pair } = pairSession;
  const { socketPaths, registrationId } = context;
  if (!pair) {
    return;
  }
  const runtimeSession = await openRuntimeSession({
    socketPath: socketPaths.runtime,
    registrationId,
    runtimeId: 'runtime-owner-ipc',
    generation: 1,
    distributionSha256: createHash('sha256').update('owner IPC runtime').digest(),
    pair,
  });
  const admitted = decode(await request(
    socketPaths.runtime,
    runtimeSubmitSchema(runtimeSession, descriptor, schema),
  )).get(4);
  assert.equal(stringValue(decode(admitted), 1), registrationId);

  const status = await request(socketPaths.owner, ownerStatus(registrationId));
  assert.ok(status.includes(Buffer.from('approved', 'ascii')));
}

async function assertTopologyAndSchedulerGuards(ownerSocket, registrationId, ownerSessionId) {
  const status = await request(ownerSocket, ownerStatus(registrationId));
  assert.ok(status.includes(Buffer.from('approved', 'ascii')));
  const topologyResponse = await request(
    ownerSocket,
    configurePlatformStorageTopology(ownerSessionId, 1, 'storage-1'),
  );
  const topology = decode(topologyResponse).get(18);
  assert.ok(topology, `storage topology configuration failed: ${errorCode(topologyResponse)}`);
  assert.equal(decode(topology).get(1), 1);
  assert.equal(decode(topology).get(2), 1);
  const eventHubResponse = await request(
    ownerSocket, configurePlatformEventHubTopology(ownerSessionId),
  );
  const eventHub = decode(eventHubResponse).get(29);
  assert.ok(eventHub, `Event Hub topology configuration failed: ${errorCode(eventHubResponse)}`);
  assert.equal(decode(eventHub).get(1), 1);
  assert.equal(decode(eventHub).get(2), 5);
  assert.equal(errorCode(await request(
    ownerSocket,
    restartSchedulerRuntime(ownerSessionId, registrationId),
  )), 'operation_denied');
}

async function exerciseSettingsMutation(context, dataDir, staleSessionId) {
  const { socketPaths, registrationId } = context;
  const settingsSession = await openOwnerSession(socketPaths.owner, dataDir);
  const settings = decode(await request(
    socketPaths.owner,
    ownerUpdateSettings(
      registrationId, 0, settingsSnapshot(registrationId), settingsSession,
    ),
  )).get(6);
  assert.equal(stringValue(decode(settings), 3), 'pending_validation');
  assert.equal(errorCode(await request(
    socketPaths.owner,
    ownerTransition(registrationId, 'suspended', staleSessionId),
  )), 'operation_denied');
}

test('owner control IPC requires a current owner-device proof for mutations', async () => {
  const root = await mkdtemp(join(tmpdir(), 'makosh-kernel-owner-control-'));
  const dataDir = join(root, 'data');
  let server;
  let ownerSocket;
  try {
    const schema = schemaArtifact();
    const descriptor = moduleDescriptor(schema);
    const enrolled = kernelCommand(
      dataDir, 'initial-owner-enroll', '--owner-id', 'owner-ipc',
      '--device-id', 'device-ipc',
    );
    assert.equal(enrolled.status, 0, enrolled.stderr);
    const started = await startKernel(dataDir);
    server = started.server;
    ownerSocket = started.socketPaths.owner;
    const registrationId = await registerModule(
      started.socketPaths.registration, descriptor,
    );
    assert.equal((await stat(ownerSocket)).mode & 0o777, 0o600);
    await proveOwnerSessionEnforcement(ownerSocket, registrationId);
    const staleSession = await exerciseOwnerMutations(
      { ...started, registrationId }, dataDir, descriptor, schema,
    );
    await stopKernel(server);
    await assert.rejects(stat(ownerSocket));
    const restarted = await startKernel(dataDir);
    server = restarted.server;
    ownerSocket = restarted.socketPaths.owner;
    await exerciseSettingsMutation(
      { ...restarted, registrationId }, dataDir, staleSession,
    );
  } finally {
    if (server) await stopKernel(server);
    if (ownerSocket) await assert.rejects(stat(ownerSocket));
    await rm(root, { recursive: true, force: true });
  }
});
