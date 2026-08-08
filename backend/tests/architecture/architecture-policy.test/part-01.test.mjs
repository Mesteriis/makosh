import assert from 'node:assert/strict';
import { mkdir, mkdtemp, rm, symlink, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';

import { validatePolicy } from '../../../scripts/lib/policy-schema.mjs';
import { collectSourceEntries } from '../../../scripts/lib/repository-scan.mjs';
import { validateSourceEntries } from '../../../scripts/lib/source-boundaries.mjs';
import { canonicalPolicyForTests as policy } from '../support/canonical-policy.mjs';

import { codes } from './support.mjs';

test('accepts the canonical registry and current development allowlist', () => {
  assert.deepEqual(validatePolicy(policy()), []);
});

test('requires architecture policy schema version 2', () => {
  const invalid = policy();
  invalid.schemaVersion = 1;

  assert.ok(codes(validatePolicy(invalid)).has('policy_schema'));
});



test('requires an exact current implementation slice inventory', () => {
  const invalid = policy();
  delete invalid.implementation;

  assert.ok(codes(validatePolicy(invalid)).has('implementation_slice_policy'));
});



test('keeps the current implementation slice closed to undeclared production packages', () => {
  const mutations = [
    (implementation) => { implementation.currentSlice = 'unsupported_phase_v1'; },
    (implementation) => { implementation.productionPackages.push({
      name: 'makosh-storage-protocol', role: 'platform', owner: 'storage', surface: 'contract',
    }); },
    (implementation) => { implementation.ownerInventory.domains.push('communications'); },
    (implementation) => { implementation.ownerInventory.businessCapabilities.push('client_rpc'); },
    (implementation) => { implementation.workspaceDependencyAllowlist['makosh-kernel'].push('makosh-events-protocol'); },
    (implementation) => { implementation.thirdPartyDependencyAllowlist['makosh-kernel'].push({ name: 'reqwest', kind: 'normal', source: 'crates_io' }); },
    (implementation) => { implementation.forbiddenDependencies.pop(); },
    (implementation) => { implementation.forbiddenDependencyPrefixes.push('makosh-ai-'); },
    (implementation) => { implementation.cargoFeaturesEnabled = true; },
    (implementation) => { implementation.cargoFeatureAllowlist['makosh-mail-api'].default.push('runtime-capability'); },
    (implementation) => { delete implementation.cargoFeatureAllowlist['makosh-mail-runtime']; },
    (implementation) => { implementation.developmentProfile.networkListenerEnabled = false; },
    (implementation) => { implementation.developmentProfile.productionGateEvidenceAllowed = true; },
    (implementation) => { implementation.developmentProfile.privateKeyStorage = 'unprotected_file'; },
    (implementation) => { implementation.developmentProfile.automaticProductionFallbackAllowed = true; },
    (implementation) => { implementation.developmentProfile.simulatedTargets.pop(); },
    (implementation) => { implementation.targetPolicy['makosh-kernel'].primaryKind = 'lib'; },
    (implementation) => { implementation.kernelProfile.maximumState = 'ready'; },
    (implementation) => { implementation.kernelProfile.allowedStates.push('ready'); },
    (implementation) => { implementation.kernelProfile.forbiddenStates.pop(); },
    (implementation) => { implementation.kernelProfile.activeComponents.push('event_hub'); },
    (implementation) => { implementation.kernelProfile.externalServices.push('postgresql'); },
    (implementation) => { implementation.kernelProfile.networkListenerEnabled = false; },
    (implementation) => { implementation.kernelProfile.managedLaunchEnabled = false; },
    (implementation) => { implementation.kernelProfile.managedChildren.pop(); },
    (implementation) => { implementation.kernelProfile.clock.moduleCapabilityEnabled = true; },
    (implementation) => { implementation.exitGates.pop(); },
    (implementation) => { implementation.compatibilityMode = true; },
  ];

  for (const mutate of mutations) {
    const invalid = policy();
    mutate(invalid.implementation);
    assert.ok(codes(validatePolicy(invalid)).has('implementation_slice_policy'));
  }
});



test('requires fail-closed phase gates before expanding the recovery-only slice', () => {
  const invalid = policy();
  delete invalid.phaseGates;

  assert.ok(codes(validatePolicy(invalid)).has('phase_gate_policy'));
});



test('requires exact evidence fields before authorizing general phases and reviewed owner exceptions', () => {
  const mutations = [
    (phaseGates) => { phaseGates.transitionAuthority = 'runtime_flag'; },
    (phaseGates) => { phaseGates.notAuthorized.push('provider_v1'); },
    (phaseGates) => { phaseGates.requiredDecisionFields.nats_data_plane_v1.pop(); },
    (phaseGates) => { delete phaseGates.requiredDecisionFields.blob_v1; },
    (phaseGates) => { phaseGates.requires.storage_control_v1.pop(); },
    (phaseGates) => { phaseGates.requires.nats_data_plane_v1.pop(); },
    (phaseGates) => { phaseGates.requires.scheduler_v1.pop(); },
    (phaseGates) => { phaseGates.requiredDecisionFields.browser_client_v1.pop(); },
    (phaseGates) => { phaseGates.requires.browser_client_v1.pop(); },
    (phaseGates) => { phaseGates.requires.client_gateway_v1.pop(); },
    (phaseGates) => { delete phaseGates.conditionalRequires.whole_instance_backup_v1.scheduler_v1; },
    (phaseGates) => { phaseGates.ownerAdmissionExceptions.push({ id: 'invalid-exception' }); },
    (phaseGates) => { delete phaseGates.ownerAdmissionExceptions; },
    (phaseGates) => { phaseGates.overrideAllowed = true; },
  ];

  for (const mutate of mutations) {
    const invalid = policy();
    mutate(invalid.phaseGates);
    assert.ok(codes(validatePolicy(invalid)).has('phase_gate_policy'));
  }
});



test('requires AI context to be supplied by explicit use-case workflows', () => {
  const invalid = policy();
  delete invalid.aiContext;

  assert.ok(codes(validatePolicy(invalid)).has('ai_context_policy'));
});



test('prevents AI read-all access, generic context APIs and durable context projections', () => {
  const mutations = [
    (invalid) => { invalid.aiContext.acquisitionMode = 'direct_database_read'; },
    (invalid) => { invalid.aiContext.aiDirectOwnerQueryAccessEnabled = true; },
    (invalid) => { invalid.aiContext.aiDirectCrossOwnerQueryOrchestrationEnabled = true; },
    (invalid) => { invalid.aiContext.aiCrossOwnerSqlEnabled = true; },
    (invalid) => { invalid.aiContext.genericContextApiEnabled = true; },
    (invalid) => { invalid.aiContext.durableContextProjectionEnabled = true; },
    (invalid) => { invalid.aiContext.wireShape = 'global_fragment_union'; },
    (invalid) => { invalid.aiContext.globalFragmentUnionEnabled = true; },
    (invalid) => { invalid.aiContext.opaquePayloadBytesEnabled = true; },
    (invalid) => { delete invalid.aiContext.schemaBinding; },
    (invalid) => { invalid.aiContext.remoteEgressRequiresExplicitPolicy = false; },
    (invalid) => { invalid.aiContext.firstConcreteUseCase = 'generic_ai'; },
    (invalid) => { invalid.aiContext.communicationsPrivateContentHandoff = 'client_ticket'; },
    (invalid) => { invalid.aiContext.clientContentTicketReuseForWorkflowEnabled = true; },
    (invalid) => { invalid.aiContext.inferenceOwnerRole = 'domain'; },
    (invalid) => { invalid.aiContext.firstProviderIntegration = 'embedded'; },
    (invalid) => { invalid.aiContext.firstProviderEgressPolicy = 'remote_allowed'; },
    (invalid) => { invalid.aiContext.callerSelectedProviderOrModelEnabled = true; },
    (invalid) => { invalid.aiContext.providerImplementationInsideInferenceOwnerEnabled = true; },
    (invalid) => { invalid.projections.blockedOwners = invalid.projections.blockedOwners.filter((owner) => owner !== 'context'); },
    (invalid) => { invalid.aiContext.readOnlyGrantsAllowed = true; },
  ];

  for (const mutate of mutations) {
    const invalid = policy();
    mutate(invalid);
    assert.ok(codes(validatePolicy(invalid)).has('ai_context_policy'));
  }
});



test('requires registered domains to be partitioned between allowed and blocked', () => {
  const invalid = policy();
  invalid.domains.developmentAllowlist = invalid.domains.developmentAllowlist.filter(
    (owner) => owner !== 'ai',
  );

  assert.ok(codes(validatePolicy(invalid)).has('domain_partition'));
});



test('requires Event Hub, Telemetry control and Settings Registry to remain constitutional Kernel components', () => {
  for (const requiredComponent of ['event_hub', 'telemetry_control', 'settings_registry']) {
    const invalid = policy();
    invalid.kernel.constitutionalComponents = invalid.kernel.constitutionalComponents.filter(
      (component) => component !== requiredComponent,
    );

    assert.ok(codes(validatePolicy(invalid)).has('kernel_components'));
  }
});



test('requires Settings Registry to remain exclusive to Kernel', () => {
  const invalid = policy();
  invalid.kernel.exclusiveComponents = invalid.kernel.exclusiveComponents.filter(
    (component) => component !== 'settings_registry',
  );

  assert.ok(codes(validatePolicy(invalid)).has('kernel_components'));
});



test('requires an exact registry of Kernel-owned packages', () => {
  const invalid = policy();
  delete invalid.kernel.packages;

  assert.ok(codes(validatePolicy(invalid)).has('kernel_package_policy'));
});



test('requires zero-config bootstrap with only OS-default and explicit CLI data-dir sources', () => {
  const mutations = [
    (bootstrap) => { bootstrap.configurationFileRequired = true; },
    (bootstrap) => { bootstrap.dataDirectorySources = ['os_standard_default']; },
    (bootstrap) => { bootstrap.dataDirectorySources.push('environment'); },
    (bootstrap) => { bootstrap.requiredExternalServicesBeforeRecoveryOnly.push('postgresql'); },
    (bootstrap) => { delete bootstrap.requiredExternalServicesBeforeRecoveryOnly; },
  ];

  for (const mutate of mutations) {
    const invalid = policy();
    mutate(invalid.kernel.bootstrap);

    assert.ok(codes(validatePolicy(invalid)).has('kernel_bootstrap_policy'));
  }
});



test('keeps Control Store failure in bounded local recovery', () => {
  const mutations = [
    (bootstrap) => { bootstrap.controlStoreUnavailable.state = 'ready'; },
    (bootstrap) => { bootstrap.controlStoreUnavailable.transport = 'http'; },
    (bootstrap) => { bootstrap.controlStoreUnavailable.onlineOperations = ['validate', 'export']; },
    (bootstrap) => { bootstrap.controlStoreUnavailable.onlineOperations.push('restore'); },
    (bootstrap) => { bootstrap.controlStoreUnavailable.lifecycleOperations = []; },
    (bootstrap) => { bootstrap.controlStoreUnavailable.lifecycleOperations.push('restart'); },
    (bootstrap) => { bootstrap.controlStoreUnavailable.onlineMutationsEnabled = true; },
    (bootstrap) => { bootstrap.controlStoreUnavailable.offlineOperations = ['restore']; },
    (bootstrap) => { bootstrap.controlStoreUnavailable.offlineOperations.push('wipe'); },
    (bootstrap) => { bootstrap.controlStoreUnavailable.kernelStoppedRequired = false; },
    (bootstrap) => { bootstrap.controlStoreUnavailable.exclusiveLockRequired = false; },
    (bootstrap) => { bootstrap.controlStoreUnavailable.explicitDataDirectoryRequired = false; },
    (bootstrap) => { bootstrap.controlStoreUnavailable.interactiveConfirmationRequired = false; },
    (bootstrap) => { bootstrap.controlStoreUnavailable.businessDataPlaneEnabled = true; },
    (bootstrap) => { bootstrap.controlStoreUnavailable.automaticResetEnabled = true; },
    (bootstrap) => { delete bootstrap.controlStoreUnavailable; },
  ];

  for (const mutate of mutations) {
    const invalid = policy();
    mutate(invalid.kernel.bootstrap);

    assert.ok(codes(validatePolicy(invalid)).has('kernel_recovery_policy'));
  }
});



test('requires per-device owner identity without shared secrets or OS-identity trust', () => {
  const mutations = [
    (identity) => { identity.ownerModel = 'shared_root_key'; },
    (identity) => { identity.deviceKeySuite = 'client_selected'; },
    (identity) => { identity.devicePrivateKeyBoundary = 'kernel'; },
    (identity) => { identity.devicePublicKeyRegistry = 'platform_signer'; },
    (identity) => { identity.initialDesktopEnrollment = 'public_endpoint'; },
    (identity) => { identity.clientSessionStorage = 'control_store'; },
    (identity) => { identity.osIdentityAloneAuthorizesOwner = true; },
    (identity) => { identity.sharedOwnerSecretAllowed = true; },
  ];

  for (const mutate of mutations) {
    const invalid = policy();
    mutate(invalid.kernel.identity);

    assert.ok(codes(validatePolicy(invalid)).has('kernel_identity_policy'));
  }

  const missing = policy();
  delete missing.kernel.identity;
  assert.ok(codes(validatePolicy(missing)).has('kernel_identity_policy'));
});



test('keeps registration open but fail-closes managed launch integrity', () => {
  const mutations = [
    (trust) => { trust.externalRegistration = 'signed_allowlist'; },
    (trust) => { trust.managedLaunchVerification = 'registration_time_only'; },
    (trust) => { trust.bundledManagedVerification = 'unsigned_manifest'; },
    (trust) => { trust.promotedExternalManagedVerification = 'self_reported_digest'; },
    (trust) => { trust.integrityFailureState = 'degraded'; },
    (trust) => { trust.kernelExecutableDownloadsEnabled = true; },
    (trust) => { trust.rollbackMode = 'automatic_previous_version'; },
    (trust) => { trust.automaticFallbackEnabled = true; },
  ];

  for (const mutate of mutations) {
    const invalid = policy();
    mutate(invalid.kernel.distributionTrust);

    assert.ok(codes(validatePolicy(invalid)).has('kernel_distribution_trust_policy'));
  }

  const missing = policy();
  delete missing.kernel.distributionTrust;
  assert.ok(codes(validatePolicy(missing)).has('kernel_distribution_trust_policy'));
});



test('requires the exact module runtime protocol policy', () => {
  const mutations = [
    (runtimeProtocol) => { runtimeProtocol.protocolPackage = 'makosh-runtime-contracts'; },
    (runtimeProtocol) => { runtimeProtocol.role = 'core'; },
    (runtimeProtocol) => { runtimeProtocol.owner = 'kernel'; },
    (runtimeProtocol) => { runtimeProtocol.surface = 'implementation'; },
    (runtimeProtocol) => { runtimeProtocol.serialization = 'json'; },
    (runtimeProtocol) => { runtimeProtocol.descriptorMajorVersion = 2; },
    (runtimeProtocol) => { runtimeProtocol.descriptorDigestSource = 'decoded_reserialization'; },
    (runtimeProtocol) => { runtimeProtocol.descriptorCarriesOwnDigest = true; },
    (runtimeProtocol) => { runtimeProtocol.approvalUnit = 'module'; },
    (runtimeProtocol) => { runtimeProtocol.grantRule = 'requested_or_approved'; },
    (runtimeProtocol) => { runtimeProtocol.dependencyBinding = 'module_id'; },
    (runtimeProtocol) => { runtimeProtocol.managedArtifactBinding = 'self_reported_digest'; },
    (runtimeProtocol) => { runtimeProtocol.unknownMajorState = 'degraded'; },
    (runtimeProtocol) => { runtimeProtocol.automaticFormatFallbackEnabled = true; },
    (runtimeProtocol) => { runtimeProtocol.wildcardPermissionsAllowed = true; },
    (runtimeProtocol) => { runtimeProtocol.arbitraryMapsAllowed = true; },
    (runtimeProtocol) => { runtimeProtocol.hostSandboxEnforcement = 'provided_by_grant_set'; },
    (runtimeProtocol) => { runtimeProtocol.forbiddenDescriptorData.pop(); },
    (runtimeProtocol) => { runtimeProtocol.forbiddenDependencies.pop(); },
    (runtimeProtocol) => { runtimeProtocol.unversionedCompatibilityAlias = true; },
  ];

  for (const mutate of mutations) {
    const invalid = policy();
    mutate(invalid.runtimeProtocol);

    assert.ok(codes(validatePolicy(invalid)).has('runtime_protocol_policy'));
  }

  const missing = policy();
  delete missing.runtimeProtocol;
  assert.ok(codes(validatePolicy(missing)).has('runtime_protocol_policy'));
});



test('requires the exact Kernel Settings Registry policy', () => {
  assert.ok(
    policy().settings.forbiddenState.includes(
      'secret_references_and_credential_bindings',
    ),
  );

  const mutations = [
    (settings) => { settings.registryOwner = 'module'; },
    (settings) => { settings.registryComponent = 'module_settings'; },
    (settings) => { settings.schemaProtocolPackage = 'makosh-settings-protocol'; },
    (settings) => { settings.schemaSerialization = 'json'; },
    (settings) => { settings.sourceOfTruth = 'postgresql'; },
    (settings) => { settings.postgresRequired = true; },
    (settings) => { settings.authorities = ['operator_managed']; },
    (settings) => { settings.clientVisibilities = ['editable', 'hidden']; },
    (settings) => { settings.authorityVisibilityRule = 'authority_implies_visibility'; },
    (settings) => { settings.targetScopes.push('runtime_instance'); },
    (settings) => { settings.composition = 'domain_merges_integrations'; },
    (settings) => { settings.crossOwnerAtomicMutationEnabled = true; },
    (settings) => { settings.schemaBinding = 'module_id_only'; },
    (settings) => { settings.valueModel = 'json_document'; },
    (settings) => { settings.arbitraryJsonMapsOrAnyAllowed = true; },
    (settings) => { settings.secretValuesAllowed = true; },
    (settings) => { settings.credentialDelivery = 'inline_setting_value'; },
    (settings) => { settings.operatorManagedWriter = 'module'; },
    (settings) => { settings.kernelManagedWriter = 'any_module'; },
    (settings) => { settings.grantMutationAllowed = true; },
    (settings) => { settings.revisionModel = 'desired_only'; },
    (settings) => { settings.mutationConcurrency = 'last_write_wins'; },
    (settings) => { settings.applyModes.pop(); },
    (settings) => { settings.managedRestart = 'kill_and_start'; },
    (settings) => { settings.externalRestart = 'kernel_process_restart'; },
    (settings) => { settings.applyFailureState = 'degraded'; },
    (settings) => { settings.automaticRollbackEnabled = true; },
    (settings) => { settings.forbiddenState.pop(); },
    (settings) => { settings.compatibilityDocument = true; },
  ];

  for (const mutate of mutations) {
    const invalid = policy();
    mutate(invalid.settings);

    assert.ok(codes(validatePolicy(invalid)).has('settings_policy'));
  }

  const missing = policy();
  delete missing.settings;
  assert.ok(codes(validatePolicy(missing)).has('settings_policy'));
});



test('requires an exact Control Store secret-exclusion policy', () => {
  const mutations = [
    (controlStore) => { controlStore.owner = 'vault'; },
    (controlStore) => { controlStore.storageBoundary = 'postgresql'; },
    (controlStore) => { controlStore.forbiddenData.pop(); },
    (controlStore) => { controlStore.forbiddenData.push('runtime_health'); },
    (controlStore) => { controlStore.secretCompatibilityAlias = true; },
  ];

  for (const mutate of mutations) {
    const invalid = policy();
    mutate(invalid.controlStore);

    assert.ok(codes(validatePolicy(invalid)).has('control_store_policy'));
  }

  const missing = policy();
  delete missing.controlStore;
  assert.ok(codes(validatePolicy(missing)).has('control_store_policy'));
});



test('requires an exact Vault process, lease and secret-carrier policy', () => {
  const canonicalVault = policy().vault;
  assert.equal(
    canonicalVault.storageFormat,
    'sqlcipher_full_database_plus_xchacha20poly1305_records',
  );
  assert.equal(
    canonicalVault.platformKeyStorage,
    'owner_private_file_adapter',
  );
  assert.equal(
    canonicalVault.recoveryModel,
    'independent_wrapped_root_key_offline_restore',
  );
  assert.deepEqual(canonicalVault.unlockModes, [
    'file_adapter_auto',
    'manual_local',
    'recovery_offline',
  ]);
  assert.equal(canonicalVault.defaultUnlockMode, 'file_adapter_auto');
  assert.equal(canonicalVault.defaultLeaseTtlSeconds, 600);
  assert.equal(canonicalVault.maximumLeaseTtlSeconds, 3600);
  assert.equal(canonicalVault.leaseResolveLimit, 1);
  assert.equal(
    canonicalVault.leaseRenewal,
    'new_lease_full_authorization_check',
  );
  assert.equal(canonicalVault.automaticInitializeOrRestoreEnabled, false);
  assert.ok(canonicalVault.leaseBindingFields.includes('secret_revision'));

  const mutations = [
    (vault) => { vault.role = 'core'; },
    (vault) => { vault.owner = 'kernel'; },
    (vault) => { vault.protocolPackage = 'makosh-vault-contracts'; },
    (vault) => { vault.managedClientPackage = 'makosh-vault-client'; },
    (vault) => { vault.keyProviderPackage = 'makosh-platform-key-provider'; },
    (vault) => { vault.runtimePackage = 'makosh-kernel'; },
    (vault) => { vault.storePackage = 'makosh-vault-store'; },
    (vault) => { vault.platformKeyAdapterPackages = []; },
    (vault) => { vault.runtimeComponent = 'vault'; },
    (vault) => { vault.processBoundary = 'kernel_process'; },
    (vault) => { vault.managementMode = 'external_allowed'; },
    (vault) => { vault.externalRegistrationEnabled = true; },
    (vault) => { vault.alternativeTopologyEnabled = true; },
    (vault) => { vault.storageFormat = 'plaintext_sqlite'; },
    (vault) => { vault.platformKeyStorage = 'unprotected_file'; },
    (vault) => { vault.recoveryModel = 'root_key_export'; },
    (vault) => { vault.recoveryKeyEncoding = 'hex'; },
    (vault) => { vault.unlockModes.pop(); },
    (vault) => { vault.defaultUnlockMode = 'recovery_offline'; },
    (vault) => { vault.automaticInitializeOrRestoreEnabled = true; },
    (vault) => { vault.transport = 'plaintext_local_rpc'; },
    (vault) => { vault.transportScope = 'shared_host_channel'; },
    (vault) => { vault.kernelPayloadVisibility = 'plaintext'; },
    (vault) => { vault.leasePersistence = 'sqlite'; },
    (vault) => { vault.defaultLeaseTtlSeconds = 3_600; },
    (vault) => { vault.maximumLeaseTtlSeconds = 86_400; },
    (vault) => { vault.leaseResolveLimit = 2; },
    (vault) => { vault.leaseRenewal = 'extend_in_place'; },
    (vault) => { vault.credentialPayloadMaxBytes = 4_194_304; },
    (vault) => { vault.sessionCredentialBlobMaxBytes = 1_073_741_824; },
    (vault) => { vault.leaseBindingFields.pop(); },
    (vault) => { vault.runtimeGenerationChangeInvalidatesLeases = false; },
    (vault) => { vault.grantEpochChangeInvalidatesLeases = false; },
    (vault) => { vault.forbiddenSecretCarriers.pop(); },
    (vault) => { vault.forbiddenProtocolDependencies.pop(); },
    (vault) => { vault.forbiddenOwnerDependencies.pop(); },
    (vault) => { vault.compatibilityProcess = true; },
  ];

  for (const mutate of mutations) {
    const invalid = policy();
    mutate(invalid.vault);

    assert.ok(codes(validatePolicy(invalid)).has('vault_policy'));
  }

  const missing = policy();
  delete missing.vault;
  assert.ok(codes(validatePolicy(missing)).has('vault_policy'));
});
