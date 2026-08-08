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

test('accepts only explicitly configured development package surfaces', () => {
  const operator = workspacePackage('makosh-development-kernel-operator', {
    role: 'development',
    owner: 'development',
    surface: 'runtime',
  });
  const assembly = workspacePackage('makosh-development-assembly', {
    role: 'development',
    owner: 'development',
    surface: 'assembly',
  });

  assert.deepEqual(
    validateCargoMetadata(canonicalPolicyForTests(), metadata([kernel(), operator, assembly])),
    [],
  );

  const invalidAssembly = workspacePackage('makosh-development-assembly', {
    role: 'development',
    owner: 'development',
    surface: 'runtime',
  });
  assert.ok(
    codes(validateCargoMetadata(canonicalPolicyForTests(), metadata([kernel(), invalidAssembly])))
      .has('invalid_development_package'),
  );
});

test('requires the canonical events protocol package when a workspace exists', () => {
  const violations = validateCargoMetadata(
    canonicalPolicyForTests(),
    fixtureMetadata([kernel(), runtimeProtocol()]),
  );

  assert.ok(codes(violations).has('missing_events_protocol_package'));
});



test('requires the canonical runtime protocol package when a workspace exists', () => {
  const violations = validateCargoMetadata(
    canonicalPolicyForTests(),
    fixtureMetadata([kernel(), eventsProtocol()]),
  );

  assert.ok(codes(violations).has('missing_runtime_protocol_package'));
});



test('accepts only the exact canonical runtime protocol metadata', () => {
  const mutations = [
    { role: 'core' },
    { owner: 'events' },
    { surface: 'implementation' },
    { components: ['module_registry'] },
  ];

  for (const metadataOverrides of mutations) {
    const packages = [kernel(), runtimeProtocol([], metadataOverrides)];
    assert.ok(
      codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)))
        .has('invalid_runtime_protocol_package'),
    );
  }
});



test('allows only one package to claim the canonical runtime protocol owner', () => {
  const packages = [
    kernel(),
    runtimeProtocol(),
    workspacePackage('makosh-runtime-protocol-alias', {
      role: 'platform',
      owner: 'runtime_protocol',
      surface: 'contract',
    }),
  ];

  assert.ok(
    codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)))
      .has('runtime_protocol_owner'),
  );
});



for (const kind of [null, 'build', 'dev']) {
  for (const forbiddenDependency of [
    'async-nats',
    'nats',
    'sqlx',
    'tokio-postgres',
    'postgres',
    'diesel',
    'sea-orm',
    'rusqlite',
    'serde_json',
  ]) {
    test(`keeps runtime protocol independent of ${forbiddenDependency} through ${kind ?? 'normal'} dependencies`, () => {
      const packages = [
        kernel(),
        runtimeProtocol([dependency(forbiddenDependency, kind)]),
      ];

      assert.ok(
        codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)))
          .has('runtime_protocol_dependency'),
      );
    });

  }
}
test('allows protobuf-only dependencies in the canonical runtime protocol', () => {
  const packages = [
    kernel(),
    runtimeProtocol([dependency('prost'), dependency('bytes')]),
  ];

  assert.deepEqual(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)), []);
});



test('requires the complete canonical Vault package set once Vault ownership appears', () => {
  const packages = [kernel(), vaultProtocol()];

  assert.ok(
    codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)))
      .has('missing_vault_package'),
  );
});



test('rejects a dependency-only reference to an undeclared Vault package', () => {
  const packages = [
    kernel(),
    workspacePackage(
      'makosh-telegram-runtime',
      { role: 'integration', owner: 'telegram', surface: 'runtime' },
      [dependency('makosh-vault-protocol')],
    ),
  ];

  assert.ok(
    codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)))
      .has('missing_vault_package'),
  );
});



test('accepts the exact canonical Vault package set', () => {
  const packages = [kernel(), ...vaultPackages()];

  assert.deepEqual(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)), []);
});



test('rejects incorrect metadata on every canonical Vault package surface', () => {
  const mutations = [
    { protocol: { surface: 'implementation' } },
    { keyProvider: { components: ['vault_service'] } },
    { runtime: { components: [] } },
    { store: { surface: 'implementation' } },
    { keyProviderFile: { owner: 'storage' } },
  ];

  for (const overrides of mutations) {
    const packages = [kernel(), ...vaultPackages({ overrides })];
    assert.ok(
      codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)))
        .has('invalid_vault_package'),
    );
  }
});



test('rejects undeclared packages claiming the Vault owner', () => {
  const packages = [
    kernel(),
    ...vaultPackages(),
    workspacePackage('makosh-vault-compat', {
      role: 'platform',
      owner: 'vault',
      surface: 'contract',
    }),
  ];

  assert.ok(
    codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)))
      .has('invalid_vault_package'),
  );
});



test('keeps the Vault service component exclusive to the canonical Vault runtime', () => {
  const packages = [
    kernel(),
    workspacePackage('makosh-telemetry-vault-helper', {
      role: 'platform',
      owner: 'telemetry',
      surface: 'runtime',
      components: ['vault_service'],
    }),
  ];

  assert.ok(
    codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)))
      .has('invalid_vault_package'),
  );
});



for (const kind of [null, 'build', 'dev']) {
  for (const forbiddenDependency of [
    'async-nats',
    'nats',
    'sqlx',
    'tokio-postgres',
    'postgres',
    'diesel',
    'sea-orm',
    'rusqlite',
    'serde_json',
  ]) {
    test(`keeps Vault protocol independent of ${forbiddenDependency} through ${kind ?? 'normal'} dependencies`, () => {
      const [, ...remainingVaultPackages] = vaultPackages();
      const packages = [
        kernel(),
        vaultProtocol([dependency(forbiddenDependency, kind)]),
        ...remainingVaultPackages,
      ];

      assert.ok(
        codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)))
          .has('vault_protocol_dependency'),
      );
    });

    test(`keeps the private Vault key-provider contract independent of ${forbiddenDependency} through ${kind ?? 'normal'} dependencies`, () => {
      const packages = [
        kernel(),
        ...vaultPackages({
          keyProviderDependencies: [dependency(forbiddenDependency, kind)],
        }),
      ];

      assert.ok(
        codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)))
          .has('vault_protocol_dependency'),
      );
    });
  }
}



test('rejects undeclared external makosh-vault package dependencies', () => {
  const packages = [
    kernel(),
    workspacePackage(
      'makosh-telegram-runtime',
      { role: 'integration', owner: 'telegram', surface: 'runtime' },
      [dependency('makosh-vault-compat')],
    ),
  ];

  assert.ok(
    codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)))
      .has('vault_private_dependency'),
  );
});



test('allows modules to use only the public Vault protocol', () => {
  const allowed = [
    kernel(),
    ...vaultPackages(),
    workspacePackage(
      'makosh-telegram-runtime',
      { role: 'integration', owner: 'telegram', surface: 'runtime' },
      [dependency('makosh-vault-protocol')],
    ),
  ];
  assert.deepEqual(validateCargoMetadata(canonicalPolicyForTests(), metadata(allowed)), []);

  const forbidden = [
    kernel(),
    ...vaultPackages(),
    workspacePackage(
      'makosh-telegram-runtime',
      { role: 'integration', owner: 'telegram', surface: 'runtime' },
      [dependency('makosh-vault-key-provider')],
    ),
  ];
  assert.ok(
    codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(forbidden)))
      .has('vault_private_dependency'),
  );
});



test('prevents Kernel from linking a private Vault package', () => {
  const packages = [
    kernel([dependency('makosh-vault-runtime')]),
    ...vaultPackages(),
  ];

  assert.ok(
    codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)))
      .has('vault_private_dependency'),
  );
});



test('prevents Vault packages from depending on Kernel or module packages', () => {
  const telegramApi = workspacePackage('makosh-telegram-api', {
    role: 'integration',
    owner: 'telegram',
    surface: 'contract',
  });
  const packages = [
    kernel(),
    telegramApi,
    ...vaultPackages({ runtimeDependencies: [dependency('makosh-telegram-api')] }),
  ];

  assert.ok(
    codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)))
      .has('vault_owner_dependency'),
  );
});



test('keeps every Vault package independent of NATS and PostgreSQL clients', () => {
  const packages = [
    kernel(),
    ...vaultPackages({ runtimeDependencies: [dependency('async-nats')] }),
  ];

  assert.ok(
    codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)))
      .has('vault_owner_dependency'),
  );
});



test('requires the complete canonical Storage Control package set once storage ownership appears', () => {
  const packages = [kernel(), storageProtocol()];

  assert.ok(
    codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)))
      .has('missing_storage_package'),
  );
});



test('accepts the exact canonical Storage Control package set', () => {
  const packages = [kernel(), ...storagePackages()];

  assert.deepEqual(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)), []);
});



test('accepts the intended isolated Storage Control dependency graph', () => {
  const packages = [
    kernel(),
    runtimeProtocol(),
    ...vaultPackages(),
    ...storagePackages({
      controlDependencies: [
        dependency('makosh-storage-protocol'),
        dependency('makosh-storage-vault'),
      ],
      vaultDependencies: [
        dependency('makosh-storage-protocol'),
        dependency('makosh-runtime-protocol'),
        dependency('makosh-vault-protocol'),
      ],
      runtimeDependencies: [
        dependency('makosh-storage-protocol'),
        dependency('makosh-storage-control'),
        dependency('makosh-storage-postgres'),
        dependency('makosh-storage-pgbouncer'),
        dependency('makosh-storage-migrations'),
        dependency('makosh-storage-vault'),
      ],
      postgresDependencies: [dependency('makosh-storage-control'), dependency('sqlx')],
      pgbouncerDependencies: [dependency('makosh-storage-control')],
      migrationsDependencies: [dependency('makosh-storage-control'), dependency('pg_query')],
    }),
  ];

  assert.deepEqual(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)), []);
});



test('rejects incorrect metadata on every canonical Storage Control package', () => {
  const mutations = [
    { protocol: { surface: 'implementation' } },
    { control: { surface: 'runtime' } },
    { vault: { surface: 'runtime' } },
    { runtime: { components: [] } },
    { postgres: { surface: 'implementation' } },
    { pgbouncer: { owner: 'events' } },
    { migrations: { surface: 'persistence' } },
  ];

  for (const overrides of mutations) {
    const packages = [kernel(), ...storagePackages({ overrides })];
    assert.ok(
      codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)))
        .has('invalid_storage_package'),
    );
  }
});



test('rejects undeclared packages claiming the Storage owner', () => {
  const packages = [
    kernel(),
    ...storagePackages(),
    workspacePackage('makosh-storage-compat', {
      role: 'platform',
      owner: 'storage',
      surface: 'contract',
    }),
  ];

  assert.ok(
    codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)))
      .has('invalid_storage_package'),
  );
});



test('keeps storage_control exclusive to the canonical Storage runtime', () => {
  const packages = [
    kernel(),
    workspacePackage('makosh-storage-helper', {
      role: 'platform',
      owner: 'events',
      surface: 'runtime',
      components: ['storage_control'],
    }),
  ];

  assert.ok(
    codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)))
      .has('invalid_storage_package'),
  );
});



for (const kind of [null, 'build', 'dev']) {
  for (const forbiddenDependency of [
    'async-nats',
    'nats',
    'sqlx',
    'tokio-postgres',
    'postgres',
    'diesel',
    'sea-orm',
    'rusqlite',
    'serde_json',
  ]) {
    test(`keeps storage protocol independent of ${forbiddenDependency} through ${kind ?? 'normal'} dependencies`, () => {
      const [, ...remainingStoragePackages] = storagePackages();
      const packages = [
        kernel(),
        storageProtocol([dependency(forbiddenDependency, kind)]),
        ...remainingStoragePackages,
      ];

      assert.ok(
        codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)))
          .has('storage_protocol_dependency'),
      );
    });
  }
}
