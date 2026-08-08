//! Converts one verified managed-runtime request into an opaque authority delivery.

use std::convert::TryInto;
use std::sync::Arc;

use makosh_kernel_control_store::ModuleRegistrationState;
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use makosh_runtime_protocol::v1::{
    ManagedRuntimeEventCredentialDeliveryV1, ManagedRuntimeEventCredentialRequestV1,
};

use crate::platform::events::credential::{
    authority::{EventAuthorityCredentialRelayErrorV1, EventAuthorityCredentialRelayV1},
    permit::{EventCredentialRequestInputV1, derive_credential_request},
};
use crate::platform::events::{catalog, topology};
use crate::runtime::lifecycle::control::{
    ManagedRuntimeEventCredentialHandler, ManagedRuntimeExpectation,
};
use crate::runtime::lifecycle::fence::current_managed_runtime_matches;
use crate::runtime::lifecycle::supervisor::ManagedRuntimeRelay;

/// Keeps approved event rights in the Kernel and returns ciphertext only.
pub(crate) struct EventCredentialHandlerV1<R> {
    store: Arc<SqliteControlStore>,
    authority: EventAuthorityCredentialRelayV1<R>,
}

impl<R> EventCredentialHandlerV1<R>
where
    R: ManagedRuntimeRelay,
{
    pub(crate) fn new(
        store: Arc<SqliteControlStore>,
        authority_registration_id: String,
        relay: R,
    ) -> Result<Self, String> {
        let authority = EventAuthorityCredentialRelayV1::new(authority_registration_id, relay)?;
        Ok(Self { store, authority })
    }
}

impl<R> ManagedRuntimeEventCredentialHandler for EventCredentialHandlerV1<R>
where
    R: ManagedRuntimeRelay,
{
    fn issue_event_credential(
        &self,
        expectation: &ManagedRuntimeExpectation,
        request: ManagedRuntimeEventCredentialRequestV1,
    ) -> Result<ManagedRuntimeEventCredentialDeliveryV1, String> {
        if !current_managed_runtime_matches(
            &*self.store,
            expectation.registration_id(),
            expectation.runtime_instance_id(),
            expectation.runtime_generation(),
            expectation.grant_epoch(),
        )
        .map_err(|_| "managed runtime Events credential request is unavailable".to_owned())?
        {
            return Err("managed runtime Events credential fence is stale".to_owned());
        }
        let registration = current_registration(&self.store, expectation.registration_id())?;
        if expectation.grant_epoch() != registration.grant_epoch() {
            return Err("managed runtime Events credential fence is stale".to_owned());
        }
        let topology = current_topology(&self.store)?;
        let authority_request = derive_credential_request(EventCredentialRequestInputV1 {
            registration: &registration,
            runtime_instance_id: expectation.runtime_instance_id(),
            runtime_generation: expectation.runtime_generation(),
            credential_revision: request.credential_revision,
            request_id: fixed_request_id(&request)?,
            recipient_public_key_x25519: fixed_recipient_key(&request)?,
            ttl_seconds: request.ttl_seconds,
            topology: &topology,
        })
        .map_err(|_| "managed runtime Events credential request is denied".to_owned())?;
        let delivery = self
            .authority
            .issue(authority_request)
            .map_err(|error| match error {
                EventAuthorityCredentialRelayErrorV1::Rejected => {
                    "managed runtime Events credential request is denied".to_owned()
                }
                EventAuthorityCredentialRelayErrorV1::Unavailable => {
                    "managed runtime Events credential authority is unavailable".to_owned()
                }
            })?;
        let consumer_bindings = topology::managed_runtime_consumer_bindings(
            &topology,
            registration.registration_id(),
            registration.grant_epoch(),
        )
        .map_err(|_| "managed runtime Events consumer binding is unavailable".to_owned())?;
        let publish_subjects = topology::managed_runtime_publish_subjects(
            &topology,
            registration.registration_id(),
            registration.grant_epoch(),
        );
        Ok(ManagedRuntimeEventCredentialDeliveryV1 {
            encapped_key: delivery.encapped_key,
            ciphertext: delivery.ciphertext,
            tag: delivery.tag,
            consumer_bindings,
            publish_subjects,
        })
    }
}

fn current_registration(
    store: &SqliteControlStore,
    registration_id: &str,
) -> Result<makosh_kernel_control_store::ModuleRegistration, String> {
    let registration = store
        .module_registration(registration_id)
        .map_err(|_| "managed runtime Events credential registration is unavailable".to_owned())?
        .ok_or_else(|| {
            "managed runtime Events credential registration is unavailable".to_owned()
        })?;
    (registration.state() == ModuleRegistrationState::Approved)
        .then_some(registration)
        .ok_or_else(|| "managed runtime Events credential registration is unavailable".to_owned())
}

fn current_topology(store: &SqliteControlStore) -> Result<topology::EventTopologyPlanV1, String> {
    let contracts = catalog::resolve_contracts(store)
        .map_err(|_| "managed runtime Events credential topology is unavailable".to_owned())?;
    let configuration = store
        .platform_event_hub_topology()
        .map_err(|_| "managed runtime Events credential topology is unavailable".to_owned())?
        .ok_or_else(|| "managed runtime Events credential topology is unavailable".to_owned())?;
    topology::plan(&contracts, &configuration)
        .map_err(|_| "managed runtime Events credential topology is unavailable".to_owned())
}

fn fixed_request_id(request: &ManagedRuntimeEventCredentialRequestV1) -> Result<[u8; 16], String> {
    request
        .request_id
        .as_slice()
        .try_into()
        .map_err(|_| "managed runtime Events credential request is denied".to_owned())
}

fn fixed_recipient_key(
    request: &ManagedRuntimeEventCredentialRequestV1,
) -> Result<[u8; 32], String> {
    request
        .recipient_public_key_x25519
        .as_slice()
        .try_into()
        .map_err(|_| "managed runtime Events credential request is denied".to_owned())
}
