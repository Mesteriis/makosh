import assert from 'node:assert/strict';
import test from 'node:test';

import { validateCargoMetadata } from '../../scripts/lib/cargo-boundaries.mjs';
import { validatePolicy } from '../../scripts/lib/policy-schema.mjs';
import { validateSourceEntries } from '../../scripts/lib/source-boundaries.mjs';
import {
  codes,
  dependency,
  kernel,
  metadata,
  workspacePackage,
} from './support/cargo-fixtures.mjs';
import { canonicalPolicyForTests } from './support/canonical-policy.mjs';

function cargoMetadata(packages) {
  const required = [
    workspacePackage('makosh-events-protocol', {
      role: 'platform',
      owner: 'events',
      surface: 'contract',
    }),
    workspacePackage('makosh-runtime-protocol', {
      role: 'platform',
      owner: 'runtime_protocol',
      surface: 'contract',
    }),
  ].filter(({ name }) => !packages.some((pkg) => pkg.name === name));

  return metadata([...required, ...packages]);
}

test('registers persons as the only central person owner', () => {
  const policy = canonicalPolicyForTests();

  assert.equal(policy.domains.registered.includes('persons'), true);
  assert.equal(policy.domains.registered.includes('contacts'), false);
  assert.equal(policy.domains.developmentAllowlist.includes('persons'), true);
  assert.equal(policy.domains.developmentAllowlist.includes('contacts'), false);
});

test('rejects a registry that restores contacts beside persons', () => {
  const policy = canonicalPolicyForTests();
  policy.domains.registered.push('contacts');
  policy.domains.developmentAllowlist.push('contacts');

  assert.ok(codes(validatePolicy(policy)).has('central_person_owner'));
});

test('rejects contacts as a replacement registry owner', () => {
  const policy = canonicalPolicyForTests();
  policy.domains.registered = policy.domains.registered.map(
    (owner) => owner === 'persons' ? 'contacts' : owner,
  );
  policy.domains.developmentAllowlist = policy.domains.developmentAllowlist.map(
    (owner) => owner === 'persons' ? 'contacts' : owner,
  );

  assert.ok(codes(validatePolicy(policy)).has('central_person_owner'));
});

test('accepts Persons development source paths and Cargo packages', () => {
  const policy = canonicalPolicyForTests();
  assert.deepEqual(validateSourceEntries(policy, [
    { path: 'src/domains/persons/contract/src/lib.rs', content: '' },
    { path: 'src/domains/persons/implementation/src/lib.rs', content: '' },
  ]), []);

  const packages = [
    kernel(),
    workspacePackage('makosh-persons-contracts', {
      role: 'domain',
      owner: 'persons',
      surface: 'contract',
    }),
  ];
  assert.deepEqual(validateCargoMetadata(policy, cargoMetadata(packages)), []);
});

test('rejects every retired Contacts package after atomic cutover', () => {
  const policy = canonicalPolicyForTests();
  const exactLegacyPackage = workspacePackage('makosh-contacts-command-api', {
    role: 'domain',
    owner: 'contacts',
    surface: 'contract',
  });
  const undeclaredLegacyPackage = workspacePackage('makosh-contacts-contracts', {
    role: 'domain',
    owner: 'contacts',
    surface: 'contract',
  });

  assert.ok(codes(validateCargoMetadata(
    policy,
    cargoMetadata([kernel(), exactLegacyPackage]),
  )).has('blocked_domain'));
  assert.ok(codes(validateCargoMetadata(
    policy,
    cargoMetadata([kernel(), undeclaredLegacyPackage]),
  )).has('blocked_domain'));

  const exactLegacyWorkflow = workspacePackage('makosh-mail-contacts-sync-api', {
    role: 'workflow',
    owner: 'mail_contacts_sync',
    surface: 'contract',
  });
  assert.ok(codes(validateCargoMetadata(
    policy,
    cargoMetadata([kernel(), exactLegacyWorkflow]),
  )).has('blocked_domain'));
});

for (const legacyPackage of [
  workspacePackage('makosh-contact-helper', {
    role: 'platform', owner: 'events', surface: 'implementation',
  }),
  workspacePackage('makosh-mail-contact-sync-copy', {
    role: 'workflow', owner: 'mail_contact_sync', surface: 'contract',
  }),
  workspacePackage('makosh-contacts-test-helper', {
    role: 'test', owner: 'test', surface: 'test_support',
  }),
  workspacePackage('makosh-contacts-development-helper', {
    role: 'development', owner: 'development', surface: 'implementation',
  }),
]) {
  test(`rejects undeclared legacy contact alias ${legacyPackage.name}`, () => {
    const result = validateCargoMetadata(
      canonicalPolicyForTests(),
      cargoMetadata([kernel(), legacyPackage]),
    );

    assert.ok(codes(result).has('blocked_domain'));
  });
}

for (const surface of ['implementation', 'persistence']) {
  test(`prevents Persons from depending on integration ${surface}`, () => {
    const policy = canonicalPolicyForTests();
    const packages = [
      kernel(),
      workspacePackage(
        'makosh-persons-runtime',
        { role: 'domain', owner: 'persons', surface: 'runtime' },
        [dependency(`makosh-mail-${surface}`)],
      ),
      workspacePackage(`makosh-mail-${surface}`, {
        role: 'integration',
        owner: 'mail',
        surface,
      }),
    ];
    const result = validateCargoMetadata(policy, cargoMetadata(packages));

    assert.ok(codes(result).has('implementation_dependency'));
  });
}
