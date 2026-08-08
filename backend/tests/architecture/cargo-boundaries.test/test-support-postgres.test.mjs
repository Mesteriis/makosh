import assert from 'node:assert/strict';
import test from 'node:test';

import { validateCargoMetadata } from '../../../scripts/lib/cargo-boundaries.mjs';
import {
  codes,
  dependency,
  kernel,
  metadata,
  runtimeProtocol,
  workspacePackage,
} from '../support/cargo-fixtures.mjs';
import { canonicalPolicyForTests } from '../support/canonical-policy.mjs';

test('allows sqlx only for exact test-only PostgreSQL conformance kits', () => {
  for (const name of ['makosh-events-jetstream-testkit', 'makosh-scheduler-testkit']) {
    const allowed = [...basePackages(), testSupport([dependency('sqlx', 'dev')], name)];
    assert.deepEqual(validateCargoMetadata(canonicalPolicyForTests(), metadata(allowed)), []);
  }

  for (const [name, metadataOverrides, kind] of [
    ['makosh-scheduler-testkit', {}, null],
    ['makosh-scheduler-testkit', {}, 'build'],
    ['makosh-events-jetstream-testkit', { surface: 'implementation' }, 'dev'],
    ['makosh-other-testkit', {}, 'dev'],
  ]) {
    const packages = [
      ...basePackages(),
      testSupport([dependency('sqlx', kind)], name, metadataOverrides),
    ];
    assert.ok(
      codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)))
        .has('storage_dependency'),
    );
  }
});

function basePackages() {
  return [
    kernel(),
    workspacePackage('makosh-events-protocol', {
      role: 'platform', owner: 'events', surface: 'contract',
    }),
    runtimeProtocol(),
  ];
}

function testSupport(dependencies, name = 'makosh-events-jetstream-testkit', overrides = {}) {
  return workspacePackage(
    name,
    { role: 'test', owner: 'test', surface: 'test_support', ...overrides },
    dependencies,
  );
}
