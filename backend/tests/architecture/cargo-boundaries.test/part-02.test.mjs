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

test('allows PostgreSQL and AST clients only in their exact Storage packages', () => {
  const allowed = [
    kernel(),
    ...storagePackages({
      postgresDependencies: [dependency('sqlx')],
      migrationsDependencies: [dependency('pg_query')],
    }),
  ];
  assert.deepEqual(validateCargoMetadata(canonicalPolicyForTests(), metadata(allowed)), []);

  const sqlInControl = [
    kernel(),
    ...storagePackages({ controlDependencies: [dependency('sqlx')] }),
  ];
  assert.ok(
    codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(sqlInControl)))
      .has('storage_dependency'),
  );

  const astInRuntime = [
    kernel(),
    ...storagePackages({ runtimeDependencies: [dependency('pg_query')] }),
  ];
  assert.ok(
    codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(astInRuntime)))
      .has('storage_ast_dependency'),
  );
});

test('allows public Storage Vault contracts while rejecting private Storage implementations', () => {
  const allowed = [
    kernel([dependency('makosh-storage-protocol')]),
    workspacePackage('makosh-scheduler-runtime', {
      role: 'platform',
      owner: 'scheduler',
      surface: 'runtime',
    }, [dependency('makosh-storage-vault')]),
    ...storagePackages(),
  ];
  assert.deepEqual(validateCargoMetadata(canonicalPolicyForTests(), metadata(allowed)), []);

  const forbidden = [
    workspacePackage('makosh-untrusted-scheduler-runtime', {
      role: 'platform',
      owner: 'scheduler',
      surface: 'runtime',
    }, [dependency('makosh-storage-control')]),
    ...storagePackages(),
  ];
  assert.ok(
    codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(forbidden)))
      .has('storage_private_dependency'),
  );
});



test('prevents Storage packages from depending on Kernel, Gateway or modules', () => {
  const contacts = workspacePackage('makosh-contacts-api', {
    role: 'domain',
    owner: 'contacts',
    surface: 'contract',
  });
  const packages = [
    kernel(),
    contacts,
    ...storagePackages({ controlDependencies: [dependency('makosh-contacts-api')] }),
  ];

  assert.ok(
    codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)))
      .has('storage_owner_dependency'),
  );
});



test('rejects SQLite clients in owner PostgreSQL persistence packages', () => {
  const packages = [
    kernel(),
    workspacePackage(
      'makosh-contacts-persistence',
      { role: 'domain', owner: 'contacts', surface: 'persistence' },
      [dependency('rusqlite')],
    ),
  ];

  assert.ok(
    codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)))
      .has('sqlite_dependency'),
  );
});



test('accepts only the exact canonical events protocol metadata', () => {
  const mutations = [
    { role: 'core' },
    { owner: 'telemetry' },
    { surface: 'implementation' },
    { components: ['event_hub'] },
  ];

  for (const metadataOverrides of mutations) {
    const packages = [kernel(), eventsProtocol([], metadataOverrides)];
    assert.ok(
      codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)))
        .has('invalid_events_protocol_package'),
    );
  }
});



test('allows only one package to claim the canonical events protocol owner', () => {
  const packages = [
    kernel(),
    eventsProtocol(),
    workspacePackage('makosh-events-protocol-alias', {
      role: 'platform',
      owner: 'events',
      surface: 'contract',
    }),
  ];

  assert.ok(
    codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)))
      .has('events_protocol_owner'),
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
    test(`keeps events protocol independent of ${forbiddenDependency} through ${kind ?? 'normal'} dependencies`, () => {
      const packages = [
        kernel(),
        eventsProtocol([dependency(forbiddenDependency, kind)]),
      ];

      assert.ok(
        codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)))
          .has('events_protocol_dependency'),
      );
    });
  }
}


test('allows protobuf-only dependencies in the canonical events protocol', () => {
  const packages = [
    kernel(),
    eventsProtocol([dependency('prost'), dependency('bytes')]),
  ];

  assert.deepEqual(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)), []);
});



test('allows an integration to publish only through explicit Communications contracts', () => {
  const communicationsContract = workspacePackage('makosh-communications-ingress', {
    role: 'domain',
    owner: 'communications',
    surface: 'contract',
  });
  const allowed = [
    kernel(),
    communicationsContract,
    workspacePackage(
      'makosh-telegram-runtime',
      { role: 'integration', owner: 'telegram', surface: 'runtime' },
      [dependency('makosh-communications-ingress')],
    ),
  ];

  assert.deepEqual(validateCargoMetadata(canonicalPolicyForTests(), metadata(allowed)), []);

  const attachmentContract = workspacePackage('makosh-communications-attachment-contract', {
    role: 'domain',
    owner: 'communications',
    surface: 'contract',
  });
  const attachmentAllowed = [
    kernel(),
    attachmentContract,
    workspacePackage(
      'makosh-mail-runtime',
      { role: 'integration', owner: 'mail', surface: 'runtime' },
      [dependency('makosh-communications-attachment-contract')],
    ),
  ];

  assert.deepEqual(
    validateCargoMetadata(canonicalPolicyForTests(), metadata(attachmentAllowed)),
    [],
  );

  const clientContract = workspacePackage('makosh-communications-api', {
    role: 'domain',
    owner: 'communications',
    surface: 'contract',
  });
  const forbidden = [
    kernel(),
    clientContract,
    workspacePackage(
      'makosh-telegram-runtime',
      { role: 'integration', owner: 'telegram', surface: 'runtime' },
      [dependency('makosh-communications-api')],
    ),
  ];

  assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(forbidden))).has('integration_domain_contract_dependency'));
});

test('allows an engine to use only the exact Communications attachment contract', () => {
  const attachmentContract = workspacePackage('makosh-communications-attachment-contract', {
    role: 'domain',
    owner: 'communications',
    surface: 'contract',
  });
  const allowed = [
    kernel(),
    attachmentContract,
    workspacePackage(
      'makosh-attachment-security-runtime',
      { role: 'engine', owner: 'attachment_security', surface: 'runtime' },
      [dependency('makosh-communications-attachment-contract')],
    ),
  ];

  assert.deepEqual(validateCargoMetadata(canonicalPolicyForTests(), metadata(allowed)), []);

  const communicationsApi = workspacePackage('makosh-communications-api', {
    role: 'domain',
    owner: 'communications',
    surface: 'contract',
  });
  const forbidden = [
    kernel(),
    communicationsApi,
    workspacePackage(
      'makosh-attachment-security-runtime',
      { role: 'engine', owner: 'attachment_security', surface: 'runtime' },
      [dependency('makosh-communications-api')],
    ),
  ];

  assert.ok(
    codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(forbidden)))
      .has('engine_domain_contract_dependency'),
  );
});

test('allows an integration to publish only through the exact Attachment Security contract', () => {
  const candidateContract = workspacePackage('makosh-attachment-security-contract', {
    role: 'engine',
    owner: 'attachment_security',
    surface: 'contract',
  });
  const allowed = [
    kernel(),
    candidateContract,
    workspacePackage(
      'makosh-mail-runtime',
      { role: 'integration', owner: 'mail', surface: 'runtime' },
      [dependency('makosh-attachment-security-contract')],
    ),
  ];

  assert.deepEqual(validateCargoMetadata(canonicalPolicyForTests(), metadata(allowed)), []);

  const engineCore = workspacePackage('makosh-other-engine-contract', {
    role: 'engine',
    owner: 'other_engine',
    surface: 'contract',
  });
  const forbidden = [
    kernel(),
    engineCore,
    workspacePackage(
      'makosh-mail-runtime',
      { role: 'integration', owner: 'mail', surface: 'runtime' },
      [dependency('makosh-other-engine-contract')],
    ),
  ];

  assert.ok(
    codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(forbidden)))
      .has('integration_engine_contract_dependency'),
  );
});

test('allows an engine runtime to use only the shared Event Hub transport implementation', () => {
  const eventTransport = workspacePackage('makosh-events-jetstream', {
    role: 'platform',
    owner: 'events',
    surface: 'implementation',
  });
  const allowed = [
    kernel(),
    eventTransport,
    workspacePackage(
      'makosh-attachment-security-runtime',
      { role: 'engine', owner: 'attachment_security', surface: 'runtime' },
      [dependency('makosh-events-jetstream')],
    ),
  ];

  assert.deepEqual(validateCargoMetadata(canonicalPolicyForTests(), metadata(allowed)), []);

  const eventAuthority = workspacePackage('makosh-events-authority', {
    role: 'platform',
    owner: 'events',
    surface: 'implementation',
  });
  const forbidden = [
    kernel(),
    eventAuthority,
    workspacePackage(
      'makosh-attachment-security-runtime',
      { role: 'engine', owner: 'attachment_security', surface: 'runtime' },
      [dependency('makosh-events-authority')],
    ),
  ];

  assert.ok(
    codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(forbidden)))
      .has('implementation_dependency'),
  );
});

test('allows a workflow runtime to use only the shared Event Hub transport implementation', () => {
  const eventTransport = workspacePackage('makosh-events-jetstream', {
    role: 'platform',
    owner: 'events',
    surface: 'implementation',
  });
  const allowed = [
    kernel(),
    eventTransport,
    workspacePackage(
      'makosh-communications-export-runtime',
      { role: 'workflow', owner: 'communications_export', surface: 'runtime' },
      [dependency('makosh-events-jetstream')],
    ),
  ];

  assert.deepEqual(validateCargoMetadata(canonicalPolicyForTests(), metadata(allowed)), []);

  const eventAuthority = workspacePackage('makosh-events-authority', {
    role: 'platform',
    owner: 'events',
    surface: 'implementation',
  });
  const forbidden = [
    kernel(),
    eventAuthority,
    workspacePackage(
      'makosh-communications-export-runtime',
      { role: 'workflow', owner: 'communications_export', surface: 'runtime' },
      [dependency('makosh-events-authority')],
    ),
  ];

  assert.ok(
    codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(forbidden)))
      .has('implementation_dependency'),
  );
});

test('forbids a domain from importing an engine contract', () => {
  const engineContract = workspacePackage('makosh-attachment-security-contract', {
    role: 'engine',
    owner: 'attachment_security',
    surface: 'contract',
  });
  const packages = [
    kernel(),
    engineContract,
    workspacePackage(
      'makosh-communications-runtime',
      { role: 'domain', owner: 'communications', surface: 'runtime' },
      [dependency('makosh-attachment-security-contract')],
    ),
  ];

  assert.ok(
    codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)))
      .has('forbidden_dependency'),
  );
});

test('allows an engine to serve only an explicitly admitted workflow ingress contract', () => {
  const ingress = workspacePackage('makosh-attachment-text-extraction-ingress', {
    role: 'workflow',
    owner: 'attachment_text_extraction',
    surface: 'contract',
  });
  const engine = workspacePackage(
    'makosh-attachment-security-runtime',
    { role: 'engine', owner: 'attachment_security', surface: 'runtime' },
    [dependency('makosh-attachment-text-extraction-ingress')],
  );
  assert.deepEqual(
    validateCargoMetadata(canonicalPolicyForTests(), metadata([kernel(), ingress, engine])),
    [],
  );

  const unadmitted = workspacePackage('makosh-unadmitted-workflow-api', {
    role: 'workflow',
    owner: 'unadmitted_workflow',
    surface: 'contract',
  });
  const forbidden = workspacePackage(
    'makosh-attachment-security-runtime',
    { role: 'engine', owner: 'attachment_security', surface: 'runtime' },
    [dependency('makosh-unadmitted-workflow-api')],
  );
  assert.ok(codes(validateCargoMetadata(
    canonicalPolicyForTests(),
    metadata([kernel(), unadmitted, forbidden]),
  )).has('engine_workflow_contract_dependency'));
});



for (const packageName of [
  'makosh-backend',
  'makosh-api',
  'makosh-worker-runtime',
  'makosh-desktop-runtime',
  'makosh-schema',
  'makosh-common',
  'makosh-provider-api',
]) {
  test(`rejects compile-graph aggregation package ${packageName}`, () => {
    const packages = [
      kernel(),
      workspacePackage(packageName, {
        role: 'platform',
        owner: 'runtime_protocol',
        surface: 'contract',
      }),
    ];

    assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages))).has('forbidden_aggregate_package'));
  });
}


test('prevents module packages from depending on Kernel implementation', () => {
  const packages = [
    kernel(),
    workspacePackage(
      'makosh-telegram-core',
      { role: 'integration', owner: 'telegram', surface: 'implementation' },
      [dependency('makosh-kernel')],
    ),
  ];

  assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages))).has('kernel_dependency'));
});



test('prevents Kernel from compiling owner-specific module contracts', () => {
  const packages = [
    kernel([dependency('makosh-contacts-contracts')]),
    workspacePackage('makosh-contacts-contracts', {
      role: 'domain',
      owner: 'contacts',
      surface: 'contract',
    }),
  ];

  assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages))).has('kernel_module_dependency'));
});



test('keeps Gateway protocol independent from owner-specific contracts', () => {
  const packages = [
    kernel(),
    workspacePackage(
      'makosh-gateway-protocol',
      { role: 'api', owner: 'gateway', surface: 'contract' },
      [dependency('makosh-contacts-contracts')],
    ),
    workspacePackage('makosh-contacts-contracts', {
      role: 'domain',
      owner: 'contacts',
      surface: 'contract',
    }),
  ];

  assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages))).has('gateway_module_dependency'));
});



test('prevents one runtime package from aggregating another runtime', () => {
  const packages = [
    kernel(),
    workspacePackage(
      'makosh-telegram-runtime',
      { role: 'integration', owner: 'telegram', surface: 'runtime' },
      [dependency('makosh-telegram-sync-runtime')],
    ),
    workspacePackage('makosh-telegram-sync-runtime', {
      role: 'integration',
      owner: 'telegram',
      surface: 'runtime',
    }),
  ];

  assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages))).has('runtime_aggregation_dependency'));
});



test('rejects persistence adapter dependencies across owners', () => {
  const packages = [
    kernel(),
    workspacePackage(
      'makosh-tasks-persistence',
      { role: 'domain', owner: 'tasks', surface: 'persistence' },
      [dependency('makosh-contacts-persistence')],
    ),
    workspacePackage('makosh-contacts-persistence', {
      role: 'domain',
      owner: 'contacts',
      surface: 'persistence',
    }),
  ];

  assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages))).has('cross_owner_persistence_dependency'));
});



for (const { owner, adapters } of [
  { owner: 'mail', adapters: ['imap', 'smtp'] },
  { owner: 'telegram', adapters: ['tdlib'] },
  { owner: 'zulip', adapters: ['http'] },
]) {
  test(`accepts an isolated ${owner} package graph without a Communications implementation dependency`, () => {
    const adapterPackages = adapters.map((adapter) => workspacePackage(
      `makosh-${owner}-${adapter}`,
      { role: 'integration', owner, surface: 'implementation' },
      [dependency(`makosh-${owner}-core`)],
    ));
    const packages = [
      kernel(),
      workspacePackage('makosh-communications-ingress', {
        role: 'domain',
        owner: 'communications',
        surface: 'contract',
      }),
      workspacePackage(`makosh-${owner}-api`, {
        role: 'integration',
        owner,
        surface: 'contract',
      }),
      workspacePackage(
        `makosh-${owner}-core`,
        { role: 'integration', owner, surface: 'implementation' },
        [dependency('makosh-communications-ingress')],
      ),
      ...adapterPackages,
      workspacePackage(
        `makosh-${owner}-persistence`,
        { role: 'integration', owner, surface: 'persistence' },
        [dependency(`makosh-${owner}-core`)],
      ),
      workspacePackage(
        `makosh-${owner}-runtime`,
        { role: 'integration', owner, surface: 'runtime' },
        [
          dependency(`makosh-${owner}-api`),
          dependency(`makosh-${owner}-core`),
          ...adapters.map((adapter) => dependency(`makosh-${owner}-${adapter}`)),
          dependency(`makosh-${owner}-persistence`),
        ],
      ),
    ];

    assert.deepEqual(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)), []);
  });
}



for (const owner of ['persons', 'organizations', 'tasks', 'calendar', 'documents', 'ai']) {
  test(`accepts an isolated package graph for enabled domain ${owner}`, () => {
    const packages = [
      kernel(),
      workspacePackage(`makosh-${owner}-contracts`, {
        role: 'domain',
        owner,
        surface: 'contract',
      }),
      workspacePackage(
        `makosh-${owner}-domain`,
        { role: 'domain', owner, surface: 'implementation' },
        [dependency(`makosh-${owner}-contracts`)],
      ),
      workspacePackage(
        `makosh-${owner}-persistence`,
        { role: 'domain', owner, surface: 'persistence' },
        [dependency(`makosh-${owner}-domain`)],
      ),
      workspacePackage(
        `makosh-${owner}-runtime`,
        { role: 'domain', owner, surface: 'runtime' },
        [
          dependency(`makosh-${owner}-contracts`),
          dependency(`makosh-${owner}-domain`),
          dependency(`makosh-${owner}-persistence`),
        ],
      ),
    ];

    assert.deepEqual(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)), []);
  });
}

test('accepts the split Communications ingress, attachment, and client API package graph', () => {
  const packages = [
    kernel(),
    workspacePackage('makosh-communications-ingress', {
      role: 'domain',
      owner: 'communications',
      surface: 'contract',
    }),
    workspacePackage('makosh-communications-api', {
      role: 'domain',
      owner: 'communications',
      surface: 'contract',
    }),
    workspacePackage('makosh-communications-attachment-contract', {
      role: 'domain',
      owner: 'communications',
      surface: 'contract',
    }),
    workspacePackage(
      'makosh-communications-domain',
      { role: 'domain', owner: 'communications', surface: 'implementation' },
      [dependency('makosh-communications-api')],
    ),
    workspacePackage(
      'makosh-communications-persistence',
      { role: 'domain', owner: 'communications', surface: 'persistence' },
      [dependency('makosh-communications-domain')],
    ),
    workspacePackage(
      'makosh-communications-runtime',
      { role: 'domain', owner: 'communications', surface: 'runtime' },
      [
        dependency('makosh-communications-attachment-contract'),
        dependency('makosh-communications-ingress'),
        dependency('makosh-communications-api'),
        dependency('makosh-communications-domain'),
        dependency('makosh-communications-persistence'),
      ],
    ),
  ];

  assert.deepEqual(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)), []);
});

test('keeps WhatsApp implementation in the hidden host WebView boundary', () => {
  const packages = [
    kernel(),
    workspacePackage('makosh-whatsapp-runtime', {
      role: 'integration',
      owner: 'whatsapp',
      surface: 'runtime',
    }, [dependency('wry')]),
  ];

  assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages))).has('host_execution_dependency'));
});



for (const owner of ['decisions']) {
  test(`rejects a Cargo package owned by blocked domain ${owner}`, () => {
    const blockedPolicy = canonicalPolicyForTests();
    blockedPolicy.domains.developmentAllowlist = blockedPolicy.domains.developmentAllowlist.filter(
      (value) => value !== owner,
    );
    blockedPolicy.domains.blocked.push(owner);
    const packages = [
      kernel(),
      workspacePackage(`makosh-${owner}-runtime`, {
        role: 'domain',
        owner,
        surface: 'runtime',
      }),
    ];

    assert.ok(codes(validateCargoMetadata(blockedPolicy, metadata(packages))).has('blocked_domain'));
  });
}
