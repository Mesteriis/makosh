import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const composePath = new URL('../../development/compose.yaml', import.meta.url);

test('clean-room development Compose stays loopback-only and excludes Docker control', async () => {
  const compose = await readFile(composePath, 'utf8');

  assert.match(compose, /^name: makosh-platform-development$/m);
  assert.match(compose, /^  postgres:$/m);
  assert.match(compose, /^  nats:$/m);
  assert.match(compose, /127\.0\.0\.1:35432:5432/);
  assert.match(compose, /127\.0\.0\.1:34222:4222/);
  assert.match(compose, /127\.0\.0\.1:36532:6432/);
  assert.doesNotMatch(compose, /internal: true/);
  assert.doesNotMatch(compose, /docker\.sock|DOCKER_HOST|privileged:/i);
  assert.doesNotMatch(compose, /\.\.?\/docker\//);
});
