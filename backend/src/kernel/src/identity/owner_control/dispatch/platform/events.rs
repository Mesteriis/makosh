//! Owner-authorized lifecycle commands for the Events credential authority.

use std::path::Path;

use makosh_gateway_protocol::v1::{
    ApplyPlatformEventsAuthorityAccountJwtRequestV1,
    ApplyPlatformEventsAuthorityAccountJwtResponseV1, BindPlatformEventsAuthorityReleaseRequestV1,
    BindPlatformEventsAuthorityReleaseResponseV1, ConfigurePlatformEventsAuthorityRequestV1,
    ConfigurePlatformEventsAuthorityResponseV1, StartPlatformEventsAuthorityRuntimeRequestV1,
    StartPlatformEventsAuthorityRuntimeResponseV1,
};
use makosh_kernel_control_store::PlatformEventsAuthorityConfigurationV1;
use makosh_kernel_control_store_sqlite::SqliteControlStore;

use super::{OwnerControlSessions, OwnerResult, stop_if_active};
use crate::platform::events::authority::{binding, launch};
use crate::runtime::lifecycle::supervisor::ManagedRuntimeSupervisor;

pub(super) fn bind_release(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    sessions: &mut OwnerControlSessions,
    request: BindPlatformEventsAuthorityReleaseRequestV1,
) -> Result<OwnerResult, String> {
    (|| {
        sessions.authorize(store, &request.owner_session_id)?;
        let binding = binding::bind_current_installed_release(store)?;
        stop_if_active(supervisor, binding::EVENTS_AUTHORITY_PROCESS_ID)?;
        Ok(binding)
    })()
    .map(|binding| {
        OwnerResult::BindPlatformEventsAuthorityRelease(
            BindPlatformEventsAuthorityReleaseResponseV1 {
                process_id: binding.process_id().to_owned(),
                binding_revision: binding.binding_revision(),
                distribution_id: binding.distribution_id().to_owned(),
                artifact_id: binding.artifact_id().to_owned(),
            },
        )
    })
}

pub(super) fn configure(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    sessions: &mut OwnerControlSessions,
    request: ConfigurePlatformEventsAuthorityRequestV1,
) -> Result<OwnerResult, String> {
    (|| {
        sessions.authorize(store, &request.owner_session_id)?;
        let configuration = PlatformEventsAuthorityConfigurationV1::new(
            next_configuration_revision(store)?,
            request.account_public_key,
            request.signer_credential_revision,
        );
        stop_if_active(supervisor, binding::EVENTS_AUTHORITY_PROCESS_ID)?;
        store
            .record_platform_events_authority_configuration(&configuration)
            .map_err(|_| "Events authority configuration cannot be recorded".to_owned())?;
        Ok(configuration)
    })()
    .map(|configuration| {
        OwnerResult::ConfigurePlatformEventsAuthority(ConfigurePlatformEventsAuthorityResponseV1 {
            configuration_revision: configuration.revision(),
            signer_credential_revision: configuration.signer_credential_revision(),
        })
    })
}

pub(super) fn start(
    store: &SqliteControlStore,
    runtime_dir: &Path,
    supervisor: &ManagedRuntimeSupervisor,
    sessions: &mut OwnerControlSessions,
    request: StartPlatformEventsAuthorityRuntimeRequestV1,
) -> Result<OwnerResult, String> {
    (|| {
        sessions.authorize(store, &request.owner_session_id)?;
        launch::start(supervisor, store, runtime_dir)
    })()
    .map(|runtime_generation| {
        OwnerResult::StartPlatformEventsAuthorityRuntime(
            StartPlatformEventsAuthorityRuntimeResponseV1 {
                process_id: binding::EVENTS_AUTHORITY_PROCESS_ID.to_owned(),
                runtime_generation,
                launch_state: "accepted".to_owned(),
            },
        )
    })
}

pub(super) fn apply_account_jwt(
    supervisor: &ManagedRuntimeSupervisor,
    sessions: &mut OwnerControlSessions,
    store: &SqliteControlStore,
    request: ApplyPlatformEventsAuthorityAccountJwtRequestV1,
) -> Result<OwnerResult, String> {
    sessions.authorize(store, &request.owner_session_id)?;
    let resolver_credential_revision = crate::platform::events::authority::account_jwt::apply(
        &supervisor.relay_port(),
        request.resolver_credential_revision,
        request.signed_account_jwt,
    )?;
    Ok(OwnerResult::ApplyPlatformEventsAuthorityAccountJwt(
        ApplyPlatformEventsAuthorityAccountJwtResponseV1 {
            resolver_credential_revision,
        },
    ))
}

fn next_configuration_revision(store: &SqliteControlStore) -> Result<u64, String> {
    store
        .platform_events_authority_configuration()
        .map_err(|_| "Events authority configuration is unavailable".to_owned())?
        .map_or(Ok(1), |current| {
            current
                .revision()
                .checked_add(1)
                .ok_or_else(|| "Events authority configuration revision overflowed".to_owned())
        })
}
