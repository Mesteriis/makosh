use makosh_organizations_api::{
    ORGANIZATIONS_MODULE_ID_V1, ORGANIZATIONS_OWNER_ID_V1, OrganizationsEnvelopeContextV1,
    build_organization_changed_outbox_record_v1,
    client_wire::{
        AddOrganizationSourceRequestV1, CreateOrganizationRequestV1, GetOrganizationRequestV1,
        ListOrganizationSourcesRequestV1, ListOrganizationSourcesResultV1,
        ListOrganizationsRequestV1, ListOrganizationsResultV1, OrganizationChangedV1,
        OrganizationMutationResultV1, OrganizationSourceStateV1 as WireSourceState,
        OrganizationSourceV1 as WireSource, OrganizationStateV1 as WireState,
        OrganizationV1 as WireOrganization, RemoveOrganizationSourceRequestV1,
        SearchOrganizationsRequestV1, SetOrganizationStateRequestV1, TimestampV1,
        UpdateOrganizationRequestV1,
    },
    organizations_client_add_source_contract_reference_v1,
    organizations_client_create_contract_reference_v1,
    organizations_client_get_contract_reference_v1,
    organizations_client_list_contract_reference_v1,
    organizations_client_list_sources_contract_reference_v1,
    organizations_client_remove_source_contract_reference_v1,
    organizations_client_search_contract_reference_v1,
    organizations_client_set_state_contract_reference_v1,
    organizations_client_update_contract_reference_v1,
};
use makosh_organizations_core::{
    OrganizationDraftV1, OrganizationRecordV1, OrganizationSourceStateV1, OrganizationSourceV1,
    OrganizationStateV1, OrganizationTimestampV1,
};
use makosh_organizations_persistence::{
    OrganizationLifecycleCommitV1, OrganizationLifecycleMutationV1,
    OrganizationLifecycleOperationOutcomeV1, OrganizationLifecycleOperationV1,
    OrganizationOutboxRecordV1, OrganizationsPersistenceErrorV1, OrganizationsPersistenceV1,
};
use makosh_runtime_protocol::v1::{ModuleClientRequestV1, ModuleClientResponseV1};
use prost::Message;
use sha2::{Digest, Sha256};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OrganizationsClientRuntimeContextV1 {
    pub runtime_instance_id: [u8; 16],
    pub runtime_generation: u64,
    pub now_unix_millis: i64,
}

pub async fn dispatch_organizations_client_request_v1(
    persistence: &OrganizationsPersistenceV1,
    logical_owner_id: &str,
    request: ModuleClientRequestV1,
    context: OrganizationsClientRuntimeContextV1,
) -> ModuleClientResponseV1 {
    let accepted = request.protocol_major == 1
        && request.module_id == ORGANIZATIONS_MODULE_ID_V1
        && request.owner_id == ORGANIZATIONS_OWNER_ID_V1
        && request.logical_owner_id == logical_owner_id
        && !request.authenticated_device_id.is_empty()
        && context.runtime_instance_id.iter().any(|byte| *byte != 0)
        && context.runtime_generation > 0
        && context.now_unix_millis > 0;
    let response = if accepted {
        dispatch(persistence, logical_owner_id, &request, context).await
    } else {
        Err("REJECTED")
    };
    match response {
        Ok(response_payload) => ModuleClientResponseV1 {
            protocol_major: 1,
            request_id: request.request_id,
            response_payload,
            error_code: String::new(),
        },
        Err(error_code) => ModuleClientResponseV1 {
            protocol_major: 1,
            request_id: request.request_id,
            response_payload: Vec::new(),
            error_code: error_code.to_owned(),
        },
    }
}

async fn dispatch(
    persistence: &OrganizationsPersistenceV1,
    logical_owner_id: &str,
    request: &ModuleClientRequestV1,
    context: OrganizationsClientRuntimeContextV1,
) -> Result<Vec<u8>, &'static str> {
    let contract = request.contract.as_ref().ok_or("REJECTED")?;
    if contract == &organizations_client_get_contract_reference_v1() {
        return get(persistence, logical_owner_id, &request.request_payload).await;
    }
    if contract == &organizations_client_list_contract_reference_v1() {
        return list(persistence, logical_owner_id, &request.request_payload).await;
    }
    if contract == &organizations_client_search_contract_reference_v1() {
        return search(persistence, logical_owner_id, &request.request_payload).await;
    }
    if contract == &organizations_client_list_sources_contract_reference_v1() {
        return list_sources(persistence, logical_owner_id, &request.request_payload).await;
    }

    let operation_id = decode_operation_id(contract, &request.request_payload)?;
    let request_sha256: [u8; 32] = Sha256::digest(&request.request_payload).into();
    if let Some(response) = persistence
        .load_operation_replay(
            logical_owner_id,
            operation_id,
            request_sha256,
            &request.request_payload,
        )
        .await
        .map_err(persistence_error)?
    {
        return Ok(response);
    }
    let mutation = decode_mutation(
        contract,
        logical_owner_id,
        &request.request_payload,
        context.now_unix_millis,
    )?;
    let operation = OrganizationLifecycleOperationV1 {
        logical_owner_id: logical_owner_id.to_owned(),
        operation_id,
        request_sha256,
        request_bytes: request.request_payload.clone(),
        received_at_unix_millis: context.now_unix_millis,
        mutation,
    };
    let envelope_context = OrganizationsEnvelopeContextV1 {
        module_id: ORGANIZATIONS_MODULE_ID_V1.to_owned(),
        runtime_instance_id: encode_id(context.runtime_instance_id),
        runtime_generation: context.runtime_generation,
        recorded_at_unix_seconds: context.now_unix_millis / 1_000,
        recorded_at_nanos: ((context.now_unix_millis % 1_000) * 1_000_000) as i32,
    };
    let outcome = persistence
        .apply_lifecycle_operation(operation, |organization| {
            build_commit(operation_id, organization, &envelope_context)
        })
        .await
        .map_err(persistence_error)?;
    Ok(match outcome {
        OrganizationLifecycleOperationOutcomeV1::Applied { response_bytes, .. }
        | OrganizationLifecycleOperationOutcomeV1::Replayed { response_bytes } => response_bytes,
    })
}

fn build_commit(
    operation_id: [u8; 16],
    organization: &OrganizationRecordV1,
    context: &OrganizationsEnvelopeContextV1,
) -> Result<OrganizationLifecycleCommitV1, OrganizationsPersistenceErrorV1> {
    let response = OrganizationMutationResultV1 {
        operation_id: operation_id.to_vec(),
        organization: Some(wire_organization(organization)),
    }
    .encode_to_vec();
    let changed = build_organization_changed_outbox_record_v1(
        operation_id,
        OrganizationChangedV1 {
            event_id: lifecycle_event_id(operation_id, organization).to_vec(),
            organization_id: organization.organization_id.to_vec(),
            logical_owner_id: organization.logical_owner_id.clone(),
            organization_revision: organization.organization_revision,
            state: encode_state(organization.state),
            occurred_at: Some(wire_timestamp(organization.updated_at)),
        },
        context,
    )
    .map_err(|_| OrganizationsPersistenceErrorV1::InvalidInput)?;
    Ok(OrganizationLifecycleCommitV1 {
        response_sha256: Sha256::digest(&response).into(),
        response_bytes: response,
        lifecycle_event: OrganizationOutboxRecordV1 {
            message_id: *changed.message_id(),
            envelope_sha256: *changed.envelope_sha256(),
            envelope_bytes: changed.exact_bytes().to_vec(),
        },
    })
}

fn decode_operation_id(
    contract: &makosh_runtime_protocol::v1::ContractReferenceV1,
    bytes: &[u8],
) -> Result<[u8; 16], &'static str> {
    macro_rules! operation_id {
        ($reference:expr, $type:ty) => {
            if contract == &$reference {
                return id16(&exact_decode::<$type>(bytes)?.operation_id);
            }
        };
    }
    operation_id!(
        organizations_client_create_contract_reference_v1(),
        CreateOrganizationRequestV1
    );
    operation_id!(
        organizations_client_update_contract_reference_v1(),
        UpdateOrganizationRequestV1
    );
    operation_id!(
        organizations_client_set_state_contract_reference_v1(),
        SetOrganizationStateRequestV1
    );
    operation_id!(
        organizations_client_add_source_contract_reference_v1(),
        AddOrganizationSourceRequestV1
    );
    operation_id!(
        organizations_client_remove_source_contract_reference_v1(),
        RemoveOrganizationSourceRequestV1
    );
    Err("REJECTED")
}

fn decode_mutation(
    contract: &makosh_runtime_protocol::v1::ContractReferenceV1,
    logical_owner_id: &str,
    bytes: &[u8],
    now_unix_millis: i64,
) -> Result<OrganizationLifecycleMutationV1, &'static str> {
    if contract == &organizations_client_create_contract_reference_v1() {
        let mut value = exact_decode::<CreateOrganizationRequestV1>(bytes)?;
        accept_owner(&mut value.logical_owner_id, logical_owner_id)?;
        Ok(OrganizationLifecycleMutationV1::Create(
            OrganizationDraftV1 {
                operation_id: id16(&value.operation_id)?,
                logical_owner_id: logical_owner_id.to_owned(),
                display_name: value.display_name,
                legal_name: value.legal_name,
                description: value.description,
                website: value.website,
                industry: value.industry,
                country_code: value.country_code,
                created_at: checked_timestamp(value.created_at, now_unix_millis)?,
            },
        ))
    } else if contract == &organizations_client_update_contract_reference_v1() {
        let mut value = exact_decode::<UpdateOrganizationRequestV1>(bytes)?;
        accept_owner(&mut value.logical_owner_id, logical_owner_id)?;
        Ok(OrganizationLifecycleMutationV1::Update {
            operation_id: id16(&value.operation_id)?,
            organization_id: id16(&value.organization_id)?,
            expected_revision: positive_revision(value.expected_organization_revision)?,
            display_name: value.display_name,
            legal_name: value.legal_name,
            description: value.description,
            website: value.website,
            industry: value.industry,
            country_code: value.country_code,
            changed_at: checked_timestamp(value.updated_at, now_unix_millis)?,
        })
    } else if contract == &organizations_client_set_state_contract_reference_v1() {
        let mut value = exact_decode::<SetOrganizationStateRequestV1>(bytes)?;
        accept_owner(&mut value.logical_owner_id, logical_owner_id)?;
        Ok(OrganizationLifecycleMutationV1::SetState {
            operation_id: id16(&value.operation_id)?,
            organization_id: id16(&value.organization_id)?,
            expected_revision: positive_revision(value.expected_organization_revision)?,
            state: decode_state(value.state)?,
            changed_at: checked_timestamp(value.changed_at, now_unix_millis)?,
        })
    } else if contract == &organizations_client_add_source_contract_reference_v1() {
        let mut value = exact_decode::<AddOrganizationSourceRequestV1>(bytes)?;
        accept_owner(&mut value.logical_owner_id, logical_owner_id)?;
        Ok(OrganizationLifecycleMutationV1::AddSource {
            operation_id: id16(&value.operation_id)?,
            organization_id: id16(&value.organization_id)?,
            expected_revision: positive_revision(value.expected_organization_revision)?,
            source_owner_id: value.source_owner_id,
            source_record_id: value.source_record_id,
            source_revision: positive_revision(value.source_revision)?,
            evidence_digest: id32(&value.evidence_digest)?,
            changed_at: checked_timestamp(value.changed_at, now_unix_millis)?,
        })
    } else if contract == &organizations_client_remove_source_contract_reference_v1() {
        let mut value = exact_decode::<RemoveOrganizationSourceRequestV1>(bytes)?;
        accept_owner(&mut value.logical_owner_id, logical_owner_id)?;
        Ok(OrganizationLifecycleMutationV1::RemoveSource {
            operation_id: id16(&value.operation_id)?,
            organization_id: id16(&value.organization_id)?,
            expected_revision: positive_revision(value.expected_organization_revision)?,
            source_id: id16(&value.source_id)?,
            changed_at: checked_timestamp(value.changed_at, now_unix_millis)?,
        })
    } else {
        Err("REJECTED")
    }
}

async fn get(
    persistence: &OrganizationsPersistenceV1,
    logical_owner_id: &str,
    bytes: &[u8],
) -> Result<Vec<u8>, &'static str> {
    let mut value = exact_decode::<GetOrganizationRequestV1>(bytes)?;
    accept_owner(&mut value.logical_owner_id, logical_owner_id)?;
    persistence
        .get_organization(logical_owner_id, id16(&value.organization_id)?)
        .await
        .map_err(persistence_error)?
        .map(|organization| wire_organization(&organization).encode_to_vec())
        .ok_or("NOT_FOUND")
}

async fn list(
    persistence: &OrganizationsPersistenceV1,
    logical_owner_id: &str,
    bytes: &[u8],
) -> Result<Vec<u8>, &'static str> {
    let mut value = exact_decode::<ListOrganizationsRequestV1>(bytes)?;
    accept_owner(&mut value.logical_owner_id, logical_owner_id)?;
    let limit = checked_limit(value.limit)?;
    let mut organizations = persistence
        .list_organizations(
            logical_owner_id,
            optional_id16(&value.after_organization_id)?,
            limit + 1,
        )
        .await
        .map_err(persistence_error)?;
    Ok(paginate_organizations(&mut organizations, limit).encode_to_vec())
}

async fn search(
    persistence: &OrganizationsPersistenceV1,
    logical_owner_id: &str,
    bytes: &[u8],
) -> Result<Vec<u8>, &'static str> {
    let mut value = exact_decode::<SearchOrganizationsRequestV1>(bytes)?;
    accept_owner(&mut value.logical_owner_id, logical_owner_id)?;
    let limit = checked_limit(value.limit)?;
    let mut organizations = persistence
        .search_organizations(
            logical_owner_id,
            &value.query,
            optional_id16(&value.after_organization_id)?,
            limit + 1,
        )
        .await
        .map_err(persistence_error)?;
    Ok(paginate_organizations(&mut organizations, limit).encode_to_vec())
}

async fn list_sources(
    persistence: &OrganizationsPersistenceV1,
    logical_owner_id: &str,
    bytes: &[u8],
) -> Result<Vec<u8>, &'static str> {
    let mut value = exact_decode::<ListOrganizationSourcesRequestV1>(bytes)?;
    accept_owner(&mut value.logical_owner_id, logical_owner_id)?;
    let limit = usize::from(checked_limit(value.limit)?);
    let after = optional_id16(&value.after_source_id)?;
    let organization = persistence
        .get_organization(logical_owner_id, id16(&value.organization_id)?)
        .await
        .map_err(persistence_error)?
        .ok_or("NOT_FOUND")?;
    let mut sources = organization
        .sources
        .into_iter()
        .filter(|source| after.is_none_or(|after| source.source_id > after))
        .collect::<Vec<_>>();
    let has_more = sources.len() > limit;
    sources.truncate(limit);
    let next = if has_more {
        sources
            .last()
            .expect("nonempty bounded page")
            .source_id
            .to_vec()
    } else {
        Vec::new()
    };
    Ok(ListOrganizationSourcesResultV1 {
        sources: sources.iter().map(wire_source).collect(),
        next_after_source_id: next,
    }
    .encode_to_vec())
}

fn paginate_organizations(
    organizations: &mut Vec<OrganizationRecordV1>,
    limit: u16,
) -> ListOrganizationsResultV1 {
    let has_more = organizations.len() > usize::from(limit);
    organizations.truncate(usize::from(limit));
    ListOrganizationsResultV1 {
        organizations: organizations.iter().map(wire_organization).collect(),
        next_after_organization_id: if has_more {
            organizations
                .last()
                .map_or_else(Vec::new, |value| value.organization_id.to_vec())
        } else {
            Vec::new()
        },
    }
}

fn wire_organization(value: &OrganizationRecordV1) -> WireOrganization {
    WireOrganization {
        organization_id: value.organization_id.to_vec(),
        logical_owner_id: value.logical_owner_id.clone(),
        display_name: value.display_name.clone(),
        legal_name: value.legal_name.clone(),
        description: value.description.clone(),
        website: value.website.clone(),
        industry: value.industry.clone(),
        country_code: value.country_code.clone(),
        state: encode_state(value.state),
        organization_revision: value.organization_revision,
        created_at: Some(wire_timestamp(value.created_at)),
        updated_at: Some(wire_timestamp(value.updated_at)),
    }
}

fn wire_source(value: &OrganizationSourceV1) -> WireSource {
    WireSource {
        source_id: value.source_id.to_vec(),
        source_owner_id: value.source_owner_id.clone(),
        source_record_id: value.source_record_id.clone(),
        source_revision: value.source_revision,
        evidence_digest: value.evidence_digest.to_vec(),
        state: match value.state {
            OrganizationSourceStateV1::Active => {
                WireSourceState::OrganizationSourceStateActive as i32
            }
            OrganizationSourceStateV1::Removed => {
                WireSourceState::OrganizationSourceStateRemoved as i32
            }
        },
        updated_at_organization_revision: value.updated_at_organization_revision,
    }
}

fn exact_decode<T: Message + Default>(bytes: &[u8]) -> Result<T, &'static str> {
    let value = T::decode(bytes).map_err(|_| "INVALID_ARGUMENT")?;
    (value.encode_to_vec() == bytes)
        .then_some(value)
        .ok_or("INVALID_ARGUMENT")
}

fn checked_timestamp(
    value: Option<TimestampV1>,
    now: i64,
) -> Result<OrganizationTimestampV1, &'static str> {
    let value = value.ok_or("INVALID_ARGUMENT")?;
    let millis = value
        .unix_seconds
        .checked_mul(1_000)
        .and_then(|base| base.checked_add(i64::from(value.nanos / 1_000_000)))
        .filter(|millis| *millis > 0 && *millis <= now)
        .ok_or("INVALID_ARGUMENT")?;
    let _ = millis;
    if !(0..1_000_000_000).contains(&value.nanos) {
        return Err("INVALID_ARGUMENT");
    }
    Ok(OrganizationTimestampV1 {
        unix_seconds: value.unix_seconds,
        nanos: value.nanos,
    })
}

fn wire_timestamp(value: OrganizationTimestampV1) -> TimestampV1 {
    TimestampV1 {
        unix_seconds: value.unix_seconds,
        nanos: value.nanos,
    }
}

fn accept_owner(payload: &mut String, authenticated: &str) -> Result<(), &'static str> {
    if payload.is_empty() {
        *payload = authenticated.to_owned();
    } else if payload != authenticated {
        return Err("REJECTED");
    }
    Ok(())
}

fn id16(value: &[u8]) -> Result<[u8; 16], &'static str> {
    fixed_id(value)
}

fn id32(value: &[u8]) -> Result<[u8; 32], &'static str> {
    fixed_id(value)
}

fn fixed_id<const N: usize>(value: &[u8]) -> Result<[u8; N], &'static str> {
    let value: [u8; N] = value.try_into().map_err(|_| "INVALID_ARGUMENT")?;
    value
        .iter()
        .any(|byte| *byte != 0)
        .then_some(value)
        .ok_or("INVALID_ARGUMENT")
}

fn optional_id16(value: &[u8]) -> Result<Option<[u8; 16]>, &'static str> {
    if value.is_empty() {
        Ok(None)
    } else {
        id16(value).map(Some)
    }
}

fn positive_revision(value: u64) -> Result<u64, &'static str> {
    (value > 0).then_some(value).ok_or("INVALID_ARGUMENT")
}

fn checked_limit(value: u32) -> Result<u16, &'static str> {
    match value {
        1..=200 => u16::try_from(value).map_err(|_| "INVALID_ARGUMENT"),
        _ => Err("INVALID_ARGUMENT"),
    }
}

fn decode_state(value: i32) -> Result<OrganizationStateV1, &'static str> {
    match WireState::try_from(value).map_err(|_| "INVALID_ARGUMENT")? {
        WireState::OrganizationStateActive => Ok(OrganizationStateV1::Active),
        WireState::OrganizationStateArchived => Ok(OrganizationStateV1::Archived),
        WireState::OrganizationStateUnspecified => Err("INVALID_ARGUMENT"),
    }
}

fn encode_state(value: OrganizationStateV1) -> i32 {
    match value {
        OrganizationStateV1::Active => WireState::OrganizationStateActive as i32,
        OrganizationStateV1::Archived => WireState::OrganizationStateArchived as i32,
    }
}

fn lifecycle_event_id(operation_id: [u8; 16], value: &OrganizationRecordV1) -> [u8; 16] {
    let mut hash = Sha256::new();
    hash.update(b"makosh.organizations.lifecycle-event-id.v1\0");
    hash.update(operation_id);
    hash.update(value.organization_id);
    hash.update(value.organization_revision.to_be_bytes());
    hash.finalize()[..16].try_into().expect("fixed digest")
}

fn encode_id(value: [u8; 16]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(32);
    for byte in value {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

fn persistence_error(value: OrganizationsPersistenceErrorV1) -> &'static str {
    match value {
        OrganizationsPersistenceErrorV1::NotFound => "NOT_FOUND",
        OrganizationsPersistenceErrorV1::RevisionConflict => "REVISION_CONFLICT",
        OrganizationsPersistenceErrorV1::OperationConflict
        | OrganizationsPersistenceErrorV1::OutboxConflict => "CONFLICT",
        OrganizationsPersistenceErrorV1::InvalidInput
        | OrganizationsPersistenceErrorV1::InvalidRow => "INVALID_ARGUMENT",
        OrganizationsPersistenceErrorV1::StorageUnavailable => "UNAVAILABLE",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn organization(id: u8) -> OrganizationRecordV1 {
        OrganizationRecordV1 {
            organization_id: [id; 16],
            logical_owner_id: "owner-1".to_owned(),
            display_name: format!("Organization {id}"),
            legal_name: String::new(),
            description: String::new(),
            website: String::new(),
            industry: String::new(),
            country_code: String::new(),
            state: OrganizationStateV1::Active,
            organization_revision: 1,
            sources: Vec::new(),
            created_at: OrganizationTimestampV1 {
                unix_seconds: 1,
                nanos: 0,
            },
            updated_at: OrganizationTimestampV1 {
                unix_seconds: 1,
                nanos: 0,
            },
        }
    }

    #[test]
    fn owner_and_cursor_are_exact_without_skipping_overflow() {
        let mut owner = String::new();
        accept_owner(&mut owner, "owner-1").expect("outer owner");
        assert_eq!(owner, "owner-1");
        assert!(accept_owner(&mut "owner-2".to_owned(), "owner-1").is_err());
        let mut values = vec![organization(1), organization(2), organization(3)];
        let first = paginate_organizations(&mut values, 2);
        assert_eq!(first.organizations.len(), 2);
        assert_eq!(first.next_after_organization_id, vec![2; 16]);
    }
}
