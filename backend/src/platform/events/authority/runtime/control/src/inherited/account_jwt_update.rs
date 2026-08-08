//! Applies an already signed NATS Account JWT through the broker resolver.

use std::os::unix::net::UnixStream;

use makosh_events_jetstream::{
    NatsAccountJwtUpdateV1, NatsResolverAccountJwtPublisherV1, NatsResolverCredentialFenceV1,
    NatsResolverCredentialLeaseAdapterV1,
};
use makosh_runtime_protocol::{
    v1::{
        ApplyEventsAccountJwtUpdateRequestV1, ApplyEventsAccountJwtUpdateResponseV1,
        EventsAuthorityRuntimeConfigurationV1, EventsAuthorityRuntimeControlResponseV1,
        events_authority_runtime_control_response_v1::Result as ResponseResult,
    },
    validation::events_authority::validate_account_jwt_update,
};

use super::{
    handshake::EventsAuthorityRuntimeIdentityV1, runtime::AUTHORITY_RUNTIME_INSTANCE_ID,
    vault_context, vault_route::InheritedVaultRoutePortV1,
};

pub(crate) fn response(
    channel: &UnixStream,
    request: ApplyEventsAccountJwtUpdateRequestV1,
    identity: &EventsAuthorityRuntimeIdentityV1,
    configuration: &EventsAuthorityRuntimeConfigurationV1,
) -> EventsAuthorityRuntimeControlResponseV1 {
    match apply(channel, request, identity, configuration) {
        Ok(revision) => EventsAuthorityRuntimeControlResponseV1 {
            result: Some(ResponseResult::AccountJwtUpdated(
                ApplyEventsAccountJwtUpdateResponseV1 {
                    resolver_credential_revision: revision,
                },
            )),
            error_code: String::new(),
        },
        Err(()) => EventsAuthorityRuntimeControlResponseV1 {
            result: None,
            error_code: "account_jwt_update_denied".to_owned(),
        },
    }
}

fn apply(
    channel: &UnixStream,
    request: ApplyEventsAccountJwtUpdateRequestV1,
    identity: &EventsAuthorityRuntimeIdentityV1,
    configuration: &EventsAuthorityRuntimeConfigurationV1,
) -> Result<u64, ()> {
    validate_account_jwt_update(&request).map_err(|_| ())?;
    let update = NatsAccountJwtUpdateV1::new(
        configuration.account_public_key.clone(),
        request.signed_account_jwt,
    )
    .map_err(|_| ())?;
    let context = vault_context::from_configuration(configuration)?;
    let route = channel.try_clone().map_err(|_| ())?;
    let fence = NatsResolverCredentialFenceV1::new(
        identity.registration_id().to_owned(),
        AUTHORITY_RUNTIME_INSTANCE_ID,
        identity.runtime_generation(),
        identity.grant_epoch(),
        request.resolver_credential_revision,
    )
    .map_err(|_| ())?;
    let mut leases =
        NatsResolverCredentialLeaseAdapterV1::new(InheritedVaultRoutePortV1::new(route), context);
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()
        .map_err(|_| ())?;
    runtime.block_on(async {
        let credentials = leases
            .resolve_system_credentials(&fence)
            .await
            .map_err(|_| ())?;
        NatsResolverAccountJwtPublisherV1::publish(
            &configuration.nats_endpoint,
            &credentials,
            &update,
        )
        .await
        .map_err(|_| ())
    })?;
    Ok(fence.credential_revision())
}
