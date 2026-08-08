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


export function eventsProtocol(dependencies = [], metadataOverrides = {}) {
  return workspacePackage(
    'makosh-events-protocol',
    {
      role: 'platform',
      owner: 'events',
      surface: 'contract',
      ...metadataOverrides,
    },
    dependencies,
  );
}
export function metadata(packages) {
  const requiredProtocols = [eventsProtocol(), runtimeProtocol()]
    .filter((protocol) => !packages.some(({ name }) => name === protocol.name));
  return fixtureMetadata([...requiredProtocols, ...packages]);
}
