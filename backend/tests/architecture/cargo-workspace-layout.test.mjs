import assert from 'node:assert/strict';
import test from 'node:test';

import {
  validateCargoMetadata,
  validateWorkspaceManifestCoverage,
  validateWorkspacePackageRoots,
} from '../../scripts/lib/cargo-boundaries.mjs';
import { associateSqlWithWorkspace } from '../../scripts/lib/cargo-workspace.mjs';
import {
  codes,
  dependency,
  kernel,
  metadata,
  runtimeProtocol,
  workspacePackage,
} from './support/cargo-fixtures.mjs';
import { canonicalPolicyForTests } from './support/canonical-policy.mjs';

test('accepts a minimal clean-room workspace', () => {
  const packages = [
    kernel([
      dependency('makosh-events-protocol'),
      dependency('makosh-runtime-protocol'),
    ]),
    workspacePackage('makosh-events-protocol', {
      role: 'platform',
      owner: 'events',
      surface: 'contract',
    }),
    runtimeProtocol(),
    workspacePackage('makosh-contacts-contracts', {
      role: 'domain',
      owner: 'contacts',
      surface: 'contract',
    }),
    workspacePackage('makosh-ai-runtime', {
      role: 'domain',
      owner: 'ai',
      surface: 'runtime',
    }),
    workspacePackage('makosh-telemetry-collector', {
      role: 'platform',
      owner: 'telemetry',
      surface: 'runtime',
      components: ['telemetry_collector'],
    }),
  ];

  assert.deepEqual(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)), []);
});

test('rejects a production Cargo manifest hidden outside workspace membership', () => {
  const violations = validateWorkspaceManifestCoverage(
    ['kernel/Cargo.toml', 'modules/tasks/Cargo.toml'],
    [],
    [],
    ['kernel/Cargo.toml'],
  );

  assert.equal(violations.length, 1);
  assert.equal(violations[0].code, 'orphan_cargo_manifest');
  assert.equal(violations[0].location, 'modules/tasks/Cargo.toml');
});

test('accepts production Cargo manifests registered in the workspace', () => {
  assert.deepEqual(
    validateWorkspaceManifestCoverage(
      ['kernel/Cargo.toml', 'modules/tasks/Cargo.toml'],
      [],
      [],
      ['kernel/Cargo.toml', 'modules/tasks/Cargo.toml'],
    ),
    [],
  );
});

test('rejects a workspace package outside configured production roots', () => {
  const violations = validateWorkspaceManifestCoverage(
    ['kernel/Cargo.toml'],
    [],
    [],
    ['kernel/Cargo.toml', 'experiments/hidden/Cargo.toml'],
  );

  assert.equal(violations.length, 1);
  assert.equal(violations[0].code, 'unscoped_workspace_package');
  assert.equal(violations[0].location, 'experiments/hidden/Cargo.toml');
});

test('accepts test-support manifests in the dedicated test-only workspace root', () => {
  assert.deepEqual(
    validateWorkspaceManifestCoverage(
      ['src/kernel/Cargo.toml'],
      ['tests/support/makosh-test-support/Cargo.toml'],
      [],
      ['src/kernel/Cargo.toml', 'tests/support/makosh-test-support/Cargo.toml'],
    ),
    [],
  );
});

test('accepts explicit development unit roots and rejects production roles there', () => {
  assert.deepEqual(
    validateWorkspaceManifestCoverage(
      ['src/kernel/Cargo.toml'],
      [],
      ['development/runtime/Cargo.toml', 'development/assembly/Cargo.toml'],
      [
        'src/kernel/Cargo.toml',
        'development/runtime/Cargo.toml',
        'development/assembly/Cargo.toml',
      ],
    ),
    [],
  );

  const violations = validateWorkspacePackageRoots(
    canonicalPolicyForTests(),
    [{ manifest: 'development/runtime/Cargo.toml', role: 'platform' }],
    new Set(),
    new Set(),
    new Set(['development/runtime/Cargo.toml']),
  );
  assert.ok(codes(violations).has('production_package_in_development_root'));
});

test('rejects production roles in the test-only workspace root', () => {
  const violations = validateWorkspacePackageRoots(
    canonicalPolicyForTests(),
    [{
      manifest: 'tests/support/makosh-tasks/Cargo.toml',
      role: 'domain',
    }],
    new Set(),
    new Set(['tests/support/makosh-tasks/Cargo.toml']),
    new Set(),
  );

  assert.ok(codes(violations).has('production_package_in_test_root'));
});

test('rejects test roles in the production workspace root', () => {
  const violations = validateWorkspacePackageRoots(
    canonicalPolicyForTests(),
    [{
      manifest: 'src/test-support/Cargo.toml',
      role: 'test',
    }],
    new Set(['src/test-support/Cargo.toml']),
    new Set(),
    new Set(),
  );

  assert.ok(codes(violations).has('test_package_in_production_root'));
});

test('associates SQL with the owning workspace package metadata', () => {
  const tasks = workspacePackage('makosh-tasks-persistence', {
    role: 'domain',
    owner: 'tasks',
    surface: 'persistence',
  });
  tasks.manifest_path = '/workspace/modules/tasks/Cargo.toml';

  assert.deepEqual(
    associateSqlWithWorkspace(
      [{ path: 'modules/tasks/migrations/0001.sql', content: 'CREATE TABLE tasks_items ();' }],
      metadata([tasks]),
      '/workspace',
      'makosh',
    ),
    [{
      path: 'modules/tasks/migrations/0001.sql',
      content: 'CREATE TABLE tasks_items ();',
      packageName: 'makosh-tasks-persistence',
      role: 'domain',
      owner: 'tasks',
      surface: 'persistence',
    }],
  );
});
