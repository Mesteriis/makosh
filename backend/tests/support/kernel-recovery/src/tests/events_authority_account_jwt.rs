//! Owner-authorized Account JWT relay conformance.

use std::sync::Mutex;

use makosh_runtime_protocol::v1::{
    ApplyEventsAccountJwtUpdateResponseV1, EventsAuthorityRuntimeControlRequestV1,
    EventsAuthorityRuntimeControlResponseV1,
    events_authority_runtime_control_request_v1::Operation,
    events_authority_runtime_control_response_v1::Result as ResponseResult,
};
use prost::Message;

use crate::platform::events::authority::{account_jwt, binding::EVENTS_AUTHORITY_PROCESS_ID};
use crate::runtime::lifecycle::supervisor::ManagedRuntimeRelay;

struct Relay {
    payload: Mutex<Option<Vec<u8>>>,
}

impl ManagedRuntimeRelay for Relay {
    fn relay(&self, registration_id: &str, payload: Vec<u8>) -> Result<Vec<u8>, String> {
        assert_eq!(registration_id, EVENTS_AUTHORITY_PROCESS_ID);
        *self.payload.lock().expect("relay state") = Some(payload);
        Ok(EventsAuthorityRuntimeControlResponseV1 {
            result: Some(ResponseResult::AccountJwtUpdated(
                ApplyEventsAccountJwtUpdateResponseV1 {
                    resolver_credential_revision: 2,
                },
            )),
            error_code: String::new(),
        }
        .encode_to_vec())
    }
}

#[test]
fn relays_only_a_valid_signed_account_jwt_to_the_events_authority() {
    let relay = Relay {
        payload: Mutex::new(None),
    };
    let jwt = signed_account_jwt();
    assert_eq!(account_jwt::apply(&relay, 2, jwt.clone()), Ok(2));
    let payload = relay
        .payload
        .lock()
        .expect("relay state")
        .take()
        .expect("payload");
    let request = EventsAuthorityRuntimeControlRequestV1::decode(payload.as_slice())
        .expect("authority request");
    assert!(
        matches!(request.operation, Some(Operation::ApplyAccountJwtUpdate(value))
        if value.resolver_credential_revision == 2 && value.signed_account_jwt == jwt)
    );
}

fn signed_account_jwt() -> String {
    let operator = nats_jwt::KeyPair::new_operator();
    let account = nats_jwt::KeyPair::new_account();
    nats_jwt::Token::new_account(account.public_key()).sign(&operator)
}
