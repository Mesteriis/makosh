import assert from 'node:assert/strict';
import { spawn, spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import tls from 'node:tls';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';

const backendRoot = new URL('../../', import.meta.url).pathname;
const runtime = join(backendRoot, 'target', 'debug', 'makosh-development-kernel-operator');

function execute(args) {
  return spawnSync(runtime, args, { encoding: 'utf8' });
}

function createPairing(stateDir, ttlSeconds = 30) {
  const result = execute(['pairing', 'create', '--state-dir', stateDir, '--ttl-seconds', String(ttlSeconds)]);
  assert.equal(result.status, 0, result.stderr);
  const token = result.stdout.match(/^development_pairing_token=([0-9a-f]{64})$/m)?.[1];
  assert.ok(token, result.stdout);
  return token;
}

function consumeAsync(stateDir, token) {
  return new Promise((resolve) => {
    const child = spawn(runtime, ['pairing', 'consume', '--state-dir', stateDir, '--token', token], { encoding: 'utf8' });
    let stdout = '';
    let stderr = '';
    child.stdout.on('data', (chunk) => { stdout += chunk; });
    child.stderr.on('data', (chunk) => { stderr += chunk; });
    child.once('exit', (status) => resolve({ status, stdout, stderr }));
  });
}

function startListener(stateDir) {
  const child = spawn(runtime, [
    'pairing', 'listen', '--state-dir', stateDir,
    '--listen-address', '127.0.0.1:0',
    '--idle-timeout-seconds', '10',
  ], { encoding: 'utf8' });
  let stdout = '';
  let stderr = '';
  let settled = false;
  const ready = new Promise((resolve, reject) => {
    const inspect = () => {
      const endpoint = stdout.match(/^development_pairing_endpoint=https:\/\/([^\n]+)$/m)?.[1];
      const fingerprint = stdout.match(/^development_pairing_tls_certificate_sha256=([0-9a-f]{64})$/m)?.[1];
      if (endpoint && fingerprint && !settled) {
        settled = true;
        resolve({ endpoint, fingerprint });
      }
    };
    child.stdout.on('data', (chunk) => { stdout += chunk; inspect(); });
    child.stderr.on('data', (chunk) => { stderr += chunk; });
    child.once('error', reject);
    child.once('exit', (status) => {
      if (!settled) {
        settled = true;
        reject(new Error(`listener exited before ready: ${status}\n${stderr}`));
      }
    });
  });
  const exited = new Promise((resolve) => {
    child.once('exit', (status) => resolve({ status, stdout, stderr }));
  });
  return { child, ready, exited, output: () => ({ stdout, stderr }) };
}

function tlsRequest(endpoint, fingerprint, request) {
  const [host, port] = endpoint.split(':');
  return new Promise((resolve, reject) => {
    const socket = tls.connect({ host, port: Number(port), servername: 'localhost', rejectUnauthorized: false });
    let response = '';
    socket.once('secureConnect', () => {
      const certificate = socket.getPeerCertificate();
      const observed = createHash('sha256').update(certificate.raw).digest('hex');
      if (observed !== fingerprint) {
        socket.destroy();
        reject(new Error('TLS certificate fingerprint mismatch'));
        return;
      }
      socket.write(request);
    });
    socket.on('data', (chunk) => { response += chunk; });
    socket.once('end', () => resolve(response));
    socket.once('error', reject);
  });
}

function responseChallenge(response) {
  assert.match(response, /^HTTP\/1\.1 200 OK/m, response);
  const challenge = response.match(/\r\n\r\nchallenge=([0-9a-f]{64})\n$/)?.[1];
  assert.ok(challenge, response);
  return challenge;
}

test('development pairing consumes a token once and stores only its digest', async () => {
  const root = await mkdtemp(join(tmpdir(), 'makosh-development-pairing-'));
  const stateDir = join(root, 'state');
  try {
    const token = createPairing(stateDir);
    const consumed = execute(['pairing', 'consume', '--state-dir', stateDir, '--token', token]);
    assert.equal(consumed.status, 0, consumed.stderr);
    assert.match(consumed.stdout, /development_pairing_consumed=true/);
    const state = await readFile(join(stateDir, 'development-remote-pairing-v1.state'), 'utf8');
    assert.match(state, /\nconsumed\n/);
    assert.doesNotMatch(state, new RegExp(token, 'u'));

    const replay = execute(['pairing', 'consume', '--state-dir', stateDir, '--token', token]);
    assert.notEqual(replay.status, 0);
    assert.match(replay.stderr, /initial enrollment is already complete/);
    const second = execute(['pairing', 'create', '--state-dir', stateDir, '--ttl-seconds', '30']);
    assert.notEqual(second.status, 0);
    assert.match(second.stderr, /initial enrollment is already complete/);
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

test('development pairing expires instead of accepting a stale token', async () => {
  const root = await mkdtemp(join(tmpdir(), 'makosh-development-pairing-expiry-'));
  const stateDir = join(root, 'state');
  try {
    const token = createPairing(stateDir, 1);
    await writeFile(
      join(stateDir, 'development-remote-pairing-v1.receipt'),
      'interrupted receipt',
      { mode: 0o600 },
    );
    await new Promise((resolve) => setTimeout(resolve, 1_100));
    const expired = execute(['pairing', 'consume', '--state-dir', stateDir, '--token', token]);
    assert.notEqual(expired.status, 0);
    assert.match(expired.stderr, /pairing token expired/);
    const replacement = createPairing(stateDir);
    assert.notEqual(replacement, token);
    await assert.rejects(readFile(join(stateDir, 'development-remote-pairing-v1.receipt')));
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

test('concurrent development pairing consumes have one winner', async () => {
  const root = await mkdtemp(join(tmpdir(), 'makosh-development-pairing-concurrent-'));
  const stateDir = join(root, 'state');
  try {
    const token = createPairing(stateDir);
    const results = await Promise.all([consumeAsync(stateDir, token), consumeAsync(stateDir, token)]);
    assert.equal(results.filter(({ status }) => status === 0).length, 1, JSON.stringify(results));
    assert.equal(results.filter(({ status }) => status !== 0).length, 1, JSON.stringify(results));
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

test('development TLS pairing pins its certificate and requires a file-signed proof', async () => {
  const root = await mkdtemp(join(tmpdir(), 'makosh-development-pairing-tls-'));
  const stateDir = join(root, 'state');
  const keyDir = join(root, 'device-key');
  const kernelDataDir = join(root, 'kernel-data');
  const token = createPairing(stateDir);
  const listener = startListener(stateDir);
  try {
    const { endpoint, fingerprint } = await listener.ready;
    const challengeResponse = await tlsRequest(
      endpoint,
      fingerprint,
      `GET /v1/pairing-challenge HTTP/1.1\r\nHost: localhost\r\nAuthorization: Bearer ${token}\r\n\r\n`,
    );
    const challenge = responseChallenge(challengeResponse);
    const proof = execute([
      'pairing', 'proof', '--key-dir', keyDir, '--challenge', challenge,
      '--owner-id', 'owner_1', '--device-id', 'device_1',
    ]);
    assert.equal(proof.status, 0, proof.stderr);
    const publicKey = proof.stdout.match(/^device_public_key_sec1=([0-9a-f]{130})$/m)?.[1];
    const signature = proof.stdout.match(/^device_signature_raw=([0-9a-f]{128})$/m)?.[1];
    assert.ok(publicKey, proof.stdout);
    assert.ok(signature, proof.stdout);

    const rejected = await tlsRequest(
      endpoint,
      fingerprint,
      `POST /v1/initial-owner-enrollment HTTP/1.1\r\nHost: localhost\r\nAuthorization: Bearer ${token}\r\nX-Макошь-Owner-Id: owner_1\r\nX-Макошь-Device-Id: device_1\r\nX-Макошь-Device-Public-Key-Sec1: ${publicKey}\r\nX-Макошь-Device-Signature-Raw: ${'00'.repeat(64)}\r\n\r\n`,
    );
    assert.match(rejected, /^HTTP\/1\.1 400 Bad Request/m, rejected);

    const accepted = await tlsRequest(
      endpoint,
      fingerprint,
      `POST /v1/initial-owner-enrollment HTTP/1.1\r\nHost: localhost\r\nAuthorization: Bearer ${token}\r\nX-Макошь-Owner-Id: owner_1\r\nX-Макошь-Device-Id: device_1\r\nX-Макошь-Device-Public-Key-Sec1: ${publicKey}\r\nX-Макошь-Device-Signature-Raw: ${signature}\r\n\r\n`,
    );
    assert.match(accepted, /^HTTP\/1\.1 201 Created/m, accepted);
    const exited = await listener.exited;
    assert.equal(exited.status, 0, `${exited.stdout}\n${exited.stderr}`);
    assert.match(exited.stdout, /development_remote_pairing_consumed=true/);
    const state = await readFile(join(stateDir, 'development-remote-pairing-v1.state'), 'utf8');
    assert.match(state, /\nconsumed\n/);
    const receipt = await readFile(join(stateDir, 'development-remote-pairing-v1.receipt'), 'utf8');
    assert.match(receipt, /^development_remote_pairing_receipt_v1\nowner_1\ndevice_1\n/m);
    assert.doesNotMatch(receipt, new RegExp(token, 'u'));

    const imported = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'makosh-development-kernel-operator', '--', '--data-dir', kernelDataDir, 'initial-owner-import-pairing', '--pairing-state-dir', stateDir],
      { cwd: backendRoot, encoding: 'utf8' },
    );
    assert.equal(imported.status, 0, imported.stderr);
    assert.match(imported.stderr, /file-backed receipt/);
    assert.match(imported.stdout, /development_remote_initial_owner_enrolled=true\nowner_id=owner_1\ndevice_id=device_1/);
    const replay = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'makosh-development-kernel-operator', '--', '--data-dir', kernelDataDir, 'initial-owner-import-pairing', '--pairing-state-dir', stateDir],
      { cwd: backendRoot, encoding: 'utf8' },
    );
    assert.notEqual(replay.status, 0);
    assert.match(replay.stderr, /initial owner is already enrolled/);
  } catch (error) {
    const output = listener.output();
    throw new Error(`${error.message}\nlistener stdout:\n${output.stdout}\nlistener stderr:\n${output.stderr}`);
  } finally {
    listener.child.kill();
    await rm(root, { recursive: true, force: true });
  }
});

test('development pairing completes after an interrupted matching receipt write', async () => {
  const root = await mkdtemp(join(tmpdir(), 'makosh-development-pairing-recovery-'));
  const stateDir = join(root, 'state');
  const keyDir = join(root, 'device-key');
  const token = createPairing(stateDir);
  const listener = startListener(stateDir);
  try {
    const { endpoint, fingerprint } = await listener.ready;
    const challenge = responseChallenge(await tlsRequest(
      endpoint,
      fingerprint,
      `GET /v1/pairing-challenge HTTP/1.1\r\nHost: localhost\r\nAuthorization: Bearer ${token}\r\n\r\n`,
    ));
    const proof = execute([
      'pairing', 'proof', '--key-dir', keyDir, '--challenge', challenge,
      '--owner-id', 'owner_1', '--device-id', 'device_1',
    ]);
    assert.equal(proof.status, 0, proof.stderr);
    const publicKey = proof.stdout.match(/^device_public_key_sec1=([0-9a-f]{130})$/m)?.[1];
    const signature = proof.stdout.match(/^device_signature_raw=([0-9a-f]{128})$/m)?.[1];
    assert.ok(publicKey, proof.stdout);
    assert.ok(signature, proof.stdout);
    await writeFile(
      join(stateDir, 'development-remote-pairing-v1.receipt'),
      `development_remote_pairing_receipt_v1\nowner_1\ndevice_1\n${challenge}\n${publicKey}\n${signature}\n`,
      { mode: 0o600 },
    );

    const accepted = await tlsRequest(
      endpoint,
      fingerprint,
      `POST /v1/initial-owner-enrollment HTTP/1.1\r\nHost: localhost\r\nAuthorization: Bearer ${token}\r\nX-Макошь-Owner-Id: owner_1\r\nX-Макошь-Device-Id: device_1\r\nX-Макошь-Device-Public-Key-Sec1: ${publicKey}\r\nX-Макошь-Device-Signature-Raw: ${signature}\r\n\r\n`,
    );
    assert.match(accepted, /^HTTP\/1\.1 201 Created/m, accepted);
    const exited = await listener.exited;
    assert.equal(exited.status, 0, `${exited.stdout}\n${exited.stderr}`);
    const state = await readFile(join(stateDir, 'development-remote-pairing-v1.state'), 'utf8');
    assert.match(state, /\nconsumed\n/);
  } catch (error) {
    const output = listener.output();
    throw new Error(`${error.message}\nlistener stdout:\n${output.stdout}\nlistener stderr:\n${output.stderr}`);
  } finally {
    listener.child.kill();
    await rm(root, { recursive: true, force: true });
  }
});
