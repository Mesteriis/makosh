import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import test from 'node:test';

const backend = new URL('../../', import.meta.url);

function cargo(arguments_) {
  return spawnSync('cargo', ['+1.97.0', ...arguments_], {
    cwd: backend,
    encoding: 'utf8',
  });
}

test('production Kernel CLI exposes only the approved operational surface', () => {
  const result = cargo(['run', '-q', '-p', 'makosh-kernel', '--', '--help']);
  assert.equal(result.status, 0, result.stderr);
  for (const command of [
    'status',
    'serve',
    'device-key-generate',
    'initial-owner-enroll',
    'server-bootstrap-pairing',
    'control-store',
  ]) {
    assert.match(result.stdout, new RegExp(`^  ${command}\\s`, 'm'));
  }
  for (const forbidden of [
    '--development-profile',
    'hold-lock',
    'initial-owner-import-pairing',
    'module-register',
    'module-approve',
    'module-transition',
  ]) {
    assert.doesNotMatch(result.stdout, new RegExp(forbidden));
  }
});

test('production Kernel dependency graph excludes the development operator', () => {
  const result = cargo(['metadata', '--locked', '--format-version', '1', '--no-deps']);
  assert.equal(result.status, 0, result.stderr);
  const metadata = JSON.parse(result.stdout);
  const kernel = metadata.packages.find(({ name }) => name === 'makosh-kernel');
  assert.ok(kernel);
  const dependencies = new Set(kernel.dependencies.map(({ name }) => name));
  assert.equal(dependencies.has('makosh-development-kernel-operator'), false);
});
