import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

const paths = {
  inventory: new URL(
    'architecture/communications-settings-reconstruction.json',
    BACKEND_ROOT,
  ),
  adr: new URL(
    'docs/adr/ADR-0310-telegram-user-only-tdlib-qr-account-identity.md',
    PROJECT_ROOT,
  ),
  api: new URL('src/telegram-api/src/lib.rs', BACKEND_ROOT),
  clientContract: new URL('src/telegram-api/src/client_contract.rs', BACKEND_ROOT),
  clientProto: new URL(
    'src/telegram-api/proto/makosh/telegram/v1/client.proto',
    BACKEND_ROOT,
  ),
  core: new URL('src/telegram-core/src/lib.rs', BACKEND_ROOT),
  runtimeBootstrap: new URL('src/telegram-runtime/src/bootstrap.rs', BACKEND_ROOT),
  tdlib: new URL('src/telegram-tdlib/src/lib.rs', BACKEND_ROOT),
  frontendGateway: new URL(
    'frontend/src/integrations/telegram/api/telegramLifecycleGateway.ts',
    PROJECT_ROOT,
  ),
  frontendWorkflow: new URL(
    'frontend/src/integrations/telegram/setup/telegramAccountSetupWorkflow.ts',
    PROJECT_ROOT,
  ),
  qrArtifact: new URL(
    'frontend/src/integrations/telegram/linking/telegramQrArtifact.ts',
    PROJECT_ROOT,
  ),
  authorizationRealtime: new URL(
    'frontend/src/integrations/telegram/api/telegramAuthorizationRealtime.ts',
    PROJECT_ROOT,
  ),
  qrCoordinator: new URL(
    'frontend/src/integrations/telegram/linking/useTelegramQrPairing.ts',
    PROJECT_ROOT,
  ),
  runtimeRealtime: new URL(
    'src/telegram-runtime/src/client_realtime.rs',
    BACKEND_ROOT,
  ),
};

test('Telegram account identity is user-only and QR authority stays with TDLib', async () => {
  const [
    inventory,
    adr,
    api,
    clientContract,
    clientProto,
    core,
    runtimeBootstrap,
    tdlib,
    frontendGateway,
    frontendWorkflow,
    qrArtifact,
    authorizationRealtime,
    qrCoordinator,
    runtimeRealtime,
  ] = await Promise.all([
    readFile(paths.inventory, 'utf8').then(JSON.parse),
    readFile(paths.adr, 'utf8'),
    readFile(paths.api, 'utf8'),
    readFile(paths.clientContract, 'utf8'),
    readFile(paths.clientProto, 'utf8'),
    readFile(paths.core, 'utf8'),
    readFile(paths.runtimeBootstrap, 'utf8'),
    readFile(paths.tdlib, 'utf8'),
    readFile(paths.frontendGateway, 'utf8'),
    readFile(paths.frontendWorkflow, 'utf8'),
    readFile(paths.qrArtifact, 'utf8'),
    readFile(paths.authorizationRealtime, 'utf8'),
    readFile(paths.qrCoordinator, 'utf8'),
    readFile(paths.runtimeRealtime, 'utf8'),
  ]);

  const gate = inventory.slices.find(
    (slice) => slice.gate === 'telegram_tdlib_user_qr_identity_v1',
  );
  assert.ok(gate, 'missing Telegram user QR identity gate');
  assert.equal(gate.role, 'integration');
  assert.equal(gate.owner, 'telegram');
  assert.equal(gate.state, 'implemented');
  assert.match(adr, /Статус: Принято/);
  assert.match(adr, /Состояние реализации: Implemented/);
  assert.match(adr, /Telegram user account через\s+TDLib/);
  assert.match(adr, /Bot API.*отдельного ADR/s);

  const activeBackend = [api, core, runtimeBootstrap].join('\n');
  assert.doesNotMatch(activeBackend, /TelegramProviderKind/);
  assert.doesNotMatch(activeBackend, /BotToken/);
  assert.doesNotMatch(activeBackend, /telegram_bot(?:_token)?/);
  assert.match(api, /TelegramCredentialPurpose::ApiHash/);
  assert.match(api, /TelegramCredentialPurpose::SessionEncryptionKey/);
  assert.match(api, /if setup\.qr_authorized[\s\S]*InvalidTransition/);

  assert.doesNotMatch(clientProto, /\bstring provider_kind\s*=/);
  assert.match(clientProto, /reserved 2;\s*reserved "provider_kind";/);
  assert.match(clientContract, /TELEGRAM_CLIENT_CONTRACT_REVISION: u32 = 9/);

  assert.doesNotMatch(frontendGateway, /providerKind/);
  assert.match(frontendGateway, /qrAuthorized: false/);
  assert.match(frontendWorkflow, /purposeId: 'telegram_api_hash'/);
  assert.match(frontendWorkflow, /purposeId: 'telegram_session_store_key'/);
  assert.match(frontendWorkflow, /purpose: 'telegram_session_encryption_key'/);
  assert.doesNotMatch(frontendWorkflow, /telegram_bot_token/);

  assert.match(tdlib, /requestQrCodeAuthentication/);
  assert.match(qrArtifact, /QRCode\.toDataURL/);
  assert.match(qrArtifact, /parsed\.protocol === 'tg:'/);
  assert.match(qrArtifact, /parsed\.hostname === 'login'/);
  assert.doesNotMatch(qrArtifact, /\bfetch\s*\(/);
  assert.doesNotMatch(qrArtifact, /window\.open|location\.assign/);
  assert.match(authorizationRealtime, /getBrowserGatewayRealtimeHub/);
  assert.match(authorizationRealtime, /telegram\.authorization\.status_changed\.v1/);
  assert.doesNotMatch(qrCoordinator, /setInterval|setTimeout|poll/i);
  assert.match(runtimeRealtime, /PublishClientRealtime/);
  assert.match(runtimeRealtime, /qr_link: None/);
  assert.match(runtimeRealtime, /password_hint: None/);
  assert.match(adr, /Periodic polling запрещён/);
});
