import assert from 'node:assert/strict';
import { existsSync, globSync, readFileSync } from 'node:fs';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import test from 'node:test';

const PROJECT_ROOT = resolve(import.meta.dirname, '../../..');
const absolute = (path) => resolve(PROJECT_ROOT, path);
const read = (path) => readFile(absolute(path), 'utf8');

const bulkPackages = [
  'makosh-communication-bulk-action-api:contract',
  'makosh-communication-bulk-action-core:implementation',
  'makosh-communication-bulk-action-persistence:persistence',
  'makosh-communication-bulk-action-runtime:runtime',
  'makosh-communication-bulk-action-assembly:assembly',
];

const delayedPackages = [
  'makosh-communication-delayed-delivery-api:contract',
  'makosh-communication-delayed-delivery-core:implementation',
  'makosh-communication-delayed-delivery-persistence:persistence',
  'makosh-communication-delayed-delivery-execution:implementation',
  'makosh-communication-delayed-delivery-event-adapters:implementation',
  'makosh-communication-delayed-delivery-runtime-adapters:implementation',
  'makosh-communication-delayed-delivery-store-adapters:persistence',
  'makosh-communication-delayed-delivery-runtime:runtime',
  'makosh-communication-delayed-delivery-assembly:assembly',
];

const task7Capabilities = [
  'communication.bulk_action.v1',
  'communication.delayed_delivery.blob.v1',
  'communication.delayed_delivery.clock.v1',
  'communication.delayed_delivery.delivery_intent.v1',
  'communication.delayed_delivery.scheduler_due.v1',
  'communication.delayed_delivery.scheduler_receipt.v1',
  'communication.delayed_delivery.scheduler_schedule_command.v1',
  'communication.delayed_delivery.scheduler_schedule_result.v1',
  'communication.delayed_delivery.storage.v1',
  'communication.delayed_delivery.v1',
  'communication_bulk_action.delivery_intent.v1',
  'communication_bulk_action.storage.v1',
];

test('Task 7 atomically admits the two existing workflow owners and exact capabilities', async () => {
  const policy = JSON.parse(await read('backend/architecture/policy.json'));
  const implementation = policy.implementation;

  assert.equal(
    implementation.currentSlice,
    'speech_to_text_whisper_admission_v1',
  );
  assert.equal(implementation.productionPackages.length, 283);
  assert.equal(implementation.ownerInventory.workflows.length, 21);
  assert.equal(implementation.ownerInventory.businessCapabilities.length, 253);
  assert.deepEqual(
    implementation.ownerInventory.workflows.filter((owner) =>
      ['communication_bulk_action', 'communication_delayed_delivery'].includes(owner),
    ),
    ['communication_bulk_action', 'communication_delayed_delivery'],
  );
  assert.deepEqual(
    implementation.ownerInventory.businessCapabilities.filter((capability) =>
      task7Capabilities.includes(capability),
    ),
    task7Capabilities,
  );
});

test('Task 7 keeps the exact existing packages and workspace inventory', async () => {
  const policy = JSON.parse(await read('backend/architecture/policy.json'));
  const packagesFor = (owner) =>
    policy.implementation.productionPackages
      .filter((descriptor) => descriptor.owner === owner)
      .map(({ name, surface }) => `${name}:${surface}`);
  const workspace = await read('backend/Cargo.toml');
  const members = [...workspace.matchAll(/^\s*"([^"]+)",?$/gm)].map(
    (match) => match[1],
  );
  const manifests = globSync('**/Cargo.toml', {
    cwd: absolute('backend'),
    exclude: ['target/**'],
  });

  assert.deepEqual(packagesFor('communication_bulk_action'), bulkPackages);
  assert.deepEqual(packagesFor('communication_delayed_delivery'), delayedPackages);
  assert.equal(members.length, 420);
  assert.equal(manifests.length, 421);
});

test('Task 7 admits only the existing five Connect routes and four release artifacts', async () => {
  const [bulkApi, delayedApi, releaseScript, developmentAssembly] = await Promise.all([
    read('backend/src/communication-bulk-action-api/src/lib.rs'),
    read('backend/src/communication-delayed-delivery-api/src/lib.rs'),
    read('backend/scripts/materialize-dev-release.sh'),
    read('backend/development/assembly/src/main.rs'),
  ]);
  const connectPaths = [
    ...bulkApi.matchAll(/pub const [A-Z_]+_CONNECT_PATH_V1: &str =\s*\n?\s*"([^"]+)"/g),
    ...delayedApi.matchAll(/pub const [A-Z_]+_CONNECT_PATH_V1: &str =\s*\n?\s*"([^"]+)"/g),
  ].map((match) => match[1]);

  assert.deepEqual(connectPaths, [
    '/makosh.communication_bulk_action.v1.CommunicationBulkDeliveryCommandService/Start',
    '/makosh.communication_bulk_action.v1.CommunicationBulkDeliveryQueryService/GetStatus',
    '/makosh.communication_delayed_delivery.v1.CommunicationDelayedDeliveryCommandService/Schedule',
    '/makosh.communication_delayed_delivery.v1.CommunicationDelayedDeliveryCommandService/Cancel',
    '/makosh.communication_delayed_delivery.v1.CommunicationDelayedDeliveryQueryService/GetStatus',
  ]);
  for (const artifact of [
    'communication_bulk_action.runtime.v1',
    'communication_bulk_action.storage.v1',
    'communication_delayed_delivery.runtime.v1',
    'communication_delayed_delivery.storage.v1',
  ]) {
    assert.equal(developmentAssembly.includes(artifact), true, artifact);
  }
  assert.equal(releaseScript.includes('communication_bulk_action.release-artifacts.json'), true);
  assert.equal(
    releaseScript.includes('communication_delayed_delivery.release-artifacts.json'),
    true,
  );
});

test('Task 7 adds no frontend or legacy compatibility surface', () => {
  const generated = globSync('src/gen/**/*{bulk,delayed}*', {
    cwd: absolute('frontend'),
  });
  const frontendSources = globSync('src/**/*.{ts,vue}', {
    cwd: absolute('frontend'),
  });
  const compatibilityMatches = frontendSources.flatMap((path) => {
    const source = readFileSync(absolute(`frontend/${path}`), 'utf8');
    return source.match(/\/api\/v1\/(?:bulk|delayed)[^'"\s]*/g) ?? [];
  });

  assert.deepEqual(generated, []);
  assert.deepEqual(compatibilityMatches, []);
  assert.equal(existsSync(absolute('frontend/src/domains/communication-bulk-action')), false);
  assert.equal(existsSync(absolute('frontend/src/domains/communication-delayed-delivery')), false);
});
