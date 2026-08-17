import assert from 'node:assert/strict';
import { readFile, readdir } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const ZULIP_PACKAGE_ROOTS = [
  'src/zulip-api/src/',
  'src/zulip-core/src/',
  'src/zulip-http/src/',
  'src/zulip-persistence/src/',
  'src/zulip-runtime/src/',
];

test('Zulip subjects, routes, diagnostics and health expose no provider-private metadata', async () => {
  const [admission, outbox, httpLib, wire, runtimeMain, managed, settings, ownerSources] =
    await Promise.all([
      readFile(new URL('src/zulip-runtime/src/admission.rs', BACKEND_ROOT), 'utf8'),
      readFile(
        new URL('src/zulip-runtime/src/communications_outbox.rs', BACKEND_ROOT),
        'utf8',
      ),
      readFile(new URL('src/zulip-http/src/lib.rs', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/zulip-http/src/wire.rs', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/zulip-runtime/src/main.rs', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/zulip-runtime/src/managed.rs', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/zulip-runtime/src/settings.rs', BACKEND_ROOT), 'utf8'),
      readZulipOwnerSources(),
    ]);

  assert.match(admission, /requests: vec!\[communication_observed_publish_request_v1\(\)\]/);
  assert.match(outbox, /\.publish_exact\(permit, record\.exact_bytes\(\)\)/);
  assert.doesNotMatch(`${admission}\n${outbox}`, /\.(?:publish|subscribe)\(\s*["'`]/);

  assert.match(httpLib, /\.field\("account", &"<redacted>"\)/);
  assert.match(httpLib, /\.field\("api_key", &"<redacted>"\)/);
  assert.doesNotMatch(httpLib, /\.field\("account", &self\.account\)/);
  assert.match(wire, /fn unavailable\(stage: &str\) -> ZulipHttpErrorV1/);
  assert.match(wire, /developer_zulip_http_unavailable stage=\{stage\}\\n/);
  assert.doesNotMatch(wire, /developer_zulip_http_unavailable[^"]*error=/);

  for (const diagnostic of [
    'developer_zulip_runtime_admission_error={error:?}',
    'developer_zulip_runtime_client_delivery_error={error:?}',
    'developer_zulip_runtime_command_schedule_error={error:?}',
    'developer_zulip_runtime_event_accept_error={error:?}',
    'developer_zulip_runtime_tick_error={error:?}',
  ]) {
    assert.ok(runtimeMain.includes(diagnostic), `missing sanitized diagnostic ${diagnostic}`);
  }
  assert.doesNotMatch(
    runtimeMain,
    /developer_zulip_runtime_[^"]*(?:realm_url|queue_id|api_key|content|payload)/,
  );
  assert.doesNotMatch(managed, /error_code:\s*format!\(/);
  assert.match(managed, /"managed_runtime_control_invalid_client_delivery"/);
  assert.match(managed, /"managed_runtime_control_unexpected_request"/);

  assert.match(
    settings,
    /client_visibility: SettingClientVisibilityV1::Editable as i32/,
  );
  assert.doesNotMatch(
    settings,
    /const API_KEY:|["']zulip\.api_key["']|credential_revision|secret_ref|record_id/,
  );
  assert.doesNotMatch(
    ownerSources,
    /(?:["'`]\/(?:health|ready)|\bHealth(?:Check|Response|Status)\b|\bfn\s+(?:health|readiness)\s*\()/i,
  );
});

async function readZulipOwnerSources() {
  const sources = await Promise.all(
    ZULIP_PACKAGE_ROOTS.map((path) => readRustTree(new URL(path, BACKEND_ROOT))),
  );
  return sources.flat().join('\n');
}

async function readRustTree(directory) {
  const entries = await readdir(directory, { withFileTypes: true });
  const sources = await Promise.all(entries.map(async (entry) => {
    const path = new URL(entry.name, directory);
    if (entry.isDirectory()) {
      return readRustTree(new URL(`${entry.name}/`, directory));
    }
    if (entry.isFile() && entry.name.endsWith('.rs')) {
      return [await readFile(path, 'utf8')];
    }
    return [];
  }));
  return sources.flat();
}
