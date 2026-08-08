import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);

test('Zulip admission uses exact route, settings and platform capability units', async () => {
  const [proto, contract, clientPort, admission, settings, core, managed] =
    await Promise.all([
      readFile(
        new URL(
          'src/zulip-api/proto/makosh/zulip/v1/client.proto',
          BACKEND_ROOT,
        ),
        'utf8',
      ),
      readFile(
        new URL('src/zulip-api/src/client_contract.rs', BACKEND_ROOT),
        'utf8',
      ),
      readFile(
        new URL('src/zulip-runtime/src/client_port.rs', BACKEND_ROOT),
        'utf8',
      ),
      readFile(
        new URL('src/zulip-runtime/src/admission.rs', BACKEND_ROOT),
        'utf8',
      ),
      readFile(
        new URL('src/zulip-runtime/src/settings.rs', BACKEND_ROOT),
        'utf8',
      ),
      readFile(new URL('src/zulip-core/src/lib.rs', BACKEND_ROOT), 'utf8'),
      readFile(
        new URL('src/zulip-runtime/src/managed.rs', BACKEND_ROOT),
        'utf8',
      ),
    ]);

  assert.match(proto, /service ZulipCommandService/);
  assert.match(proto, /rpc ExecuteCommand\(ZulipProviderCommandV1\)/);
  assert.match(proto, /service ZulipQueryService/);
  assert.match(
    proto,
    /rpc GetOperationStatus\(ZulipOperationStatusQueryV1\)/,
  );
  assert.doesNotMatch(proto, /service ZulipOperationalService/);
  assert.doesNotMatch(proto, /message ZulipClientRequestV1/);
  assert.doesNotMatch(proto, /message ZulipClientResponseV1/);

  assert.match(contract, /"zulip\.command\.v1"/);
  assert.match(contract, /"zulip\.query\.v1"/);
  assert.match(contract, /"zulip\.operational\.query\.v1"/);
  assert.match(contract, /"zulip\.operational\.realtime\.v1"/);
  assert.match(contract, /ZULIP_CLIENT_DESCRIPTOR_SET_V1/);
  assert.match(contract, /ZULIP_OPERATIONAL_DESCRIPTOR_SET_V1/);
  assert.match(contract, /ZULIP_OPERATIONAL_REALTIME_DESCRIPTOR_SET_V1/);
  assert.match(contract, /pub const fn descriptor_set\(self\)/);
  assert.match(
    contract,
    /\/makosh\.zulip\.v1\.ZulipCommandService\/ExecuteCommand/,
  );
  assert.match(
    contract,
    /\/makosh\.zulip\.v1\.ZulipQueryService\/GetOperationStatus/,
  );

  assert.match(clientPort, /Sha256::digest\(contract\.descriptor_set\(\)\)/);
  assert.match(clientPort, /match contract/);
  assert.doesNotMatch(clientPort, /schema_sha256: Vec::new\(\)/);

  for (const capability of [
    'ZULIP_BLOB_CAPABILITY_ID',
    'ZULIP_CREDENTIALS_CAPABILITY_ID',
    'ZULIP_EVENTS_CAPABILITY_ID',
    'ZULIP_STORAGE_CAPABILITY_ID',
  ]) {
    assert.match(admission, new RegExp(capability));
  }
  assert.match(admission, /communication_observed_publish_request_v1\(\)/);
  assert.match(admission, /minimum_major: 2/);
  assert.match(admission, /maximum_major: 2/);
  assert.match(admission, /zulip_settings_schema_bytes_v3\(\)/);

  assert.match(settings, /pub fn zulip_settings_schema_v3\(\)/);
  assert.match(settings, /SettingClientVisibilityV1::Editable/);
  assert.doesNotMatch(settings, /api_key_revision/);
  assert.match(core, /ZULIP_API_KEY_PURPOSE_ID: &str = "zulip_api_key"/);
  assert.match(
    core,
    /VaultPurposeRequestV1::new\(\s*ZULIP_API_KEY_PURPOSE_ID\.to_owned\(\)/,
  );
  assert.doesNotMatch(core, /format!\([^)]*api_key/);

  assert.match(managed, /capability_id: ZULIP_BLOB_CAPABILITY_ID/);
  assert.match(managed, /ttl_seconds: ZULIP_CREDENTIAL_LEASE_TTL_SECONDS/);
});
