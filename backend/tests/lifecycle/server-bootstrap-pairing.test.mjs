import assert from 'node:assert/strict';
import { spawn, spawnSync } from 'node:child_process';
import { createHash, generateKeyPairSync, sign } from 'node:crypto';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';
import tls from 'node:tls';

const backendRoot = new URL('../../', import.meta.url).pathname;
const proofDomain = Buffer.from('makosh.server-bootstrap-pairing.v1\0', 'ascii');

function startPairing(dataDir) {
  const child = spawn('cargo', [
    'run', '-q', '-p', 'makosh-kernel', '--', '--data-dir', dataDir,
    'server-bootstrap-pairing', '--listen-address', '127.0.0.1:0', '--ttl-seconds', '10',
  ], { cwd: backendRoot, encoding: 'utf8' });
  let stdout = '';
  let stderr = '';
  let settled = false;
  const ready = new Promise((resolve, reject) => {
    const inspect = () => {
      const endpoint = stdout.match(/^server_bootstrap_pairing_endpoint=https:\/\/([^\n]+)$/m)?.[1];
      const fingerprint = stdout.match(/^server_bootstrap_pairing_tls_certificate_sha256=([0-9a-f]{64})$/m)?.[1];
      const token = stdout.match(/^server_bootstrap_pairing_token=([0-9a-f]{64})$/m)?.[1];
      const challenge = stdout.match(/^server_bootstrap_pairing_challenge=([0-9a-f]{64})$/m)?.[1];
      if (endpoint && fingerprint && token && challenge && !settled) {
        settled = true;
        resolve({ endpoint, fingerprint, token, challenge });
      }
    };
    child.stdout.on('data', (chunk) => { stdout += chunk; inspect(); });
    child.stderr.on('data', (chunk) => { stderr += chunk; });
    child.once('error', reject);
    child.once('exit', (status) => {
      if (!settled) {
        settled = true;
        reject(new Error(`pairing listener exited before ready: ${status}\n${stderr}`));
      }
    });
  });
  const exited = new Promise((resolve) => {
    child.once('exit', (status) => resolve({ status, stdout, stderr }));
  });
  return { ready, exited };
}

function proof(challengeHex, ownerId, deviceId) {
  const keys = generateKeyPairSync('ec', {
    namedCurve: 'prime256v1',
  });
  const publicKey = keys.publicKey.export({ format: 'der', type: 'spki' }).subarray(-65);
  assert.equal(publicKey[0], 0x04);
  const owner = Buffer.from(ownerId, 'ascii');
  const device = Buffer.from(deviceId, 'ascii');
  const message = Buffer.concat([
    proofDomain,
    Buffer.from(challengeHex, 'hex'),
    publicKey,
    Buffer.from([owner.length >> 8, owner.length & 0xff]), owner,
    Buffer.from([device.length >> 8, device.length & 0xff]), device,
  ]);
  const signature = sign('sha256', message, { key: keys.privateKey, dsaEncoding: 'ieee-p1363' });
  assert.equal(signature.length, 64);
  return { publicKey: publicKey.toString('hex'), signature: signature.toString('hex') };
}

function request(endpoint, fingerprint, token, signedProof, path = '/v1/initial-owner-enrollment') {
  const [host, port] = endpoint.split(':');
  return new Promise((resolve, reject) => {
    const socket = tls.connect({ host, port: Number(port), servername: 'localhost', rejectUnauthorized: false });
    let response = '';
    let settled = false;
    const finish = (callback, value) => {
      if (!settled) {
        settled = true;
        callback(value);
      }
    };
    socket.once('secureConnect', () => {
      const observed = createHash('sha256').update(socket.getPeerCertificate().raw).digest('hex');
      if (observed !== fingerprint) {
        socket.destroy();
        finish(reject, new Error('TLS certificate fingerprint mismatch'));
        return;
      }
      socket.write(
        `POST ${path} HTTP/1.1\r\nHost: localhost\r\nAuthorization: Bearer ${token}\r\nX-Макошь-Owner-Id: owner_1\r\nX-Макошь-Device-Id: device_1\r\nX-Макошь-Device-Public-Key-Sec1: ${signedProof.publicKey}\r\nX-Макошь-Device-Signature-Raw: ${signedProof.signature}\r\n\r\n`,
      );
    });
    socket.on('data', (chunk) => { response += chunk; });
    socket.once('end', () => finish(resolve, response));
    socket.once('error', (error) => {
      finish(reject, new Error(`${error.message}; response=${response}`));
    });
  });
}

test('server bootstrap pairing pins TLS, rejects a bad token and consumes a valid file-ES256 proof once', async () => {
  const root = await mkdtemp(join(tmpdir(), 'makosh-server-bootstrap-pairing-'));
  const dataDir = join(root, 'data');
  const storePath = join(dataDir, 'kernel-control-store.sqlite');
  try {
    const listener = startPairing(dataDir);
    const descriptor = await listener.ready;
    const signedProof = proof(descriptor.challenge, 'owner_1', 'device_1');
    await assert.rejects(
      request(descriptor.endpoint, 'f'.repeat(64), descriptor.token, signedProof),
      /TLS certificate fingerprint mismatch/,
    );
    const wrongEndpoint = await request(descriptor.endpoint, descriptor.fingerprint, descriptor.token, signedProof, '/v1/not-pairing');
    assert.match(wrongEndpoint, /^HTTP\/1\.1 400 Bad Request/m);
    const rejected = await request(descriptor.endpoint, descriptor.fingerprint, '0'.repeat(64), signedProof);
    assert.match(rejected, /^HTTP\/1\.1 400 Bad Request/m);
    const accepted = await request(descriptor.endpoint, descriptor.fingerprint, descriptor.token, signedProof);
    assert.match(accepted, /^HTTP\/1\.1 201 Created/m);
    const exited = await listener.exited;
    assert.equal(exited.status, 0, `${exited.stdout}\n${exited.stderr}`);
    assert.match(exited.stdout, /server_bootstrap_pairing_consumed=true/);

    const persisted = spawnSync(
      'sqlite3',
      [storePath, "SELECT status, length(token_sha256), length(certificate_sha256), length(challenge) FROM makosh_kernel_server_bootstrap_pairing;"],
      { encoding: 'utf8' },
    );
    assert.equal(persisted.status, 0, persisted.stderr);
    assert.equal(persisted.stdout, 'consumed|32|32|32\n');

    const replay = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'makosh-kernel', '--', '--data-dir', dataDir, 'server-bootstrap-pairing', '--listen-address', '127.0.0.1:0'],
      { cwd: backendRoot, encoding: 'utf8' },
    );
    assert.notEqual(replay.status, 0);
    assert.match(replay.stderr, /initial owner is already enrolled/);
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

test('server bootstrap pairing stops after its bounded failed-request limit', async () => {
  const root = await mkdtemp(join(tmpdir(), 'makosh-server-bootstrap-rate-limit-'));
  const dataDir = join(root, 'data');
  try {
    const listener = startPairing(dataDir);
    const descriptor = await listener.ready;
    const signedProof = proof(descriptor.challenge, 'owner_1', 'device_1');
    for (let attempt = 0; attempt < 8; attempt += 1) {
      const rejected = await request(descriptor.endpoint, descriptor.fingerprint, '0'.repeat(64), signedProof);
      assert.match(rejected, /^HTTP\/1\.1 400 Bad Request/m);
    }
    const exited = await listener.exited;
    assert.notEqual(exited.status, 0);
    assert.match(exited.stderr, /rate limit exceeded/);
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});
