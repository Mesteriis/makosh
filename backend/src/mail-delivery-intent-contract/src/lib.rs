#![forbid(unsafe_code)]

use makosh_runtime_protocol::v1::{
    CapabilityRequestV1, ContractReferenceV1, DurableEnvelopeKindV1, EventRouteDirectionV1,
    EventRouteRequestV1, EventSubscriptionRequirementV1, capability_request_v1::Request,
};

pub const PACKAGE: &str = "makosh-mail-delivery-intent-contract";
pub const MAIL_DELIVERY_INTENT_OWNER_ID_V1: &str = "mail";
pub const MAIL_DELIVERY_INTENT_SOURCE_MODULE_ID_V1: &str =
    "makosh-communication-delivery-intent-runtime";
pub const MAIL_DELIVERY_INTENT_TARGET_MODULE_ID_V1: &str = "makosh-mail-runtime";
pub const MAIL_DELIVERY_INTENT_TARGET_CAPABILITY_ID_V1: &str = "mail.delivery-intent.v1";
pub const MAIL_DELIVERY_INTENT_TARGET_BLOB_CAPABILITY_ID_V1: &str = "mail.blob.v1";
pub const MAIL_DELIVERY_INTENT_CUSTODY_SCOPE_ID_V1: &str = "mail.delivery-intent-body.v1";
pub const MAIL_DELIVERY_INTENT_EXECUTE_CONTRACT_NAME_V1: &str = "mail_delivery_intent_execute";
pub const MAIL_DELIVERY_INTENT_SUCCEEDED_CONTRACT_NAME_V1: &str = "mail_delivery_intent_succeeded";
pub const MAIL_DELIVERY_INTENT_REJECTED_CONTRACT_NAME_V1: &str = "mail_delivery_intent_rejected";
pub const MAIL_DELIVERY_INTENT_CONTRACT_MAJOR_V1: u32 = 1;
pub const MAIL_DELIVERY_INTENT_CONTRACT_REVISION_V1: u32 = 1;
pub const MAIL_DELIVERY_INTENT_MAX_BODY_BYTES_V1: u64 = 64 * 1024;
pub const MAIL_DELIVERY_INTENT_MAX_SOURCE_PROOF_BYTES_V1: usize = 2_048;
pub const MAIL_DELIVERY_INTENT_MAX_PROVIDER_OPERATION_ID_BYTES_V1: usize = 256;
pub const MAIL_DELIVERY_INTENT_MAX_IN_FLIGHT_V1: u32 = 32;

pub mod wire {
    include!(concat!(
        env!("OUT_DIR"),
        "/makosh.mail.delivery_intent.v1.rs"
    ));
}

include!(concat!(env!("OUT_DIR"), "/mail_delivery_intent_schema.rs"));

pub const MAIL_DELIVERY_INTENT_DESCRIPTOR_SET_V1: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/mail-delivery-intent-v1.bin"));

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailDeliveryIntentValidationErrorV1 {
    InvalidIntentId,
    InvalidLogicalOwner,
    InvalidRouteCursor,
    InvalidBodySource,
    InvalidProviderOperationId,
    InvalidRejectionCode,
}

pub fn validate_mail_delivery_intent_execute_v1(
    command: &wire::ExecuteMailDeliveryIntentCommandV1,
) -> Result<(), MailDeliveryIntentValidationErrorV1> {
    validate_identity(&command.intent_id, &command.logical_owner_id)?;
    if !valid_fixed_id(&command.account_source_cursor, 32)
        || !valid_fixed_id(&command.conversation_source_cursor, 32)
        || command
            .reply_to_source_cursor
            .as_deref()
            .is_some_and(|cursor| !valid_fixed_id(cursor, 32))
    {
        return Err(MailDeliveryIntentValidationErrorV1::InvalidRouteCursor);
    }
    let Some(body) = command.body_source.as_ref() else {
        return Err(MailDeliveryIntentValidationErrorV1::InvalidBodySource);
    };
    if !valid_fixed_id(&body.reference_id, 16)
        || !(1..=MAIL_DELIVERY_INTENT_MAX_BODY_BYTES_V1).contains(&body.declared_bytes)
        || !valid_fixed_id(&body.sha256, 32)
        || body.custody_transfer_source_proof.is_empty()
        || body.custody_transfer_source_proof.len() > MAIL_DELIVERY_INTENT_MAX_SOURCE_PROOF_BYTES_V1
    {
        return Err(MailDeliveryIntentValidationErrorV1::InvalidBodySource);
    }
    Ok(())
}

pub fn validate_mail_delivery_intent_succeeded_v1(
    result: &wire::MailDeliveryIntentSucceededV1,
) -> Result<(), MailDeliveryIntentValidationErrorV1> {
    validate_identity(&result.intent_id, &result.logical_owner_id)?;
    if result.provider_operation_id.is_empty()
        || result.provider_operation_id.len()
            > MAIL_DELIVERY_INTENT_MAX_PROVIDER_OPERATION_ID_BYTES_V1
    {
        return Err(MailDeliveryIntentValidationErrorV1::InvalidProviderOperationId);
    }
    Ok(())
}

pub fn validate_mail_delivery_intent_rejected_v1(
    result: &wire::MailDeliveryIntentRejectedV1,
) -> Result<(), MailDeliveryIntentValidationErrorV1> {
    validate_identity(&result.intent_id, &result.logical_owner_id)?;
    if !(1..=6).contains(&result.code) {
        return Err(MailDeliveryIntentValidationErrorV1::InvalidRejectionCode);
    }
    Ok(())
}

#[must_use]
pub fn mail_delivery_intent_execute_contract_reference_v1() -> ContractReferenceV1 {
    contract_reference(MAIL_DELIVERY_INTENT_EXECUTE_CONTRACT_NAME_V1)
}

#[must_use]
pub fn mail_delivery_intent_succeeded_contract_reference_v1() -> ContractReferenceV1 {
    contract_reference(MAIL_DELIVERY_INTENT_SUCCEEDED_CONTRACT_NAME_V1)
}

#[must_use]
pub fn mail_delivery_intent_rejected_contract_reference_v1() -> ContractReferenceV1 {
    contract_reference(MAIL_DELIVERY_INTENT_REJECTED_CONTRACT_NAME_V1)
}

#[must_use]
pub fn mail_delivery_intent_execute_publish_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Command,
        mail_delivery_intent_execute_contract_reference_v1(),
        EventRouteDirectionV1::Publish,
        EventSubscriptionRequirementV1::Unspecified,
    )
}

#[must_use]
pub fn mail_delivery_intent_execute_consume_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Command,
        mail_delivery_intent_execute_contract_reference_v1(),
        EventRouteDirectionV1::Consume,
        EventSubscriptionRequirementV1::Required,
    )
}

#[must_use]
pub fn mail_delivery_intent_succeeded_publish_request_v1() -> CapabilityRequestV1 {
    result_route(
        mail_delivery_intent_succeeded_contract_reference_v1(),
        EventRouteDirectionV1::Publish,
        EventSubscriptionRequirementV1::Unspecified,
    )
}

#[must_use]
pub fn mail_delivery_intent_succeeded_consume_request_v1() -> CapabilityRequestV1 {
    result_route(
        mail_delivery_intent_succeeded_contract_reference_v1(),
        EventRouteDirectionV1::Consume,
        EventSubscriptionRequirementV1::Required,
    )
}

#[must_use]
pub fn mail_delivery_intent_rejected_publish_request_v1() -> CapabilityRequestV1 {
    result_route(
        mail_delivery_intent_rejected_contract_reference_v1(),
        EventRouteDirectionV1::Publish,
        EventSubscriptionRequirementV1::Unspecified,
    )
}

#[must_use]
pub fn mail_delivery_intent_rejected_consume_request_v1() -> CapabilityRequestV1 {
    result_route(
        mail_delivery_intent_rejected_contract_reference_v1(),
        EventRouteDirectionV1::Consume,
        EventSubscriptionRequirementV1::Required,
    )
}

fn result_route(
    contract: ContractReferenceV1,
    direction: EventRouteDirectionV1,
    requirement: EventSubscriptionRequirementV1,
) -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Result,
        contract,
        direction,
        requirement,
    )
}

fn contract_reference(name: &str) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: MAIL_DELIVERY_INTENT_OWNER_ID_V1.to_owned(),
        name: name.to_owned(),
        major: MAIL_DELIVERY_INTENT_CONTRACT_MAJOR_V1,
        revision: MAIL_DELIVERY_INTENT_CONTRACT_REVISION_V1,
        schema_sha256: MAIL_DELIVERY_INTENT_SCHEMA_SHA256.to_vec(),
    }
}

fn event_route(
    envelope_kind: DurableEnvelopeKindV1,
    contract: ContractReferenceV1,
    direction: EventRouteDirectionV1,
    subscription_requirement: EventSubscriptionRequirementV1,
) -> CapabilityRequestV1 {
    CapabilityRequestV1 {
        request: Some(Request::EventRoute(EventRouteRequestV1 {
            envelope_kind: envelope_kind as i32,
            contract: Some(contract),
            direction: direction as i32,
            max_in_flight: MAIL_DELIVERY_INTENT_MAX_IN_FLIGHT_V1,
            subscription_requirement: subscription_requirement as i32,
            max_deliver: u32::from(direction == EventRouteDirectionV1::Consume) * 10,
            ack_wait_millis: u32::from(direction == EventRouteDirectionV1::Consume) * 30_000,
        })),
    }
}

fn validate_identity(
    intent_id: &[u8],
    logical_owner_id: &str,
) -> Result<(), MailDeliveryIntentValidationErrorV1> {
    if !valid_fixed_id(intent_id, 16) {
        return Err(MailDeliveryIntentValidationErrorV1::InvalidIntentId);
    }
    if logical_owner_id.is_empty() || logical_owner_id.len() > 128 || !logical_owner_id.is_ascii() {
        return Err(MailDeliveryIntentValidationErrorV1::InvalidLogicalOwner);
    }
    Ok(())
}

fn valid_fixed_id(value: &[u8], width: usize) -> bool {
    value.len() == width && value.iter().any(|byte| *byte != 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routes_are_exact_and_directional() {
        let Some(Request::EventRoute(command)) =
            mail_delivery_intent_execute_consume_request_v1().request
        else {
            panic!("command route");
        };
        let Some(Request::EventRoute(result)) =
            mail_delivery_intent_succeeded_publish_request_v1().request
        else {
            panic!("result route");
        };
        assert_eq!(command.envelope_kind, DurableEnvelopeKindV1::Command as i32);
        assert_eq!(result.envelope_kind, DurableEnvelopeKindV1::Result as i32);
        assert_eq!(command.direction, EventRouteDirectionV1::Consume as i32);
        assert_eq!(result.direction, EventRouteDirectionV1::Publish as i32);
        assert_eq!(
            command.subscription_requirement,
            EventSubscriptionRequirementV1::Required as i32
        );
    }

    #[test]
    fn blob_target_is_exact_mail_runtime() {
        assert_eq!(MAIL_DELIVERY_INTENT_OWNER_ID_V1, "mail");
        assert_eq!(
            MAIL_DELIVERY_INTENT_TARGET_MODULE_ID_V1,
            "makosh-mail-runtime"
        );
        assert_eq!(
            MAIL_DELIVERY_INTENT_TARGET_BLOB_CAPABILITY_ID_V1,
            "mail.blob.v1"
        );
        assert_eq!(
            MAIL_DELIVERY_INTENT_CUSTODY_SCOPE_ID_V1,
            "mail.delivery-intent-body.v1"
        );
    }

    #[test]
    fn execute_validation_enforces_fixed_ids_and_bounded_proof() {
        let mut command = wire::ExecuteMailDeliveryIntentCommandV1 {
            intent_id: vec![1; 16],
            logical_owner_id: "owner-1".to_owned(),
            account_source_cursor: vec![2; 32],
            conversation_source_cursor: vec![3; 32],
            reply_to_source_cursor: Some(vec![4; 32]),
            body_source: Some(wire::MailDeliveryIntentBodySourceReceiptV1 {
                reference_id: vec![5; 16],
                declared_bytes: 32,
                sha256: vec![6; 32],
                custody_transfer_source_proof: vec![7; 128],
            }),
        };
        assert_eq!(validate_mail_delivery_intent_execute_v1(&command), Ok(()));
        command.account_source_cursor.pop();
        assert_eq!(
            validate_mail_delivery_intent_execute_v1(&command),
            Err(MailDeliveryIntentValidationErrorV1::InvalidRouteCursor)
        );
    }
}
