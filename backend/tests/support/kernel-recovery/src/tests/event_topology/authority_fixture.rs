use makosh_runtime_protocol::v1::{
    EventsAuthorityRuntimeControlRequestV1, EventsAuthorityRuntimeControlResponseV1,
    EventsRuntimeCredentialDeliveryV1,
    events_authority_runtime_control_request_v1::Operation as AuthorityOperation,
    events_authority_runtime_control_response_v1::Result as AuthorityResult,
};
use prost::Message;

use crate::runtime::lifecycle::supervisor::ManagedRuntimeRelay;

pub(super) struct CapturingAuthorityRelay;

impl ManagedRuntimeRelay for CapturingAuthorityRelay {
    fn relay(&self, registration_id: &str, payload: Vec<u8>) -> Result<Vec<u8>, String> {
        assert_eq!(registration_id, "events_authority");
        let request = EventsAuthorityRuntimeControlRequestV1::decode(payload.as_slice())
            .expect("authority request");
        assert!(
            matches!(request.operation, Some(AuthorityOperation::IssueRuntimeCredential(value))
            if value.registration_id == "registration_notes"
                && value.runtime_instance_id == "runtime_1"
                && value.runtime_generation == 3
                && value.grant_epoch == 2
                && value.publish_subjects == ["makosh.event.v1.owner_notes.changed.v1"])
        );
        Ok(EventsAuthorityRuntimeControlResponseV1 {
            result: Some(AuthorityResult::CredentialDelivery(
                EventsRuntimeCredentialDeliveryV1 {
                    encapped_key: vec![1; 32],
                    ciphertext: vec![2; 32],
                    tag: vec![3; 16],
                },
            )),
            error_code: String::new(),
        }
        .encode_to_vec())
    }
}
