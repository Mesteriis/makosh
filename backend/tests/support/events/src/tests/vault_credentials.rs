use makosh_events_jetstream::vault::NatsVaultRouteFailureV1;
use makosh_events_jetstream::{
    EventHubCredentialFenceV1, EventHubCredentialLeaseAdapterV1, NatsCredentialLeaseAdapterV1,
    NatsRuntimeCredentialFenceV1, NatsVaultRouteContextV1,
};

use super::support::ScriptedVaultRoute;

#[tokio::test]
async fn nats_credential_creation_and_resolution_stay_on_the_encrypted_vault_route() {
    let mut adapter = NatsCredentialLeaseAdapterV1::new(
        ScriptedVaultRoute::new(vec![
            Err(NatsVaultRouteFailureV1::Rejected),
            Ok(vec![b'a'; 32]),
            Ok(vec![3; 16]),
            Ok(vec![b'b'; 32]),
            Ok(b"nats-runtime-password".to_vec()),
        ]),
        context(),
    );
    let fence = fence();

    let credential = adapter
        .ensure_runtime_credential(&fence)
        .await
        .expect("Vault-backed NATS credential");
    assert_eq!(credential.as_slice(), b"nats-runtime-password");
    let route_port = adapter.into_route_port();
    assert_eq!(route_port.routes.len(), 5);
    assert!(route_port.routes.iter().all(|route| {
        route.registration_id == "registration_notes"
            && route.runtime_instance_id == "notes_runtime"
            && route.caller_runtime_generation == 2
            && route.grant_epoch == 5
            && !route.ciphertext.windows(4).any(|value| value == b"nats")
    }));
}

#[tokio::test]
async fn unavailable_vault_route_does_not_fall_back_to_a_local_broker_secret() {
    let mut adapter = NatsCredentialLeaseAdapterV1::new(
        ScriptedVaultRoute::new(vec![Err(NatsVaultRouteFailureV1::Unavailable)]),
        context(),
    );
    assert_eq!(
        adapter.ensure_runtime_credential(&fence()).await,
        Err(makosh_events_jetstream::vault::NatsCredentialLeaseErrorV1::Unavailable),
    );
}

#[tokio::test]
async fn runtime_lease_revoke_stays_on_the_same_fenced_vault_route() {
    let mut adapter =
        NatsCredentialLeaseAdapterV1::new(ScriptedVaultRoute::new(vec![Ok(vec![1])]), context());
    adapter
        .revoke_runtime_leases(&fence())
        .await
        .expect("revoke Vault audience");
    let route_port = adapter.into_route_port();
    assert_eq!(route_port.routes.len(), 1);
    assert_eq!(route_port.routes[0].grant_epoch, 5);
    assert_eq!(route_port.routes[0].caller_runtime_generation, 2);
}

#[tokio::test]
async fn event_hub_identity_is_resolved_through_the_authority_fenced_vault_audience() {
    let mut adapter = EventHubCredentialLeaseAdapterV1::new(
        ScriptedVaultRoute::new(vec![Ok(vec![b'a'; 32]), Ok(b"event-hub-password".to_vec())]),
        context(),
    );

    let identity = adapter
        .resolve_event_hub_identity(&event_hub_fence())
        .await
        .expect("Vault-backed Event Hub identity");
    let diagnostic = format!("{identity:?}");
    assert!(diagnostic.contains("[redacted]"));
    assert!(!diagnostic.contains("event-hub-password"));

    let route_port = adapter.into_route_port();
    assert_eq!(route_port.routes.len(), 2);
    assert!(route_port.routes.iter().all(|route| {
        route.registration_id == "events_authority"
            && route.runtime_instance_id == "events_authority_runtime"
            && route.caller_runtime_generation == 3
            && route.grant_epoch == 7
            && !route.ciphertext.windows(4).any(|value| value == b"nats")
    }));
}

#[tokio::test]
async fn event_hub_lease_revoke_stays_on_the_authority_fenced_vault_audience() {
    let mut adapter = EventHubCredentialLeaseAdapterV1::new(
        ScriptedVaultRoute::new(vec![Ok(vec![1])]),
        context(),
    );
    adapter
        .revoke_event_hub_leases(&event_hub_fence())
        .await
        .expect("revoke Event Hub Vault audience");

    let route_port = adapter.into_route_port();
    assert_eq!(route_port.routes.len(), 1);
    assert_eq!(route_port.routes[0].registration_id, "events_authority");
    assert_eq!(
        route_port.routes[0].runtime_instance_id,
        "events_authority_runtime"
    );
    assert_eq!(route_port.routes[0].grant_epoch, 7);
}

fn context() -> NatsVaultRouteContextV1 {
    NatsVaultRouteContextV1::new("vault_instance".to_owned(), 7, vault_public_key())
        .expect("Vault context")
}

fn fence() -> NatsRuntimeCredentialFenceV1 {
    NatsRuntimeCredentialFenceV1::new("notes", "registration_notes", "notes_runtime", 2, 5, 3)
        .expect("runtime fence")
}

fn event_hub_fence() -> EventHubCredentialFenceV1 {
    EventHubCredentialFenceV1::new(
        "events_authority",
        "events_authority_runtime",
        "event_hub_main",
        3,
        7,
        1,
        "event_hub",
    )
    .expect("Event Hub fence")
}

fn vault_public_key() -> [u8; 32] {
    makosh_vault_protocol::VaultResponseRecipientV1::generate()
        .public_key()
        .as_bytes()
        .to_owned()
}
