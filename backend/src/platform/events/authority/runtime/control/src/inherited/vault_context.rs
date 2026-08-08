//! Builds the verified Vault route context from authority runtime configuration.

use makosh_events_jetstream::NatsVaultRouteContextV1;
use makosh_runtime_protocol::v1::EventsAuthorityRuntimeConfigurationV1;

pub(crate) fn from_configuration(
    configuration: &EventsAuthorityRuntimeConfigurationV1,
) -> Result<NatsVaultRouteContextV1, ()> {
    let public_key = configuration
        .vault_hpke_public_key_x25519
        .as_slice()
        .try_into()
        .map_err(|_| ())?;
    NatsVaultRouteContextV1::new(
        configuration.vault_instance_id.clone(),
        configuration.vault_runtime_generation,
        public_key,
    )
    .map_err(|_| ())
}
