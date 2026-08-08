import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdtemp, rm } from 'node:fs/promises';
import { spawnSync } from 'node:child_process';
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
  varint,
} from './support/protobuf-ipc.mjs';

function schemaArtifact(displayName = 'Poll interval') {
  const flags = Buffer.from([0x18, 0x06, 0x20, 0x01, 0x28, 0x01, 0x30, 0x01, 0x38, 0x01]);
  const definition = Buffer.concat([
    text(1, 'poll_interval'), text(2, 'capability.read'), flags, text(10, displayName),
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
    uint64(1, 1), uint64(2, 1), text(3, 'module-settings'), text(4, 'owner-settings'),
    uint64(5, 2), text(6, '1'), text(7, 'build-settings'),
    field(9, capability), field(10, schemaRef),
  ]);
}

function settingsSnapshot(registrationId, revision = 1) {
  const value = Buffer.concat([Buffer.from([0x30]), varint(1000)]);
  const entry = Buffer.concat([text(1, 'poll_interval'), field(2, value)]);
  return Buffer.concat([
    text(1, registrationId), uint64(2, revision), field(3, entry),
  ]);
}

function sqlite(dataDir, statement) {
  const result = spawnSync(
    'sqlite3', [join(dataDir, 'kernel-control-store.sqlite'), statement],
    { encoding: 'utf8' },
  );
  assert.equal(result.status, 0, result.stderr);
  return result.stdout;
}

async function prepareRuntime(dataDir, schema, descriptor) {
  const started = await startKernel(dataDir);
  const registrationId = await registerModule(started.socketPaths.registration, descriptor);
  const ownerSessionId = await openOwnerSession(started.socketPaths.owner, dataDir);
  await request(started.socketPaths.owner, ownerApprove(registrationId, ownerSessionId));
  const { pair, publicKey } = runtimeKeyPair();
  const bindingSession = await openOwnerSession(started.socketPaths.owner, dataDir);
  await request(
    started.socketPaths.owner,
    ownerBindExternal(registrationId, publicKey, bindingSession),
  );
  const runtimeSessionId = await openRuntimeSession({
    socketPath: started.socketPaths.runtime,
    registrationId,
    runtimeId: 'runtime-settings',
    generation: 1,
    distributionSha256: createHash('sha256').update('settings runtime').digest(),
    pair,
  });
  const admitted = decode(await request(
    started.socketPaths.runtime,
    runtimeSubmitSchema(runtimeSessionId, descriptor, schema),
  )).get(4);
  assert.equal(stringValue(decode(admitted), 1), registrationId);
  return { ...started, registrationId, runtimeSessionId };
}

async function exerciseAcceptedLifecycle(context, dataDir, schemaLength) {
  const { socketPaths, registrationId, runtimeSessionId } = context;
  assert.equal(sqlite(
    dataDir, 'SELECT length(schema_bytes) FROM makosh_kernel_settings_schema_artifact;',
  ), `${schemaLength}\n`);
  const ownerSessionId = await openOwnerSession(socketPaths.owner, dataDir);
  const snapshot = settingsSnapshot(registrationId);
  const updated = decode(await request(
    socketPaths.owner,
    ownerUpdateSettings(registrationId, 0, snapshot, ownerSessionId),
  )).get(6);
  assert.equal(stringValue(decode(updated), 3), 'pending_validation');
  assert.equal(errorCode(await request(
    socketPaths.owner,
    ownerUpdateSettings(registrationId, 0, snapshot, ownerSessionId),
  )), 'operation_denied');
  for (const acknowledgement of ['validation_accepted', 'apply_started', 'runtime_applied']) {
    await request(
      socketPaths.runtime,
      runtimeAcknowledge(runtimeSessionId, 1, acknowledgement),
    );
  }
  assert.equal(sqlite(
    dataDir,
    'SELECT desired_revision || ":" || effective_revision || ":" || apply_state FROM makosh_kernel_settings_schema_binding;',
  ), '1:1:current\n');
}

async function exerciseRejectedLifecycle(context, dataDir) {
  const { socketPaths, registrationId, runtimeSessionId } = context;
  const ownerSessionId = await openOwnerSession(socketPaths.owner, dataDir);
  await request(
    socketPaths.owner,
    ownerUpdateSettings(registrationId, 1, settingsSnapshot(registrationId, 2), ownerSessionId),
  );
  await request(
    socketPaths.runtime,
    runtimeAcknowledge(
      runtimeSessionId, 2, 'validation_rejected', 'validation.invalid_interval',
    ),
  );
  assert.equal(sqlite(
    dataDir,
    'SELECT apply_state || ":" || sanitized_reason_code FROM makosh_kernel_settings_schema_binding;',
  ), 'blocked_config:validation.invalid_interval\n');
  const changedSchema = schemaArtifact('Updated interval');
  assert.equal(errorCode(await request(
    socketPaths.runtime,
    runtimeSubmitSchema(runtimeSessionId, moduleDescriptor(schemaArtifact()), changedSchema),
  )), 'runtime_session_denied');
}

test('settings schema admission persists only exact approved descriptor artifacts', async () => {
  const root = await mkdtemp(join(tmpdir(), 'makosh-settings-schema-'));
  const dataDir = join(root, 'data');
  let server;
  try {
    const schema = schemaArtifact();
    const descriptor = moduleDescriptor(schema);
    const enrolled = kernelCommand(
      dataDir, 'initial-owner-enroll', '--owner-id', 'owner-settings',
      '--device-id', 'device-settings',
    );
    assert.equal(enrolled.status, 0, enrolled.stderr);
    const context = await prepareRuntime(dataDir, schema, descriptor);
    server = context.server;
    await exerciseAcceptedLifecycle(context, dataDir, schema.length);
    await exerciseRejectedLifecycle(context, dataDir);
  } finally {
    if (server) await stopKernel(server);
    await rm(root, { recursive: true, force: true });
  }
});
