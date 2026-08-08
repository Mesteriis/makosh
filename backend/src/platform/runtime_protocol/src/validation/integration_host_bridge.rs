//! Structural validation for a Kernel-issued host-only integration endpoint.

use crate::v1::ManagedIntegrationHostBridgeConfigurationV1;

const MAX_UNIX_SOCKET_PATH_BYTES: usize = 100;
const MAX_IDENTIFIER_BYTES: usize = 128;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntegrationHostBridgeValidationErrorV1 {
    InvalidConfiguration,
}

pub fn validate_managed_integration_host_bridge_configuration(
    configuration: &ManagedIntegrationHostBridgeConfigurationV1,
) -> Result<(), IntegrationHostBridgeValidationErrorV1> {
    if configuration.major != 1
        || !valid_identifier(&configuration.kernel_instance_id)
        || !valid_identifier(&configuration.owner_id)
        || !valid_identifier(&configuration.registration_id)
        || !valid_identifier(&configuration.runtime_instance_id)
        || configuration.runtime_generation == 0
        || configuration.grant_epoch == 0
        || !configuration.socket_path.starts_with('/')
        || configuration.socket_path.len() > MAX_UNIX_SOCKET_PATH_BYTES
        || configuration.route_binding_sha256.len() != 32
    {
        return Err(IntegrationHostBridgeValidationErrorV1::InvalidConfiguration);
    }
    Ok(())
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_IDENTIFIER_BYTES
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_only_a_fenced_absolute_local_endpoint() {
        let configuration = ManagedIntegrationHostBridgeConfigurationV1 {
            major: 1,
            kernel_instance_id: "kernel_1".to_owned(),
            owner_id: "whatsapp".to_owned(),
            registration_id: "whatsapp_runtime".to_owned(),
            runtime_instance_id: "whatsapp_runtime_1".to_owned(),
            runtime_generation: 1,
            grant_epoch: 1,
            socket_path: "/private/tmp/makosh/host-bridge.sock".to_owned(),
            route_binding_sha256: vec![7; 32],
        };

        assert_eq!(
            validate_managed_integration_host_bridge_configuration(&configuration),
            Ok(())
        );
    }

    #[test]
    fn rejects_a_relative_or_unbound_endpoint() {
        let configuration = ManagedIntegrationHostBridgeConfigurationV1 {
            major: 1,
            kernel_instance_id: "kernel_1".to_owned(),
            owner_id: "whatsapp".to_owned(),
            registration_id: "whatsapp_runtime".to_owned(),
            runtime_instance_id: "whatsapp_runtime_1".to_owned(),
            runtime_generation: 1,
            grant_epoch: 1,
            socket_path: "host-bridge.sock".to_owned(),
            route_binding_sha256: Vec::new(),
        };

        assert_eq!(
            validate_managed_integration_host_bridge_configuration(&configuration),
            Err(IntegrationHostBridgeValidationErrorV1::InvalidConfiguration)
        );
    }
}
