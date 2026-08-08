import assert from 'node:assert/strict';
import { readdir, readFile } from 'node:fs/promises';
import { join } from 'node:path';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const GATEWAY_RUNTIME_ROOT = new URL('src/api/gateway/runtime/src/', BACKEND_ROOT);
const GATEWAY_CONTRACT_ROOT = new URL('src/api/gateway/contracts/proto/', BACKEND_ROOT);
const GATEWAY_SESSION_CONTRACT = new URL('src/api/gateway/session_contract/src/browser/client_bootstrap.rs', BACKEND_ROOT);
const GATEWAY_CONTRACT_MANIFEST = new URL('src/api/gateway/contracts/Cargo.toml', BACKEND_ROOT);
const KERNEL_MANIFEST = new URL('src/kernel/Cargo.toml', BACKEND_ROOT);
const KERNEL_GATEWAY = new URL('src/kernel/src/platform/gateway.rs', BACKEND_ROOT);
const KERNEL_CLIENT_BLOB_GATEWAY = new URL(
  'src/kernel/src/platform/gateway/client_blob.rs',
  BACKEND_ROOT,
);
const KERNEL_BROWSER_GATEWAY = new URL('src/kernel/src/identity/browser_gateway.rs', BACKEND_ROOT);
const GATEWAY_CONTRACT_BUILD = new URL('src/api/gateway/contracts/build.rs', BACKEND_ROOT);
const COMMUNICATIONS_QUERY_CONTRACT = new URL(
  'src/communications-api/proto/makosh/communications/query/v1/query.proto',
  BACKEND_ROOT,
);
const FORBIDDEN_PROVIDER_HTTP_SURFACES = [
  '/api/v1/integrations/',
  '/api/v1/communications/email/',
];
const FORBIDDEN_PROVIDER_ROUTER_MARKERS = [
  'MailGatewayIntegrationRouter',
  'TelegramGatewayIntegrationRouter',
  'WhatsAppGatewayIntegrationRouter',
  'makosh_mail_',
  'makosh_telegram_',
  'makosh_whatsapp_',
];
const FORBIDDEN_PROVIDER_COMMAND_CONTRACTS = [
  'ExecuteMailRuntimeOwnerCommand',
  'ExecuteCommunicationsRuntimeOwnerCommand',
  'MailRuntimeOwnerCommand',
  'CommunicationsRuntimeOwnerCommand',
];

test('Gateway runtime has no provider-specific HTTP facade', async () => {
  const sources = await rustSources(GATEWAY_RUNTIME_ROOT);

  assert.ok(sources.length > 0);
  for (const source of sources) {
    for (const surface of FORBIDDEN_PROVIDER_HTTP_SURFACES) {
      assert.ok(!source.content.includes(surface), `${source.path} exposes provider HTTP surface ${surface}`);
    }
    for (const marker of FORBIDDEN_PROVIDER_ROUTER_MARKERS) {
      assert.ok(!source.content.includes(marker), `${source.path} depends on provider implementation ${marker}`);
    }
  }
});

test('Gateway contracts do not expose provider or Communications runtime command facades', async () => {
  const entries = await readdir(GATEWAY_CONTRACT_ROOT, { recursive: true, withFileTypes: true });
  const sources = await Promise.all(entries
    .filter((entry) => entry.isFile() && entry.name.endsWith('.proto'))
    .map(async (entry) => {
      const path = entry.parentPath.startsWith(GATEWAY_CONTRACT_ROOT.pathname)
        ? join(entry.parentPath, entry.name)
        : join(GATEWAY_CONTRACT_ROOT.pathname, entry.parentPath, entry.name);
      return { path, content: await readFile(path, 'utf8') };
    }));

  for (const source of sources) {
    for (const marker of FORBIDDEN_PROVIDER_COMMAND_CONTRACTS) {
      assert.ok(!source.content.includes(marker), `${source.path} exposes runtime command facade ${marker}`);
    }
  }
});

test('Kernel and Gateway contracts do not compile Communications owner packages', async () => {
  const [gatewayManifest, kernelManifest] = await Promise.all([
    readFile(GATEWAY_CONTRACT_MANIFEST, 'utf8'),
    readFile(KERNEL_MANIFEST, 'utf8'),
  ]);

  for (const manifest of [gatewayManifest, kernelManifest]) {
    assert.doesNotMatch(manifest, /makosh-communications-(api|domain|ingress|persistence|runtime)/);
  }
});

test('Communications query remains an owner contract, not a Gateway wrapper service', async () => {
  const [entries, ownerContract] = await Promise.all([
    readdir(GATEWAY_CONTRACT_ROOT, { recursive: true, withFileTypes: true }),
    readFile(COMMUNICATIONS_QUERY_CONTRACT, 'utf8'),
  ]);
  const gatewayContracts = await Promise.all(entries
    .filter((entry) => entry.isFile() && entry.name.endsWith('.proto'))
    .map((entry) => readFile(
      entry.parentPath.startsWith(GATEWAY_CONTRACT_ROOT.pathname)
        ? join(entry.parentPath, entry.name)
        : join(GATEWAY_CONTRACT_ROOT.pathname, entry.parentPath, entry.name),
      'utf8',
    )));

  assert.match(ownerContract, /service CommunicationsQueryService/);
  for (const contract of gatewayContracts) {
    assert.doesNotMatch(contract, /CommunicationsQueryService/);
  }
});

test('Gateway route composition is owner-neutral and has no owner schema build edge', async () => {
  const [kernelGateway, kernelClientBlobGateway, gatewayBuild] = await Promise.all([
    readFile(KERNEL_GATEWAY, 'utf8'),
    readFile(KERNEL_CLIENT_BLOB_GATEWAY, 'utf8'),
    readFile(GATEWAY_CONTRACT_BUILD, 'utf8'),
  ]);

  for (const marker of ['COMMUNICATIONS_', 'CommunicationsQuery', 'communications.query']) {
    for (const source of [kernelGateway, kernelClientBlobGateway]) {
      assert.ok(!source.includes(marker), `Kernel Gateway hardcodes owner route marker ${marker}`);
    }
  }
  for (const marker of ['communications-api', 'communications_query_schema', 'communications-query-v1.bin']) {
    assert.ok(!gatewayBuild.includes(marker), `Gateway contracts retain owner schema build edge ${marker}`);
  }
});

test('Core Gateway client Blob delivery stays descriptor and capability bound', async () => {
  const adapter = await readFile(KERNEL_CLIENT_BLOB_GATEWAY, 'utf8');

  assert.match(adapter, /approved_module_client_blob_routes/);
  assert.match(adapter, /module_grant_snapshot/);
  assert.match(adapter, /capability_ids\(\)/);
  assert.match(adapter, /effective_managed_launch_record/);
  assert.match(adapter, /effective_bundled_managed_launch_binding/);
  assert.match(adapter, /ManagedRuntimeBlobSessionRequestV1/);
  assert.match(adapter, /BlobDataOperationReadRangeV1/);
  assert.match(adapter, /channel_binding_sha256:\s*Sha256::digest\(channel_binding\)\.to_vec\(\)/);
  assert.match(adapter, /client\.read_range\(\s*grant,\s*channel_binding\.to_vec\(\)/);
  assert.match(adapter, /expected_plaintext_sha256/);
  assert.doesNotMatch(adapter, /makosh_(communications|mail|telegram|whatsapp|zulip)/);
});

test('Core client bootstrap admits Communications and provider surfaces independently', async () => {
  const [sessionContract, kernelBootstrap, gatewayProto] = await Promise.all([
    readFile(GATEWAY_SESSION_CONTRACT, 'utf8'),
    readFile(KERNEL_BROWSER_GATEWAY, 'utf8'),
    readFile(new URL('makosh/gateway/v1/client_bootstrap.proto', GATEWAY_CONTRACT_ROOT), 'utf8'),
  ]);

  for (const source of [sessionContract, kernelBootstrap, gatewayProto]) {
    assert.doesNotMatch(source, /Communications(Mail|Telegram|Whatsapp)|COMMUNICATIONS_(MAIL|TELEGRAM|WHATSAPP)/);
  }
  assert.match(sessionContract, /Self::Communications => Some\("communications\.query\.v1"\)/);
  for (const [surface, capability] of [
    ['Mail', 'mail.delivery.query.v1'],
    ['Telegram', 'telegram.query.v1'],
    ['WhatsApp', 'whatsapp.query.v1'],
    ['Zulip', 'zulip.query.v1'],
  ]) {
    assert.match(
      sessionContract,
      new RegExp(`Self::${surface} => Some\\("${capability.replaceAll('.', '\\.')}\"\\)`),
    );
    assert.match(kernelBootstrap, new RegExp(`\\b${surface}\\b`));
  }
  for (const declaration of [
    'CLIENT_SURFACE_ID_V1_COMMUNICATIONS = 2;',
    'CLIENT_SURFACE_ID_V1_MAIL = 3;',
    'CLIENT_SURFACE_ID_V1_TELEGRAM = 4;',
    'CLIENT_SURFACE_ID_V1_WHATSAPP = 12;',
    'CLIENT_SURFACE_ID_V1_ZULIP = 13;',
  ]) {
    assert.ok(gatewayProto.includes(declaration), `missing exact surface declaration ${declaration}`);
  }
});

async function rustSources(directory) {
  const entries = await readdir(directory, { recursive: true, withFileTypes: true });
  const files = entries
    .filter((entry) => entry.isFile() && entry.name.endsWith('.rs'))
    .map((entry) => join(
      entry.parentPath.startsWith(directory.pathname)
        ? entry.parentPath
        : directory.pathname,
      entry.parentPath.startsWith(directory.pathname) ? entry.name : entry.parentPath,
      entry.parentPath.startsWith(directory.pathname) ? '' : entry.name,
    ));

  return Promise.all(files.map(async (path) => ({
    path,
    content: await readFile(path, 'utf8'),
  })));
}
