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

test('rejects a singular retired Contacts domain hidden in metadata owner', () => {
  const packages = [
    kernel(),
    workspacePackage('makosh-generic-runtime', {
      role: 'integration',
      owner: 'contact',
      surface: 'runtime',
    }),
  ];

  assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages))).has('blocked_domain'));
});

for (const provider of ['mail', 'telegram', 'whatsapp', 'zulip']) {
  test(`rejects provider ${provider} as a business domain`, () => {
    const packages = [
      kernel(),
      workspacePackage(`makosh-${provider}-runtime`, {
        role: 'domain',
        owner: provider,
        surface: 'runtime',
      }),
    ];

    assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages))).has('blocked_domain'));
  });
}

test('prevents an integration from claiming an enabled business domain identity', () => {
  const packages = [
    kernel(),
    workspacePackage('makosh-telegram-runtime', {
      role: 'integration',
      owner: 'communications',
      surface: 'runtime',
    }),
  ];

  assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages))).has('invalid_owner'));
});



for (const owner of ['graph', 'timeline', 'search', 'context']) {
  test(`rejects a Cargo package for blocked projection ${owner}`, () => {
    const packages = [
      kernel(),
      workspacePackage(`makosh-${owner}-runtime`, {
        role: 'engine',
        owner,
        surface: 'runtime',
      }),
    ];

    assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages))).has('blocked_projection'));
  });
}



test('rejects missing and unknown package roles', () => {
  const packages = [
    kernel(),
    workspacePackage('makosh-no-role', {
      owner: 'events',
      surface: 'contract',
    }),
    workspacePackage('makosh-many-roles', {
      role: ['platform', 'domain'],
      owner: 'events',
      surface: 'contract',
    }),
  ];

  const resultCodes = codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)));
  assert.ok(resultCodes.has('invalid_role'));
});



for (const kind of [null, 'build', 'dev']) {
  test(`rejects a direct ${kind ?? 'normal'} dependency between domains`, () => {
    const packages = [
      kernel(),
      workspacePackage(
        'makosh-tasks-runtime',
        { role: 'domain', owner: 'tasks', surface: 'runtime' },
        [dependency('makosh-contacts-contracts', kind)],
      ),
      workspacePackage('makosh-contacts-contracts', {
        role: 'domain',
        owner: 'contacts',
        surface: 'contract',
      }),
    ];

    assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages))).has('forbidden_dependency'));
  });
}



for (const target of [
  { name: 'makosh-contacts-contracts', role: 'domain', owner: 'contacts' },
  { name: 'makosh-mail-analysis-contracts', role: 'workflow', owner: 'mail_analysis' },
  { name: 'makosh-telegram-contracts', role: 'integration', owner: 'telegram' },
]) {
  test(`prevents AI from acquiring cross-owner context through ${target.role} ${target.owner}`, () => {
    const packages = [
      kernel(),
      workspacePackage(
        'makosh-ai-runtime',
        { role: 'domain', owner: 'ai', surface: 'runtime' },
        [dependency(target.name)],
      ),
      workspacePackage(target.name, {
        role: target.role,
        owner: target.owner,
        surface: 'contract',
      }),
    ];

    assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages))).has('forbidden_dependency'));
  });
}



test('allows a use-case workflow to assemble AI context from explicit owner contracts', () => {
  const packages = [
    kernel(),
    workspacePackage('makosh-ai-contracts', {
      role: 'domain',
      owner: 'ai',
      surface: 'contract',
    }),
    workspacePackage('makosh-persons-contracts', {
      role: 'domain',
      owner: 'persons',
      surface: 'contract',
    }),
    workspacePackage(
      'makosh-person-summary-workflow',
      { role: 'workflow', owner: 'person_summary', surface: 'runtime' },
      [
        dependency('makosh-ai-contracts'),
        dependency('makosh-persons-contracts'),
      ],
    ),
  ];

  assert.deepEqual(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)), []);
});



test('rejects the blocked projection role independently of its owner name', () => {
  const packages = [
    kernel(),
    workspacePackage('makosh-derived-reader', {
      role: 'projection',
      owner: 'derived_reader',
      surface: 'runtime',
    }),
  ];

  assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages))).has('blocked_projection'));
});



test('rejects singular aliases of the retired Contacts domain in package names', () => {
  const packages = [
    kernel(),
    workspacePackage('makosh-contact-runtime', {
      role: 'platform',
      owner: 'events',
      surface: 'runtime',
    }),
  ];

  assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages))).has('blocked_domain'));
});



test('allows a workflow to use contracts but not implementations', () => {
  const personsContract = workspacePackage('makosh-persons-contracts', {
    role: 'domain',
    owner: 'persons',
    surface: 'contract',
  });
  const personsRuntime = workspacePackage('makosh-persons-runtime', {
    role: 'domain',
    owner: 'persons',
    surface: 'runtime',
  });

  const allowed = [
    kernel(),
    personsContract,
    workspacePackage(
      'makosh-person-import-workflow',
      { role: 'workflow', owner: 'person_import', surface: 'runtime' },
      [dependency('makosh-persons-contracts')],
    ),
  ];
  assert.deepEqual(validateCargoMetadata(canonicalPolicyForTests(), metadata(allowed)), []);

  const workflowContract = workspacePackage('makosh-delivery-intent-contract', {
    role: 'workflow',
    owner: 'delivery_intent',
    surface: 'contract',
  });
  const workflowToWorkflowContract = [
    kernel(),
    workflowContract,
    workspacePackage(
      'makosh-bulk-delivery-workflow',
      { role: 'workflow', owner: 'bulk_delivery', surface: 'runtime' },
      [dependency('makosh-delivery-intent-contract')],
    ),
  ];
  assert.deepEqual(
    validateCargoMetadata(canonicalPolicyForTests(), metadata(workflowToWorkflowContract)),
    [],
  );

  const forbidden = [
    kernel(),
    personsRuntime,
    workspacePackage(
      'makosh-person-import-workflow',
      { role: 'workflow', owner: 'person_import', surface: 'runtime' },
      [dependency('makosh-persons-runtime')],
    ),
  ];
  assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(forbidden))).has('implementation_dependency'));
});

test('allows the Kernel runtime to compose only the exact Core Gateway adapters', () => {
  const gatewaySession = workspacePackage('makosh-gateway-session', {
    role: 'api',
    owner: 'gateway',
    surface: 'implementation',
  });
  const gatewayRuntime = workspacePackage('makosh-gateway-runtime', {
    role: 'api',
    owner: 'gateway',
    surface: 'implementation',
  });
  const packages = [
    kernel([dependency('makosh-gateway-session'), dependency('makosh-gateway-runtime')]),
    gatewaySession,
    gatewayRuntime,
  ];

  assert.deepEqual(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)), []);
});



test('keeps a contract independent from its owner runtime and persistence', () => {
  for (const targetSurface of ['runtime', 'persistence']) {
    const targetName = `makosh-contacts-${targetSurface}`;
    const packages = [
      kernel(),
      workspacePackage(
        'makosh-contacts-contracts',
        { role: 'domain', owner: 'contacts', surface: 'contract' },
        [dependency(targetName)],
      ),
      workspacePackage(targetName, {
        role: 'domain',
        owner: 'contacts',
        surface: targetSurface,
      }),
    ];

    assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages))).has('forbidden_dependency'));
  }
});



test('keeps domain implementation independent from persistence while runtime composes both', () => {
  const implementation = workspacePackage(
    'makosh-persons-implementation',
    { role: 'domain', owner: 'persons', surface: 'implementation' },
    [dependency('makosh-persons-persistence')],
  );
  const persistence = workspacePackage('makosh-persons-persistence', {
    role: 'domain',
    owner: 'persons',
    surface: 'persistence',
  });
  const forbidden = [kernel(), implementation, persistence];
  assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(forbidden))).has('forbidden_dependency'));

  const allowed = [
    kernel(),
    workspacePackage(
      'makosh-persons-runtime',
      { role: 'domain', owner: 'persons', surface: 'runtime' },
      [
        dependency('makosh-persons-implementation'),
        dependency('makosh-persons-persistence'),
      ],
    ),
    workspacePackage('makosh-persons-implementation', {
      role: 'domain',
      owner: 'persons',
      surface: 'implementation',
    }),
    persistence,
  ];
  assert.deepEqual(validateCargoMetadata(canonicalPolicyForTests(), metadata(allowed)), []);
});

test('allows an owner assembly unit to compose runtime and persistence only downstream', () => {
  const runtime = workspacePackage('makosh-telegram-runtime', {
    role: 'integration',
    owner: 'telegram',
    surface: 'runtime',
  });
  const persistence = workspacePackage('makosh-telegram-persistence', {
    role: 'integration',
    owner: 'telegram',
    surface: 'persistence',
  });
  const assembly = workspacePackage(
    'makosh-telegram-assembly',
    { role: 'integration', owner: 'telegram', surface: 'assembly' },
    [
      dependency('makosh-telegram-persistence'),
      dependency('makosh-telegram-runtime'),
    ],
  );

  assert.deepEqual(
    validateCargoMetadata(
      canonicalPolicyForTests(),
      metadata([kernel(), runtime, persistence, assembly]),
    ),
    [],
  );

  const reversedRuntime = workspacePackage(
    'makosh-telegram-runtime',
    { role: 'integration', owner: 'telegram', surface: 'runtime' },
    [dependency('makosh-telegram-assembly')],
  );
  assert.ok(codes(validateCargoMetadata(
    canonicalPolicyForTests(),
    metadata([kernel(), reversedRuntime, persistence, assembly]),
  )).has('forbidden_dependency'));
});

test('keeps the Mail release assembly downstream from Mail runtime and persistence', () => {
  const runtime = workspacePackage('makosh-mail-runtime', {
    role: 'integration',
    owner: 'mail',
    surface: 'runtime',
  });
  const persistence = workspacePackage('makosh-mail-persistence', {
    role: 'integration',
    owner: 'mail',
    surface: 'persistence',
  });
  const assembly = workspacePackage(
    'makosh-mail-assembly',
    { role: 'integration', owner: 'mail', surface: 'assembly' },
    [
      dependency('makosh-mail-persistence'),
      dependency('makosh-mail-runtime'),
    ],
  );

  assert.deepEqual(
    validateCargoMetadata(
      canonicalPolicyForTests(),
      metadata([kernel(), runtime, persistence, assembly]),
    ),
    [],
  );

  for (const [forbiddenConsumer, expectedCode] of [
    [
      workspacePackage(
        'makosh-mail-runtime',
        { role: 'integration', owner: 'mail', surface: 'runtime' },
        [dependency('makosh-mail-assembly')],
      ),
      'forbidden_dependency',
    ],
    [
      workspacePackage(
        'makosh-communications-runtime',
        { role: 'domain', owner: 'communications', surface: 'runtime' },
        [dependency('makosh-mail-assembly')],
      ),
      'implementation_dependency',
    ],
  ]) {
    assert.ok(codes(validateCargoMetadata(
      canonicalPolicyForTests(),
      metadata([kernel(), forbiddenConsumer, persistence, assembly]),
    )).has(expectedCode));
  }
});



for (const sqlClient of ['sqlx']) {
  test(`allows ${sqlClient} only in a persistence surface`, () => {
    const forbidden = [
      kernel(),
      workspacePackage(
        'makosh-persons-runtime',
        { role: 'domain', owner: 'persons', surface: 'runtime' },
        [dependency(sqlClient)],
      ),
    ];
    assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(forbidden))).has('storage_dependency'));

    const allowed = [
      kernel(),
      workspacePackage(
        'makosh-persons-persistence',
        { role: 'domain', owner: 'persons', surface: 'persistence' },
        [dependency(sqlClient)],
      ),
    ];
    assert.deepEqual(validateCargoMetadata(canonicalPolicyForTests(), metadata(allowed)), []);
  });
}



for (const alternativeClient of ['tokio-postgres', 'postgres', 'diesel', 'sea-orm']) {
  test(`rejects unselected PostgreSQL client ${alternativeClient} in owner persistence`, () => {
    const packages = [
      kernel(),
      workspacePackage(
        'makosh-contacts-persistence',
        { role: 'domain', owner: 'contacts', surface: 'persistence' },
        [dependency(alternativeClient)],
      ),
    ];

    assert.ok(
      codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)))
        .has('storage_dependency'),
    );
  });
}



test('isolates the Kernel SQLite client in its persistence adapter', () => {
  const contract = workspacePackage('makosh-kernel-control-store', {
    role: 'core',
    owner: 'kernel',
    surface: 'contract',
  });
  const sqliteAdapter = workspacePackage(
    'makosh-kernel-control-store-sqlite',
    { role: 'core', owner: 'kernel', surface: 'persistence' },
    [dependency('makosh-kernel-control-store'), dependency('rusqlite')],
  );
  const allowed = [
    kernel([
      dependency('makosh-kernel-control-store'),
      dependency('makosh-kernel-control-store-sqlite'),
    ]),
    contract,
    sqliteAdapter,
  ];

  assert.deepEqual(validateCargoMetadata(canonicalPolicyForTests(), metadata(allowed)), []);

  const directRuntimeDependency = [
    kernel([dependency('rusqlite')]),
  ];
  assert.ok(
    codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(directRuntimeDependency)))
      .has('sqlite_dependency'),
  );

  const moduleBypass = [
    kernel(),
    sqliteAdapter,
    workspacePackage(
      'makosh-telegram-runtime',
      { role: 'integration', owner: 'telegram', surface: 'runtime' },
      [dependency('makosh-kernel-control-store-sqlite')],
    ),
  ];
  assert.ok(
    codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(moduleBypass)))
      .has('kernel_dependency'),
  );
});



test('rejects an unregistered core-owned package', () => {
  const packages = [
    kernel(),
    workspacePackage('makosh-kernel-unlisted-helper', {
      role: 'core',
      owner: 'kernel',
      surface: 'contract',
    }),
  ];

  assert.ok(
    codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)))
      .has('invalid_core_package'),
  );
});



test('keeps Event Hub, telemetry control and settings registry exclusive to Kernel', () => {
  const packages = [
    kernel(),
    workspacePackage('makosh-events-runtime', {
      role: 'platform',
      owner: 'events',
      surface: 'runtime',
      components: ['event_hub'],
    }),
  ];

  assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages))).has('exclusive_kernel_component'));
});



test('keeps settings registry exclusive to Kernel', () => {
  const packages = [
    kernel(),
    workspacePackage('makosh-runtime-settings', {
      role: 'platform',
      owner: 'runtime_protocol',
      surface: 'runtime',
      components: ['settings_registry'],
    }),
  ];

  assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages))).has('exclusive_kernel_component'));
});



test('rejects a settings registry package outside Kernel without component metadata', () => {
  const packages = [
    kernel(),
    workspacePackage('makosh-settings-registry', {
      role: 'platform',
      owner: 'runtime_protocol',
      surface: 'runtime',
    }),
  ];

  assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages))).has('exclusive_kernel_component'));
});

test('rejects an Event Hub package outside Kernel even without component metadata', () => {
  const packages = [
    kernel(),
    workspacePackage('makosh-event-hub', {
      role: 'platform',
      owner: 'events',
      surface: 'runtime',
    }),
  ];

  assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages))).has('exclusive_kernel_component'));
});

test('rejects Kernel components outside the constitutional registry', () => {
  const packages = [
    kernel([], {
      components: ['supervisor', 'unapproved_component'],
    }),
  ];

  assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages))).has('unknown_kernel_component'));
});
