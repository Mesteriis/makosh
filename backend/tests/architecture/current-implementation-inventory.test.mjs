import assert from 'node:assert/strict';
import test from 'node:test';

import {
  validateCargoMetadata,
  validateCurrentImplementationInventory,
  validateCurrentImplementationSourceCoverage,
} from '../../scripts/lib/cargo-boundaries.mjs';
import {
  codes,
  dependency,
  metadata,
  workspacePackage,
} from './support/cargo-fixtures.mjs';
import { canonicalPolicyForTests } from './support/canonical-policy.mjs';

function recoveryOnlyPackages() {
  const policy = canonicalPolicyForTests();
  return policy.implementation.productionPackages.map((descriptor) => {
    const makosh = {
      role: descriptor.role,
      owner: descriptor.owner,
      surface: descriptor.surface,
    };
    if (descriptor.name === policy.kernel.package) {
      makosh.components = [...policy.implementation.kernelProfile.activeComponents];
    }
    if (descriptor.name === policy.vault.runtimePackage) {
      makosh.components = [policy.vault.runtimeComponent];
    }
    if (descriptor.name === policy.storage.runtimePackage) {
      makosh.components = [policy.storage.runtimeComponent];
    }
    const dependencies = policy.implementation.workspaceDependencyAllowlist[descriptor.name]
      .map(({ name, kind }) => dependency(name, kind));
    dependencies.push(
      ...policy.implementation.thirdPartyDependencyAllowlist[descriptor.name]
        .map(reviewedCratesIoDependency),
    );
    return {
      ...workspacePackage(descriptor.name, makosh, dependencies),
      targets: [{ kind: [policy.implementation.targetPolicy[descriptor.name].primaryKind] }],
      features: structuredClone(
        policy.implementation.cargoFeatureAllowlist[descriptor.name] ?? {},
      ),
    };
  });
}

function cratesIoDependency(name, kind = 'normal', overrides = {}) {
  return {
    ...dependency(name, kind),
    source: 'registry+https://github.com/rust-lang/crates.io-index',
    path: null,
    req: '=0.0.0',
    uses_default_features: false,
    features: [],
    ...overrides,
  };
}

function reviewedCratesIoDependency(specification) {
  return cratesIoDependency(specification.name, specification.kind, {
    req: specification.version,
    uses_default_features: specification.defaultFeatures,
    features: [...specification.features],
  });
}

test('allows an empty workspace before the first production package exists', () => {
  assert.deepEqual(
    validateCurrentImplementationInventory(canonicalPolicyForTests(), metadata([])),
    [],
  );
});

test('allows an empty production source root before the first package exists', () => {
  assert.deepEqual(
    validateCurrentImplementationSourceCoverage(
      canonicalPolicyForTests(),
      [],
      [],
    ),
    [],
  );
});

test('requires every production source file to belong to an authorized package root', () => {
  const packageRoots = [{
    name: 'makosh-kernel',
    role: 'core',
    root: 'src/core/kernel/runtime',
  }];
  const ownedEntries = [
    { path: 'src/core/kernel/runtime/Cargo.toml', isDirectory: false },
    { path: 'src/core/kernel/runtime/src/main.rs', isDirectory: false },
    { path: 'src/core/kernel/runtime/src', isDirectory: true },
  ];

  assert.deepEqual(
    validateCurrentImplementationSourceCoverage(
      canonicalPolicyForTests(),
      ownedEntries,
      packageRoots,
    ),
    [],
  );

  for (const hiddenPath of [
    'src/domains/tasks/src/lib.rs',
    'src/core/kernel/runtime-copy/src/main.rs',
    'src/platform/events/proto/hidden.proto',
    'src/build.rs',
  ]) {
    assert.ok(codes(validateCurrentImplementationSourceCoverage(
      canonicalPolicyForTests(),
      [{ path: hiddenPath, isDirectory: false }],
      packageRoots,
    )).has('implementation_source_coverage'));
  }
});

test('allows staged integration and engine sources outside the active owner inventory only', () => {
  const policy = canonicalPolicyForTests();
  const integration = workspacePackage('makosh-telegram-core', {
    role: 'integration',
    owner: 'telegram',
    surface: 'implementation',
  });
  const engine = workspacePackage('makosh-content-security-core', {
    role: 'engine',
    owner: 'content_security',
    surface: 'implementation',
  });
  const extraDomain = workspacePackage('makosh-extra-domain', {
    role: 'domain',
    owner: 'extra',
    surface: 'implementation',
  });

  assert.deepEqual(
    validateCurrentImplementationInventory(policy, metadata([...recoveryOnlyPackages(), integration])),
    [],
  );
  assert.deepEqual(
    validateCurrentImplementationInventory(policy, metadata([...recoveryOnlyPackages(), engine])),
    [],
  );
  assert.deepEqual(
    validateCurrentImplementationSourceCoverage(policy, [
      { path: 'src/telegram-core/Cargo.toml', isDirectory: false },
      { path: 'src/telegram-core/src/lib.rs', isDirectory: false },
    ], [{
      name: 'makosh-telegram-core',
      role: 'integration',
      root: 'src/telegram-core',
    }]),
    [],
  );
  assert.deepEqual(
    validateCurrentImplementationSourceCoverage(policy, [
      { path: 'src/content-security-core/Cargo.toml', isDirectory: false },
      { path: 'src/content-security-core/src/lib.rs', isDirectory: false },
    ], [{
      name: 'makosh-content-security-core',
      role: 'engine',
      root: 'src/content-security-core',
    }]),
    [],
  );
  assert.ok(codes(validateCurrentImplementationInventory(
    policy,
    metadata([...recoveryOnlyPackages(), extraDomain]),
  )).has('implementation_inventory'));
  assert.ok(codes(validateCurrentImplementationSourceCoverage(policy, [
    { path: 'src/extra-domain/src/lib.rs', isDirectory: false },
  ], [{ name: 'makosh-extra-domain', role: 'domain', root: 'src/extra-domain' }])).has('implementation_source_coverage'));
});

test('ignores a test-only workspace when enforcing the production slice', () => {
  const packages = [workspacePackage('makosh-test-support', {
    role: 'test',
    owner: 'test',
    surface: 'test_support',
  })];
  assert.deepEqual(
    validateCurrentImplementationInventory(canonicalPolicyForTests(), metadata(packages)),
    [],
  );
  assert.deepEqual(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)), []);
});

test('allows only the explicit development runtime and assembly outside production inventory', () => {
  const policy = canonicalPolicyForTests();
  const development = policy.implementation.developmentProfile.packages.map((entry) => (
    workspacePackage(entry.package, {
      role: 'development',
      owner: 'development',
      surface: entry.surface,
      components: [],
    })
  ));
  assert.deepEqual(
    validateCurrentImplementationInventory(policy, metadata([...recoveryOnlyPackages(), ...development])),
    [],
  );

  development[1].metadata.makosh.components = ['hidden_component'];
  assert.ok(codes(validateCurrentImplementationInventory(
    policy,
    metadata([...recoveryOnlyPackages(), ...development]),
  )).has('development_runtime_inventory'));
});

test('accepts exactly the current production inventory', () => {
  assert.deepEqual(
    validateCurrentImplementationInventory(
      canonicalPolicyForTests(),
      metadata(recoveryOnlyPackages()),
    ),
    [],
  );
});

test('rejects missing or extra production packages', () => {
  const missing = recoveryOnlyPackages().slice(1);
  const extra = [
    ...recoveryOnlyPackages(),
    workspacePackage('makosh-storage-protocol', {
      role: 'platform',
      owner: 'storage',
      surface: 'contract',
    }),
  ];

  assert.ok(codes(validateCurrentImplementationInventory(
    canonicalPolicyForTests(),
    metadata(missing),
  )).has('implementation_inventory'));
  assert.ok(codes(validateCurrentImplementationInventory(
    canonicalPolicyForTests(),
    metadata(extra),
  )).has('implementation_inventory'));
});

test('rejects descriptor drift in an authorized package', () => {
  const packages = recoveryOnlyPackages();
  packages[0].metadata.makosh.owner = 'telemetry';

  assert.ok(codes(validateCurrentImplementationInventory(
    canonicalPolicyForTests(),
    metadata(packages),
  )).has('implementation_inventory'));
});

test('requires only the active recovery-only Kernel components', () => {
  for (const components of [
    ['supervisor'],
    ['supervisor', 'core_gateway', 'event_hub'],
    ['core_gateway', 'supervisor'],
  ]) {
    const packages = recoveryOnlyPackages();
    packages.find(({ name }) => name === 'makosh-kernel').metadata.makosh.components = components;

    assert.ok(codes(validateCurrentImplementationInventory(
      canonicalPolicyForTests(),
      metadata(packages),
    )).has('implementation_inventory'));
  }
});

test('allows only the declared internal current-slice dependency graph', () => {
  const packages = recoveryOnlyPackages();

  assert.deepEqual(validateCurrentImplementationInventory(
    canonicalPolicyForTests(),
    metadata(packages),
  ), []);
});

for (const [packageName, dependencyName] of [
  ['makosh-kernel', 'makosh-events-protocol'],
  ['makosh-events-protocol', 'makosh-kernel'],
  ['makosh-kernel', 'async-nats'],
  ['makosh-kernel-control-store-sqlite', 'sqlx'],
  ['makosh-kernel', 'reqwest'],
  ['makosh-kernel', 'teloxide'],
  ['makosh-kernel', 'mongodb'],
  ['makosh-kernel', 'makosh-vault-protocol'],
  ['makosh-kernel', 'makosh-provider-zulip'],
]) {
  test(`rejects ${packageName} dependency on ${dependencyName} in recovery-only`, () => {
    const packages = recoveryOnlyPackages();
    packages.find(({ name }) => name === packageName).dependencies = [
      dependency(dependencyName),
    ];

    assert.ok(codes(validateCurrentImplementationInventory(
      canonicalPolicyForTests(),
      metadata(packages),
    )).has('implementation_dependency'));
  });
}

test('allows only reviewed crates.io dependencies with their exact dependency kind', () => {
  const packages = recoveryOnlyPackages();

  assert.deepEqual(validateCurrentImplementationInventory(
    canonicalPolicyForTests(),
    metadata(packages),
  ), []);

  const wrongKind = recoveryOnlyPackages();
  wrongKind.find(({ name }) => name === 'makosh-events-protocol')
    .dependencies.find(({ name }) => name === 'prost-build').kind = 'normal';
  assert.ok(codes(validateCurrentImplementationInventory(
    canonicalPolicyForTests(),
    metadata(wrongKind),
  )).has('implementation_dependency'));

  const renamed = recoveryOnlyPackages();
  renamed.find(({ name }) => name === 'makosh-events-protocol')
    .dependencies.find(({ name }) => name === 'prost').rename = 'wire';
  assert.ok(codes(validateCurrentImplementationInventory(
    canonicalPolicyForTests(),
    metadata(renamed),
  )).has('implementation_dependency'));
});

test('rejects a registry URL that only looks like crates.io', () => {
  const packages = recoveryOnlyPackages();
  packages.find(({ name }) => name === 'makosh-kernel')
    .dependencies.find(({ name }) => name === 'clap').source =
      'registry+https://packages.example.invalid/crates.io-index';

  assert.ok(codes(validateCurrentImplementationInventory(
    canonicalPolicyForTests(),
    metadata(packages),
  )).has('implementation_dependency'));
});

test('rejects a reviewed dependency resolved through a patched source', () => {
  const cargoMetadata = metadata(recoveryOnlyPackages());
  cargoMetadata.packages.find(({ name }) => name === 'clap').source =
    'git+https://example.invalid/patched-clap';

  assert.ok(codes(validateCurrentImplementationInventory(
    canonicalPolicyForTests(),
    cargoMetadata,
  )).has('implementation_dependency'));
});

test('requires exact third-party versions, default features and feature sets', () => {
  const mutations = [
    (dependency) => { dependency.req = '^4'; },
    (dependency) => { dependency.uses_default_features = true; },
    (dependency) => { dependency.features.push('env'); },
  ];

  for (const mutate of mutations) {
    const packages = recoveryOnlyPackages();
    const clap = packages.find(({ name }) => name === 'makosh-kernel')
      .dependencies.find(({ name }) => name === 'clap');
    mutate(clap);

    assert.ok(codes(validateCurrentImplementationInventory(
      canonicalPolicyForTests(),
      metadata(packages),
    )).has('implementation_dependency'));
  }
});

test('does not accept a registry namesake as a required workspace package-ID edge', () => {
  const packages = recoveryOnlyPackages();
  const kernel = packages.find(({ name }) => name === 'makosh-kernel');
  const runtimeIndex = kernel.dependencies.findIndex(
    ({ name }) => name === 'makosh-runtime-protocol',
  );
  kernel.dependencies[runtimeIndex] = cratesIoDependency(
    'makosh-runtime-protocol',
    'normal',
    { rename: 'evil_runtime', req: '=999.0.0' },
  );

  assert.ok(codes(validateCurrentImplementationInventory(
    canonicalPolicyForTests(),
    metadata(packages),
  )).has('implementation_dependency'));
});

test('rejects Cargo feature definitions outside the exact current-slice allowlist', () => {
  const packages = recoveryOnlyPackages();
  packages.find(({ name }) => name === 'makosh-kernel').features = {
    nats_data_plane_v1: [],
  };

  assert.ok(codes(validateCurrentImplementationInventory(
    canonicalPolicyForTests(),
    metadata(packages),
  )).has('implementation_features'));
});

test('requires one binary Kernel target without a hidden build target', () => {
  for (const targets of [
    [{ kind: ['lib'] }],
    [{ kind: ['bin'] }, { kind: ['bin'] }],
    [{ kind: ['bin'] }, { kind: ['custom-build'] }],
  ]) {
    const packages = recoveryOnlyPackages();
    packages.find(({ name }) => name === 'makosh-kernel').targets = targets;

    assert.ok(codes(validateCurrentImplementationInventory(
      canonicalPolicyForTests(),
      metadata(packages),
    )).has('implementation_target'));
  }
});

test('permits only a protocol codegen build target in addition to its library', () => {
  const packages = recoveryOnlyPackages();
  packages.find(({ name }) => name === 'makosh-events-protocol').targets.push({
    kind: ['custom-build'],
  });

  assert.deepEqual(validateCurrentImplementationInventory(
    canonicalPolicyForTests(),
    metadata(packages),
  ), []);
});
