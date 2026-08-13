use makosh_events_protocol::validation::envelope::decode_envelope_v1;
use makosh_persons_api::{
    PERSONS_CLIENT_CREATE_CONTRACT_NAME_V1, PERSONS_CLIENT_GET_PROFILE_CONTRACT_NAME_V1,
    PERSONS_CLIENT_LIST_DIRECTORY_CONTRACT_NAME_V1,
    PERSONS_CLIENT_LIST_SOURCE_LINKS_CONTRACT_NAME_V1,
    PERSONS_CLIENT_UPDATE_PROFILE_CONTRACT_NAME_V1, PERSONS_MODULE_ID_V1, PERSONS_OWNER_ID_V1,
    wire::{
        ManualCreatePersonCommandV1, PersonCommandSucceededV1, PersonDirectoryEntryV1,
        PersonDirectoryResultV1, PersonLifecycleV1 as WireLifecycle, PersonProfileResultV1,
        PersonProfileV1, PersonSourceLinkV1, PersonSourceLinksResultV1, PersonsCommandV1,
        ReadPersonDirectoryRequestV1, ReadPersonProfileRequestV1, ReadPersonSourceLinksRequestV1,
        TimestampV1, UpdatePersonOwnerProfileCommandV1, persons_command_v1::Command,
    },
};
use makosh_persons_core::{PersonIdV1, PersonLifecycleV1, PersonV1};
use makosh_persons_persistence::PersonsPersistenceV1;
use makosh_runtime_protocol::v1::{ModuleClientRequestV1, ModuleClientResponseV1};
use prost::Message;

use crate::{
    PersonsCommandRuntimeContextV1, PersonsEnvelopeContextV1,
    build_persons_command_outbox_record_v1, execute_persons_command_record_v1,
};

pub async fn dispatch_persons_client_request_v1(
    persistence: &PersonsPersistenceV1,
    runtime_instance_id: &str,
    runtime_generation: u64,
    logical_owner_id: &str,
    request: ModuleClientRequestV1,
    now_unix_millis: i64,
) -> ModuleClientResponseV1 {
    let accepted_identity = request.protocol_major == 1
        && request.module_id == PERSONS_MODULE_ID_V1
        && request.owner_id == PERSONS_OWNER_ID_V1
        && request.logical_owner_id == logical_owner_id
        && now_unix_millis > 0;
    let response = if !accepted_identity {
        Err("REJECTED")
    } else {
        dispatch(
            persistence,
            runtime_instance_id,
            runtime_generation,
            logical_owner_id,
            &request,
            now_unix_millis,
        )
        .await
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
    persistence: &PersonsPersistenceV1,
    runtime_instance_id: &str,
    runtime_generation: u64,
    logical_owner_id: &str,
    request: &ModuleClientRequestV1,
    now_unix_millis: i64,
) -> Result<Vec<u8>, &'static str> {
    let contract = request.contract.as_ref().ok_or("REJECTED")?;
    match contract.name.as_str() {
        PERSONS_CLIENT_CREATE_CONTRACT_NAME_V1 | PERSONS_CLIENT_UPDATE_PROFILE_CONTRACT_NAME_V1 => {
            let command = match contract.name.as_str() {
                PERSONS_CLIENT_CREATE_CONTRACT_NAME_V1 => {
                    let mut value =
                        ManualCreatePersonCommandV1::decode(request.request_payload.as_slice())
                            .map_err(|_| "INVALID_ARGUMENT")?;
                    if !value.logical_owner_id.is_empty()
                        && value.logical_owner_id != logical_owner_id
                    {
                        return Err("REJECTED");
                    }
                    value.logical_owner_id = logical_owner_id.to_owned();
                    Command::ManualCreate(value)
                }
                PERSONS_CLIENT_UPDATE_PROFILE_CONTRACT_NAME_V1 => {
                    let mut value = UpdatePersonOwnerProfileCommandV1::decode(
                        request.request_payload.as_slice(),
                    )
                    .map_err(|_| "INVALID_ARGUMENT")?;
                    if !value.logical_owner_id.is_empty()
                        && value.logical_owner_id != logical_owner_id
                    {
                        return Err("REJECTED");
                    }
                    value.logical_owner_id = logical_owner_id.to_owned();
                    Command::OwnerProfileUpdate(value)
                }
                _ => return Err("REJECTED"),
            };
            let payload = PersonsCommandV1 {
                command: Some(command),
            };
            let seconds = now_unix_millis / 1_000;
            let record = build_persons_command_outbox_record_v1(
                payload,
                seconds.checked_add(30).ok_or("INVALID_ARGUMENT")?,
                &PersonsEnvelopeContextV1 {
                    module_id: PERSONS_MODULE_ID_V1.to_owned(),
                    runtime_instance_id: runtime_instance_id.to_owned(),
                    runtime_generation,
                    recorded_at_unix_seconds: seconds,
                    recorded_at_nanos: ((now_unix_millis % 1_000) * 1_000_000) as i32,
                },
            )
            .map_err(|_| "INVALID_ARGUMENT")?;
            let outcome = execute_persons_command_record_v1(
                persistence,
                &record,
                &PersonsCommandRuntimeContextV1 {
                    logical_owner_id: logical_owner_id.to_owned(),
                    runtime_instance_id: runtime_instance_id.to_owned(),
                    runtime_generation,
                    now_unix_millis,
                },
            )
            .await
            .map_err(|_| "UNAVAILABLE")?;
            let envelope = decode_envelope_v1(&outcome.terminal_result.envelope_bytes)
                .map_err(|_| "UNAVAILABLE")?;
            PersonCommandSucceededV1::decode(envelope.payload.as_slice())
                .map(|payload| payload.encode_to_vec())
                .map_err(|_| "FAILED_PRECONDITION")
        }
        PERSONS_CLIENT_LIST_DIRECTORY_CONTRACT_NAME_V1
        | PERSONS_CLIENT_GET_PROFILE_CONTRACT_NAME_V1
        | PERSONS_CLIENT_LIST_SOURCE_LINKS_CONTRACT_NAME_V1 => {
            let loaded = persistence
                .load_owner(logical_owner_id)
                .await
                .map_err(|_| "UNAVAILABLE")?;
            match contract.name.as_str() {
                PERSONS_CLIENT_LIST_DIRECTORY_CONTRACT_NAME_V1 => {
                    let query =
                        ReadPersonDirectoryRequestV1::decode(request.request_payload.as_slice())
                            .map_err(|_| "INVALID_ARGUMENT")?;
                    if !accepted_payload_owner_v1(&query.logical_owner_id, logical_owner_id)
                        || !(1..=200).contains(&query.limit)
                    {
                        return Err("INVALID_ARGUMENT");
                    }
                    let after = optional_id(&query.after_person_id)?;
                    let mut persons = loaded
                        .state
                        .persons()
                        .filter(|person| after.is_none_or(|id| person.person_id.0 > id))
                        .take(query.limit as usize + 1)
                        .collect::<Vec<_>>();
                    let has_more = persons.len() > query.limit as usize;
                    persons.truncate(query.limit as usize);
                    let next = page_cursor_v1(
                        &persons
                            .iter()
                            .map(|person| person.person_id.0)
                            .collect::<Vec<_>>(),
                        query.limit as usize,
                        has_more,
                    )
                    .map_or_else(Vec::new, |id| id.to_vec());
                    Ok(PersonDirectoryResultV1 {
                        persons: persons.into_iter().map(directory_entry).collect(),
                        next_after_person_id: next,
                    }
                    .encode_to_vec())
                }
                PERSONS_CLIENT_GET_PROFILE_CONTRACT_NAME_V1 => {
                    let query =
                        ReadPersonProfileRequestV1::decode(request.request_payload.as_slice())
                            .map_err(|_| "INVALID_ARGUMENT")?;
                    let id = required_id(&query.person_id)?;
                    if !accepted_payload_owner_v1(&query.logical_owner_id, logical_owner_id) {
                        return Err("REJECTED");
                    }
                    let person = loaded.state.person(PersonIdV1(id)).ok_or("NOT_FOUND")?;
                    Ok(profile_result(person).encode_to_vec())
                }
                PERSONS_CLIENT_LIST_SOURCE_LINKS_CONTRACT_NAME_V1 => {
                    let query =
                        ReadPersonSourceLinksRequestV1::decode(request.request_payload.as_slice())
                            .map_err(|_| "INVALID_ARGUMENT")?;
                    let id = required_id(&query.person_id)?;
                    let after = optional_id(&query.after_source_link_id)?;
                    if !accepted_payload_owner_v1(&query.logical_owner_id, logical_owner_id)
                        || !(1..=200).contains(&query.limit)
                    {
                        return Err("INVALID_ARGUMENT");
                    }
                    let person = loaded.state.person(PersonIdV1(id)).ok_or("NOT_FOUND")?;
                    let mut links = person
                        .source_links
                        .values()
                        .filter(|link| after.is_none_or(|id| source_link_id(link) > id))
                        .take(query.limit as usize + 1)
                        .collect::<Vec<_>>();
                    let has_more = links.len() > query.limit as usize;
                    links.truncate(query.limit as usize);
                    let next = page_cursor_v1(
                        &links
                            .iter()
                            .map(|link| source_link_id(link))
                            .collect::<Vec<_>>(),
                        query.limit as usize,
                        has_more,
                    )
                    .map_or_else(Vec::new, |id| id.to_vec());
                    Ok(PersonSourceLinksResultV1 {
                        source_links: links
                            .into_iter()
                            .map(|link| PersonSourceLinkV1 {
                                source_link_id: source_link_id(link).to_vec(),
                                person_id: person.person_id.0.to_vec(),
                                source: Some(makosh_persons_api::wire::ProviderSourceIdentityV1 {
                                    integration_public_id: link
                                        .key
                                        .integration_public_id
                                        .0
                                        .to_vec(),
                                    account_public_id: link.key.account_public_id.0.to_vec(),
                                    provider_source_contact_public_id: link
                                        .key
                                        .provider_source_contact_public_id
                                        .0
                                        .to_vec(),
                                }),
                                claims: Some(makosh_persons_api::wire::ProviderSourceClaimsV1 {
                                    display_name: link.claims.display_name.clone(),
                                    normalized_emails: link.claims.emails.clone(),
                                    normalized_phones: link.claims.phones.clone(),
                                }),
                                provenance: Some(
                                    makosh_persons_api::wire::ProviderSourceProvenanceV1 {
                                        source_revision: link.provenance.revision,
                                        source_digest: link.provenance.digest.0.to_vec(),
                                        observed_at: Some(timestamp(link.provenance.observed_at)),
                                    },
                                ),
                            })
                            .collect(),
                        next_after_source_link_id: next,
                    }
                    .encode_to_vec())
                }
                _ => Err("REJECTED"),
            }
        }
        _ => Err("REJECTED"),
    }
}

fn required_id(value: &[u8]) -> Result<[u8; 16], &'static str> {
    let id: [u8; 16] = value.try_into().map_err(|_| "INVALID_ARGUMENT")?;
    id.iter()
        .any(|byte| *byte != 0)
        .then_some(id)
        .ok_or("INVALID_ARGUMENT")
}

fn optional_id(value: &[u8]) -> Result<Option<[u8; 16]>, &'static str> {
    if value.is_empty() {
        Ok(None)
    } else {
        required_id(value).map(Some)
    }
}

fn accepted_payload_owner_v1(payload_owner: &str, authenticated_owner: &str) -> bool {
    payload_owner.is_empty() || payload_owner == authenticated_owner
}

fn page_cursor_v1(returned_ids: &[[u8; 16]], limit: usize, has_more: bool) -> Option<[u8; 16]> {
    has_more
        .then(|| returned_ids.get(limit.saturating_sub(1)).copied())
        .flatten()
}

fn lifecycle(value: PersonLifecycleV1) -> i32 {
    (match value {
        PersonLifecycleV1::Provisional => WireLifecycle::PersonLifecycleProvisional,
        PersonLifecycleV1::Active => WireLifecycle::PersonLifecycleActive,
        PersonLifecycleV1::Merged => WireLifecycle::PersonLifecycleMerged,
        PersonLifecycleV1::Archived => WireLifecycle::PersonLifecycleArchived,
    }) as i32
}

fn profile(value: Option<&makosh_persons_core::OwnerProfileV1>) -> PersonProfileV1 {
    value.map_or_else(PersonProfileV1::default, |profile| PersonProfileV1 {
        display_name: profile.display_name.clone(),
        given_name: profile.given_name.clone(),
        family_name: profile.family_name.clone(),
        normalized_emails: profile.emails.clone(),
        normalized_phones: profile.phones.clone(),
    })
}

fn timestamp(value: makosh_persons_core::TimestampV1) -> TimestampV1 {
    TimestampV1 {
        unix_seconds: value.unix_seconds,
        nanos: value.nanos,
    }
}

fn directory_entry(person: &PersonV1) -> PersonDirectoryEntryV1 {
    PersonDirectoryEntryV1 {
        person_id: person.person_id.0.to_vec(),
        lifecycle: lifecycle(person.lifecycle),
        person_revision: person.revision,
        display_name: person
            .owner_profile
            .as_ref()
            .and_then(|profile| profile.display_name.clone()),
        source_count: person.source_links.len() as u32,
    }
}

fn profile_result(person: &PersonV1) -> PersonProfileResultV1 {
    PersonProfileResultV1 {
        person_id: person.person_id.0.to_vec(),
        logical_owner_id: person.logical_owner_id.clone(),
        lifecycle: lifecycle(person.lifecycle),
        person_revision: person.revision,
        owner_profile: Some(profile(person.owner_profile.as_ref())),
        created_at: Some(timestamp(person.created_at)),
        updated_at: Some(timestamp(person.updated_at)),
    }
}

fn source_link_id(link: &makosh_persons_core::SourceLinkV1) -> [u8; 16] {
    let mut bytes = Vec::with_capacity(48);
    bytes.extend_from_slice(&link.key.integration_public_id.0);
    bytes.extend_from_slice(&link.key.account_public_id.0);
    bytes.extend_from_slice(&link.key.provider_source_contact_public_id.0);
    crate::transport::persons_deterministic_public_id_v1(
        b"persons-source-link-v1",
        &bytes,
        b"public",
    )
}

#[cfg(test)]
mod client_boundary_tests {
    use super::{accepted_payload_owner_v1, page_cursor_v1};

    #[test]
    fn authenticated_owner_accepts_empty_or_exact_payload_owner_only() {
        assert!(accepted_payload_owner_v1("", "owner-1"));
        assert!(accepted_payload_owner_v1("owner-1", "owner-1"));
        assert!(!accepted_payload_owner_v1("owner-2", "owner-1"));
    }

    #[test]
    fn page_cursor_is_last_returned_and_never_the_overflow_item() {
        let one = vec![[1_u8; 16]];
        assert_eq!(page_cursor_v1(&one, 1, false), None);

        let exact = vec![[1_u8; 16], [2_u8; 16]];
        assert_eq!(page_cursor_v1(&exact, 2, false), None);

        let overflow = vec![[1_u8; 16], [2_u8; 16], [3_u8; 16]];
        assert_eq!(page_cursor_v1(&overflow, 2, true), Some([2_u8; 16]));

        let all = [[1_u8; 16], [2_u8; 16], [3_u8; 16], [4_u8; 16], [5_u8; 16]];
        let first_cursor = page_cursor_v1(&all[..2], 2, true).expect("first cursor");
        let second = all
            .iter()
            .copied()
            .filter(|id| *id > first_cursor)
            .take(2)
            .collect::<Vec<_>>();
        let second_cursor = page_cursor_v1(&second, 2, true).expect("second cursor");
        let third = all
            .iter()
            .copied()
            .filter(|id| *id > second_cursor)
            .collect::<Vec<_>>();
        assert_eq!(second, vec![[3_u8; 16], [4_u8; 16]]);
        assert_eq!(third, vec![[5_u8; 16]]);
        assert_eq!(page_cursor_v1(&third, 2, false), None);
    }
}
