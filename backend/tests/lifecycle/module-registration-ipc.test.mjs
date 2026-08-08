import assert from 'node:assert/strict';
import { mkdtemp, rm, stat } from 'node:fs/promises';
import { spawn, spawnSync } from 'node:child_process';
import { createConnection } from 'node:net';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';

const backendRoot = new URL('../../', import.meta.url);

function waitForChildExit(child) {
  if (child.exitCode !== null || child.signalCode !== null) return Promise.resolve();
  return new Promise((resolve) => child.once('exit', resolve));
}

function encodeVarint(value) {
  const bytes = [];
  while (value >= 0x80) {
    bytes.push((value & 0x7f) | 0x80);
    value >>>= 7;
  }
  return Buffer.from([...bytes, value]);
}

function field(fieldNumber, value) {
  return Buffer.concat([Buffer.from([(fieldNumber << 3) | 2]), encodeVarint(value.length), value]);
}

function stringField(fieldNumber, value) {
  return field(fieldNumber, Buffer.from(value, 'ascii'));
}

function request(socketPath, message) {
  return new Promise((resolve, reject) => {
    const socket = createConnection(socketPath);
    const chunks = [];
    socket.once('error', reject);
    socket.on('data', (chunk) => chunks.push(chunk));
    socket.once('end', () => resolve(Buffer.concat(chunks)));
    socket.once('connect', () => socket.end(Buffer.concat([encodeVarint(message.length), message])));
  });
}

function command(dataDir, ...args) {
  return spawnSync(
    'cargo',
    ['run', '-q', '-p', 'makosh-kernel', '--', '--data-dir', dataDir, ...args],
    { cwd: backendRoot, encoding: 'utf8' },
  );
}

function moduleDescriptor() {
  const text = (value) => [value.length, ...Buffer.from(value, 'ascii')];
  const capability = [0x0a, ...text('capability.read'), 0x10, 0x01, 0x18, 0x01];
  return Buffer.from([
    0x08, 0x01, 0x10, 0x01,
    0x1a, ...text('module-registration-ipc'),
    0x22, ...text('owner-registration-ipc'),
    0x28, 0x02,
    0x32, ...text('1'),
    0x3a, ...text('build-1'),
    0x4a, capability.length, ...capability,
  ]);
}

async function startRegistrationServer(dataDir) {
  const server = spawn(
    'cargo',
    ['run', '-q', '-p', 'makosh-kernel', '--', '--data-dir', dataDir, 'serve'],
    { cwd: backendRoot, stdio: ['ignore', 'pipe', 'pipe'] },
  );
  const socketPath = await new Promise((resolve, reject) => {
    const timeout = setTimeout(() => reject(new Error('module registration socket did not start')), 15_000);
    let output = '';
    let stderr = '';
    server.stderr.on('data', (chunk) => { stderr += chunk.toString('utf8'); });
    server.stdout.on('data', (chunk) => {
      output += chunk.toString('utf8');
      const match = output.match(/module_registration_socket=(.+)\n/);
      if (match) {
        clearTimeout(timeout);
        resolve(match[1]);
      }
    });
    server.once('exit', (code) => reject(new Error(`module registration exited early: ${code}: ${stderr}`)));
  });
  return { server, socketPath };
}

function beginRequest() {
  return Buffer.from([0x12, 0x00]);
}

function describeRequest(sessionId, descriptor) {
  return field(3, Buffer.concat([stringField(1, sessionId), field(2, descriptor)]));
}

function ownStatusRequest(sessionId) {
  return field(4, stringField(1, sessionId));
}

function sessionId(response) {
  return response.toString('ascii').match(/[0-9a-f]{32}/)?.[0];
}

test('module registration IPC bounds pending admission and exposes only own status', async () => {
  const root = await mkdtemp(join(tmpdir(), 'makosh-kernel-module-registration-'));
  const dataDir = join(root, 'data');
  let server;
  let socketPath;
  try {
    const enrolled = command(dataDir, 'initial-owner-enroll', '--owner-id', 'owner-registration-ipc', '--device-id', 'device-registration-ipc');
    assert.equal(enrolled.status, 0, enrolled.stderr);
    ({ server, socketPath } = await startRegistrationServer(dataDir));
    assert.equal((await stat(socketPath)).mode & 0o777, 0o600);

    const hello = await request(socketPath, Buffer.from([0x0a, 0x00]));
    assert.deepEqual(hello, Buffer.from([0x04, 0x0a, 0x02, 0x08, 0x01]));
    const malformed = await request(socketPath, Buffer.from([0x08]));
    assert.ok(malformed.includes(Buffer.from('invalid_request', 'ascii')));
    const oversized = await request(socketPath, Buffer.from([0x80, 0x80, 0x04]));
    assert.ok(oversized.includes(Buffer.from('invalid_request', 'ascii')));

    const session = sessionId(await request(socketPath, beginRequest()));
    assert.ok(session);
    const described = await request(socketPath, describeRequest(session, moduleDescriptor()));
    const registrationId = sessionId(described);
    assert.ok(registrationId);
    assert.ok(described.includes(Buffer.from('pending', 'ascii')));
    const ownStatus = await request(socketPath, ownStatusRequest(session));
    assert.ok(ownStatus.includes(Buffer.from(registrationId, 'ascii')));
    assert.ok(ownStatus.includes(Buffer.from('pending', 'ascii')));

    const duplicate = await request(socketPath, describeRequest(session, moduleDescriptor()));
    assert.ok(duplicate.includes(Buffer.from('registration_session_unavailable', 'ascii')));
    const statusAfterDuplicate = await request(socketPath, ownStatusRequest(session));
    assert.ok(statusAfterDuplicate.includes(Buffer.from(registrationId, 'ascii')));

    const invalidSession = sessionId(await request(socketPath, beginRequest()));
    assert.ok(invalidSession);
    const rejectedDescriptor = await request(socketPath, describeRequest(invalidSession, Buffer.from([0x08])));
    assert.ok(rejectedDescriptor.includes(Buffer.from('registration_denied', 'ascii')));
    const reusedInvalidSession = await request(socketPath, describeRequest(invalidSession, moduleDescriptor()));
    assert.ok(reusedInvalidSession.includes(Buffer.from('registration_session_unavailable', 'ascii')));

    for (let index = 0; index < 14; index += 1) {
      const accepted = await request(socketPath, beginRequest());
      assert.ok(sessionId(accepted));
    }
    const rateLimited = await request(socketPath, beginRequest());
    assert.ok(rateLimited.includes(Buffer.from('registration_rate_limited', 'ascii')));
  } finally {
    if (server) {
      server.kill('SIGTERM');
      await waitForChildExit(server);
      await assert.rejects(stat(socketPath));
    }
    await rm(root, { recursive: true, force: true });
  }
});
