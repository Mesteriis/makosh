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
  registerModule,
  runtimeKeyPair,
  startKernel,
  stopKernel,
} from '../support/production-kernel-ipc.mjs';
import {
  decode,
  field,
  request,
  stringValue,
  text,
  uint64,
} from '../support/protobuf-ipc.mjs';

const ownerId = 'owner_storage';
const capabilityId = 'storage.access';
const runtimeId = 'runtime_storage';

test('owner IPC issues and rotates one external Storage binding behind current attestation', async () => {
  const root = await mkdtemp(join(tmpdir(), 'makosh-external-storage-binding-'));
  const dataDir = join(root, 'data');
  let server;
  try {
    const enrolled = kernelCommand(
      dataDir, 'initial-owner-enroll', '--owner-id', ownerId, '--device-id', 'device_storage',
    );
    assert.equal(enrolled.status, 0, enrolled.stderr);
    const started = await startKernel(dataDir);
    server = started.server;
    const registrationId = await registerModule(
      started.socketPaths.registration, storageDescriptor(),
    );
    const ownerSessionId = await openOwnerSession(started.socketPaths.owner, dataDir);
    await request(
      started.socketPaths.owner,
      ownerApprove(registrationId, ownerSessionId, [capabilityId]),
    );
    const { pair, publicKey } = runtimeKeyPair();
    await request(
      started.socketPaths.owner,
      ownerBindExternal(registrationId, publicKey, ownerSessionId),
    );
    const bundle = storageBundle();
    await configureTopology(started.socketPaths.owner, ownerSessionId);
    await admitBundle(started.socketPaths.owner, ownerSessionId, bundle);
    await openRuntimeSession({
      socketPath: started.socketPaths.runtime,
      registrationId,
      runtimeId,
      generation: 1,
      distributionSha256: createHash('sha256').update('external storage runtime').digest(),
      pair,
    });

    await issueBinding(started.socketPaths.owner, ownerSessionId, registrationId, 1, 1, bundle.digest);
    await reserveRevocation(started.socketPaths.owner, ownerSessionId, registrationId);
    const rotated = await issueBinding(
      started.socketPaths.owner, ownerSessionId, registrationId, 2, 2, bundle.digest,
    );
    assert.equal(rotated.get(3), 2, 'the durable binding revision advances after reservation');
    assert.equal(rotated.get(4), 1, 'the binding remains on the current topology revision');
    assert.equal(rotated.get(5), 1, 'the binding remains on the current storage generation');
  } finally {
    if (server) await stopKernel(server);
    await rm(root, { recursive: true, force: true });
  }
});

function storageDescriptor() {
  const storageRequest = Buffer.concat([
    text(1, ownerId), uint64(2, 4), uint64(3, 5_000),
  ]);
  const request = field(1, storageRequest);
  const capability = Buffer.concat([
    text(1, capabilityId), uint64(2, 1), uint64(3, 1), field(5, request),
  ]);
  return Buffer.concat([
    uint64(1, 1), uint64(2, 1), text(3, 'module_storage'), text(4, ownerId),
    uint64(5, 2), text(6, '1'), text(7, 'build_storage'), field(9, capability),
  ]);
}

function storageBundle() {
  const sql = Buffer.from('CREATE TABLE makosh_data.owner_storage_probe (probe_id uuid);', 'utf8');
  const step = Buffer.concat([
    uint64(1, 1), text(2, 'create_probe'), field(3, sql),
    field(4, createHash('sha256').update(sql).digest()),
  ]);
  const bytes = Buffer.concat([
    uint64(1, 1), uint64(2, 1), text(3, 'owner_storage_bundle'), text(4, ownerId),
    field(5, step),
  ]);
  return { bytes, digest: createHash('sha256').update(bytes).digest() };
}

async function configureTopology(socketPath, ownerSessionId) {
  const response = await request(socketPath, field(17, Buffer.concat([
    text(1, ownerSessionId), uint64(2, 1), text(3, 'storage_main'), text(4, 'makosh'),
    uint64(5, 1), field(6, Buffer.alloc(32, 3)), field(7, Buffer.alloc(32, 4)),
    text(8, '127.0.0.1'), uint64(9, 5_432), text(10, '127.0.0.1'), uint64(11, 6_432),
  ])));
  assert.equal(decode(decode(response).get(18)).get(1), 1);
}

async function admitBundle(socketPath, ownerSessionId, bundle) {
  const response = await request(socketPath, field(19, Buffer.concat([
    text(1, ownerSessionId), field(2, bundle.bytes),
  ])));
  const admitted = decode(decode(response).get(20));
  assert.equal(stringValue(admitted, 1), ownerId);
  assert.equal(admitted.get(2), 1);
  assert.deepEqual(admitted.get(3), bundle.digest);
}

async function issueBinding(socketPath, ownerSessionId, registrationId, roleEpoch, credentialRevision, digest) {
  const response = await request(socketPath, field(21, Buffer.concat([
    text(1, ownerSessionId), text(2, registrationId), text(3, capabilityId), text(4, runtimeId),
    uint64(5, 1), uint64(6, roleEpoch), uint64(7, credentialRevision), uint64(8, 1),
    field(9, digest),
  ])));
  const binding = decode(response).get(22);
  assert.ok(binding, 'owner control returns a durable external binding');
  return decode(binding);
}

async function reserveRevocation(socketPath, ownerSessionId, registrationId) {
  const response = await request(socketPath, field(22, Buffer.concat([
    text(1, ownerSessionId), text(2, registrationId), text(3, capabilityId), uint64(4, 1),
    text(5, runtimeId), uint64(6, 1),
  ])));
  assert.equal(decode(decode(response).get(23)).get(3), 1);
}
