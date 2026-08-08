import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);

test('Mail provider transport seams are compile-time conformance-only', async () => {
  const [
    apiManifest,
    api,
    gmailManifest,
    gmail,
    imapManifest,
    imap,
    runtimeManifest,
    settings,
    harness,
  ] =
    await Promise.all([
      readFile(new URL('src/mail-api/Cargo.toml', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/mail-api/src/lib.rs', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/mail-gmail/Cargo.toml', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/mail-gmail/src/lib.rs', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/mail-imap/Cargo.toml', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/mail-imap/src/lib.rs', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/mail-runtime/Cargo.toml', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/mail-runtime/src/settings.rs', BACKEND_ROOT), 'utf8'),
      readFile(
        new URL('scripts/test-authenticated-storage.mjs', BACKEND_ROOT),
        'utf8',
      ),
    ]);

  for (const manifest of [
    apiManifest,
    gmailManifest,
    imapManifest,
    runtimeManifest,
  ]) {
    assert.match(manifest, /\[features\]\s+default = \[\]/);
  }
  assert.match(
    runtimeManifest,
    /"makosh-mail-api\/conformance-test-support"/,
  );
  assert.match(
    runtimeManifest,
    /"makosh-mail-imap\/conformance-test-support"/,
  );
  assert.match(
    runtimeManifest,
    /"makosh-mail-gmail\/conformance-test-support"/,
  );

  assert.match(
    api,
    /pub fn valid_port[\s\S]*#\[cfg\(not\(feature = "conformance-test-support"\)\)\]/,
  );
  assert.match(api, /port == IMAP_PORT/);
  assert.match(
    imap,
    /#\[cfg\(not\(feature = "conformance-test-support"\)\)\]\s+async fn open_session/,
  );
  assert.match(imap, /TlsConnector::new\(\)/);
  assert.match(
    imap,
    /#\[cfg\(feature = "conformance-test-support"\)\]\s+async fn open_session/,
  );
  assert.match(
    imap,
    /matches!\(host, "127\.0\.0\.1" \| "::1" \| "localhost"\)/,
  );
  assert.match(api, /endpoint\.host == GMAIL_API_HOST/);
  assert.match(api, /endpoint\.port == GMAIL_API_HTTPS_PORT/);
  assert.match(api, /endpoint\.ca_certificate_pem\.is_none\(\)/);
  assert.match(api, /"127\.0\.0\.1" \| "localhost"/);
  assert.match(
    gmail,
    /#\[cfg\(any\(test, feature = "conformance-test-support"\)\)\]\s+pub fn for_conformance_endpoint/,
  );
  assert.match(gmail, /"127\.0\.0\.1" \| "localhost"/);
  assert.match(gmail, /valid_bearer_token\(access_token\)/);
  assert.match(gmail, /GMAIL_OPERATION_TIMEOUT/);
  assert.match(gmail, /TlsConnector::new\(\)\.add_root_certificate/);
  assert.doesNotMatch(`${gmail}\n${settings}`, /std::env|var_os|GMAIL_API_URL/);
  assert.match(
    harness,
    /--features',\s+'[^']*makosh-mail-runtime\/conformance-test-support[^']*'/,
  );
});

test('Mail event routes are independent capability approval units', async () => {
  const [admission, liveSetup] = await Promise.all([
    readFile(new URL('src/mail-runtime/src/admission.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/mail_managed_setup.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
  ]);

  for (const [symbol, capabilityId] of [
    [
      'MAIL_ATTACHMENT_ANCHOR_CONSUME_CAPABILITY_ID',
      'mail.attachment-anchor.consume.v1',
    ],
    [
      'MAIL_ATTACHMENT_BLOB_ADMISSION_PUBLISH_CAPABILITY_ID',
      'mail.attachment-blob-admission.publish.v1',
    ],
    [
      'MAIL_COMMUNICATION_OBSERVED_PUBLISH_CAPABILITY_ID',
      'mail.communication-observed.publish.v1',
    ],
  ]) {
    assert.match(
      admission,
      new RegExp(`${symbol}: &str =[\\s\\S]{0,80}"${capabilityId.replaceAll('.', '\\.')}"`),
    );
  }
  assert.doesNotMatch(`${admission}\n${liveSetup}`, /MAIL_EVENTS_CAPABILITY_ID|mail\.events\.v1/);
  assert.match(
    admission,
    /mail_attachment_anchor_consume_capability_v1\(\)[\s\S]*requests: vec!\[CapabilityRequestV1/,
  );
  assert.match(
    admission,
    /mail_attachment_blob_admission_publish_capability_v1\(\)[\s\S]*requests: vec!\[communication_attachment_blob_admission_observed_publish_request_v1\(\)\]/,
  );
  assert.match(
    admission,
    /mail_communication_observed_publish_capability_v1\(\)[\s\S]*requests: vec!\[communication_observed_publish_request_v1\(\)\]/,
  );
});

test('Gmail OAuth is a Mail-owned durable workflow with exact Vault actions', async () => {
  const [
    proto,
    api,
    clientContract,
    adapter,
    persistence,
    runtime,
    admission,
    settings,
  ] = await Promise.all([
    readFile(
      new URL('src/mail-api/proto/makosh/mail/v1/client.proto', BACKEND_ROOT),
      'utf8',
    ),
    readFile(new URL('src/mail-api/src/oauth.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL('src/mail-api/src/client_contract.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(new URL('src/mail-gmail/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/mail-persistence/src/oauth.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/mail-runtime/src/gmail_oauth.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/mail-runtime/src/admission.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/mail-runtime/src/settings.rs', BACKEND_ROOT), 'utf8'),
  ]);

  for (const [service, operation] of [
    ['GmailOAuthStartService', 'Start'],
    ['GmailOAuthCompleteService', 'Complete'],
    ['GmailOAuthRefreshService', 'Refresh'],
    ['GmailOAuthQueryService', 'GetOperationStatus'],
  ]) {
    assert.match(
      proto,
      new RegExp(`service ${service} \\{[\\s\\S]*rpc ${operation}\\(`),
    );
  }
  for (const capabilityId of [
    'mail.oauth.start.v1',
    'mail.oauth.complete.v1',
    'mail.oauth.refresh.v1',
    'mail.oauth.query.v1',
  ]) {
    assert.match(
      clientContract,
      new RegExp(capabilityId.replaceAll('.', '\\.')),
    );
  }
  assert.doesNotMatch(
    proto,
    /client_secret|authorization_host|token_host|token_path|scope/,
  );
  assert.match(api, /GMAIL_OAUTH_AUTHORIZATION_HOST: &str = "accounts\.google\.com"/);
  assert.match(api, /GMAIL_OAUTH_TOKEN_HOST: &str = "oauth2\.googleapis\.com"/);
  assert.match(proto, /enum GmailOAuthAuthorityV1[\s\S]*OPERATIONAL[\s\S]*PERMANENT_DELETE/);
  assert.match(adapter, /code_challenge_method", "S256"/);
  assert.match(adapter, /GMAIL_OPERATIONAL_OAUTH_SCOPES: \[&str; 5\]/);
  assert.match(adapter, /"https:\/\/www\.googleapis\.com\/auth\/contacts"/);
  assert.match(adapter, /GMAIL_PERMANENT_DELETE_OAUTH_SCOPES: \[&str; 3\]/);
  assert.match(adapter, /"https:\/\/mail\.google\.com\/"/);
  assert.doesNotMatch(
    adapter,
    /makosh_communications|makosh_mail_persistence|makosh_managed_vault_client/,
  );

  assert.match(persistence, /mail_gmail_oauth_attempts/);
  assert.match(persistence, /mail_gmail_oauth_operations/);
  assert.match(persistence, /state_sha256 BYTEA NOT NULL/);
  assert.match(persistence, /authorization_code = NULL/);
  assert.match(runtime, /gmail_oauth_credential_binding/);
  assert.match(runtime, /ManagedProviderCredentialClientV2/);
  assert.doesNotMatch(runtime, /makosh_communications/);
  assert.doesNotMatch(settings, /gmail_access_token_revision/);

  assert.match(
    admission,
    /mail_gmail_access_token[\s\S]*VaultSecretClassV1::ProviderCredential[\s\S]*VaultActionV1::Create[\s\S]*VaultActionV1::ReplaceCas/,
  );
  assert.match(
    admission,
    /mail_gmail_refresh_credential[\s\S]*VaultSecretClassV1::OauthRefreshCredential[\s\S]*VaultActionV1::Resolve[\s\S]*VaultActionV1::ReplaceCas/,
  );
  assert.match(runtime, /SecretClassV1::ProviderCredential/);
  assert.match(runtime, /SecretClassV1::OAuthRefreshCredential/);
});
