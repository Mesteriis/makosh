import { readFile } from 'node:fs/promises';

import { validateAiContextPolicy } from './ai-context-policy.mjs';
import { validateImplementationSlicePolicy } from './implementation-slice-policy.mjs';
import { validatePhaseGatePolicy } from './phase-gate-policy.mjs';
import { duplicates, list, violation } from './validation-diagnostics.mjs';

const CONTROL_STORE_POLICY_KEYS = [
  'owner',
  'storageBoundary',
  'forbiddenData',
];

const CONTROL_STORE_FORBIDDEN_DATA = [
  'secret_values',
  'credential_leases',
  'vault_root_record_and_wrapping_keys',
  'vault_key_slots',
  'vault_secret_record_ids_and_bindings',
  'provider_sessions',
  'private_content',
];

const EVENT_POLICY_KEYS = [
  'protocolPackage',
  'role',
  'owner',
  'surface',
  'serialization',
  'envelopeMajorVersion',
  'kinds',
  'kindMetadata',
  'payloadBinding',
  'payloadVisibility',
  'outboxPublishMode',
  'clientEnvelopeReuseAllowed',
  'brokerAckIsEnvelopeAck',
  'unknownMajorVersion',
  'automaticFormatFallbackEnabled',
  'forbiddenPayloadData',
  'forbiddenDependencies',
];

const EVENT_KINDS = ['command', 'event', 'observation', 'result', 'ack'];

const EVENT_FORBIDDEN_PAYLOAD_DATA = [
  'secret_values',
  'credential_leases',
  'provider_sessions',
  'private_keys',
  'private_content',
];

const EVENT_FORBIDDEN_DEPENDENCIES = [
  'async-nats',
  'nats',
  'sqlx',
  'tokio-postgres',
  'postgres',
  'diesel',
  'sea-orm',
  'rusqlite',
  'serde_json',
];

const RUNTIME_PROTOCOL_POLICY_KEYS = [
  'protocolPackage',
  'role',
  'owner',
  'surface',
  'serialization',
  'descriptorMajorVersion',
  'descriptorDigestSource',
  'descriptorCarriesOwnDigest',
  'approvalUnit',
  'grantRule',
  'dependencyBinding',
  'managedArtifactBinding',
  'unknownMajorState',
  'automaticFormatFallbackEnabled',
  'wildcardPermissionsAllowed',
  'arbitraryMapsAllowed',
  'hostSandboxEnforcement',
  'forbiddenDescriptorData',
  'forbiddenDependencies',
];

const RUNTIME_PROTOCOL_FORBIDDEN_DESCRIPTOR_DATA = [
  'secret_values',
  'provider_sessions',
  'account_private_identifiers',
  'private_content',
  'sql_and_storage_details',
  'broker_topology_and_credentials',
  'executable_or_remote_code',
  'filesystem_paths_and_process_addresses',
  'active_settings_values',
  'runtime_state',
  'business_entities_and_inline_owner_payload_schemas',
];

const RUNTIME_PROTOCOL_FORBIDDEN_DEPENDENCIES = [
  'async-nats',
  'nats',
  'sqlx',
  'tokio-postgres',
  'postgres',
  'diesel',
  'sea-orm',
  'rusqlite',
  'serde_json',
];

const SETTINGS_POLICY_KEYS = [
  'registryOwner',
  'registryComponent',
  'schemaProtocolPackage',
  'schemaSerialization',
  'sourceOfTruth',
  'postgresRequired',
  'authorities',
  'clientVisibilities',
  'authorityVisibilityRule',
  'targetScopes',
  'composition',
  'crossOwnerAtomicMutationEnabled',
  'schemaBinding',
  'valueModel',
  'arbitraryJsonMapsOrAnyAllowed',
  'secretValuesAllowed',
  'credentialDelivery',
  'operatorManagedWriter',
  'kernelManagedWriter',
  'grantMutationAllowed',
  'revisionModel',
  'mutationConcurrency',
  'applyModes',
  'managedRestart',
  'externalRestart',
  'applyFailureState',
  'automaticRollbackEnabled',
  'forbiddenState',
];

const SETTINGS_AUTHORITIES = ['operator_managed', 'kernel_managed'];

const SETTINGS_CLIENT_VISIBILITIES = ['editable', 'read_only', 'hidden'];

const SETTINGS_TARGET_SCOPES = [
  'module_registration',
  'capability',
  'configuration_instance',
];

const SETTINGS_APPLY_MODES = [
  'hot_reload',
  'restart_capability',
  'restart_module',
];

const SETTINGS_FORBIDDEN_STATE = [
  'business_state',
  'provider_sessions_and_accounts',
  'inbox_outbox_events_and_results',
  'cursors_checkpoints_and_sync_state',
  'scheduler_schedules_runs_and_leases',
  'process_health_and_restart_history',
  'prompts_documents_embeddings_and_indexes',
  'credentials_keys_and_secrets',
  'secret_references_and_credential_bindings',
  'arbitrary_cache',
];

const VAULT_POLICY_KEYS = [
  'role',
  'owner',
  'protocolPackage',
  'managedClientPackage',
  'keyProviderPackage',
  'runtimePackage',
  'storePackage',
  'platformKeyAdapterPackages',
  'runtimeComponent',
  'processBoundary',
  'managementMode',
  'externalRegistrationEnabled',
  'alternativeTopologyEnabled',
  'storageFormat',
  'platformKeyStorage',
  'recoveryModel',
  'recoveryKeyEncoding',
  'unlockModes',
  'defaultUnlockMode',
  'automaticInitializeOrRestoreEnabled',
  'transport',
  'transportScope',
  'kernelPayloadVisibility',
  'leasePersistence',
  'defaultLeaseTtlSeconds',
  'maximumLeaseTtlSeconds',
  'leaseResolveLimit',
  'leaseRenewal',
  'credentialPayloadMaxBytes',
  'sessionCredentialBlobMaxBytes',
  'leaseBindingFields',
  'runtimeGenerationChangeInvalidatesLeases',
  'grantEpochChangeInvalidatesLeases',
  'forbiddenSecretCarriers',
  'forbiddenProtocolDependencies',
  'forbiddenOwnerDependencies',
];

const VAULT_PLATFORM_KEY_ADAPTER_PACKAGES = ['makosh-vault-key-provider-file'];

const VAULT_UNLOCK_MODES = [
  'file_adapter_auto',
  'manual_local',
  'recovery_offline',
];

const VAULT_LEASE_BINDING_FIELDS = [
  'lease_id',
  'vault_instance_id',
  'vault_runtime_generation',
  'secret_revision',
  'logical_owner_id',
  'configuration_instance_id',
  'purpose_id',
  'actions',
  'audience_module_registration_id',
  'audience_runtime_instance_id',
  'grant_epoch',
  'issued_at',
  'expires_at',
  'single_resolve',
];

const VAULT_FORBIDDEN_SECRET_CARRIERS = [
  'nats',
  'durable_events',
  'sse',
  'settings',
  'kernel_control_store',
  'argv',
  'environment',
  'logs',
  'errors',
  'health',
  'crash_reports',
  'filesystem_spool',
];

const VAULT_FORBIDDEN_PROTOCOL_DEPENDENCIES = [
  'async-nats',
  'nats',
  'sqlx',
  'tokio-postgres',
  'postgres',
  'diesel',
  'sea-orm',
  'rusqlite',
  'serde_json',
];

const VAULT_FORBIDDEN_OWNER_DEPENDENCIES = [
  'async-nats',
  'nats',
  'sqlx',
  'tokio-postgres',
  'postgres',
  'diesel',
  'sea-orm',
];

const STORAGE_POLICY_KEYS = [
  'role',
  'owner',
  'protocolPackage',
  'controlPackage',
  'vaultPackage',
  'sharedVaultRouteConsumers',
  'runtimePackage',
  'postgresPackage',
  'pgbouncerPackage',
  'migrationsPackage',
  'runtimeComponent',
  'managementMode',
  'protocolSerialization',
  'clusterTopology',
  'databaseTopology',
  'fixedSchemas',
  'runtimePath',
  'poolMode',
  'minimumPgbouncerVersion',
  'runtimePrincipalScope',
  'ownerDdlRole',
  'runtimeLoginRole',
  'runtimePoolAliasScope',
  'directConnectionAudiences',
  'credentialDelivery',
  'bindingFields',
  'revocationSequence',
  'migrationArtifact',
  'migrationTrustBinding',
  'migrationExecution',
  'migrationDirection',
  'migrationValidation',
  'sharedTechnicalAccess',
  'sharedTechnicalFunctions',
  'moduleSelfMigrationsEnabled',
  'destructiveMigrationsEnabled',
  'kernelSqlProxyEnabled',
  'regexOnlyValidationAllowed',
  'crossOwnerBusinessSqlEnabled',
  'directSharedTechnicalDmlEnabled',
  'postgresRoleHardLimitsRequired',
  'pgbouncerSoleBudgetBoundary',
  'clientDependencies',
  'postgresClientDependencies',
  'allowedPostgresClientDependencies',
  'testSupportPostgresClientAllowlist',
  'sqliteClientDependencies',
  'sqlitePackages',
  'astParserDependencies',
  'forbiddenProtocolDependencies',
  'forbiddenOwnerDependencies',
];

const STORAGE_FIXED_SCHEMAS = [
  'makosh_data',
  'makosh_platform',
  'makosh_extensions',
];

const STORAGE_DIRECT_CONNECTION_AUDIENCES = [
  'bootstrap',
  'migration',
  'backup_restore',
  'controlled_admin',
];

const STORAGE_BINDING_FIELDS = [
  'storage_instance_id',
  'storage_generation',
  'database_id',
  'owner',
  'registration_id',
  'runtime_instance_id',
  'runtime_generation',
  'grant_epoch',
  'role_epoch',
  'runtime_principal',
  'pool_alias',
  'effective_budgets',
  'credential_lease_revision',
  'storage_bundle_revision',
  'storage_bundle_digest',
];

const STORAGE_REVOCATION_SEQUENCE = [
  'mark_revoking_and_increment_role_epoch',
  'stop_new_sessions_and_leases',
  'quiesce_runtime_and_set_nologin',
  'disable_drain_kill_pool_alias',
  'terminate_postgresql_backends_and_verify_zero',
  'rotate_role_credential',
  'audit_and_issue_new_generation_binding',
];

const STORAGE_SHARED_TECHNICAL_FUNCTIONS = [
  'makosh_platform.events_append_outbox_v1',
  'makosh_platform.events_accept_inbox_v1',
];

const STORAGE_CLIENT_DEPENDENCIES = [
  'sqlx',
  'tokio-postgres',
  'postgres',
  'diesel',
  'sea-orm',
  'rusqlite',
];

const STORAGE_POSTGRES_CLIENT_DEPENDENCIES = [
  'sqlx',
  'tokio-postgres',
  'postgres',
  'diesel',
  'sea-orm',
];

const STORAGE_SQLITE_PACKAGES = [
  'makosh-kernel-control-store-sqlite',
  'makosh-vault-store-sqlcipher',
];

const STORAGE_FORBIDDEN_PROTOCOL_DEPENDENCIES = [
  'async-nats',
  'nats',
  ...STORAGE_CLIENT_DEPENDENCIES,
  'serde_json',
];

const STORAGE_FORBIDDEN_OWNER_DEPENDENCIES = ['async-nats', 'nats'];

function hasExactKeys(value, expectedKeys) {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return false;
  const keys = Object.keys(value);
  return keys.length === expectedKeys.length
    && keys.every((key) => expectedKeys.includes(key));
}

function isExactOrderedStringList(value, expected) {
  return Array.isArray(value)
    && value.length === expected.length
    && duplicates(value).length === 0
    && value.every((entry, index) => entry === expected[index]);
}

export async function loadPolicy(path) {
  return JSON.parse(await readFile(path, 'utf8'));
}

export function validatePolicy(policy) {
  const violations = [];
  if (policy?.schemaVersion !== 2) {
    violations.push(violation('policy_schema', 'architecture/policy.json', 'schemaVersion must be 2'));
  }

  violations.push(
    ...validateImplementationSlicePolicy(policy),
    ...validatePhaseGatePolicy(policy),
    ...validateAiContextPolicy(policy),
  );

  const registered = list(policy?.domains?.registered);
  const allowed = list(policy?.domains?.developmentAllowlist);
  const blocked = list(policy?.domains?.blocked);
  for (const [name, values] of [['registered', registered], ['developmentAllowlist', allowed], ['blocked', blocked]]) {
    const repeated = duplicates(values);
    if (repeated.length > 0) {
      violations.push(violation('duplicate_policy_value', `domains.${name}`, `duplicate values: ${repeated.join(', ')}`));
    }
  }

  const expected = new Set(registered);
  const partition = [...allowed, ...blocked];
  const partitionSet = new Set(partition);
  const overlaps = allowed.filter((owner) => blocked.includes(owner));
  const missing = registered.filter((owner) => !partitionSet.has(owner));
  const unknown = partition.filter((owner) => !expected.has(owner));
  if (overlaps.length > 0 || missing.length > 0 || unknown.length > 0 || partition.length !== registered.length) {
    violations.push(violation(
      'domain_partition',
      'domains',
      `allowed/blocked must partition registered domains; overlaps=${overlaps.join(',') || '-'} missing=${missing.join(',') || '-'} unknown=${unknown.join(',') || '-'}`,
    ));
  }

  const centralPersonOwners = registered.filter(
    (owner) => ['contacts', 'persons'].includes(owner),
  );
  if (centralPersonOwners.length !== 1
    || centralPersonOwners[0] !== 'persons'
    || !allowed.includes('persons')
    || allowed.includes('contacts')) {
    violations.push(violation(
      'central_person_owner',
      'domains',
      'persons must be the only registered and development-allowlisted central person owner',
    ));
  }

  const constitutionalCore = new Set(list(policy?.kernel?.constitutionalComponents));
  const exclusiveCore = new Set(list(policy?.kernel?.exclusiveComponents));
  if (!constitutionalCore.has('event_hub')
    || !constitutionalCore.has('telemetry_control')
    || !constitutionalCore.has('settings_registry')) {
    violations.push(violation(
      'kernel_components',
      'kernel.constitutionalComponents',
      'Event Hub, telemetry_control and settings_registry are constitutional Kernel components',
    ));
  }
  if (!exclusiveCore.has('settings_registry')) {
    violations.push(violation(
      'kernel_components',
      'kernel.exclusiveComponents',
      'settings_registry is exclusive to Kernel',
    ));
  }
  for (const component of list(policy?.kernel?.exclusiveComponents)) {
    if (!constitutionalCore.has(component)) {
      violations.push(violation('kernel_components', 'kernel.exclusiveComponents', `${component} must also be constitutional to Kernel`));
    }
  }

  const kernelPackages = list(policy?.kernel?.packages);
  const kernelPackageNames = kernelPackages.map((entry) => entry?.name);
  const invalidKernelPackage = kernelPackages.some((entry) => !entry
    || typeof entry.name !== 'string'
    || !entry.name.startsWith(policy?.cargo?.packagePrefix ?? '')
    || !list(policy?.cargo?.surfaces).includes(entry.surface));
  const runtimeKernelPackage = kernelPackages.find((entry) => entry?.name === policy?.kernel?.package);
  if (!kernelPackages.length
    || invalidKernelPackage
    || duplicates(kernelPackageNames).length > 0
    || runtimeKernelPackage?.surface !== 'runtime') {
    violations.push(violation(
      'kernel_package_policy',
      'kernel.packages',
      'Kernel-owned packages must be explicit, unique, valid surfaces and include the configured runtime package',
    ));
  }

  const bootstrap = policy?.kernel?.bootstrap;
  const dataDirectorySources = bootstrap?.dataDirectorySources;
  const requiredDataDirectorySources = ['os_standard_default', 'explicit_cli'];
  const externalServices = bootstrap?.requiredExternalServicesBeforeRecoveryOnly;
  if (bootstrap?.configurationFileRequired !== false
    || !Array.isArray(dataDirectorySources)
    || dataDirectorySources.length !== requiredDataDirectorySources.length
    || duplicates(dataDirectorySources).length > 0
    || requiredDataDirectorySources.some((source) => !dataDirectorySources.includes(source))
    || !Array.isArray(externalServices)
    || externalServices.length !== 0) {
    violations.push(violation(
      'kernel_bootstrap_policy',
      'kernel.bootstrap',
      'Kernel bootstrap requires no configuration file or external service and permits only OS-standard or explicit CLI data directories',
    ));
  }

  const controlStoreUnavailable = bootstrap?.controlStoreUnavailable;
  const onlineRecoveryOperations = controlStoreUnavailable?.onlineOperations;
  const requiredOnlineRecoveryOperations = ['status', 'validate', 'export'];
  const lifecycleRecoveryOperations = controlStoreUnavailable?.lifecycleOperations;
  const requiredLifecycleRecoveryOperations = ['shutdown'];
  const offlineRecoveryOperations = controlStoreUnavailable?.offlineOperations;
  const requiredOfflineRecoveryOperations = ['restore', 'reset'];
  if (controlStoreUnavailable?.state !== 'recovery_only'
    || controlStoreUnavailable?.transport !== 'local_ipc'
    || !Array.isArray(onlineRecoveryOperations)
    || onlineRecoveryOperations.length !== requiredOnlineRecoveryOperations.length
    || duplicates(onlineRecoveryOperations).length > 0
    || requiredOnlineRecoveryOperations.some(
      (operation) => !onlineRecoveryOperations.includes(operation),
    )
    || !Array.isArray(lifecycleRecoveryOperations)
    || lifecycleRecoveryOperations.length !== requiredLifecycleRecoveryOperations.length
    || duplicates(lifecycleRecoveryOperations).length > 0
    || requiredLifecycleRecoveryOperations.some(
      (operation) => !lifecycleRecoveryOperations.includes(operation),
    )
    || controlStoreUnavailable?.onlineMutationsEnabled !== false
    || !Array.isArray(offlineRecoveryOperations)
    || offlineRecoveryOperations.length !== requiredOfflineRecoveryOperations.length
    || duplicates(offlineRecoveryOperations).length > 0
    || requiredOfflineRecoveryOperations.some(
      (operation) => !offlineRecoveryOperations.includes(operation),
    )
    || controlStoreUnavailable?.kernelStoppedRequired !== true
    || controlStoreUnavailable?.exclusiveLockRequired !== true
    || controlStoreUnavailable?.explicitDataDirectoryRequired !== true
    || controlStoreUnavailable?.interactiveConfirmationRequired !== true
    || controlStoreUnavailable?.businessDataPlaneEnabled !== false
    || controlStoreUnavailable?.automaticResetEnabled !== false) {
    violations.push(violation(
      'kernel_recovery_policy',
      'kernel.bootstrap.controlStoreUnavailable',
      'an unavailable Control Store permits only bounded local recovery without data plane or automatic reset',
    ));
  }

  const identity = policy?.kernel?.identity;
  if (identity?.ownerModel !== 'logical_authority'
    || identity?.deviceKeySuite !== 'es256'
    || identity?.devicePrivateKeyBoundary !== 'platform_signer'
    || identity?.devicePublicKeyRegistry !== 'kernel_control_store'
    || identity?.initialDesktopEnrollment !== 'inherited_fd_pristine_instance'
    || identity?.clientSessionStorage !== 'memory_only'
    || identity?.osIdentityAloneAuthorizesOwner !== false
    || identity?.sharedOwnerSecretAllowed !== false) {
    violations.push(violation(
      'kernel_identity_policy',
      'kernel.identity',
      'owner authority requires per-device ES256 keys, platform signing, Control Store public records and inherited-FD first enrollment',
    ));
  }

  const distributionTrust = policy?.kernel?.distributionTrust;
  if (distributionTrust?.externalRegistration !== 'open_pending'
    || distributionTrust?.managedLaunchVerification !== 'exact_bytes_before_every_launch'
    || distributionTrust?.bundledManagedVerification !== 'signed_distribution_manifest_digest'
    || distributionTrust?.promotedExternalManagedVerification !== 'owner_pinned_executable_digest'
    || distributionTrust?.integrityFailureState !== 'blocked_integrity'
    || distributionTrust?.kernelExecutableDownloadsEnabled !== false
    || distributionTrust?.rollbackMode !== 'explicit_verified_target'
    || distributionTrust?.automaticFallbackEnabled !== false) {
    violations.push(violation(
      'kernel_distribution_trust_policy',
      'kernel.distributionTrust',
      'external registration stays open/pending, every managed launch verifies exact bytes, Kernel never downloads code and rollback is explicit',
    ));
  }

  const controlStore = policy?.controlStore;
  if (!hasExactKeys(controlStore, CONTROL_STORE_POLICY_KEYS)
    || controlStore?.owner !== 'kernel'
    || controlStore?.owner !== policy?.owners?.core
    || controlStore?.storageBoundary !== 'private_sqlite'
    || !isExactOrderedStringList(
      controlStore?.forbiddenData,
      CONTROL_STORE_FORBIDDEN_DATA,
    )) {
    violations.push(violation(
      'control_store_policy',
      'controlStore',
      'Kernel Control Store must remain private SQLite and exclude Vault keys, secret bindings, credential leases, provider sessions and private content',
    ));
  }

  const runtimeProtocol = policy?.runtimeProtocol;
  if (!hasExactKeys(runtimeProtocol, RUNTIME_PROTOCOL_POLICY_KEYS)
    || runtimeProtocol?.protocolPackage !== 'makosh-runtime-protocol'
    || runtimeProtocol?.role !== 'platform'
    || runtimeProtocol?.owner !== 'runtime_protocol'
    || !list(policy?.owners?.platform).includes(runtimeProtocol?.owner)
    || runtimeProtocol?.surface !== 'contract'
    || runtimeProtocol?.serialization !== 'protobuf_binary'
    || runtimeProtocol?.descriptorMajorVersion !== 1
    || runtimeProtocol?.descriptorDigestSource !== 'received_exact_bytes'
    || runtimeProtocol?.descriptorCarriesOwnDigest !== false
    || runtimeProtocol?.approvalUnit !== 'capability'
    || runtimeProtocol?.grantRule !== 'requested_approved_hard_policy_intersection'
    || runtimeProtocol?.dependencyBinding !== 'contract_capability_or_platform_capability'
    || runtimeProtocol?.managedArtifactBinding !== 'distribution_manifest_or_owner_pinned_artifact_digests'
    || runtimeProtocol?.unknownMajorState !== 'blocked_incompatible'
    || runtimeProtocol?.automaticFormatFallbackEnabled !== false
    || runtimeProtocol?.wildcardPermissionsAllowed !== false
    || runtimeProtocol?.arbitraryMapsAllowed !== false
    || runtimeProtocol?.hostSandboxEnforcement !== 'not_provided_by_grant_set'
    || !isExactOrderedStringList(
      runtimeProtocol?.forbiddenDescriptorData,
      RUNTIME_PROTOCOL_FORBIDDEN_DESCRIPTOR_DATA,
    )
    || !isExactOrderedStringList(
      runtimeProtocol?.forbiddenDependencies,
      RUNTIME_PROTOCOL_FORBIDDEN_DEPENDENCIES,
    )) {
    violations.push(violation(
      'runtime_protocol_policy',
      'runtimeProtocol',
      'module runtime protocol must declare the exact Protobuf v1, capability-level, fail-closed and externally bound descriptor invariants',
    ));
  }

  const settings = policy?.settings;
  if (!hasExactKeys(settings, SETTINGS_POLICY_KEYS)
    || settings?.registryOwner !== 'kernel'
    || settings?.registryOwner !== policy?.owners?.core
    || settings?.registryComponent !== 'settings_registry'
    || settings?.schemaProtocolPackage !== 'makosh-runtime-protocol'
    || settings?.schemaProtocolPackage !== runtimeProtocol?.protocolPackage
    || settings?.schemaSerialization !== 'protobuf_binary'
    || settings?.sourceOfTruth !== 'kernel_control_store'
    || settings?.postgresRequired !== false
    || !isExactOrderedStringList(settings?.authorities, SETTINGS_AUTHORITIES)
    || !isExactOrderedStringList(
      settings?.clientVisibilities,
      SETTINGS_CLIENT_VISIBILITIES,
    )
    || settings?.authorityVisibilityRule !== 'independent_axes_kernel_managed_never_editable'
    || !isExactOrderedStringList(settings?.targetScopes, SETTINGS_TARGET_SCOPES)
    || settings?.composition !== 'kernel_catalog_preserves_owner_sections'
    || settings?.crossOwnerAtomicMutationEnabled !== false
    || settings?.schemaBinding !== 'descriptor_ref_schema_major_revision_size_sha256'
    || settings?.valueModel !== 'closed_typed_union'
    || settings?.arbitraryJsonMapsOrAnyAllowed !== false
    || settings?.secretValuesAllowed !== false
    || settings?.credentialDelivery !== 'vault_lease_outside_settings'
    || settings?.operatorManagedWriter !== 'authenticated_owner_device'
    || settings?.kernelManagedWriter !== 'allowlisted_kernel_controller'
    || settings?.grantMutationAllowed !== false
    || settings?.revisionModel !== 'schema_desired_effective_runtime_generation'
    || settings?.mutationConcurrency !== 'expected_desired_revision'
    || !isExactOrderedStringList(settings?.applyModes, SETTINGS_APPLY_MODES)
    || settings?.managedRestart !== 'supervised_quiesce_drain_optional_checkpoint_stop_start_readiness'
    || settings?.externalRestart !== 'awaiting_external_restart'
    || settings?.applyFailureState !== 'blocked_config'
    || settings?.automaticRollbackEnabled !== false
    || !isExactOrderedStringList(settings?.forbiddenState, SETTINGS_FORBIDDEN_STATE)) {
    violations.push(violation(
      'settings_policy',
      'settings',
      'Kernel settings policy must preserve owner sections, typed authority/scopes, Control Store revisions and explicit supervised application',
    ));
  }

  const vault = policy?.vault;
  if (!hasExactKeys(vault, VAULT_POLICY_KEYS)
    || vault?.role !== 'platform'
    || vault?.owner !== 'vault'
    || !list(policy?.owners?.platform).includes(vault?.owner)
    || vault?.protocolPackage !== 'makosh-vault-protocol'
    || vault?.managedClientPackage !== 'makosh-managed-vault-client'
    || vault?.keyProviderPackage !== 'makosh-vault-key-provider'
    || vault?.runtimePackage !== 'makosh-vault-runtime'
    || vault?.storePackage !== 'makosh-vault-store-sqlcipher'
    || !isExactOrderedStringList(
      vault?.platformKeyAdapterPackages,
      VAULT_PLATFORM_KEY_ADAPTER_PACKAGES,
    )
    || vault?.runtimeComponent !== 'vault_service'
    || vault?.processBoundary !== 'separate_managed_process'
    || vault?.managementMode !== 'bundled_managed_only'
    || vault?.externalRegistrationEnabled !== false
    || vault?.alternativeTopologyEnabled !== false
    || vault?.storageFormat !== 'sqlcipher_full_database_plus_xchacha20poly1305_records'
    || vault?.platformKeyStorage !== 'owner_private_file_adapter'
    || vault?.recoveryModel !== 'independent_wrapped_root_key_offline_restore'
    || vault?.recoveryKeyEncoding !== 'bip39_24_word_entropy_only'
    || !isExactOrderedStringList(vault?.unlockModes, VAULT_UNLOCK_MODES)
    || vault?.defaultUnlockMode !== 'file_adapter_auto'
    || vault?.automaticInitializeOrRestoreEnabled !== false
    || vault?.transport !== 'hpke_ciphertext_over_capability_router'
    || vault?.transportScope !== 'authenticated_session_bound_local_ipc'
    || vault?.kernelPayloadVisibility !== 'ciphertext_only'
    || vault?.leasePersistence !== 'memory_only'
    || vault?.defaultLeaseTtlSeconds !== 600
    || vault?.maximumLeaseTtlSeconds !== 3600
    || vault?.leaseResolveLimit !== 1
    || vault?.leaseRenewal !== 'new_lease_full_authorization_check'
    || vault?.credentialPayloadMaxBytes !== 65_536
    || vault?.sessionCredentialBlobMaxBytes !== 4_194_304
    || !isExactOrderedStringList(vault?.leaseBindingFields, VAULT_LEASE_BINDING_FIELDS)
    || vault?.runtimeGenerationChangeInvalidatesLeases !== true
    || vault?.grantEpochChangeInvalidatesLeases !== true
    || !isExactOrderedStringList(
      vault?.forbiddenSecretCarriers,
      VAULT_FORBIDDEN_SECRET_CARRIERS,
    )
    || !isExactOrderedStringList(
      vault?.forbiddenProtocolDependencies,
      VAULT_FORBIDDEN_PROTOCOL_DEPENDENCIES,
    )
    || !isExactOrderedStringList(
      vault?.forbiddenOwnerDependencies,
      VAULT_FORBIDDEN_OWNER_DEPENDENCIES,
    )) {
    violations.push(violation(
      'vault_policy',
      'vault',
      'Vault must remain a separate bundled managed process with exact packages, HPKE-routed session binding, memory-only leases and forbidden secret carriers',
    ));
  }

  const events = policy?.events;
  if (!hasExactKeys(events, EVENT_POLICY_KEYS)
    || events?.protocolPackage !== 'makosh-events-protocol'
    || events?.role !== 'platform'
    || events?.owner !== 'events'
    || events?.surface !== 'contract'
    || events?.serialization !== 'protobuf_binary'
    || events?.envelopeMajorVersion !== 1
    || !isExactOrderedStringList(events?.kinds, EVENT_KINDS)
    || events?.kindMetadata !== 'oneof'
    || events?.payloadBinding !== 'catalog_contract_version_schema_sha256'
    || events?.payloadVisibility !== 'opaque_to_kernel_and_event_hub'
    || events?.outboxPublishMode !== 'canonical_bytes_byte_for_byte'
    || events?.clientEnvelopeReuseAllowed !== false
    || events?.brokerAckIsEnvelopeAck !== false
    || events?.unknownMajorVersion !== 'reject'
    || events?.automaticFormatFallbackEnabled !== false
    || !isExactOrderedStringList(
      events?.forbiddenPayloadData,
      EVENT_FORBIDDEN_PAYLOAD_DATA,
    )
    || !isExactOrderedStringList(
      events?.forbiddenDependencies,
      EVENT_FORBIDDEN_DEPENDENCIES,
    )) {
    violations.push(violation(
      'events_protocol_policy',
      'events',
      'canonical durable envelopes require the exact protobuf v1 contract, five message kinds, opaque catalog-bound payloads and fail-closed versioning',
    ));
  }

  if (!list(policy?.cargo?.roles).length || !list(policy?.cargo?.surfaces).length) {
    violations.push(violation('cargo_policy', 'cargo', 'roles and surfaces must be non-empty'));
  }
  if (!list(policy?.projections?.blockedOwners).length) {
    violations.push(violation('projection_policy', 'projections', 'blocked projection owners must be explicit'));
  }
  if (!list(policy?.telemetry?.forbiddenDependencies).length) {
    violations.push(violation('telemetry_policy', 'telemetry', 'Telemetry Collector forbidden dependencies must be explicit'));
  }
  const packagePrefix = policy?.cargo?.packagePrefix;
  const engineEngineContracts = list(policy?.dependencies?.engineEngineContractPackages);
  const engineDomainContracts = list(policy?.dependencies?.engineDomainContractPackages);
  const engineWorkflowContracts = list(policy?.dependencies?.engineWorkflowContractPackages);
  const integrationEngineContracts = list(
    policy?.dependencies?.integrationEngineContractPackages,
  );
  const integrationDomainContracts = list(policy?.dependencies?.integrationDomainContractPackages);
  const integrationWorkflowContracts = list(
    policy?.dependencies?.integrationWorkflowContractPackages,
  );
  if (!engineEngineContracts.length
    || engineEngineContracts.some((packageName) => typeof packagePrefix !== 'string'
      || !packageName.startsWith(packagePrefix))
    || !engineDomainContracts.length
    || engineDomainContracts.some((packageName) => typeof packagePrefix !== 'string'
      || !packageName.startsWith(packagePrefix))
    || !engineWorkflowContracts.length
    || engineWorkflowContracts.some((packageName) => typeof packagePrefix !== 'string'
      || !packageName.startsWith(packagePrefix))
    || !integrationEngineContracts.length
    || integrationEngineContracts.some((packageName) => typeof packagePrefix !== 'string'
      || !packageName.startsWith(packagePrefix))
    || !integrationDomainContracts.length
    || integrationDomainContracts.some((packageName) => typeof packagePrefix !== 'string'
      || !packageName.startsWith(packagePrefix))
    || !integrationWorkflowContracts.length
    || integrationWorkflowContracts.some((packageName) => typeof packagePrefix !== 'string'
      || !packageName.startsWith(packagePrefix))) {
    violations.push(violation(
      'dependency_policy',
      'dependencies',
      'engine-engine, engine-domain, engine-workflow, integration-engine, integration-domain and integration-workflow contracts must use explicit package allowlists',
    ));
  }
  const forbiddenAggregatePackages = list(policy?.compileIsolation?.forbiddenAggregatePackages);
  const moduleRoles = list(policy?.compileIsolation?.moduleRoles);
  if (!forbiddenAggregatePackages.length
    || duplicates(forbiddenAggregatePackages).length > 0
    || forbiddenAggregatePackages.some((packageName) => typeof packagePrefix !== 'string'
      || !packageName.startsWith(packagePrefix))
    || !moduleRoles.length
    || moduleRoles.some((role) => !list(policy?.cargo?.roles).includes(role))
    || policy?.compileIsolation?.forbidSameOwnerRuntimeDependencies !== true
    || policy?.compileIsolation?.forbidCrossOwnerPersistenceDependencies !== true) {
    violations.push(violation(
      'compile_isolation_policy',
      'compileIsolation',
      'compile-isolation package, role, runtime and persistence rules must be explicit',
    ));
  }
  const hostOnlyProviderExecutionOwners = list(
    policy?.integrations?.hostOnlyProviderExecutionOwners,
  );
  if (!hostOnlyProviderExecutionOwners.length
    || duplicates(hostOnlyProviderExecutionOwners).length > 0
    || hostOnlyProviderExecutionOwners.some((owner) => typeof owner !== 'string' || owner.length === 0)) {
    violations.push(violation(
      'integration_policy',
      'integrations.hostOnlyProviderExecutionOwners',
      'host-only provider execution owners must be explicit and unique',
    ));
  }
  const storage = policy?.storage;
  if (!hasExactKeys(storage, STORAGE_POLICY_KEYS)
    || storage?.role !== 'platform'
    || storage?.owner !== 'storage'
    || storage?.protocolPackage !== 'makosh-storage-protocol'
    || storage?.controlPackage !== 'makosh-storage-control'
    || storage?.vaultPackage !== 'makosh-storage-vault'
    || !isExactOrderedStringList(
      storage?.sharedVaultRouteConsumers,
      ['makosh-scheduler-runtime'],
    )
    || storage?.runtimePackage !== 'makosh-storage-runtime'
    || storage?.postgresPackage !== 'makosh-storage-postgres'
    || storage?.pgbouncerPackage !== 'makosh-storage-pgbouncer'
    || storage?.migrationsPackage !== 'makosh-storage-migrations'
    || storage?.runtimeComponent !== 'storage_control'
    || storage?.managementMode !== 'bundled_managed_only'
    || storage?.protocolSerialization !== 'protobuf_binary'
    || storage?.clusterTopology !== 'one_managed_postgresql_cluster'
    || storage?.databaseTopology !== 'one_application_database'
    || !isExactOrderedStringList(storage?.fixedSchemas, STORAGE_FIXED_SCHEMAS)
    || storage?.runtimePath !== 'pgbouncer_only'
    || storage?.poolMode !== 'transaction'
    || storage?.minimumPgbouncerVersion !== '1.25.2'
    || storage?.runtimePrincipalScope !== 'registration_runtime_generation'
    || storage?.ownerDdlRole !== 'nologin'
    || storage?.runtimeLoginRole !== 'login_noinherit'
    || storage?.runtimePoolAliasScope !== 'runtime_generation'
    || !isExactOrderedStringList(
      storage?.directConnectionAudiences,
      STORAGE_DIRECT_CONNECTION_AUDIENCES,
    )
    || storage?.credentialDelivery !== 'vault_scoped_lease'
    || !isExactOrderedStringList(storage?.bindingFields, STORAGE_BINDING_FIELDS)
    || !isExactOrderedStringList(storage?.revocationSequence, STORAGE_REVOCATION_SEQUENCE)
    || storage?.migrationArtifact !== 'protobuf_storage_bundle_v1'
    || storage?.migrationTrustBinding !== 'distribution_manifest_or_owner_pinned_sha256'
    || storage?.migrationExecution !== 'coordinator_only_transactional_steps'
    || storage?.migrationDirection !== 'forward_only'
    || storage?.migrationValidation !== 'digest_postgresql_ast_owner_role_privilege_audit'
    || storage?.sharedTechnicalAccess !== 'versioned_functions_only'
    || !isExactOrderedStringList(
      storage?.sharedTechnicalFunctions,
      STORAGE_SHARED_TECHNICAL_FUNCTIONS,
    )
    || storage?.moduleSelfMigrationsEnabled !== false
    || storage?.destructiveMigrationsEnabled !== false
    || storage?.kernelSqlProxyEnabled !== false
    || storage?.regexOnlyValidationAllowed !== false
    || storage?.crossOwnerBusinessSqlEnabled !== false
    || storage?.directSharedTechnicalDmlEnabled !== false
    || storage?.postgresRoleHardLimitsRequired !== true
    || storage?.pgbouncerSoleBudgetBoundary !== false
    || !isExactOrderedStringList(storage?.clientDependencies, STORAGE_CLIENT_DEPENDENCIES)
    || !isExactOrderedStringList(
      storage?.postgresClientDependencies,
      STORAGE_POSTGRES_CLIENT_DEPENDENCIES,
    )
    || !isExactOrderedStringList(
      storage?.allowedPostgresClientDependencies,
      ['sqlx', 'tokio-postgres'],
    )
    || !isExactOrderedStringList(
      storage?.testSupportPostgresClientAllowlist,
      [
        'makosh-communication-delayed-delivery-testkit:dev:sqlx',
        'makosh-events-jetstream-testkit:dev:sqlx',
        'makosh-kernel-recovery-testkit:dev:sqlx',
        'makosh-scheduler-testkit:dev:sqlx',
      ],
    )
    || !isExactOrderedStringList(storage?.sqliteClientDependencies, ['rusqlite'])
    || !isExactOrderedStringList(storage?.sqlitePackages, STORAGE_SQLITE_PACKAGES)
    || !isExactOrderedStringList(storage?.astParserDependencies, ['pg_query'])
    || !isExactOrderedStringList(
      storage?.forbiddenProtocolDependencies,
      STORAGE_FORBIDDEN_PROTOCOL_DEPENDENCIES,
    )
    || !isExactOrderedStringList(
      storage?.forbiddenOwnerDependencies,
      STORAGE_FORBIDDEN_OWNER_DEPENDENCIES,
    )) {
    violations.push(violation(
      'storage_policy',
      'storage',
      'Storage Control topology, owner isolation, PgBouncer path, fencing, bundle admission and dependency policy must be exact and fail closed',
    ));
  }
  if (!list(policy?.source?.ownerPathMarkers).length) {
    violations.push(violation('source_policy', 'source', 'ownerPathMarkers must be explicit'));
  }
  if (!list(policy?.source?.roots).length
    || !list(policy?.source?.contentExtensions).length
    || !isExactOrderedStringList(policy?.source?.srpRoots, ['src', 'development', 'tests'])
    || !isExactOrderedStringList(policy?.source?.srpContentExtensions, ['.rs', '.mjs'])
    || !isExactOrderedStringList(policy?.source?.generatedPathSegments, ['gen', 'generated'])
    || policy?.source?.forbidSymlinks !== true) {
    violations.push(violation(
      'source_policy',
      'source',
      'production roots, readable content extensions and symlink prohibition must be explicit',
    ));
  }

  const testRoots = list(policy?.tests?.workspaceRoots);
  const forbiddenTestDirectories = list(policy?.tests?.forbiddenProductionDirectories);
  const forbiddenTestFiles = list(policy?.tests?.forbiddenProductionFilePatterns);
  if (!testRoots.length
    || !forbiddenTestDirectories.length
    || !forbiddenTestFiles.length
    || policy?.tests?.forbidInlineRustTests !== true) {
    violations.push(violation(
      'test_layout_policy',
      'tests',
      'test-only roots and production test-code prohibitions must be explicit',
    ));
  }

  if (!list(policy?.layout?.requiredBackendPaths).length
    || !list(policy?.layout?.forbiddenProjectPaths).length
    || !list(policy?.layout?.forbiddenBackendPaths).length) {
    violations.push(violation(
      'layout_policy',
      'layout',
      'required backend paths and forbidden legacy paths must be explicit',
    ));
  }

  return violations;
}
