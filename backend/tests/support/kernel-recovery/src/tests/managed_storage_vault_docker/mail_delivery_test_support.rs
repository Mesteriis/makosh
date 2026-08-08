//! Shared managed Mail delivery routing and terminal-status assertions.

use std::time::{Duration, Instant};

use makosh_mail_api::{
    MailClientRequestV1, MailClientResponseV1, MailDeliveryOutcomeV1, MailDeliveryStatusRequestV1,
    client_contract::MailClientContractV1,
};
use makosh_mail_runtime::client_port::{
    MailClientPortErrorV1, decode_module_response, encode_module_request,
};

use crate::modules::capability::router::{
    ManagedCapabilityRouteRequest, route_managed_client_request,
};

use super::*;

pub(super) const MAIL_DELIVERY_OBSERVATION_SUBJECT: &str =
    "makosh.observation.v1.communications.communication_observed.v1";
pub(super) const MAIL_DELIVERY_CANONICAL_EVENT_SUBJECT: &str =
    "makosh.event.v1.communications.communication_evidence_recorded.v1";

pub(super) fn route_mail_client(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
    contract: MailClientContractV1,
    request_id: u64,
    request: &MailClientRequestV1,
) -> MailClientResponseV1 {
    let relay = supervisor.relay_port();
    let deadline = Instant::now() + Duration::from_secs(10);
    while relay.is_ready(&mail.registration_id) != Ok(true) {
        assert!(
            Instant::now() < deadline,
            "managed Mail runtime did not become ready: {:?}",
            supervisor.last_failure(&mail.registration_id)
        );
        std::thread::sleep(Duration::from_millis(25));
    }
    loop {
        let last_error =
            match route_mail_client_once(store, supervisor, mail, contract, request_id, request) {
                Ok(response) => return response,
                Err(error) => error,
            };
        assert!(
            Instant::now() < deadline,
            "Mail client route remained unavailable: {last_error}; managed failure: {:?}",
            supervisor.last_failure(&mail.registration_id)
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

pub(super) fn route_mail_client_once(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
    contract: MailClientContractV1,
    request_id: u64,
    request: &MailClientRequestV1,
) -> Result<MailClientResponseV1, String> {
    let encoded =
        encode_module_request(request_id, request).map_err(|_| "encode Mail request".to_owned())?;
    let route = ManagedCapabilityRouteRequest::new(
        &mail.registration_id,
        &mail.runtime_instance_id,
        mail.runtime_generation,
        mail.grant_epoch,
        contract.capability_id(),
        &encoded,
    );
    let bytes = route_managed_client_request(store, &supervisor.relay_port(), &route)?;
    match decode_module_response(contract, &bytes) {
        Ok((response_id, response)) if response_id == request_id => Ok(response),
        Ok((response_id, _)) => Err(format!("unexpected response id {response_id}")),
        Err(MailClientPortErrorV1::Protocol) => Err("invalid Mail route response".to_owned()),
        Err(MailClientPortErrorV1::Runtime) => Err("Mail route runtime error".to_owned()),
    }
}

pub(super) fn assert_delivery_completed(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
    operation_id: &str,
    expected_response_code: u16,
) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let response = route_mail_client(
            store,
            supervisor,
            mail,
            MailClientContractV1::DeliveryQuery,
            72,
            &MailClientRequestV1::DeliveryStatus(MailDeliveryStatusRequestV1 {
                operation_id: operation_id.to_owned(),
                connection_id: MAIL_ACCOUNT_ID.to_owned(),
            }),
        );
        let MailClientResponseV1::DeliveryStatus(Some(status)) = response else {
            panic!("Mail delivery status query returned no operation");
        };
        assert_eq!(status.operation_id, operation_id);
        assert_eq!(status.connection_id, MAIL_ACCOUNT_ID);
        match status.outcome {
            MailDeliveryOutcomeV1::Accepted => {
                assert_eq!(status.response_code, Some(expected_response_code));
                assert!(status.completed_at_unix_seconds.is_some());
                return;
            }
            MailDeliveryOutcomeV1::Rejected | MailDeliveryOutcomeV1::OutcomeUnknown => {
                panic!("Mail provider delivery did not complete successfully")
            }
            MailDeliveryOutcomeV1::Pending => {}
        }
        assert!(
            Instant::now() < deadline,
            "Mail provider delivery did not reach a terminal result"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

pub(super) fn assert_delivery_outcome_unknown(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
    operation_id: &str,
) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let response = route_mail_client(
            store,
            supervisor,
            mail,
            MailClientContractV1::DeliveryQuery,
            73,
            &MailClientRequestV1::DeliveryStatus(MailDeliveryStatusRequestV1 {
                operation_id: operation_id.to_owned(),
                connection_id: MAIL_ACCOUNT_ID.to_owned(),
            }),
        );
        let MailClientResponseV1::DeliveryStatus(Some(status)) = response else {
            panic!("Mail delivery status query returned no operation");
        };
        assert_eq!(status.operation_id, operation_id);
        assert_eq!(status.connection_id, MAIL_ACCOUNT_ID);
        match status.outcome {
            MailDeliveryOutcomeV1::OutcomeUnknown => {
                assert_eq!(status.response_code, None);
                assert!(status.completed_at_unix_seconds.is_some());
                return;
            }
            MailDeliveryOutcomeV1::Accepted | MailDeliveryOutcomeV1::Rejected => {
                panic!("ambiguous Mail provider delivery did not remain outcome_unknown")
            }
            MailDeliveryOutcomeV1::Pending => {}
        }
        assert!(
            Instant::now() < deadline,
            "Mail provider ambiguity did not reach a terminal result"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

pub(super) fn wait_for_mail_ready(
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while supervisor.relay_port().is_ready(&mail.registration_id) != Ok(true) {
        assert!(
            Instant::now() < deadline,
            "managed Mail runtime did not become ready: {:?}",
            supervisor.last_failure(&mail.registration_id)
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}
