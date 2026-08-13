import {
  duplicates,
  list,
  ownerAliases,
  pathTokens,
  violation,
} from './validation-diagnostics.mjs';

const MAKOSH_METADATA_KEYS = new Set(['role', 'owner', 'surface', 'components']);
const HOST_EXECUTION_DEPENDENCIES = new Set([
  'tauri',
  'wry',
  'webkit2gtk',
  'webview2-com',
]);

function claimsCanonicalEventsOwner(policy, descriptor) {
  return descriptor?.role === policy.events.role
    && descriptor?.owner === policy.events.owner
    && descriptor?.surface === policy.events.surface;
}

function isExactEventsProtocolDescriptor(policy, descriptor) {
  return claimsCanonicalEventsOwner(policy, descriptor)
    && descriptor.components.length === 0;
}

function claimsCanonicalRuntimeProtocolOwner(policy, descriptor) {
  return descriptor?.role === policy.runtimeProtocol.role
    && descriptor?.owner === policy.runtimeProtocol.owner
    && descriptor?.surface === policy.runtimeProtocol.surface;
}

function isExactRuntimeProtocolDescriptor(policy, descriptor) {
  return claimsCanonicalRuntimeProtocolOwner(policy, descriptor)
    && descriptor.components.length === 0;
}

function isExactDeclaredPreCutoverContactTokenPackage(policy, pkg, descriptor) {
  if (!policy.domains.registered.includes('persons')
    || policy.domains.registered.includes('contacts')
    || !policy.domains.developmentAllowlist.includes('persons')
    || policy.domains.developmentAllowlist.includes('contacts')) return false;

  return list(policy?.implementation?.productionPackages).some((entry) => (
    entry?.name === pkg.name
    && entry?.role === descriptor.role
    && entry?.owner === descriptor.owner
    && entry?.surface === descriptor.surface
  ));
}

function isExactPreCutoverContactsInventoryPackage(policy, pkg, descriptor) {
  return descriptor?.owner === 'contacts'
    && isExactDeclaredPreCutoverContactTokenPackage(policy, pkg, descriptor);
}

function vaultPackageSpecifications(policy) {
  return [
    { name: policy.vault.protocolPackage, surface: 'contract', components: [] },
    { name: policy.vault.managedClientPackage, surface: 'contract', components: [] },
    { name: policy.vault.keyProviderPackage, surface: 'contract', components: [] },
    {
      name: policy.vault.runtimePackage,
      surface: 'runtime',
      components: [policy.vault.runtimeComponent],
    },
    { name: policy.vault.storePackage, surface: 'persistence', components: [] },
    ...list(policy.vault.platformKeyAdapterPackages).map((name) => ({
      name,
      surface: 'implementation',
      components: [],
    })),
  ];
}

function claimsVaultOwner(policy, descriptor) {
  return descriptor?.role === policy.vault.role
    && descriptor?.owner === policy.vault.owner;
}

function isExactVaultPackageDescriptor(policy, descriptor, specification) {
  return claimsVaultOwner(policy, descriptor)
    && descriptor.surface === specification.surface
    && descriptor.components.length === specification.components.length
    && descriptor.components.every(
      (component, index) => component === specification.components[index],
    );
}

function storagePackageSpecifications(policy) {
  return [
    { name: policy.storage.protocolPackage, surface: 'contract', components: [] },
    { name: policy.storage.controlPackage, surface: 'implementation', components: [] },
    { name: policy.storage.vaultPackage, surface: 'contract', components: [] },
    {
      name: policy.storage.runtimePackage,
      surface: 'runtime',
      components: [policy.storage.runtimeComponent],
    },
    { name: policy.storage.postgresPackage, surface: 'persistence', components: [] },
    { name: policy.storage.pgbouncerPackage, surface: 'implementation', components: [] },
    { name: policy.storage.migrationsPackage, surface: 'implementation', components: [] },
  ];
}

function claimsStorageOwner(policy, descriptor) {
  return descriptor?.role === policy.storage.role
    && descriptor?.owner === policy.storage.owner;
}

function isExactStoragePackageDescriptor(policy, descriptor, specification) {
  return claimsStorageOwner(policy, descriptor)
    && descriptor.surface === specification.surface
    && descriptor.components.length === specification.components.length
    && descriptor.components.every(
      (component, index) => component === specification.components[index],
    );
}

function metadataDescriptor(policy, pkg, violations) {
  const metadataKey = policy.cargo.metadataKey;
  const metadata = pkg?.metadata?.[metadataKey];
  const location = `cargo:${pkg?.name ?? '<unknown>'}`;

  if (!metadata || typeof metadata !== 'object' || Array.isArray(metadata)) {
    violations.push(violation('missing_metadata', location, `missing [package.metadata.${metadataKey}]`));
    return null;
  }

  for (const key of Object.keys(metadata)) {
    if (!MAKOSH_METADATA_KEYS.has(key)) {
      violations.push(violation('unknown_metadata_key', location, `unknown Макошь metadata key: ${key}`));
    }
  }

  const { role, owner, surface } = metadata;
  const components = metadata.components ?? [];

  if (typeof role !== 'string' || !policy.cargo.roles.includes(role)) {
    const blockedRole = typeof role === 'string' && policy.projections.blockedRoles.includes(role);
    violations.push(violation(
      blockedRole ? 'blocked_projection' : 'invalid_role',
      location,
      `role must be exactly one of: ${policy.cargo.roles.join(', ')}`,
    ));
  }
  if (typeof owner !== 'string' || owner.length === 0) {
    violations.push(violation('invalid_owner', location, 'owner must be a non-empty string'));
  }
  if (typeof surface !== 'string' || !policy.cargo.surfaces.includes(surface)) {
    violations.push(violation(
      'invalid_surface',
      location,
      `surface must be one of: ${policy.cargo.surfaces.join(', ')}`,
    ));
  }
  if (!Array.isArray(components) || components.some((component) => typeof component !== 'string')) {
    violations.push(violation('invalid_components', location, 'components must be an array of strings'));
  } else if (duplicates(components).length > 0) {
    violations.push(violation('duplicate_component', location, 'components must be unique'));
  }

  if (typeof role !== 'string' || typeof owner !== 'string' || typeof surface !== 'string') return null;
  return { role, owner, surface, components: Array.isArray(components) ? components : [] };
}

function validateDescriptor(policy, pkg, descriptor, violations) {
  const location = `cargo:${pkg.name}`;
  if (policy.compileIsolation.forbiddenAggregatePackages.includes(pkg.name)) {
    violations.push(violation(
      'forbidden_aggregate_package',
      location,
      'aggregate packages collapse module ownership and compile isolation',
    ));
  }
  if (!descriptor) return;
  const { role, owner, surface, components } = descriptor;
  const blockedDomains = new Set(policy.domains.blocked);
  const blockedProjections = new Set(policy.projections.blockedOwners);
  const ownerTokens = new Set(pathTokens(owner));

  for (const blocked of blockedDomains) {
    if ([...ownerAliases(blocked)].some((alias) => ownerTokens.has(alias))) {
      violations.push(violation('blocked_domain', location, `owner contains blocked domain ${blocked}`));
    }
  }
  for (const blocked of blockedProjections) {
    if ([...ownerAliases(blocked)].some((alias) => ownerTokens.has(alias))) {
      violations.push(violation('blocked_projection', location, `owner contains blocked projection ${blocked}`));
    }
  }

  const packageTokens = new Set(pathTokens(pkg.name));
  for (const blocked of blockedDomains) {
    if ([...ownerAliases(blocked)].some((alias) => packageTokens.has(alias))) {
      violations.push(violation('blocked_domain', location, `package name contains blocked domain ${blocked}`));
    }
  }
  for (const blocked of blockedProjections) {
    if ([...ownerAliases(blocked)].some((alias) => packageTokens.has(alias))) {
      violations.push(violation('blocked_projection', location, `package name contains blocked projection ${blocked}`));
    }
  }

  const claimsRetiredContactsOwner = [...ownerAliases('contacts')].some(
    (alias) => ownerTokens.has(alias) || packageTokens.has(alias),
  );
  if (policy.domains.registered.includes('persons')
    && !policy.domains.registered.includes('contacts')
    && claimsRetiredContactsOwner
    && !isExactDeclaredPreCutoverContactTokenPackage(policy, pkg, descriptor)) {
    violations.push(violation(
      'blocked_domain',
      location,
      'retired contacts identity is allowed only for the exact pre-cutover production inventory',
    ));
  }

  if (!pkg.name.startsWith(policy.cargo.packagePrefix)) {
    violations.push(violation('package_prefix', location, `workspace package must start with ${policy.cargo.packagePrefix}`));
  }
  if (role === 'domain'
    && !policy.domains.developmentAllowlist.includes(owner)
    && !isExactPreCutoverContactsInventoryPackage(policy, pkg, descriptor)) {
    violations.push(violation('blocked_domain', location, `domain owner ${owner} is not in the development allowlist`));
  }
  if (role === 'integration' && policy.domains.registered.some(
    (domain) => [...ownerAliases(domain)].some((alias) => ownerTokens.has(alias)),
  )) {
    violations.push(violation('invalid_owner', location, 'integration owner cannot use a business domain identity'));
  }
  if (role === 'integration'
    && list(policy.integrations.hostOnlyProviderExecutionOwners).includes(owner)
    && list(pkg.dependencies).some(({ name }) => HOST_EXECUTION_DEPENDENCIES.has(name))) {
    violations.push(violation(
      'host_execution_dependency',
      location,
      `${owner} provider execution is host-only and cannot depend on a browser or WebView runtime from backend`,
    ));
  }
  if (role === 'platform' && !policy.owners.platform.includes(owner)) {
    violations.push(violation('invalid_owner', location, `unknown platform owner ${owner}`));
  }
  if (role === 'api' && !policy.owners.api.includes(owner)) {
    violations.push(violation('invalid_owner', location, `unknown API owner ${owner}`));
  }
  const configuredCorePackage = list(policy.kernel.packages).find((entry) => entry?.name === pkg.name);
  if (role === 'core' && (owner !== policy.owners.core
    || !configuredCorePackage
    || surface !== configuredCorePackage.surface)) {
    violations.push(violation(
      'invalid_core_package',
      location,
      'core role is reserved for an explicitly configured Kernel-owned package and surface',
    ));
  }
  if (configuredCorePackage && (role !== 'core'
    || owner !== policy.owners.core
    || surface !== configuredCorePackage.surface)) {
    violations.push(violation(
      'invalid_core_package',
      location,
      'configured Kernel-owned package must use the core role, Kernel owner and declared surface',
    ));
  }
  if (role === 'core' && pkg.name !== policy.kernel.package && components.length > 0) {
    violations.push(violation(
      'invalid_core_component',
      location,
      'only the configured Kernel runtime package may declare Kernel components',
    ));
  }
  if (role === 'test' && (owner !== policy.owners.test || surface !== 'test_support')) {
    violations.push(violation('invalid_test_package', location, 'test role requires owner=test and surface=test_support'));
  }
  const configuredDevelopmentPackage = list(
    policy.implementation?.developmentProfile?.packages,
  ).find((entry) => entry?.package === pkg.name);
  if (role === 'development' && (
    owner !== policy.owners.development
    || !configuredDevelopmentPackage
    || surface !== configuredDevelopmentPackage.surface
  )) {
    violations.push(violation(
      'invalid_development_package',
      location,
      'development role requires owner=development and an exact configured package surface',
    ));
  }
  if (configuredDevelopmentPackage && (
    role !== 'development'
    || owner !== policy.owners.development
    || surface !== configuredDevelopmentPackage.surface
  )) {
    violations.push(violation(
      'invalid_development_package',
      location,
      'configured development package must use the development role, owner and declared surface',
    ));
  }

  if (components.includes(policy.vault.runtimeComponent)
    && pkg.name !== policy.vault.runtimePackage) {
    violations.push(violation(
      'invalid_vault_package',
      location,
      `${policy.vault.runtimeComponent} is exclusive to ${policy.vault.runtimePackage}`,
    ));
  }

  if (components.includes(policy.storage.runtimeComponent)
    && pkg.name !== policy.storage.runtimePackage) {
    violations.push(violation(
      'invalid_storage_package',
      location,
      `${policy.storage.runtimeComponent} is exclusive to ${policy.storage.runtimePackage}`,
    ));
  }

  for (const component of components) {
    if (policy.kernel.exclusiveComponents.includes(component) && pkg.name !== policy.kernel.package) {
      violations.push(violation('exclusive_kernel_component', location, `${component} is exclusive to ${policy.kernel.package}`));
    }
  }
  for (const component of policy.kernel.exclusiveComponents) {
    const packageToken = component.replaceAll('_', '-');
    if (pkg.name.includes(packageToken) && pkg.name !== policy.kernel.package) {
      violations.push(violation('exclusive_kernel_component', location, `${packageToken} package is forbidden outside Kernel`));
    }
  }

  const collectorPackageToken = policy.telemetry.collectorComponent.replaceAll('_', '-');
  const declaresCollector = components.includes(policy.telemetry.collectorComponent);
  if (pkg.name.includes(collectorPackageToken) && !declaresCollector) {
    violations.push(violation('invalid_telemetry_collector', location, 'Telemetry Collector package must declare its component'));
  }
  if (declaresCollector) {
    const validCollector = role === 'platform'
      && owner === policy.telemetry.owner
      && pkg.name !== policy.kernel.package
      && ['implementation', 'runtime'].includes(surface);
    if (!validCollector) {
      violations.push(violation(
        'invalid_telemetry_collector',
        location,
        'Telemetry Collector must be a separate telemetry platform implementation/runtime',
      ));
    }
  }
}

export function inspectWorkspacePackages(policy, cargoMetadata) {
  const violations = [];
  const workspaceIds = new Set(list(cargoMetadata?.workspace_members));
  const packages = list(cargoMetadata?.packages).filter(
    (pkg) => workspaceIds.size === 0 || workspaceIds.has(pkg.id),
  );
  const byName = new Map();
  const descriptors = new Map();

  for (const pkg of packages) {
    if (byName.has(pkg.name)) {
      violations.push(violation('duplicate_package', `cargo:${pkg.name}`, 'workspace package names must be unique'));
    }
    byName.set(pkg.name, pkg);
    const descriptor = metadataDescriptor(policy, pkg, violations);
    descriptors.set(pkg.name, descriptor);
    validateDescriptor(policy, pkg, descriptor, violations);
  }

  const productionPackages = packages.filter(
    (pkg) => ![policy.owners.test, policy.owners.development].includes(
      descriptors.get(pkg.name)?.role,
    ),
  );
  const productionWorkspaceExists = productionPackages.length > 0;

  if (productionWorkspaceExists && !byName.has(policy.kernel.package)) {
    violations.push(violation('missing_kernel_package', 'cargo:workspace', `${policy.kernel.package} is required when a workspace exists`));
  }

  const eventsProtocolPackage = policy.events.protocolPackage;
  if (productionWorkspaceExists && !byName.has(eventsProtocolPackage)) {
    violations.push(violation(
      'missing_events_protocol_package',
      'cargo:workspace',
      `${eventsProtocolPackage} is required when a workspace exists`,
    ));
  }

  const eventsProtocolDescriptor = descriptors.get(eventsProtocolPackage);
  if (byName.has(eventsProtocolPackage)
    && !isExactEventsProtocolDescriptor(policy, eventsProtocolDescriptor)) {
    violations.push(violation(
      'invalid_events_protocol_package',
      `cargo:${eventsProtocolPackage}`,
      `${eventsProtocolPackage} must be the component-free ${policy.events.role}:${policy.events.owner}:${policy.events.surface} package`,
    ));
  }

  const eventsOwnerPackages = [...descriptors.entries()]
    .filter(([, descriptor]) => claimsCanonicalEventsOwner(policy, descriptor))
    .map(([packageName]) => packageName);
  if (productionWorkspaceExists
    && (eventsOwnerPackages.length !== 1 || eventsOwnerPackages[0] !== eventsProtocolPackage)) {
    violations.push(violation(
      'events_protocol_owner',
      'cargo:workspace',
      `${eventsProtocolPackage} must be the only package claiming the canonical events protocol owner`,
    ));
  }

  const runtimeProtocolPackage = policy.runtimeProtocol.protocolPackage;
  if (productionWorkspaceExists && !byName.has(runtimeProtocolPackage)) {
    violations.push(violation(
      'missing_runtime_protocol_package',
      'cargo:workspace',
      `${runtimeProtocolPackage} is required when a workspace exists`,
    ));
  }

  const runtimeProtocolDescriptor = descriptors.get(runtimeProtocolPackage);
  if (byName.has(runtimeProtocolPackage)
    && !isExactRuntimeProtocolDescriptor(policy, runtimeProtocolDescriptor)) {
    violations.push(violation(
      'invalid_runtime_protocol_package',
      `cargo:${runtimeProtocolPackage}`,
      `${runtimeProtocolPackage} must be the component-free ${policy.runtimeProtocol.role}:${policy.runtimeProtocol.owner}:${policy.runtimeProtocol.surface} package`,
    ));
  }

  const runtimeProtocolOwnerPackages = [...descriptors.entries()]
    .filter(([, descriptor]) => claimsCanonicalRuntimeProtocolOwner(policy, descriptor))
    .map(([packageName]) => packageName);
  if (productionWorkspaceExists
    && (runtimeProtocolOwnerPackages.length !== 1
      || runtimeProtocolOwnerPackages[0] !== runtimeProtocolPackage)) {
    violations.push(violation(
      'runtime_protocol_owner',
      'cargo:workspace',
      `${runtimeProtocolPackage} must be the only package claiming the canonical runtime protocol owner`,
    ));
  }

  const vaultSpecifications = vaultPackageSpecifications(policy);
  const configuredVaultPackageNames = new Set(
    vaultSpecifications.map(({ name }) => name),
  );
  const claimedVaultPackages = [...descriptors.entries()]
    .filter(([, descriptor]) => claimsVaultOwner(policy, descriptor))
    .map(([packageName]) => packageName);
  const vaultBoundaryPresent = claimedVaultPackages.length > 0
    || packages.some(({ name, dependencies }) => configuredVaultPackageNames.has(name)
      || list(dependencies).some(({ name: dependencyName }) => (
        configuredVaultPackageNames.has(dependencyName)
      )));

  if (vaultBoundaryPresent) {
    for (const specification of vaultSpecifications) {
      if (!byName.has(specification.name)) {
        violations.push(violation(
          'missing_vault_package',
          'cargo:workspace',
          `${specification.name} is required once the Vault package boundary exists`,
        ));
        continue;
      }

      if (!isExactVaultPackageDescriptor(
        policy,
        descriptors.get(specification.name),
        specification,
      )) {
        violations.push(violation(
          'invalid_vault_package',
          `cargo:${specification.name}`,
          `${specification.name} must be exactly ${policy.vault.role}:${policy.vault.owner}:${specification.surface} with declared Vault components only`,
        ));
      }
    }

    for (const packageName of claimedVaultPackages) {
      if (!configuredVaultPackageNames.has(packageName)) {
        violations.push(violation(
          'invalid_vault_package',
          `cargo:${packageName}`,
          'Vault owner packages are closed to the exact ADR-0223 package set',
        ));
      }
    }
  }


  const storageSpecifications = storagePackageSpecifications(policy);
  const configuredStoragePackageNames = new Set(
    storageSpecifications.map(({ name }) => name),
  );
  const claimedStoragePackages = [...descriptors.entries()]
    .filter(([, descriptor]) => claimsStorageOwner(policy, descriptor))
    .map(([packageName]) => packageName);
  const storageBoundaryPresent = claimedStoragePackages.length > 0
    || packages.some(({ name, dependencies }) => configuredStoragePackageNames.has(name)
      || list(dependencies).some(({ name: dependencyName }) => (
        configuredStoragePackageNames.has(dependencyName)
      )));

  if (storageBoundaryPresent) {
    for (const specification of storageSpecifications) {
      if (!byName.has(specification.name)) {
        violations.push(violation(
          'missing_storage_package',
          'cargo:workspace',
          `${specification.name} is required once the Storage Control package boundary exists`,
        ));
        continue;
      }

      if (!isExactStoragePackageDescriptor(
        policy,
        descriptors.get(specification.name),
        specification,
      )) {
        violations.push(violation(
          'invalid_storage_package',
          `cargo:${specification.name}`,
          `${specification.name} must be exactly ${policy.storage.role}:${policy.storage.owner}:${specification.surface} with declared Storage components only`,
        ));
      }
    }

    for (const packageName of claimedStoragePackages) {
      if (!configuredStoragePackageNames.has(packageName)) {
        violations.push(violation(
          'invalid_storage_package',
          `cargo:${packageName}`,
          'Storage owner packages are closed to the exact ADR-0224 package set',
        ));
      }
    }
  }

  const kernelDescriptor = descriptors.get(policy.kernel.package);
  if (kernelDescriptor) {
    for (const component of kernelDescriptor.components) {
      if (!policy.kernel.constitutionalComponents.includes(component)) {
        violations.push(violation(
          'unknown_kernel_component',
          `cargo:${policy.kernel.package}`,
          `${component} is not a constitutional Kernel component`,
        ));
      }
    }
  }

  return { violations, packages, descriptors };
}
