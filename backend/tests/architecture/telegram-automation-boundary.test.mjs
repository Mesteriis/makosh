import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);

async function source(path) {
  return readFile(new URL(path, BACKEND_ROOT), 'utf8');
}

test('Telegram automation packages are independent integration build units', async () => {
  const [apiManifest, coreManifest, persistenceManifest, runtimeManifest] = await Promise.all([
    source('src/telegram-automation-api/Cargo.toml'),
    source('src/telegram-automation-core/Cargo.toml'),
    source('src/telegram-automation-persistence/Cargo.toml'),
    source('src/telegram-runtime/Cargo.toml'),
  ]);

  for (const manifest of [apiManifest, coreManifest, persistenceManifest]) {
    assert.match(manifest, /role = "integration"/);
    assert.match(manifest, /owner = "telegram"/);
    assert.doesNotMatch(manifest, /communications-domain|kernel|gateway/);
  }
  assert.doesNotMatch(apiManifest, /telegram-automation-core|sqlx|telegram-runtime/);
  assert.doesNotMatch(coreManifest, /sqlx|prost|telegram-runtime|telegram-persistence/);
  assert.match(persistenceManifest, /makosh-telegram-automation-core/);
  assert.doesNotMatch(persistenceManifest, /makosh-telegram-api|makosh-telegram-tdlib/);
  assert.match(runtimeManifest, /makosh-telegram-automation-api/);
  assert.match(runtimeManifest, /makosh-telegram-automation-core/);
  assert.match(runtimeManifest, /makosh-telegram-automation-persistence/);
});

test('Telegram automation exposes exact query and command routes without a generic engine', async () => {
  const [contract, proto, admission, runtimePort] = await Promise.all([
    source('src/telegram-automation-api/src/contract.rs'),
    source('src/telegram-automation-api/proto/makosh/telegram/automation/v1/automation.proto'),
    source('src/telegram-runtime/src/admission.rs'),
    source('src/telegram-runtime/src/automation_client_port.rs'),
  ]);

  for (const identity of [
    'telegram.automation.query.v1',
    'telegram.automation.command.v1',
  ]) {
    assert.match(contract, new RegExp(identity.replaceAll('.', '\\.')));
    assert.match(admission, /TelegramAutomationContractV1/);
  }
  assert.match(proto, /service TelegramAutomationQueryService/);
  assert.match(proto, /service TelegramAutomationCommandService/);
  assert.doesNotMatch(proto, /\bgoogle\.protobuf\.Any\b|\bmap\s*</);
  assert.doesNotMatch(runtimePort, /makosh_communications|makosh_telegram_tdlib/);
  assert.doesNotMatch(runtimePort, /serde_json|Value|HashMap/);
});

test('Telegram assembly composes automation storage without becoming runtime', async () => {
  const [assemblyManifest, assembly, schema] = await Promise.all([
    source('src/telegram-assembly/Cargo.toml'),
    source('src/telegram-assembly/src/lib.rs'),
    source('src/telegram-automation-persistence/src/schema.rs'),
  ]);

  assert.match(assemblyManifest, /surface = "assembly"/);
  assert.match(assembly, /telegram_storage_bundle_with_automation_v2/);
  assert.match(assembly, /telegram_automation_storage_migration_v1/);
  assert.match(schema, /telegram_automation_templates/);
  assert.match(schema, /telegram_automation_preview_receipts/);
  assert.doesNotMatch(schema, /makosh_data\.communications_/);
  assert.doesNotMatch(assembly, /render_preview|AutomationPolicyDraft/);
});
