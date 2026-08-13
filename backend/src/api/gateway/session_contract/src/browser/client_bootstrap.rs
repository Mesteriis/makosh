// Client-safe bootstrap projection contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientBootstrapProjectionV1 {
    modules: Vec<ClientModuleProjectionV1>,
    surfaces: Vec<ClientSurfaceAvailabilityProjectionV1>,
    system_status: Vec<ClientSystemComponentStatusProjectionV1>,
}

impl ClientBootstrapProjectionV1 {
    #[must_use]
    pub fn new(modules: Vec<ClientModuleProjectionV1>) -> Self {
        Self::with_surfaces(modules, Vec::new())
    }

    #[must_use]
    pub fn with_surfaces(
        modules: Vec<ClientModuleProjectionV1>,
        surfaces: Vec<ClientSurfaceAvailabilityProjectionV1>,
    ) -> Self {
        Self {
            modules,
            surfaces,
            system_status: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_system_status(
        modules: Vec<ClientModuleProjectionV1>,
        surfaces: Vec<ClientSurfaceAvailabilityProjectionV1>,
        system_status: Vec<ClientSystemComponentStatusProjectionV1>,
    ) -> Self {
        Self {
            modules,
            surfaces,
            system_status,
        }
    }

    #[must_use]
    pub fn modules(&self) -> &[ClientModuleProjectionV1] {
        &self.modules
    }

    #[must_use]
    pub fn surfaces(&self) -> &[ClientSurfaceAvailabilityProjectionV1] {
        &self.surfaces
    }

    #[must_use]
    pub fn system_status(&self) -> &[ClientSystemComponentStatusProjectionV1] {
        &self.system_status
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientSystemComponentIdV1 {
    Kernel,
    ControlStore,
    ModuleControlPlane,
    Gateway,
    Vault,
    StorageControl,
    Postgresql,
    Pgbouncer,
    Nats,
    EventHub,
    Scheduler,
    Clock,
    Blob,
    Telemetry,
    Sse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientSystemComponentStateV1 {
    Healthy,
    Degraded,
    Unavailable,
    NotAdmitted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientSystemComponentStatusProjectionV1 {
    component_id: ClientSystemComponentIdV1,
    state: ClientSystemComponentStateV1,
    sanitized_reason_code: Option<String>,
}

impl ClientSystemComponentStatusProjectionV1 {
    #[must_use]
    pub fn new(
        component_id: ClientSystemComponentIdV1,
        state: ClientSystemComponentStateV1,
        sanitized_reason_code: Option<String>,
    ) -> Self {
        Self {
            component_id,
            state,
            sanitized_reason_code,
        }
    }

    #[must_use]
    pub const fn component_id(&self) -> ClientSystemComponentIdV1 {
        self.component_id
    }

    #[must_use]
    pub const fn state(&self) -> ClientSystemComponentStateV1 {
        self.state
    }

    #[must_use]
    pub fn sanitized_reason_code(&self) -> Option<&str> {
        self.sanitized_reason_code.as_deref()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ClientSurfaceIdV1 {
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
}

impl ClientSurfaceIdV1 {
    /// Exact owner-approved capability required before a compiled product route
    /// may be admitted. Settings is the local recovery surface and therefore
    /// never derives admission from a module grant.
    #[must_use]
    pub const fn admission_capability_id(self) -> Option<&'static str> {
        match self {
            Self::Dashboard => Some("client.surface.dashboard.v1"),
            Self::Communications => Some("communications.query.v1"),
            Self::Mail => Some("mail.delivery.query.v1"),
            Self::Telegram => Some("telegram.query.v1"),
            Self::Review => Some("client.surface.review.v1"),
            Self::Personas => Some("client.surface.personas.v1"),
            Self::Knowledge => Some("knowledge.client.v1"),
            Self::Tasks => Some("tasks.client.v1"),
            Self::Calendar => Some("calendar.client.v1"),
            Self::Documents => Some("documents.client.v1"),
            Self::Settings => None,
            Self::WhatsApp => Some("whatsapp.query.v1"),
            Self::Zulip => Some("zulip.query.v1"),
            Self::Organizations => Some("organizations.client.v1"),
            Self::Projects => Some("projects.client.v1"),
            Self::Obligations => Some("obligations.client.v1"),
            Self::Decisions => Some("decisions.client.v1"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientSurfaceAvailabilityStateV1 {
    Available,
    NotAdmitted,
    Starting,
    Blocked,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientSurfaceAvailabilityProjectionV1 {
    surface_id: ClientSurfaceIdV1,
    state: ClientSurfaceAvailabilityStateV1,
    sanitized_reason_code: Option<String>,
    supported_client_contract_major: u32,
}

impl ClientSurfaceAvailabilityProjectionV1 {
    pub fn new(
        surface_id: ClientSurfaceIdV1,
        state: ClientSurfaceAvailabilityStateV1,
        sanitized_reason_code: Option<String>,
        supported_client_contract_major: u32,
    ) -> Result<Self, String> {
        (supported_client_contract_major > 0)
            .then_some(Self {
                surface_id,
                state,
                sanitized_reason_code,
                supported_client_contract_major,
            })
            .ok_or_else(|| "client surface availability is invalid".to_owned())
    }
    #[must_use]
    pub const fn surface_id(&self) -> ClientSurfaceIdV1 {
        self.surface_id
    }
    #[must_use]
    pub const fn state(&self) -> ClientSurfaceAvailabilityStateV1 {
        self.state
    }
    #[must_use]
    pub fn sanitized_reason_code(&self) -> Option<&str> {
        self.sanitized_reason_code.as_deref()
    }
    #[must_use]
    pub const fn supported_client_contract_major(&self) -> u32 {
        self.supported_client_contract_major
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientModuleProjectionV1 {
    registration_id: String,
    module_id: String,
    grant_epoch: u64,
    capability_ids: Vec<String>,
    sections_enabled: bool,
    settings: Option<ClientModuleSettingsProjectionV1>,
    settings_targets: Vec<ClientModuleSettingsTargetProjectionV1>,
}

impl ClientModuleProjectionV1 {
    #[must_use]
    pub fn new(
        registration_id: String,
        module_id: String,
        grant_epoch: u64,
        capability_ids: Vec<String>,
        sections_enabled: bool,
        settings: Option<ClientModuleSettingsProjectionV1>,
        settings_targets: Vec<ClientModuleSettingsTargetProjectionV1>,
    ) -> Self {
        Self {
            registration_id,
            module_id,
            grant_epoch,
            capability_ids,
            sections_enabled,
            settings,
            settings_targets,
        }
    }
    #[must_use]
    pub fn registration_id(&self) -> &str {
        &self.registration_id
    }
    #[must_use]
    pub fn module_id(&self) -> &str {
        &self.module_id
    }
    #[must_use]
    pub const fn grant_epoch(&self) -> u64 {
        self.grant_epoch
    }
    #[must_use]
    pub fn capability_ids(&self) -> &[String] {
        &self.capability_ids
    }
    #[must_use]
    pub const fn sections_enabled(&self) -> bool {
        self.sections_enabled
    }
    #[must_use]
    pub fn settings(&self) -> Option<&ClientModuleSettingsProjectionV1> {
        self.settings.as_ref()
    }
    #[must_use]
    pub fn settings_targets(&self) -> &[ClientModuleSettingsTargetProjectionV1] {
        &self.settings_targets
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientModuleSettingsTargetProjectionV1 {
    configuration_instance_id: String,
    desired_revision: u64,
    effective_revision: u64,
    apply_state: String,
    sanitized_reason_code: Option<String>,
    values: Vec<ClientSettingValueEntryV1>,
}

impl ClientModuleSettingsTargetProjectionV1 {
    #[must_use]
    pub fn new(
        configuration_instance_id: String,
        desired_revision: u64,
        effective_revision: u64,
        apply_state: String,
        sanitized_reason_code: Option<String>,
        values: Vec<ClientSettingValueEntryV1>,
    ) -> Self {
        Self {
            configuration_instance_id,
            desired_revision,
            effective_revision,
            apply_state,
            sanitized_reason_code,
            values,
        }
    }
    #[must_use]
    pub fn configuration_instance_id(&self) -> &str {
        &self.configuration_instance_id
    }
    #[must_use]
    pub const fn desired_revision(&self) -> u64 {
        self.desired_revision
    }
    #[must_use]
    pub const fn effective_revision(&self) -> u64 {
        self.effective_revision
    }
    #[must_use]
    pub fn apply_state(&self) -> &str {
        &self.apply_state
    }
    #[must_use]
    pub fn sanitized_reason_code(&self) -> Option<&str> {
        self.sanitized_reason_code.as_deref()
    }
    #[must_use]
    pub fn values(&self) -> &[ClientSettingValueEntryV1] {
        &self.values
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientModuleSettingsProjectionV1 {
    schema_major: u32,
    schema_revision: u32,
    desired_revision: u64,
    effective_revision: u64,
    apply_state: String,
    sanitized_reason_code: Option<String>,
    values: Vec<ClientSettingValueEntryV1>,
}

impl ClientModuleSettingsProjectionV1 {
    #[must_use]
    pub fn new(
        schema_major: u32,
        schema_revision: u32,
        desired_revision: u64,
        effective_revision: u64,
        apply_state: String,
        sanitized_reason_code: Option<String>,
        values: Vec<ClientSettingValueEntryV1>,
    ) -> Self {
        Self {
            schema_major,
            schema_revision,
            desired_revision,
            effective_revision,
            apply_state,
            sanitized_reason_code,
            values,
        }
    }
    #[must_use]
    pub const fn schema_major(&self) -> u32 {
        self.schema_major
    }
    #[must_use]
    pub const fn schema_revision(&self) -> u32 {
        self.schema_revision
    }
    #[must_use]
    pub const fn desired_revision(&self) -> u64 {
        self.desired_revision
    }
    #[must_use]
    pub const fn effective_revision(&self) -> u64 {
        self.effective_revision
    }
    #[must_use]
    pub fn apply_state(&self) -> &str {
        &self.apply_state
    }
    #[must_use]
    pub fn sanitized_reason_code(&self) -> Option<&str> {
        self.sanitized_reason_code.as_deref()
    }
    #[must_use]
    pub fn values(&self) -> &[ClientSettingValueEntryV1] {
        &self.values
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientSettingValueEntryV1 {
    setting_id: String,
    value: ClientSettingValueV1,
    display_name: String,
    editable: bool,
}
impl ClientSettingValueEntryV1 {
    #[must_use]
    pub fn new(
        setting_id: String,
        value: ClientSettingValueV1,
        display_name: String,
        editable: bool,
    ) -> Self {
        Self {
            setting_id,
            value,
            display_name,
            editable,
        }
    }
    #[must_use]
    pub fn setting_id(&self) -> &str {
        &self.setting_id
    }
    #[must_use]
    pub fn value(&self) -> &ClientSettingValueV1 {
        &self.value
    }
    #[must_use]
    pub fn display_name(&self) -> &str {
        &self.display_name
    }
    #[must_use]
    pub const fn editable(&self) -> bool {
        self.editable
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientSettingValueV1 {
    Boolean(bool),
    SignedInteger(i64),
    UnsignedInteger(u64),
    Decimal(String),
    String(String),
    DurationMillis(u64),
    TimestampUnixMillis(i64),
    Enum(String),
    ResourceReference(String),
}

pub trait ClientBootstrapAuthority {
    fn client_bootstrap(
        &self,
        owner_id: &str,
        device_id: &str,
    ) -> Result<ClientBootstrapProjectionV1, String>;
}

#[cfg(test)]
mod tests {
    use super::ClientSurfaceIdV1;

    #[test]
    fn provider_surfaces_require_their_own_exact_query_capability() {
        assert_eq!(
            ClientSurfaceIdV1::Communications.admission_capability_id(),
            Some("communications.query.v1")
        );
        assert_eq!(
            ClientSurfaceIdV1::Mail.admission_capability_id(),
            Some("mail.delivery.query.v1")
        );
        assert_eq!(
            ClientSurfaceIdV1::Telegram.admission_capability_id(),
            Some("telegram.query.v1")
        );
        assert_eq!(
            ClientSurfaceIdV1::WhatsApp.admission_capability_id(),
            Some("whatsapp.query.v1")
        );
        assert_eq!(
            ClientSurfaceIdV1::Zulip.admission_capability_id(),
            Some("zulip.query.v1")
        );
        assert_eq!(
            ClientSurfaceIdV1::Tasks.admission_capability_id(),
            Some("tasks.client.v1")
        );
        assert_eq!(
            ClientSurfaceIdV1::Organizations.admission_capability_id(),
            Some("organizations.client.v1")
        );
        assert_eq!(
            ClientSurfaceIdV1::Documents.admission_capability_id(),
            Some("documents.client.v1")
        );
        assert_eq!(
            ClientSurfaceIdV1::Projects.admission_capability_id(),
            Some("projects.client.v1")
        );
    }
}
