//! Kernel implementation of the Gateway browser-session authority contract.

use std::sync::Arc;

use makosh_gateway_session_contract::{
    BrowserAssertionAuthority, BrowserAuthenticationAuthority, BrowserDeviceAuthority,
    BrowserDeviceCredentialV1, BrowserDevicePrincipalV1, BrowserEnrollmentAuthority,
    BrowserEnrollmentV1, BrowserPairingAuthority, ClientBootstrapAuthority,
    ClientBootstrapProjectionV1, ClientModuleProjectionV1, ClientModuleSettingsProjectionV1,
    ClientModuleSettingsTargetProjectionV1, ClientSettingValueEntryV1, ClientSettingValueV1,
    ClientSurfaceAvailabilityProjectionV1, ClientSurfaceAvailabilityStateV1, ClientSurfaceIdV1,
    GatewayIdentityFenceV1,
};
use makosh_kernel_control_store::{
    BrowserDeviceEnrollmentInputV1, BrowserDeviceEnrollmentV1, BrowserDeviceIdentityV1,
    BrowserDeviceStateV1, ModuleGrantSnapshot, SettingsApplyState,
};
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use makosh_runtime_protocol::{
    SETTINGS_CONFIGURATION_CATALOG_CAPABILITY_ID,
    v1::{SettingClientVisibilityV1, SettingsSchemaV1, setting_value_v1::Value},
    validation::descriptor::{
        decode_settings_schema_v1, decode_settings_snapshot_v1,
        validate_settings_snapshot_against_schema_v1,
    },
};

use crate::runtime::lifecycle::supervisor::ManagedRuntimeSupervisor;

#[path = "browser_gateway/system_status.rs"]
pub(crate) mod system_status;

#[derive(Clone)]
pub(crate) struct ControlStoreBrowserAuthority {
    store: Arc<SqliteControlStore>,
    supervisor: ManagedRuntimeSupervisor,
    developer_realtime_enabled: bool,
}

impl ControlStoreBrowserAuthority {
    #[must_use]
    pub(crate) fn new(
        store: Arc<SqliteControlStore>,
        supervisor: ManagedRuntimeSupervisor,
    ) -> Self {
        Self {
            store,
            supervisor,
            developer_realtime_enabled: false,
        }
    }

    #[must_use]
    pub(crate) fn with_developer_realtime(mut self) -> Self {
        self.developer_realtime_enabled = true;
        self
    }
}

impl BrowserDeviceAuthority for ControlStoreBrowserAuthority {
    fn current_identity_fence(&self) -> Result<GatewayIdentityFenceV1, String> {
        let snapshot = self.store.snapshot();
        GatewayIdentityFenceV1::new(
            snapshot.instance_id(),
            snapshot.generation(),
            self.store.current_identity_epoch().map_err(store_error)?,
        )
    }

    fn active_browser_device(&self, device_id: &str) -> Result<BrowserDevicePrincipalV1, String> {
        self.store
            .browser_device_identity(device_id)
            .map_err(store_error)
            .and_then(active_principal)
    }

    fn active_browser_device_by_credential(
        &self,
        credential_id: &[u8],
    ) -> Result<BrowserDevicePrincipalV1, String> {
        self.store
            .browser_device_identity_by_credential_id(credential_id)
            .map_err(store_error)
            .and_then(active_principal)
    }
}

impl BrowserAssertionAuthority for ControlStoreBrowserAuthority {
    fn accept_verified_browser_assertion(
        &self,
        credential_id: &[u8],
        sign_count: u32,
        backup_eligible: bool,
        backup_state: bool,
    ) -> Result<BrowserDevicePrincipalV1, String> {
        let identity_epoch = self.store.current_identity_epoch().map_err(store_error)?;
        self.store
            .record_verified_browser_assertion(
                credential_id,
                sign_count,
                backup_eligible,
                backup_state,
                identity_epoch,
            )
            .map_err(store_error)
            .and_then(|device| active_principal(Some(device)))
    }
}

impl BrowserAuthenticationAuthority for ControlStoreBrowserAuthority {
    fn active_browser_credential(
        &self,
        credential_id: &[u8],
    ) -> Result<BrowserDeviceCredentialV1, String> {
        self.store
            .browser_device_identity_by_credential_id(credential_id)
            .map_err(store_error)
            .and_then(|device| {
                let device = device.ok_or_else(|| "browser device is unavailable".to_owned())?;
                (device.state() == BrowserDeviceStateV1::Active)
                    .then_some(device)
                    .ok_or_else(|| "browser device is revoked".to_owned())
            })
            .and_then(|device| {
                BrowserDeviceCredentialV1::new(
                    device.enrollment().credential_id().to_vec(),
                    device.enrollment().cose_public_key().to_vec(),
                    device.enrollment().browser_key_public_key().to_vec(),
                    device.enrollment().sign_count(),
                    device.enrollment().backup_eligible(),
                    device.enrollment().backup_state(),
                )
            })
    }
}

impl ClientBootstrapAuthority for ControlStoreBrowserAuthority {
    fn client_bootstrap(
        &self,
        owner_id: &str,
        _device_id: &str,
    ) -> Result<ClientBootstrapProjectionV1, String> {
        self.require_current_owner(owner_id)?;
        let snapshots = self
            .store
            .approved_module_grant_snapshots()
            .map_err(store_error)?;
        let mut modules = snapshots
            .into_iter()
            .map(|snapshot| project_module(&self.store, snapshot))
            .collect::<Result<Vec<_>, _>>()?;
        if modules.len() > 128 {
            return Err("client bootstrap is unavailable".to_owned());
        }
        modules.sort_by(|left, right| left.registration_id().cmp(right.registration_id()));
        let surfaces = client_surface_availability(&modules)?;
        let system_status = system_status::client_system_status(
            &self.store,
            &self.supervisor,
            self.developer_realtime_enabled,
        );
        Ok(ClientBootstrapProjectionV1::with_system_status(
            modules,
            surfaces,
            system_status,
        ))
    }
}

fn client_surface_availability(
    modules: &[ClientModuleProjectionV1],
) -> Result<Vec<ClientSurfaceAvailabilityProjectionV1>, String> {
    use ClientSurfaceAvailabilityStateV1::{Available, Blocked, NotAdmitted};
    use ClientSurfaceIdV1::{
        Calendar, Communications, Dashboard, Decisions, Documents, Knowledge, Mail, Obligations,
        Organizations, Personas, Projects, Review, Settings, Tasks, Telegram, WhatsApp, Zulip,
    };

    [
        Dashboard,
        Communications,
        Mail,
        Telegram,
        Review,
        Personas,
        Knowledge,
        Tasks,
        Calendar,
        Documents,
        Settings,
        WhatsApp,
        Zulip,
        Organizations,
        Projects,
        Obligations,
        Decisions,
    ]
    .into_iter()
    .map(|surface_id| {
        let (state, reason) = match surface_id.admission_capability_id() {
            None => (Available, None),
            Some(capability_id)
                if modules.iter().any(|module| {
                    module.sections_enabled()
                        && module
                            .capability_ids()
                            .iter()
                            .any(|candidate| candidate == capability_id)
                }) =>
            {
                (Available, None)
            }
            Some(capability_id)
                if modules.iter().any(|module| {
                    module
                        .capability_ids()
                        .iter()
                        .any(|candidate| candidate == capability_id)
                }) =>
            {
                (Blocked, Some("surface_settings_not_current".to_owned()))
            }
            Some(_) => (NotAdmitted, Some("surface_not_admitted".to_owned())),
        };
        ClientSurfaceAvailabilityProjectionV1::new(surface_id, state, reason, 1)
    })
    .collect()
}

impl BrowserPairingAuthority for ControlStoreBrowserAuthority {
    fn current_identity_fence(&self) -> Result<GatewayIdentityFenceV1, String> {
        BrowserDeviceAuthority::current_identity_fence(self)
    }

    fn require_current_owner(&self, owner_id: &str) -> Result<(), String> {
        let owner = self
            .store
            .initial_owner_identity()
            .map_err(store_error)?
            .ok_or_else(|| "owner is unavailable".to_owned())?;
        (owner.owner_id() == owner_id)
            .then_some(())
            .ok_or_else(|| "owner is unavailable".to_owned())
    }
}

impl BrowserEnrollmentAuthority for ControlStoreBrowserAuthority {
    fn admit_browser_device(
        &self,
        enrollment: &BrowserEnrollmentV1,
    ) -> Result<BrowserDevicePrincipalV1, String> {
        self.require_current_owner(enrollment.owner_id())?;
        let current = BrowserDeviceAuthority::current_identity_fence(self)?;
        (current == *enrollment.identity_fence())
            .then_some(())
            .ok_or_else(|| "browser pairing is stale".to_owned())?;
        let enrollment = BrowserDeviceEnrollmentV1::new(BrowserDeviceEnrollmentInputV1 {
            owner_id: enrollment.owner_id().to_owned(),
            device_id: enrollment.device_id().to_owned(),
            credential_id: enrollment.credential_id().to_vec(),
            cose_public_key: enrollment.cose_public_key().to_vec(),
            browser_key_public_key: enrollment.browser_key_public_key().to_vec(),
            rp_id: enrollment.rp_id().to_owned(),
            sign_count: enrollment.sign_count(),
            backup_eligible: enrollment.backup_eligible(),
            backup_state: enrollment.backup_state(),
        })?;
        self.store
            .admit_browser_device(&enrollment, current.identity_epoch())
            .map_err(store_error)
            .and_then(|device| active_principal(Some(device)))
    }
}

fn active_principal(
    device: Option<BrowserDeviceIdentityV1>,
) -> Result<BrowserDevicePrincipalV1, String> {
    let device = device.ok_or_else(|| "browser device is unavailable".to_owned())?;
    (device.state() == BrowserDeviceStateV1::Active)
        .then_some(device)
        .ok_or_else(|| "browser device is revoked".to_owned())
        .and_then(|device| {
            BrowserDevicePrincipalV1::new(
                device.enrollment().owner_id(),
                device.enrollment().device_id(),
            )
        })
}

fn project_module(
    store: &SqliteControlStore,
    snapshot: ModuleGrantSnapshot,
) -> Result<ClientModuleProjectionV1, String> {
    let registration = snapshot.registration();
    let grants = snapshot
        .effective_grants()
        .ok_or_else(|| "client bootstrap is unavailable".to_owned())?;
    if grants.capability_ids().len() > 256 {
        return Err("client bootstrap is unavailable".to_owned());
    }
    let (default_target_current, settings) =
        project_settings(store, registration.registration_id())?;
    let settings_targets = project_settings_targets(
        store,
        registration.registration_id(),
        grants.capability_ids(),
    )?;
    let sections_enabled = default_target_current
        || catalog_has_current_configuration(
            store,
            registration.registration_id(),
            grants.capability_ids(),
        )?;
    Ok(ClientModuleProjectionV1::new(
        registration.registration_id().to_owned(),
        registration.module_id().to_owned(),
        registration.grant_epoch(),
        grants.capability_ids().to_vec(),
        sections_enabled,
        settings,
        settings_targets,
    ))
}

fn project_settings_targets(
    store: &SqliteControlStore,
    registration_id: &str,
    capability_ids: &[String],
) -> Result<Vec<ClientModuleSettingsTargetProjectionV1>, String> {
    if !capability_ids
        .iter()
        .any(|capability_id| capability_id == SETTINGS_CONFIGURATION_CATALOG_CAPABILITY_ID)
    {
        return Ok(Vec::new());
    }
    let targets = store
        .settings_configuration_targets(registration_id)
        .map_err(store_error)?;
    if targets.len() > 64 {
        return Err(bootstrap_unavailable());
    }
    targets
        .into_iter()
        .map(|target| {
            let values = if target.apply_state() == SettingsApplyState::BlockedConfig {
                Vec::new()
            } else {
                visible_settings_for_target(
                    store,
                    registration_id,
                    target.configuration_instance_id(),
                    target.desired_revision(),
                )?
            };
            Ok(ClientModuleSettingsTargetProjectionV1::new(
                target.configuration_instance_id().to_owned(),
                target.desired_revision(),
                target.effective_revision(),
                target.apply_state().as_str().to_owned(),
                target.sanitized_reason_code().map(str::to_owned),
                values,
            ))
        })
        .collect()
}

fn catalog_has_current_configuration(
    store: &SqliteControlStore,
    registration_id: &str,
    capability_ids: &[String],
) -> Result<bool, String> {
    if !capability_ids
        .iter()
        .any(|capability_id| capability_id == SETTINGS_CONFIGURATION_CATALOG_CAPABILITY_ID)
    {
        return Ok(false);
    }
    Ok(store
        .settings_configuration_targets(registration_id)
        .map_err(store_error)?
        .iter()
        .any(|target| {
            target.effective_revision() > 0
                && target.desired_revision() == target.effective_revision()
                && target.apply_state() == SettingsApplyState::Current
        }))
}

fn project_settings(
    store: &SqliteControlStore,
    registration_id: &str,
) -> Result<(bool, Option<ClientModuleSettingsProjectionV1>), String> {
    let Some(binding) = store
        .settings_schema_binding(registration_id)
        .map_err(store_error)?
    else {
        return Ok((true, None));
    };
    let current = binding.desired_revision() == binding.effective_revision()
        && binding.apply_state() == SettingsApplyState::Current;
    let values = if current {
        visible_settings_for_target(
            store,
            registration_id,
            registration_id,
            binding.desired_revision(),
        )?
    } else {
        Vec::new()
    };
    let settings = ClientModuleSettingsProjectionV1::new(
        binding.schema_major(),
        binding.schema_revision(),
        binding.desired_revision(),
        binding.effective_revision(),
        binding.apply_state().as_str().to_owned(),
        binding.sanitized_reason_code().map(str::to_owned),
        values,
    );
    Ok((current, Some(settings)))
}

fn visible_settings_for_target(
    store: &SqliteControlStore,
    registration_id: &str,
    configuration_instance_id: &str,
    desired_revision: u64,
) -> Result<Vec<ClientSettingValueEntryV1>, String> {
    let schema = settings_schema(store, registration_id)?;
    let Some((revision, bytes)) = store
        .desired_settings_snapshot_for_target(registration_id, configuration_instance_id)
        .map_err(store_error)?
    else {
        return (desired_revision == 0)
            .then_some(Vec::new())
            .ok_or_else(bootstrap_unavailable);
    };
    let snapshot = decode_settings_snapshot_v1(&bytes).map_err(|_| bootstrap_unavailable())?;
    if revision != desired_revision
        || snapshot.target_id != configuration_instance_id
        || snapshot.revision != desired_revision
    {
        return Err(bootstrap_unavailable());
    }
    validate_settings_snapshot_against_schema_v1(&schema, &snapshot)
        .map_err(|_| bootstrap_unavailable())?;
    snapshot
        .values
        .iter()
        .filter_map(|entry| visible_entry(&schema, entry))
        .collect::<Result<Vec<_>, _>>()
}

fn settings_schema(
    store: &SqliteControlStore,
    registration_id: &str,
) -> Result<SettingsSchemaV1, String> {
    let bytes = store
        .settings_schema_artifact(registration_id)
        .map_err(store_error)?
        .ok_or_else(bootstrap_unavailable)?;
    decode_settings_schema_v1(&bytes).map_err(|_| bootstrap_unavailable())
}

fn visible_entry(
    schema: &SettingsSchemaV1,
    entry: &makosh_runtime_protocol::v1::SettingsValueEntryV1,
) -> Option<Result<ClientSettingValueEntryV1, String>> {
    let definition = schema
        .definitions
        .binary_search_by(|definition| definition.setting_id.cmp(&entry.setting_id))
        .ok()
        .map(|index| &schema.definitions[index])?;
    let visibility = SettingClientVisibilityV1::try_from(definition.client_visibility).ok()?;
    matches!(
        visibility,
        SettingClientVisibilityV1::Editable | SettingClientVisibilityV1::ReadOnly
    )
    .then(|| {
        entry
            .value
            .as_ref()
            .and_then(project_value)
            .map(|value| {
                Ok(ClientSettingValueEntryV1::new(
                    entry.setting_id.clone(),
                    value,
                    definition.display_name.clone(),
                    visibility == SettingClientVisibilityV1::Editable,
                ))
            })
            .unwrap_or_else(|| Err(bootstrap_unavailable()))
    })
}

fn project_value(
    value: &makosh_runtime_protocol::v1::SettingValueV1,
) -> Option<ClientSettingValueV1> {
    Some(match value.value.as_ref()? {
        Value::BooleanValue(value) => ClientSettingValueV1::Boolean(*value),
        Value::SignedIntegerValue(value) => ClientSettingValueV1::SignedInteger(*value),
        Value::UnsignedIntegerValue(value) => ClientSettingValueV1::UnsignedInteger(*value),
        Value::DecimalValue(value) => ClientSettingValueV1::Decimal(value.clone()),
        Value::StringValue(value) => ClientSettingValueV1::String(value.clone()),
        Value::DurationMillis(value) => ClientSettingValueV1::DurationMillis(*value),
        Value::TimestampUnixMillis(value) => ClientSettingValueV1::TimestampUnixMillis(*value),
        Value::EnumValue(value) => ClientSettingValueV1::Enum(value.clone()),
        Value::ResourceReference(value) => ClientSettingValueV1::ResourceReference(value.clone()),
    })
}

fn bootstrap_unavailable() -> String {
    "client bootstrap is unavailable".to_owned()
}

fn store_error(error: impl std::fmt::Debug) -> String {
    format!("{error:?}")
}
