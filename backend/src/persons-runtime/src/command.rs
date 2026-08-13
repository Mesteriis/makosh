use makosh_persons_api::wire::{
    self, ConfirmAttachPersonSourceCommandV1, ConfirmDetachPersonSourceCommandV1,
    ConfirmMergePersonsCommandV1, ConfirmSplitPersonCommandV1, ManualCreatePersonCommandV1,
    ObserveProviderSourceContactCommandV1, RemoveProviderSourceContactCommandV1,
    UpdatePersonOwnerProfileCommandV1, UpdateProviderSourceContactCommandV1,
    persons_command_v1::Command,
};
use makosh_persons_core::{
    AttachSourceActionV1, DecisionProvenanceV1, DetachSourceActionV1, DigestV1,
    ManualPersonDraftV1, MergePersonsActionV1, OwnerProfileV1, PersonIdV1, PublicIdV1,
    SourceClaimsV1, SourceLinkKeyV1, SourceObservationV1, SourceProvenanceV1, SplitPersonActionV1,
    SplitProfileFactKindV1, SplitSourceSelectionV1, TimestampV1,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DecodedPersonsCommandV1 {
    ManualCreate(ManualPersonDraftV1),
    OwnerProfileUpdate {
        logical_owner_id: String,
        person_id: PersonIdV1,
        expected_person_revision: u64,
        owner_profile: OwnerProfileV1,
        updated_at: TimestampV1,
    },
    SourceObserve(SourceObservationV1),
    SourceUpdate(SourceObservationV1),
    SourceRemove {
        logical_owner_id: String,
        source: SourceLinkKeyV1,
        provenance: SourceProvenanceV1,
    },
    ConfirmedAttach(AttachSourceActionV1, DecisionProvenanceV1),
    ConfirmedDetach(DetachSourceActionV1, DecisionProvenanceV1),
    ConfirmedMerge(MergePersonsActionV1, DecisionProvenanceV1),
    ConfirmedSplit(SplitPersonActionV1, DecisionProvenanceV1),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PersonsCommandDecodeErrorV1 {
    InvalidPayload,
    OwnerMismatch,
    CommandMismatch,
}

pub fn decode_typed_command_v1(
    payload: wire::PersonsCommandV1,
    expected_owner: &str,
    expected_command_id: [u8; 16],
) -> Result<DecodedPersonsCommandV1, PersonsCommandDecodeErrorV1> {
    let command = payload
        .command
        .ok_or(PersonsCommandDecodeErrorV1::InvalidPayload)?;
    let (command_id, owner) = command_identity(&command)?;
    if command_id != expected_command_id {
        return Err(PersonsCommandDecodeErrorV1::CommandMismatch);
    }
    if owner != expected_owner {
        return Err(PersonsCommandDecodeErrorV1::OwnerMismatch);
    }
    match command {
        Command::ManualCreate(value) => decode_manual(value),
        Command::OwnerProfileUpdate(value) => decode_profile(value),
        Command::SourceObserve(value) => {
            decode_source(value).map(DecodedPersonsCommandV1::SourceObserve)
        }
        Command::SourceUpdate(value) => {
            decode_source_update(value).map(DecodedPersonsCommandV1::SourceUpdate)
        }
        Command::SourceRemove(value) => decode_remove(value),
        Command::ConfirmedAttach(value) => decode_attach(value),
        Command::ConfirmedDetach(value) => decode_detach(value),
        Command::ConfirmedMerge(value) => decode_merge(value),
        Command::ConfirmedSplit(value) => decode_split(value),
    }
}

pub fn persons_wire_command_identity_v1(
    payload: &wire::PersonsCommandV1,
) -> Result<([u8; 16], String), PersonsCommandDecodeErrorV1> {
    let command = payload
        .command
        .as_ref()
        .ok_or(PersonsCommandDecodeErrorV1::InvalidPayload)?;
    let (command_id, owner) = command_identity(command)?;
    Ok((command_id, owner.to_owned()))
}

fn command_identity(command: &Command) -> Result<([u8; 16], &str), PersonsCommandDecodeErrorV1> {
    let (id, owner) = match command {
        Command::ManualCreate(value) => (&value.command_id, value.logical_owner_id.as_str()),
        Command::OwnerProfileUpdate(value) => (&value.command_id, value.logical_owner_id.as_str()),
        Command::SourceObserve(value) => (&value.command_id, value.logical_owner_id.as_str()),
        Command::SourceUpdate(value) => (&value.command_id, value.logical_owner_id.as_str()),
        Command::SourceRemove(value) => (&value.command_id, value.logical_owner_id.as_str()),
        Command::ConfirmedAttach(value) => (&value.command_id, value.logical_owner_id.as_str()),
        Command::ConfirmedDetach(value) => (&value.command_id, value.logical_owner_id.as_str()),
        Command::ConfirmedMerge(value) => (&value.command_id, value.logical_owner_id.as_str()),
        Command::ConfirmedSplit(value) => (&value.command_id, value.logical_owner_id.as_str()),
    };
    if owner.is_empty() || owner.len() > 128 {
        return Err(PersonsCommandDecodeErrorV1::InvalidPayload);
    }
    Ok((id16(id)?, owner))
}

fn decode_manual(
    value: ManualCreatePersonCommandV1,
) -> Result<DecodedPersonsCommandV1, PersonsCommandDecodeErrorV1> {
    Ok(DecodedPersonsCommandV1::ManualCreate(ManualPersonDraftV1 {
        person_id: PersonIdV1(id16(&value.person_id)?),
        logical_owner_id: value.logical_owner_id,
        owner_profile: profile(value.owner_profile)?,
        created_at: timestamp(value.created_at)?,
    }))
}

fn decode_profile(
    value: UpdatePersonOwnerProfileCommandV1,
) -> Result<DecodedPersonsCommandV1, PersonsCommandDecodeErrorV1> {
    Ok(DecodedPersonsCommandV1::OwnerProfileUpdate {
        logical_owner_id: value.logical_owner_id,
        person_id: PersonIdV1(id16(&value.person_id)?),
        expected_person_revision: nonzero_revision(value.expected_person_revision)?,
        owner_profile: profile(value.owner_profile)?,
        updated_at: timestamp(value.updated_at)?,
    })
}

fn decode_source(
    value: ObserveProviderSourceContactCommandV1,
) -> Result<SourceObservationV1, PersonsCommandDecodeErrorV1> {
    source_observation(
        value.logical_owner_id,
        value.source,
        value.claims,
        value.provenance,
    )
}

fn decode_source_update(
    value: UpdateProviderSourceContactCommandV1,
) -> Result<SourceObservationV1, PersonsCommandDecodeErrorV1> {
    source_observation(
        value.logical_owner_id,
        value.source,
        value.claims,
        value.provenance,
    )
}

fn decode_remove(
    value: RemoveProviderSourceContactCommandV1,
) -> Result<DecodedPersonsCommandV1, PersonsCommandDecodeErrorV1> {
    Ok(DecodedPersonsCommandV1::SourceRemove {
        logical_owner_id: value.logical_owner_id,
        source: source(value.source)?,
        provenance: provenance(value.provenance)?,
    })
}

fn decode_attach(
    value: ConfirmAttachPersonSourceCommandV1,
) -> Result<DecodedPersonsCommandV1, PersonsCommandDecodeErrorV1> {
    Ok(DecodedPersonsCommandV1::ConfirmedAttach(
        AttachSourceActionV1 {
            logical_owner_id: value.logical_owner_id,
            from_person_id: PersonIdV1(id16(&value.from_person_id)?),
            expected_from_person_revision: nonzero_revision(value.expected_from_person_revision)?,
            to_person_id: PersonIdV1(id16(&value.to_person_id)?),
            expected_to_person_revision: nonzero_revision(value.expected_to_person_revision)?,
            source: source(value.source)?,
            expected_source_revision: nonzero_revision(value.expected_source_revision)?,
        },
        decision(value.decision)?,
    ))
}

fn decode_detach(
    value: ConfirmDetachPersonSourceCommandV1,
) -> Result<DecodedPersonsCommandV1, PersonsCommandDecodeErrorV1> {
    Ok(DecodedPersonsCommandV1::ConfirmedDetach(
        DetachSourceActionV1 {
            logical_owner_id: value.logical_owner_id,
            person_id: PersonIdV1(id16(&value.person_id)?),
            expected_person_revision: nonzero_revision(value.expected_person_revision)?,
            source: source(value.source)?,
            expected_source_revision: nonzero_revision(value.expected_source_revision)?,
            expected_detached_person_revision: value.expected_detached_person_revision,
        },
        decision(value.decision)?,
    ))
}

fn decode_merge(
    value: ConfirmMergePersonsCommandV1,
) -> Result<DecodedPersonsCommandV1, PersonsCommandDecodeErrorV1> {
    Ok(DecodedPersonsCommandV1::ConfirmedMerge(
        MergePersonsActionV1 {
            logical_owner_id: value.logical_owner_id,
            source_person_id: PersonIdV1(id16(&value.source_person_id)?),
            expected_source_person_revision: nonzero_revision(
                value.expected_source_person_revision,
            )?,
            target_person_id: PersonIdV1(id16(&value.target_person_id)?),
            expected_target_person_revision: nonzero_revision(
                value.expected_target_person_revision,
            )?,
        },
        decision(value.decision)?,
    ))
}

fn decode_split(
    value: ConfirmSplitPersonCommandV1,
) -> Result<DecodedPersonsCommandV1, PersonsCommandDecodeErrorV1> {
    let source_selection = value
        .source_selection
        .into_iter()
        .map(|value| {
            Ok(SplitSourceSelectionV1 {
                source: source(value.source)?,
                expected_source_revision: nonzero_revision(value.expected_source_revision)?,
            })
        })
        .collect::<Result<Vec<_>, PersonsCommandDecodeErrorV1>>()?;
    let profile_fact_selection = value
        .profile_fact_selection
        .into_iter()
        .map(
            |value| match wire::SplitProfileFactKindV1::try_from(value) {
                Ok(wire::SplitProfileFactKindV1::SplitProfileFactKindDisplayName) => {
                    Ok(SplitProfileFactKindV1::DisplayName)
                }
                Ok(wire::SplitProfileFactKindV1::SplitProfileFactKindGivenName) => {
                    Ok(SplitProfileFactKindV1::GivenName)
                }
                Ok(wire::SplitProfileFactKindV1::SplitProfileFactKindFamilyName) => {
                    Ok(SplitProfileFactKindV1::FamilyName)
                }
                Ok(wire::SplitProfileFactKindV1::SplitProfileFactKindEmails) => {
                    Ok(SplitProfileFactKindV1::Emails)
                }
                Ok(wire::SplitProfileFactKindV1::SplitProfileFactKindPhones) => {
                    Ok(SplitProfileFactKindV1::Phones)
                }
                _ => Err(PersonsCommandDecodeErrorV1::InvalidPayload),
            },
        )
        .collect::<Result<Vec<_>, _>>()?;
    Ok(DecodedPersonsCommandV1::ConfirmedSplit(
        SplitPersonActionV1 {
            logical_owner_id: value.logical_owner_id,
            merged_person_id: PersonIdV1(id16(&value.merged_person_id)?),
            expected_merged_person_revision: nonzero_revision(
                value.expected_merged_person_revision,
            )?,
            target_person_id: PersonIdV1(id16(&value.target_person_id)?),
            expected_target_person_revision: nonzero_revision(
                value.expected_target_person_revision,
            )?,
            source_selection,
            profile_fact_selection,
        },
        decision(value.decision)?,
    ))
}

fn source_observation(
    owner: String,
    source_value: Option<wire::ProviderSourceIdentityV1>,
    claims_value: Option<wire::ProviderSourceClaimsV1>,
    provenance_value: Option<wire::ProviderSourceProvenanceV1>,
) -> Result<SourceObservationV1, PersonsCommandDecodeErrorV1> {
    let claims = claims_value.ok_or(PersonsCommandDecodeErrorV1::InvalidPayload)?;
    Ok(SourceObservationV1 {
        logical_owner_id: owner,
        key: source(source_value)?,
        claims: SourceClaimsV1 {
            display_name: claims.display_name,
            emails: claims.normalized_emails,
            phones: claims.normalized_phones,
        },
        provenance: provenance(provenance_value)?,
    })
}

fn source(
    value: Option<wire::ProviderSourceIdentityV1>,
) -> Result<SourceLinkKeyV1, PersonsCommandDecodeErrorV1> {
    let value = value.ok_or(PersonsCommandDecodeErrorV1::InvalidPayload)?;
    Ok(SourceLinkKeyV1 {
        integration_public_id: PublicIdV1(id16(&value.integration_public_id)?),
        account_public_id: PublicIdV1(id16(&value.account_public_id)?),
        provider_source_contact_public_id: PublicIdV1(id16(
            &value.provider_source_contact_public_id,
        )?),
    })
}

fn provenance(
    value: Option<wire::ProviderSourceProvenanceV1>,
) -> Result<SourceProvenanceV1, PersonsCommandDecodeErrorV1> {
    let value = value.ok_or(PersonsCommandDecodeErrorV1::InvalidPayload)?;
    Ok(SourceProvenanceV1 {
        revision: nonzero_revision(value.source_revision)?,
        digest: DigestV1(id32(&value.source_digest)?),
        observed_at: timestamp(value.observed_at)?,
    })
}

fn decision(
    value: Option<wire::DecisionProvenanceV1>,
) -> Result<DecisionProvenanceV1, PersonsCommandDecodeErrorV1> {
    let value = value.ok_or(PersonsCommandDecodeErrorV1::InvalidPayload)?;
    Ok(DecisionProvenanceV1 {
        decision_id: PublicIdV1(id16(&value.decision_id)?),
        review_id: PublicIdV1(id16(&value.review_id)?),
        revision: nonzero_revision(value.decision_revision)?,
        decided_by_owner_device_id: PublicIdV1(id16(&value.decided_by_owner_device_id)?),
        decided_at: timestamp(value.decided_at)?,
        approved_action_digest: DigestV1(id32(&value.approved_action_digest)?),
    })
}

fn profile(
    value: Option<wire::PersonProfileV1>,
) -> Result<OwnerProfileV1, PersonsCommandDecodeErrorV1> {
    let value = value.ok_or(PersonsCommandDecodeErrorV1::InvalidPayload)?;
    Ok(OwnerProfileV1 {
        display_name: value.display_name,
        given_name: value.given_name,
        family_name: value.family_name,
        emails: value.normalized_emails,
        phones: value.normalized_phones,
    })
}

fn timestamp(value: Option<wire::TimestampV1>) -> Result<TimestampV1, PersonsCommandDecodeErrorV1> {
    let value = value.ok_or(PersonsCommandDecodeErrorV1::InvalidPayload)?;
    if value.unix_seconds <= 0 || !(0..1_000_000_000).contains(&value.nanos) {
        return Err(PersonsCommandDecodeErrorV1::InvalidPayload);
    }
    Ok(TimestampV1 {
        unix_seconds: value.unix_seconds,
        nanos: value.nanos,
    })
}

fn id16(value: &[u8]) -> Result<[u8; 16], PersonsCommandDecodeErrorV1> {
    let value: [u8; 16] = value
        .try_into()
        .map_err(|_| PersonsCommandDecodeErrorV1::InvalidPayload)?;
    if value == [0; 16] {
        return Err(PersonsCommandDecodeErrorV1::InvalidPayload);
    }
    Ok(value)
}

fn id32(value: &[u8]) -> Result<[u8; 32], PersonsCommandDecodeErrorV1> {
    let value: [u8; 32] = value
        .try_into()
        .map_err(|_| PersonsCommandDecodeErrorV1::InvalidPayload)?;
    if value == [0; 32] {
        return Err(PersonsCommandDecodeErrorV1::InvalidPayload);
    }
    Ok(value)
}

fn nonzero_revision(value: u64) -> Result<u64, PersonsCommandDecodeErrorV1> {
    if value == 0 {
        Err(PersonsCommandDecodeErrorV1::InvalidPayload)
    } else {
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const OWNER: &str = "owner-a";

    #[test]
    fn all_nine_wire_commands_decode_to_exact_typed_variants() {
        let id = [9_u8; 16];
        let commands = vec![
            Command::ManualCreate(ManualCreatePersonCommandV1 {
                command_id: id.to_vec(),
                person_id: [1; 16].to_vec(),
                logical_owner_id: OWNER.to_owned(),
                owner_profile: Some(profile_wire()),
                created_at: Some(time_wire()),
            }),
            Command::OwnerProfileUpdate(UpdatePersonOwnerProfileCommandV1 {
                command_id: id.to_vec(),
                person_id: [1; 16].to_vec(),
                logical_owner_id: OWNER.to_owned(),
                expected_person_revision: 7,
                owner_profile: Some(profile_wire()),
                updated_at: Some(time_wire()),
            }),
            Command::SourceObserve(source_observe_wire(id)),
            Command::SourceUpdate(UpdateProviderSourceContactCommandV1 {
                command_id: id.to_vec(),
                logical_owner_id: OWNER.to_owned(),
                source: Some(source_wire()),
                claims: Some(claims_wire()),
                provenance: Some(provenance_wire()),
            }),
            Command::SourceRemove(RemoveProviderSourceContactCommandV1 {
                command_id: id.to_vec(),
                logical_owner_id: OWNER.to_owned(),
                source: Some(source_wire()),
                provenance: Some(provenance_wire()),
            }),
            Command::ConfirmedAttach(ConfirmAttachPersonSourceCommandV1 {
                command_id: id.to_vec(),
                from_person_id: [1; 16].to_vec(),
                to_person_id: [2; 16].to_vec(),
                logical_owner_id: OWNER.to_owned(),
                source: Some(source_wire()),
                decision: Some(decision_wire()),
                expected_from_person_revision: 3,
                expected_to_person_revision: 4,
                expected_source_revision: 5,
            }),
            Command::ConfirmedDetach(ConfirmDetachPersonSourceCommandV1 {
                command_id: id.to_vec(),
                person_id: [1; 16].to_vec(),
                logical_owner_id: OWNER.to_owned(),
                source: Some(source_wire()),
                decision: Some(decision_wire()),
                expected_person_revision: 3,
                expected_source_revision: 5,
                expected_detached_person_revision: 4,
            }),
            Command::ConfirmedMerge(ConfirmMergePersonsCommandV1 {
                command_id: id.to_vec(),
                source_person_id: [1; 16].to_vec(),
                target_person_id: [2; 16].to_vec(),
                logical_owner_id: OWNER.to_owned(),
                decision: Some(decision_wire()),
                expected_source_person_revision: 3,
                expected_target_person_revision: 4,
            }),
            Command::ConfirmedSplit(ConfirmSplitPersonCommandV1 {
                command_id: id.to_vec(),
                merged_person_id: [1; 16].to_vec(),
                logical_owner_id: OWNER.to_owned(),
                target_person_id: [2; 16].to_vec(),
                expected_merged_person_revision: 3,
                expected_target_person_revision: 4,
                source_selection: vec![wire::SplitPersonSourceSelectionV1 {
                    source: Some(source_wire()),
                    expected_source_revision: 5,
                }],
                profile_fact_selection: vec![
                    wire::SplitProfileFactKindV1::SplitProfileFactKindEmails as i32,
                ],
                decision: Some(decision_wire()),
            }),
        ];
        for (index, command) in commands.into_iter().enumerate() {
            let decoded = decode_typed_command_v1(
                wire::PersonsCommandV1 {
                    command: Some(command),
                },
                OWNER,
                id,
            )
            .expect("decode");
            assert_eq!(variant_index(&decoded), index);
        }
        let DecodedPersonsCommandV1::ConfirmedSplit(action, decision) = decode_typed_command_v1(
            wire::PersonsCommandV1 {
                command: Some(Command::ConfirmedSplit(ConfirmSplitPersonCommandV1 {
                    command_id: id.to_vec(),
                    merged_person_id: [1; 16].to_vec(),
                    logical_owner_id: OWNER.to_owned(),
                    target_person_id: [2; 16].to_vec(),
                    expected_merged_person_revision: 3,
                    expected_target_person_revision: 4,
                    source_selection: vec![wire::SplitPersonSourceSelectionV1 {
                        source: Some(source_wire()),
                        expected_source_revision: 5,
                    }],
                    profile_fact_selection: vec![
                        wire::SplitProfileFactKindV1::SplitProfileFactKindEmails as i32,
                    ],
                    decision: Some(decision_wire()),
                })),
            },
            OWNER,
            id,
        )
        .expect("split") else {
            panic!("split");
        };
        assert_eq!(action.expected_merged_person_revision, 3);
        assert_eq!(action.source_selection[0].expected_source_revision, 5);
        assert_eq!(
            action.profile_fact_selection,
            vec![SplitProfileFactKindV1::Emails]
        );
        assert_eq!(decision.revision, 6);
    }

    fn variant_index(value: &DecodedPersonsCommandV1) -> usize {
        match value {
            DecodedPersonsCommandV1::ManualCreate(_) => 0,
            DecodedPersonsCommandV1::OwnerProfileUpdate { .. } => 1,
            DecodedPersonsCommandV1::SourceObserve(_) => 2,
            DecodedPersonsCommandV1::SourceUpdate(_) => 3,
            DecodedPersonsCommandV1::SourceRemove { .. } => 4,
            DecodedPersonsCommandV1::ConfirmedAttach(..) => 5,
            DecodedPersonsCommandV1::ConfirmedDetach(..) => 6,
            DecodedPersonsCommandV1::ConfirmedMerge(..) => 7,
            DecodedPersonsCommandV1::ConfirmedSplit(..) => 8,
        }
    }

    fn source_observe_wire(id: [u8; 16]) -> ObserveProviderSourceContactCommandV1 {
        ObserveProviderSourceContactCommandV1 {
            command_id: id.to_vec(),
            logical_owner_id: OWNER.to_owned(),
            source: Some(source_wire()),
            claims: Some(claims_wire()),
            provenance: Some(provenance_wire()),
        }
    }
    fn source_wire() -> wire::ProviderSourceIdentityV1 {
        wire::ProviderSourceIdentityV1 {
            integration_public_id: [3; 16].to_vec(),
            account_public_id: [4; 16].to_vec(),
            provider_source_contact_public_id: [5; 16].to_vec(),
        }
    }
    fn claims_wire() -> wire::ProviderSourceClaimsV1 {
        wire::ProviderSourceClaimsV1 {
            display_name: Some("public".to_owned()),
            normalized_emails: vec!["a@example.test".to_owned()],
            normalized_phones: Vec::new(),
        }
    }
    fn provenance_wire() -> wire::ProviderSourceProvenanceV1 {
        wire::ProviderSourceProvenanceV1 {
            source_revision: 5,
            source_digest: [7; 32].to_vec(),
            observed_at: Some(time_wire()),
        }
    }
    fn profile_wire() -> wire::PersonProfileV1 {
        wire::PersonProfileV1 {
            display_name: Some("Ada".to_owned()),
            given_name: None,
            family_name: None,
            normalized_emails: Vec::new(),
            normalized_phones: Vec::new(),
        }
    }
    fn time_wire() -> wire::TimestampV1 {
        wire::TimestampV1 {
            unix_seconds: 1_800_000_000,
            nanos: 0,
        }
    }
    fn decision_wire() -> wire::DecisionProvenanceV1 {
        wire::DecisionProvenanceV1 {
            decision_id: [6; 16].to_vec(),
            review_id: [7; 16].to_vec(),
            decision_revision: 6,
            decided_by_owner_device_id: [8; 16].to_vec(),
            decided_at: Some(time_wire()),
            approved_action_digest: [9; 32].to_vec(),
        }
    }
}
