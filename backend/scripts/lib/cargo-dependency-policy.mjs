import { list, violation } from './validation-diagnostics.mjs';

function isAllowedSameOwnerDependency(source, target) {
  if (source.role !== target.role || source.owner !== target.owner) return false;
  if (target.surface === 'contract') return true;

  switch (source.surface) {
    case 'implementation':
      return target.surface === 'implementation';
    case 'persistence':
      return ['implementation', 'persistence'].includes(target.surface);
    case 'runtime':
      return ['implementation', 'persistence'].includes(target.surface);
    case 'assembly':
      return ['implementation', 'persistence', 'runtime'].includes(target.surface);
    default:
      return false;
  }
}

function isKernelCoreGatewayAdapter(policy, source, target, targetPackageName) {
  return source.role === 'core'
    && source.owner === policy.owners.core
    && source.surface === 'runtime'
    && target.role === 'api'
    && target.owner === 'gateway'
    && target.surface === 'implementation'
    && ['makosh-gateway-runtime', 'makosh-gateway-session'].includes(targetPackageName);
}

function isEventHubTransportRuntimeConsumer(policy, source, target, targetPackageName) {
  return targetPackageName === 'makosh-events-jetstream'
    && target
    && target.role === policy.events.role
    && target.owner === policy.events.owner
    && target.surface === 'implementation'
    && ['domain', 'engine', 'integration', 'workflow'].includes(source.role)
    && source.surface === 'runtime';
}

function isAllowedDependency(policy, source, target, targetPackageName) {
  if (isKernelCoreGatewayAdapter(policy, source, target, targetPackageName)) return true;
  if (isEventHubTransportRuntimeConsumer(policy, source, target, targetPackageName)) return true;
  if (source.role === target.role && source.owner === target.owner) {
    return isAllowedSameOwnerDependency(source, target);
  }
  if (target.surface !== 'contract') return false;

  // The Review queue consumes the canonical sanitized Persons candidate event.
  // This is the only domain-to-domain contract edge: it grants no access to
  // Persons implementation, persistence, runtime, or any arbitrary domain.
  if (source.role === 'domain'
    && source.owner === 'review'
    && source.surface === 'runtime'
    && target.role === 'domain'
    && target.owner === 'persons'
    && target.surface === 'contract'
    && targetPackageName === 'makosh-persons-api') return true;

  switch (source.role) {
    case 'domain':
      return target.role === 'platform';
    case 'integration':
      return target.role === 'platform'
        || (target.role === 'engine'
          && policy.dependencies.integrationEngineContractPackages.includes(targetPackageName))
        || (target.role === 'domain'
          && policy.dependencies.integrationDomainContractPackages.includes(targetPackageName))
        || (target.role === 'workflow'
          && policy.dependencies.integrationWorkflowContractPackages.includes(targetPackageName));
    case 'workflow':
      return ['domain', 'integration', 'platform', 'engine', 'api', 'workflow'].includes(
        target.role,
      );
    case 'engine':
      return target.role === 'platform'
        || (target.role === 'engine'
          && policy.dependencies.engineEngineContractPackages.includes(targetPackageName))
        || (target.role === 'domain'
          && policy.dependencies.engineDomainContractPackages.includes(targetPackageName))
        || (target.role === 'workflow'
          && policy.dependencies.engineWorkflowContractPackages.includes(targetPackageName));
    case 'platform':
      return target.role === 'platform';
    case 'api':
      return target.role === 'platform';
    case 'core':
      return ['platform', 'api'].includes(target.role);
    case 'test':
      return target.role !== 'core';
    default:
      return false;
  }
}

export function validateDependencyEdges(policy, packages, descriptors) {
  const violations = [];

  for (const pkg of packages) {
    const source = descriptors.get(pkg.name);
    if (!source) continue;
    const isTelemetryImplementation = source.role === 'platform'
      && source.owner === policy.telemetry.owner
      && source.surface !== 'contract';
    const isEventsProtocol = pkg.name === policy.events.protocolPackage;
    const isRuntimeProtocol = pkg.name === policy.runtimeProtocol.protocolPackage;
    const isVaultContract = [
      policy.vault.protocolPackage,
      policy.vault.keyProviderPackage,
    ].includes(pkg.name);
    const sourceIsVaultOwner = source.role === policy.vault.role
      && source.owner === policy.vault.owner;
    const isStorageProtocol = pkg.name === policy.storage.protocolPackage;
    const sourceIsStorageOwner = source.role === policy.storage.role
      && source.owner === policy.storage.owner;
    const isDevelopmentOperator = source.role === 'development'
      && policy.implementation.developmentProfile.packages
        .some((entry) => entry.package === pkg.name);

    for (const dependency of list(pkg.dependencies)) {
      const kind = dependency.kind ?? 'normal';
      const target = descriptors.get(dependency.name);

      const isSqliteClient = policy.storage.sqliteClientDependencies.includes(dependency.name);
      const isPostgresClient = policy.storage.postgresClientDependencies.includes(dependency.name);
      const testSupportDevDependency = source.role === 'test' && kind === 'dev';

      if (isSqliteClient && !policy.storage.sqlitePackages.includes(pkg.name)
        && !isDevelopmentOperator && !testSupportDevDependency) {
        violations.push(violation(
          'sqlite_dependency',
          `cargo:${pkg.name}:${kind}:${dependency.name}`,
          `${dependency.name} is reserved for the exact private SQLite/SQLCipher packages`,
        ));
      } else if (isPostgresClient && !isAllowedStoragePostgresClient(
        policy,
        source,
        pkg.name,
        kind,
        dependency.name,
        sourceIsStorageOwner,
      )) {
        violations.push(violation(
          'storage_dependency',
          `cargo:${pkg.name}:${kind}:${dependency.name}`,
          `${dependency.name} is allowed only in an owner persistence surface and the exact Storage PostgreSQL adapter`,
        ));
      }

      if (policy.storage.astParserDependencies.includes(dependency.name)
        && pkg.name !== policy.storage.migrationsPackage) {
        violations.push(violation(
          'storage_ast_dependency',
          `cargo:${pkg.name}:${kind}:${dependency.name}`,
          `${dependency.name} is reserved for the exact Storage migration admission package`,
        ));
      }

      if (isStorageProtocol
        && policy.storage.forbiddenProtocolDependencies.includes(dependency.name)) {
        violations.push(violation(
          'storage_protocol_dependency',
          `cargo:${pkg.name}:${kind}:${dependency.name}`,
          `Storage protocol cannot depend on ${dependency.name}`,
        ));
      }

      if (sourceIsStorageOwner
        && !isStorageProtocol
        && policy.storage.forbiddenOwnerDependencies.includes(dependency.name)) {
        violations.push(violation(
          'storage_owner_dependency',
          `cargo:${pkg.name}:${kind}:${dependency.name}`,
          `Storage owner packages cannot depend on broker client ${dependency.name}`,
        ));
      }

      if (isEventsProtocol && policy.events.forbiddenDependencies.includes(dependency.name)) {
        violations.push(violation(
          'events_protocol_dependency',
          `cargo:${pkg.name}:${kind}:${dependency.name}`,
          `canonical events protocol cannot depend on ${dependency.name}`,
        ));
      }

      if (isRuntimeProtocol
        && policy.runtimeProtocol.forbiddenDependencies.includes(dependency.name)) {
        violations.push(violation(
          'runtime_protocol_dependency',
          `cargo:${pkg.name}:${kind}:${dependency.name}`,
          `canonical runtime protocol cannot depend on ${dependency.name}`,
        ));
      }

      if (isVaultContract
        && policy.vault.forbiddenProtocolDependencies.includes(dependency.name)) {
        violations.push(violation(
          'vault_protocol_dependency',
          `cargo:${pkg.name}:${kind}:${dependency.name}`,
          `Vault contract packages cannot depend on ${dependency.name}`,
        ));
      }

      if (sourceIsVaultOwner
        && !isVaultContract
        && policy.vault.forbiddenOwnerDependencies.includes(dependency.name)) {
        violations.push(violation(
          'vault_owner_dependency',
          `cargo:${pkg.name}:${kind}:${dependency.name}`,
          `Vault owner packages cannot depend on broker or PostgreSQL client ${dependency.name}`,
        ));
      }

      if (!target && dependency.name.startsWith('makosh-vault-')) {
        violations.push(violation(
          'vault_private_dependency',
          `cargo:${pkg.name}:${kind}:${dependency.name}`,
          'Vault dependencies must resolve to an exact ADR-0223 workspace package',
        ));
      }

      if (!target && dependency.name.startsWith('makosh-storage-')) {
        violations.push(violation(
          'storage_private_dependency',
          `cargo:${pkg.name}:${kind}:${dependency.name}`,
          'Storage dependencies must resolve to an exact ADR-0224 workspace package',
        ));
      }

      if (!target) {
        if (isTelemetryImplementation && policy.telemetry.forbiddenDependencies.includes(dependency.name)) {
          violations.push(violation(
            'telemetry_dependency',
            `cargo:${pkg.name}:${kind}:${dependency.name}`,
            `telemetry implementation cannot depend on ${dependency.name}`,
          ));
        }
        continue;
      }

      // Dedicated test-support packages exercise real production adapters and
      // runtimes. They never participate in the production dependency graph.
      if (source.role === 'test') continue;

      if (source.role !== 'test' && target.role === 'test') {
        if (kind === 'dev') continue;
        violations.push(violation(
          'production_test_dependency',
          `cargo:${pkg.name}:${kind}:${dependency.name}`,
          'production packages may use test support only as a dev dependency',
        ));
        continue;
      }

      const sourceIsModule = policy.compileIsolation.moduleRoles.includes(source.role);
      const targetIsModule = policy.compileIsolation.moduleRoles.includes(target.role);
      const targetIsVaultOwner = target.role === policy.vault.role
        && target.owner === policy.vault.owner;
      const targetIsStorageOwner = target.role === policy.storage.role
        && target.owner === policy.storage.owner;
      const isSharedStorageVaultRouteConsumer = dependency.name === policy.storage.vaultPackage
        && list(policy.storage.sharedVaultRouteConsumers).includes(pkg.name);
      const isCoreGatewayAdapter = isKernelCoreGatewayAdapter(
        policy,
        source,
        target,
        dependency.name,
      );
      const isEventHubTransportRuntimeConsumerEdge = isEventHubTransportRuntimeConsumer(
        policy,
        source,
        target,
        dependency.name,
      );

      if (!sourceIsVaultOwner
        && source.role !== 'test'
        && targetIsVaultOwner
        && ![
          policy.vault.protocolPackage,
          policy.vault.managedClientPackage,
        ].includes(dependency.name)) {
        violations.push(violation(
          'vault_private_dependency',
          `cargo:${pkg.name}:${kind}:${dependency.name}`,
          'production packages outside Vault may depend only on public Vault contracts',
        ));
        continue;
      }

      if (sourceIsVaultOwner
        && (target.role === 'core' || target.role === 'api' || targetIsModule)) {
        violations.push(violation(
          'vault_owner_dependency',
          `cargo:${pkg.name}:${kind}:${dependency.name}`,
          'Vault owner packages cannot depend on Kernel, Gateway or module packages',
        ));
        continue;
      }

      if (!sourceIsStorageOwner
        && source.role !== 'test'
        && targetIsStorageOwner
        && dependency.name !== policy.storage.protocolPackage
        && target.surface !== 'contract'
        && !isSharedStorageVaultRouteConsumer) {
        violations.push(violation(
          'storage_private_dependency',
          `cargo:${pkg.name}:${kind}:${dependency.name}`,
          'production packages outside Storage may depend only on public Storage contracts',
        ));
        continue;
      }

      if (sourceIsStorageOwner
        && (target.role === 'core' || target.role === 'api' || targetIsModule)) {
        violations.push(violation(
          'storage_owner_dependency',
          `cargo:${pkg.name}:${kind}:${dependency.name}`,
          'Storage owner packages cannot depend on Kernel, Gateway or module packages',
        ));
        continue;
      }

      if (sourceIsModule && target.role === 'core') {
        violations.push(violation(
          'kernel_dependency',
          `cargo:${pkg.name}:${kind}:${dependency.name}`,
          'modules depend on runtime protocols, never on the Kernel implementation',
        ));
        continue;
      }

      if (source.role === 'core' && targetIsModule) {
        violations.push(violation(
          'kernel_module_dependency',
          `cargo:${pkg.name}:${kind}:${dependency.name}`,
          'Kernel discovers modules through protocols and cannot compile owner-specific packages',
        ));
        continue;
      }

      if (source.role === 'api' && targetIsModule) {
        violations.push(violation(
          'gateway_module_dependency',
          `cargo:${pkg.name}:${kind}:${dependency.name}`,
          'Gateway protocol cannot compile owner-specific packages',
        ));
        continue;
      }

      if (policy.compileIsolation.forbidSameOwnerRuntimeDependencies
        && source.role === target.role
        && source.owner === target.owner
        && source.surface === 'runtime'
        && target.surface === 'runtime') {
        violations.push(violation(
          'runtime_aggregation_dependency',
          `cargo:${pkg.name}:${kind}:${dependency.name}`,
          'a runtime composes its implementation and adapters, not another runtime',
        ));
        continue;
      }

      if (isDevelopmentOperator) continue;

      if (policy.compileIsolation.forbidCrossOwnerPersistenceDependencies
        && source.owner !== target.owner
        && source.surface === 'persistence'
        && target.surface === 'persistence') {
        violations.push(violation(
          'cross_owner_persistence_dependency',
          `cargo:${pkg.name}:${kind}:${dependency.name}`,
          'persistence adapters cannot depend on another owner persistence adapter',
        ));
        continue;
      }

      if (source.role === 'integration'
        && target.role === 'engine'
        && target.surface === 'contract'
        && !policy.dependencies.integrationEngineContractPackages.includes(dependency.name)) {
        violations.push(violation(
          'integration_engine_contract_dependency',
          `cargo:${pkg.name}:${kind}:${dependency.name}`,
          'integrations may publish engine observations only through an explicitly allowed contract package',
        ));
        continue;
      }

      if (source.role === 'integration'
        && target.role === 'domain'
        && target.surface === 'contract'
        && !policy.dependencies.integrationDomainContractPackages.includes(dependency.name)) {
        violations.push(violation(
          'integration_domain_contract_dependency',
          `cargo:${pkg.name}:${kind}:${dependency.name}`,
          'integrations may publish domain-neutral evidence only through an explicitly allowed ingress package',
        ));
        continue;
      }

      if (source.role === 'integration'
        && target.role === 'workflow'
        && target.surface === 'contract'
        && !policy.dependencies.integrationWorkflowContractPackages.includes(dependency.name)) {
        violations.push(violation(
          'integration_workflow_contract_dependency',
          `cargo:${pkg.name}:${kind}:${dependency.name}`,
          'integrations may hand off provider-neutral work only through an explicitly allowed target-owned workflow contract package',
        ));
        continue;
      }

      if (source.role === 'engine'
        && target.role === 'engine'
        && source.owner !== target.owner
        && target.surface === 'contract'
        && !policy.dependencies.engineEngineContractPackages.includes(dependency.name)) {
        violations.push(violation(
          'engine_engine_contract_dependency',
          `cargo:${pkg.name}:${kind}:${dependency.name}`,
          'engines may exchange durable facts only through an explicitly allowed target-owned contract package',
        ));
        continue;
      }

      if (source.role === 'engine'
        && target.role === 'domain'
        && target.surface === 'contract'
        && !policy.dependencies.engineDomainContractPackages.includes(dependency.name)) {
        violations.push(violation(
          'engine_domain_contract_dependency',
          `cargo:${pkg.name}:${kind}:${dependency.name}`,
          'engines may consume or publish domain facts only through an explicitly allowed contract package',
        ));
        continue;
      }

      if (source.role === 'engine'
        && target.role === 'workflow'
        && target.surface === 'contract'
        && !policy.dependencies.engineWorkflowContractPackages.includes(dependency.name)) {
        violations.push(violation(
          'engine_workflow_contract_dependency',
          `cargo:${pkg.name}:${kind}:${dependency.name}`,
          'engines may serve workflow custody only through an explicitly allowed target-owned contract package',
        ));
        continue;
      }

      if (source.owner !== target.owner
        && target.surface !== 'contract'
        && !isSharedStorageVaultRouteConsumer
        && !isCoreGatewayAdapter
        && !isEventHubTransportRuntimeConsumerEdge) {
        violations.push(violation(
          'implementation_dependency',
          `cargo:${pkg.name}:${kind}:${dependency.name}`,
          'cross-owner dependencies must target contracts, not implementations',
        ));
        continue;
      }

      if (!isSharedStorageVaultRouteConsumer
        && !isCoreGatewayAdapter
        && !isEventHubTransportRuntimeConsumerEdge
        && !isAllowedDependency(policy, source, target, dependency.name)) {
        violations.push(violation(
          'forbidden_dependency',
          `cargo:${pkg.name}:${kind}:${dependency.name}`,
          `${source.role}:${source.owner} cannot depend on ${target.role}:${target.owner}`,
        ));
      }
    }
  }

  return violations;
}

function isAllowedStoragePostgresClient(
  policy,
  source,
  packageName,
  dependencyKind,
  dependencyName,
  sourceIsStorageOwner,
) {
  if (!policy.storage.allowedPostgresClientDependencies.includes(dependencyName)) return false;
  const testOnlyKey = `${packageName}:${dependencyKind}:${dependencyName}`;
  if (policy.storage.testSupportPostgresClientAllowlist.includes(testOnlyKey)) {
    return source.role === 'test' && source.surface === 'test_support' && dependencyKind === 'dev';
  }
  if (dependencyName === 'sqlx') {
    return source.surface === 'persistence'
      && (!sourceIsStorageOwner || packageName === policy.storage.postgresPackage);
  }
  return dependencyName === 'tokio-postgres'
    && sourceIsStorageOwner
    && source.surface === 'implementation'
    && packageName === policy.storage.pgbouncerPackage;
}
