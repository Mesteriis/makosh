import assert from 'node:assert/strict';
import { readdirSync, readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const backendRoot = join(dirname(fileURLToPath(import.meta.url)), '..', '..');

test('every Communications integration relay publishes exact durable envelopes', () => {
  const canonicalRelay = readFileSync(
    join(backendRoot, 'src', 'platform', 'events', 'src', 'delivery', 'relay.rs'),
    'utf8',
  );
  assert.match(canonicalRelay, /publisher\.publish_exact\(entry\.record\(\)\)\.await/);
  assert.match(canonicalRelay, /store\.mark_published\(&entry, &receipt\)\.await/);

  for (const owner of ['mail', 'telegram', 'zulip', 'whatsapp']) {
    const source = readFileSync(
      join(backendRoot, 'src', `${owner}-runtime`, 'src', 'communications_outbox.rs'),
      'utf8',
    );

    const publishesInline =
      /publish_exact\(permit, record\.exact_bytes\(\)\)/.test(source)
      && /mark_communications_outbox_published\(record\.message_id\(\), published_at_unix_seconds\)/.test(source);
    const delegatesToCanonicalRelay =
      /RuntimeOutboxPublisherV1::new\(connection, permit\)/.test(source)
      && /relay_once\(&mut store, &publisher\)\.await/.test(source);
    assert.ok(
      publishesInline || delegatesToCanonicalRelay,
      `${owner} relay neither publishes exact bytes inline nor delegates to the canonical exact relay`,
    );
  }
});

test('integration packages reach Communications only through explicit public contract units', () => {
  const communicationsPackages = new Set([
    'makosh-communications-api',
    'makosh-communications-attachment-contract',
    'makosh-communications-domain',
    'makosh-communications-ingress',
    'makosh-communications-persistence',
    'makosh-communications-runtime',
  ]);

  const integrationManifests = readdirSync(join(backendRoot, 'src'), { withFileTypes: true })
    .filter((entry) => entry.isDirectory())
    .map((entry) => join(backendRoot, 'src', entry.name, 'Cargo.toml'))
    .filter((path) => {
      try {
        return readFileSync(path, 'utf8').includes('role = "integration"');
      } catch {
        return false;
      }
    });

  assert.ok(integrationManifests.length > 0, 'missing integration package manifests');
  for (const manifestPath of integrationManifests) {
    const manifest = readFileSync(manifestPath, 'utf8');
    const communicationsDependencies = [...manifest.matchAll(/^([\w-]+)\s*=.*$/gm)]
      .map((match) => match[1])
      .filter((name) => communicationsPackages.has(name));
    assert.ok(
      communicationsDependencies.every((name) => [
        'makosh-communications-attachment-contract',
        'makosh-communications-ingress',
      ].includes(name)),
      `${manifestPath} has a direct Communications implementation edge`,
    );
  }
});
