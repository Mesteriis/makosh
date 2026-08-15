import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import test from 'node:test';

const root = new URL('../../..', import.meta.url);
const read = (path) => readFileSync(new URL(path, root), 'utf8');
const policy = JSON.parse(read('backend/architecture/policy.json'));

test('final matrix preserves the last authorized production admission', () => {
  assert.equal(policy.implementation.currentSlice, 'speech_to_text_whisper_admission_v1');
  assert.equal(policy.implementation.productionPackages.length, 283);
  assert.equal(policy.implementation.ownerInventory.businessCapabilities.length, 253);
});

test('final matrix counts admitted staged and support packages exactly', () => {
  const metadata = JSON.parse(execFileSync('cargo', ['metadata', '--no-deps', '--format-version', '1'], {
    cwd: new URL('backend', root), encoding: 'utf8', maxBuffer: 32 * 1024 * 1024,
  }));
  const metadataKey = policy.cargo.metadataKey;
  const workspaceIds = new Set(metadata.workspace_members);
  const workspace = metadata.packages.filter(({ id }) => workspaceIds.has(id));
  const production = workspace.filter(({ metadata: packageMetadata }) => {
    const role = packageMetadata?.[metadataKey]?.role;
    return role !== policy.owners.test && role !== policy.owners.development;
  });
  const manifests = execFileSync('rg', ['--files', 'backend', '-g', 'Cargo.toml'], {
    cwd: root, encoding: 'utf8',
  }).trim().split('\n');

  assert.equal(workspace.length, 420);
  assert.equal(manifests.length, 421);
  assert.equal(production.length, 403);
  assert.equal(production.length - policy.implementation.productionPackages.length, 120);
  assert.equal(workspace.length - production.length, 17);
});

test('every completed implementation task has a durable report', () => {
  for (let task = 0; task <= 26; task += 1) {
    if (task === 5) {
      for (const part of ['5a', '5b', '5c']) {
        assert.ok(existsSync(new URL(`.superpowers/sdd/reference-clean-room-implementation/task-${part}-report.md`, root)), `task ${part}`);
      }
      continue;
    }
    assert.ok(existsSync(new URL(`.superpowers/sdd/reference-clean-room-implementation/task-${task}-report.md`, root)), `task ${task}`);
  }
});

test('final active docs state admitted staged and blocked evidence without overclaim', () => {
  const protectedAudit = readFileSync(new URL('REFERENCE_CLEAN_ROOM_CAPABILITY_AUDIT.md', root));
  assert.equal(
    createHash('sha256').update(protectedAudit).digest('hex'),
    'af622174e06f3e32e7852c38f7db05b0cf5f6e1ad4c96c313e1bf81ae383983b',
  );

  for (const path of ['README.md', 'backend/README.md', 'AGENTS.md', 'CORE_CLOSURE_PLAN.md']) {
    const text = read(path);
    assert.match(text, /speech_to_text_whisper_admission_v1/);
    assert.match(text, /implemented-not-admitted/);
    assert.match(text, /Telegram/);
    assert.match(text, /Zoom/);
    assert.match(text, /Telemost/);
    assert.match(text, /OmniRoute/);
    assert.doesNotMatch(text, /make -C backend architecture-check/);
  }
});

test('final report records the external resumption boundary and validation truth', () => {
  const report = read('.superpowers/sdd/reference-clean-room-implementation/task-27-report.md');
  assert.match(report, /120 implemented-not-admitted/);
  assert.match(report, /Apple\/Xcode\/Telegram/);
  assert.match(report, /Zoom.*Telemost.*OmniRoute/s);
  assert.match(report, /cargo-boundaries-check.*RED/s);
  assert.match(report, /make pre-push/);
});
