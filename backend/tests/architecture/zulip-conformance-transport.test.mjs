import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);

test('Zulip loopback CA trust is compile-time conformance-only', async () => {
  const [httpManifest, runtimeManifest, wire, launcher, harness] =
    await Promise.all([
    readFile(new URL('src/zulip-http/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/zulip-runtime/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/zulip-http/src/wire.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL(
        'src/kernel/src/runtime/managed/execution.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('scripts/test-authenticated-storage.mjs', BACKEND_ROOT),
      'utf8',
    ),
    ]);

  for (const manifest of [httpManifest, runtimeManifest]) {
    assert.match(manifest, /\[features\]\s+default = \[\]/);
  }
  assert.match(
    runtimeManifest,
    /conformance-test-support = \["makosh-zulip-http\/conformance-test-support"\]/,
  );
  assert.match(
    wire,
    /#\[cfg\(not\(feature = "conformance-test-support"\)\)\]\s+fn tls_connector/,
  );
  assert.match(
    wire,
    /#\[cfg\(feature = "conformance-test-support"\)\]\s+fn tls_connector/,
  );
  assert.match(wire, /host != "localhost"/);
  assert.match(wire, /Certificate::from_pem/);
  assert.doesNotMatch(wire, /danger_accept_invalid_(?:certs|hostnames)/);
  assert.match(
    launcher,
    /#\[cfg\(test\)\]\s+if let Some\(certificate_path\)/,
  );
  assert.match(
    launcher,
    /MAKOSH_MANAGED_RUNTIME_CONFORMANCE_CA_CERT_FILE/,
  );
  assert.match(
    harness,
    /makosh-zulip-runtime\/conformance-test-support/,
  );
});
