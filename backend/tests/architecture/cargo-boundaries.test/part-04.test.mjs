import assert from 'node:assert/strict';
import test from 'node:test';

import { validateCargoMetadata } from '../../../scripts/lib/cargo-boundaries.mjs';
import {
  codes,
  dependency,
  kernel,
  metadata as fixtureMetadata,
  runtimeProtocol,
  storagePackages,
  storageProtocol,
  vaultPackages,
  vaultProtocol,
  workspacePackage,
} from '../support/cargo-fixtures.mjs';
import { canonicalPolicyForTests } from '../support/canonical-policy.mjs';

import { eventsProtocol, metadata } from './support.mjs';

test('allows a phase-specific subset of constitutional Kernel components', () => {
  const packages = [
    kernel([
      dependency('makosh-events-protocol'),
      dependency('makosh-runtime-protocol'),
    ], {
      components: ['supervisor', 'core_gateway'],
    }),
    workspacePackage('makosh-events-protocol', {
      role: 'platform',
      owner: 'events',
      surface: 'contract',
    }),
    runtimeProtocol(),
  ];

  assert.deepEqual(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)), []);
});


for (const forbiddenDependency of ['async-nats', 'nats', 'sqlx', 'tokio-postgres', 'postgres', 'diesel', 'sea-orm']) {
  test(`keeps Telemetry Collector independent of ${forbiddenDependency}`, () => {
    const packages = [
      kernel(),
      workspacePackage(
        'makosh-telemetry-collector',
        {
          role: 'platform',
          owner: 'telemetry',
          surface: 'runtime',
          components: ['telemetry_collector'],
        },
        [dependency(forbiddenDependency)],
      ),
    ];

    assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages))).has('telemetry_dependency'));
  });
}



test('prevents a telemetry implementation helper from bypassing collector dependency rules', () => {
  const packages = [
    kernel(),
    workspacePackage(
      'makosh-telemetry-exporter',
      {
        role: 'platform',
        owner: 'telemetry',
        surface: 'implementation',
      },
      [dependency('sqlx')],
    ),
  ];

  assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages))).has('telemetry_dependency'));
});



test('prevents Kernel from linking Telemetry Collector implementation', () => {
  const collector = workspacePackage('makosh-telemetry-collector', {
    role: 'platform',
    owner: 'telemetry',
    surface: 'runtime',
    components: ['telemetry_collector'],
  });
  const packages = [kernel([dependency('makosh-telemetry-collector')]), collector];

  assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages))).has('implementation_dependency'));
});



test('allows test support only through a dev dependency', () => {
  const support = workspacePackage('makosh-test-support', {
    role: 'test',
    owner: 'test',
    surface: 'test_support',
  });

  const allowed = [
    kernel(),
    support,
    workspacePackage(
      'makosh-persons-runtime',
      { role: 'domain', owner: 'persons', surface: 'runtime' },
      [dependency('makosh-test-support', 'dev')],
    ),
  ];
  assert.deepEqual(validateCargoMetadata(canonicalPolicyForTests(), metadata(allowed)), []);

  const forbidden = [
    kernel(),
    support,
    workspacePackage(
      'makosh-persons-runtime',
      { role: 'domain', owner: 'persons', surface: 'runtime' },
      [dependency('makosh-test-support')],
    ),
  ];
  assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(forbidden))).has('production_test_dependency'));
});



test('rejects production use of test support from build dependencies', () => {
  const support = workspacePackage('makosh-test-support', {
    role: 'test',
    owner: 'test',
    surface: 'test_support',
  });
  const packages = [
    kernel(),
    support,
    workspacePackage(
      'makosh-contacts-runtime',
      { role: 'domain', owner: 'contacts', surface: 'runtime' },
      [dependency('makosh-test-support', 'build')],
    ),
  ];

  assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages))).has('production_test_dependency'));
});
