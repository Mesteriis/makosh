#![forbid(unsafe_code)]

use makosh_mail_address_book_contract::{
    validate_mail_person_source_observed_v1, validate_mail_person_source_removed_v1,
    validate_mail_person_source_updated_v1,
    wire_person_source::{
        MailPersonSourceClaimsV1, MailPersonSourceIdentityV1, MailPersonSourceObservedV1,
        MailPersonSourceProvenanceV1, MailPersonSourceRemovedV1, MailPersonSourceUpdatedV1,
    },
};
use makosh_mail_persons_sync_api::{
    mail_persons_sync_page_fingerprint_v1, wire::MailPersonsSyncPageIdentityV1,
};
use makosh_persons_api::wire::{
    ObserveProviderSourceContactCommandV1, PersonsCommandV1, ProviderSourceClaimsV1,
    ProviderSourceIdentityV1, ProviderSourceProvenanceV1, RemoveProviderSourceContactCommandV1,
    TimestampV1, UpdateProviderSourceContactCommandV1, persons_command_v1::Command,
};
use sha2::{Digest, Sha256};

pub const PACKAGE: &str = "makosh-mail-persons-sync-core";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailPersonsSyncReplayDecisionV1 {
    Apply,
    ExactReplay,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailPersonsSyncCoreErrorV1 {
    InvalidInput,
    Conflict,
}

pub fn classify_page_replay_v1(
    stored: Option<(MailPersonsSyncPageIdentityV1, [u8; 32])>,
    identity: &MailPersonsSyncPageIdentityV1,
    page_digest: [u8; 32],
) -> Result<MailPersonsSyncReplayDecisionV1, MailPersonsSyncCoreErrorV1> {
    mail_persons_sync_page_fingerprint_v1(identity, page_digest)
        .map_err(|_| MailPersonsSyncCoreErrorV1::InvalidInput)?;
    match stored {
        None => Ok(MailPersonsSyncReplayDecisionV1::Apply),
        Some((existing_identity, existing_digest))
            if existing_identity == *identity && existing_digest == page_digest =>
        {
            Ok(MailPersonsSyncReplayDecisionV1::ExactReplay)
        }
        Some(_) => Err(MailPersonsSyncCoreErrorV1::Conflict),
    }
}

pub fn map_observed_to_persons_v1(
    value: &MailPersonSourceObservedV1,
) -> Result<PersonsCommandV1, MailPersonsSyncCoreErrorV1> {
    validate_mail_person_source_observed_v1(value)
        .map_err(|_| MailPersonsSyncCoreErrorV1::InvalidInput)?;
    let source = value
        .source
        .as_ref()
        .ok_or(MailPersonsSyncCoreErrorV1::InvalidInput)?;
    let provenance = value
        .provenance
        .as_ref()
        .ok_or(MailPersonsSyncCoreErrorV1::InvalidInput)?;
    Ok(PersonsCommandV1 {
        command: Some(Command::SourceObserve(
            ObserveProviderSourceContactCommandV1 {
                command_id: command_id(
                    b"observe",
                    value.observation_id.as_slice(),
                    &value.logical_owner_id,
                    source,
                    provenance,
                )
                .to_vec(),
                logical_owner_id: value.logical_owner_id.clone(),
                source: Some(map_source(source)),
                claims: value.claims.as_ref().map(map_claims),
                provenance: Some(map_provenance(provenance)?),
            },
        )),
    })
}

pub fn map_updated_to_persons_v1(
    value: &MailPersonSourceUpdatedV1,
) -> Result<PersonsCommandV1, MailPersonsSyncCoreErrorV1> {
    validate_mail_person_source_updated_v1(value)
        .map_err(|_| MailPersonsSyncCoreErrorV1::InvalidInput)?;
    let source = value
        .source
        .as_ref()
        .ok_or(MailPersonsSyncCoreErrorV1::InvalidInput)?;
    let provenance = value
        .provenance
        .as_ref()
        .ok_or(MailPersonsSyncCoreErrorV1::InvalidInput)?;
    Ok(PersonsCommandV1 {
        command: Some(Command::SourceUpdate(
            UpdateProviderSourceContactCommandV1 {
                command_id: command_id(
                    b"update",
                    value.observation_id.as_slice(),
                    &value.logical_owner_id,
                    source,
                    provenance,
                )
                .to_vec(),
                logical_owner_id: value.logical_owner_id.clone(),
                source: Some(map_source(source)),
                claims: value.claims.as_ref().map(map_claims),
                provenance: Some(map_provenance(provenance)?),
            },
        )),
    })
}

pub fn map_removed_to_persons_v1(
    value: &MailPersonSourceRemovedV1,
) -> Result<PersonsCommandV1, MailPersonsSyncCoreErrorV1> {
    validate_mail_person_source_removed_v1(value)
        .map_err(|_| MailPersonsSyncCoreErrorV1::InvalidInput)?;
    let source = value
        .source
        .as_ref()
        .ok_or(MailPersonsSyncCoreErrorV1::InvalidInput)?;
    let provenance = value
        .provenance
        .as_ref()
        .ok_or(MailPersonsSyncCoreErrorV1::InvalidInput)?;
    Ok(PersonsCommandV1 {
        command: Some(Command::SourceRemove(
            RemoveProviderSourceContactCommandV1 {
                command_id: command_id(
                    b"remove",
                    value.observation_id.as_slice(),
                    &value.logical_owner_id,
                    source,
                    provenance,
                )
                .to_vec(),
                logical_owner_id: value.logical_owner_id.clone(),
                source: Some(map_source(source)),
                provenance: Some(map_provenance(provenance)?),
            },
        )),
    })
}

fn command_id(
    kind: &[u8],
    observation_id: &[u8],
    owner: &str,
    source: &MailPersonSourceIdentityV1,
    provenance: &MailPersonSourceProvenanceV1,
) -> [u8; 16] {
    let mut hash = Sha256::new();
    hash.update(b"makosh.mail-persons-sync.persons-command.v1");
    for field in [
        kind,
        observation_id,
        owner.as_bytes(),
        source.integration_public_id.as_slice(),
        source.account_public_id.as_slice(),
        source.provider_source_contact_public_id.as_slice(),
        provenance.source_digest.as_slice(),
    ] {
        hash.update((field.len() as u64).to_be_bytes());
        hash.update(field);
    }
    hash.update(provenance.source_revision.to_be_bytes());
    hash.finalize()[..16].try_into().expect("SHA-256 prefix")
}

fn map_source(value: &MailPersonSourceIdentityV1) -> ProviderSourceIdentityV1 {
    ProviderSourceIdentityV1 {
        integration_public_id: value.integration_public_id.clone(),
        account_public_id: value.account_public_id.clone(),
        provider_source_contact_public_id: value.provider_source_contact_public_id.clone(),
    }
}

fn map_claims(value: &MailPersonSourceClaimsV1) -> ProviderSourceClaimsV1 {
    ProviderSourceClaimsV1 {
        display_name: value.display_name.clone(),
        normalized_emails: value.normalized_emails.clone(),
        normalized_phones: value.normalized_phones.clone(),
    }
}

fn map_provenance(
    value: &MailPersonSourceProvenanceV1,
) -> Result<ProviderSourceProvenanceV1, MailPersonsSyncCoreErrorV1> {
    let time = value
        .observed_at
        .as_ref()
        .ok_or(MailPersonsSyncCoreErrorV1::InvalidInput)?;
    Ok(ProviderSourceProvenanceV1 {
        source_revision: value.source_revision,
        source_digest: value.source_digest.clone(),
        observed_at: Some(TimestampV1 {
            unix_seconds: time.seconds,
            nanos: time.nanos,
        }),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use makosh_mail_address_book_contract::{
        mail_person_source_claims_digest_v1, mail_person_source_tombstone_digest_v1,
    };

    fn source(account: u8) -> MailPersonSourceIdentityV1 {
        MailPersonSourceIdentityV1 {
            integration_public_id: vec![1; 16],
            account_public_id: vec![account; 16],
            provider_source_contact_public_id: vec![3; 16],
        }
    }

    fn claims() -> MailPersonSourceClaimsV1 {
        MailPersonSourceClaimsV1 {
            display_name: Some("Ada".to_owned()),
            normalized_emails: vec!["ada@example.test".to_owned()],
            normalized_phones: vec!["+34910000000".to_owned()],
        }
    }

    fn provenance(
        revision: u64,
        source: &MailPersonSourceIdentityV1,
    ) -> MailPersonSourceProvenanceV1 {
        let claims = claims();
        MailPersonSourceProvenanceV1 {
            source_revision: revision,
            source_digest: mail_person_source_claims_digest_v1(source, &claims)
                .expect("claims digest")
                .to_vec(),
            observed_at: Some(prost_types::Timestamp {
                seconds: 10,
                nanos: 0,
            }),
        }
    }

    fn observed(owner: &str, account: u8) -> MailPersonSourceObservedV1 {
        let source = source(account);
        MailPersonSourceObservedV1 {
            observation_id: vec![5; 16],
            run_id: vec![6; 16],
            logical_owner_id: owner.to_owned(),
            page_sequence: 1,
            source: Some(source.clone()),
            claims: Some(claims()),
            provenance: Some(provenance(1, &source)),
        }
    }

    fn command_id(command: &PersonsCommandV1) -> Vec<u8> {
        match command.command.as_ref().expect("variant") {
            Command::SourceObserve(value) => value.command_id.clone(),
            Command::SourceUpdate(value) => value.command_id.clone(),
            Command::SourceRemove(value) => value.command_id.clone(),
            _ => panic!("source command"),
        }
    }

    #[test]
    fn observed_updated_and_removed_map_to_exact_existing_persons_variants() {
        let observed = observed("owner-1", 2);
        let observe = map_observed_to_persons_v1(&observed).expect("observe command");
        assert!(matches!(observe.command, Some(Command::SourceObserve(_))));

        let updated = MailPersonSourceUpdatedV1 {
            observation_id: vec![7; 16],
            run_id: observed.run_id.clone(),
            logical_owner_id: observed.logical_owner_id.clone(),
            page_sequence: 1,
            source: observed.source.clone(),
            claims: observed.claims.clone(),
            provenance: Some(provenance(2, observed.source.as_ref().expect("source"))),
        };
        let update = map_updated_to_persons_v1(&updated).expect("update command");
        assert!(matches!(update.command, Some(Command::SourceUpdate(_))));

        let removed_source = observed.source.clone().expect("source");
        let removed = MailPersonSourceRemovedV1 {
            observation_id: vec![8; 16],
            run_id: observed.run_id,
            logical_owner_id: observed.logical_owner_id,
            page_sequence: 1,
            source: Some(removed_source.clone()),
            provenance: Some(MailPersonSourceProvenanceV1 {
                source_revision: 3,
                source_digest: mail_person_source_tombstone_digest_v1(&removed_source)
                    .expect("tombstone digest")
                    .to_vec(),
                observed_at: Some(prost_types::Timestamp {
                    seconds: 10,
                    nanos: 0,
                }),
            }),
        };
        let remove = map_removed_to_persons_v1(&removed).expect("remove command");
        assert!(matches!(remove.command, Some(Command::SourceRemove(_))));
        assert!(!format!("{remove:?}").contains("person_id"));
    }

    #[test]
    fn command_identity_is_deterministic_and_owner_account_isolated() {
        let first = map_observed_to_persons_v1(&observed("owner-1", 2)).expect("first");
        let replay = map_observed_to_persons_v1(&observed("owner-1", 2)).expect("replay");
        let other_account = map_observed_to_persons_v1(&observed("owner-1", 9)).expect("account");
        let other_owner = map_observed_to_persons_v1(&observed("owner-2", 2)).expect("owner");
        assert_eq!(command_id(&first), command_id(&replay));
        assert_ne!(command_id(&first), command_id(&other_account));
        assert_ne!(command_id(&first), command_id(&other_owner));
    }

    #[test]
    fn mapped_source_commands_preserve_persons_compatible_public_semantics() {
        let observation = observed("owner-1", 2);
        let observe = map_observed_to_persons_v1(&observation).expect("observe mapping");
        let Command::SourceObserve(observe) = observe.command.expect("variant") else {
            panic!("source observe")
        };
        assert_eq!(observe.logical_owner_id, "owner-1");
        assert!(observe.logical_owner_id.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        }));
        let claims = observe.claims.expect("claims");
        assert!(claims.display_name.as_ref().is_some_and(|display| {
            display.chars().count() <= 240
                && display.trim() == display
                && !display.chars().any(char::is_control)
        }));
        assert!(
            claims
                .normalized_emails
                .windows(2)
                .all(|pair| pair[0] < pair[1])
        );
        assert!(
            claims
                .normalized_phones
                .windows(2)
                .all(|pair| pair[0] < pair[1])
        );
    }

    #[test]
    fn exact_page_replay_is_idempotent_and_changed_bytes_conflict() {
        let identity = MailPersonsSyncPageIdentityV1 {
            logical_owner_id: "owner-1".to_owned(),
            account_public_id: vec![3; 16],
            run_id: vec![1; 16],
            page_sequence: 1,
        };
        assert_eq!(
            classify_page_replay_v1(None, &identity, [2; 32]).expect("new"),
            MailPersonsSyncReplayDecisionV1::Apply
        );
        assert_eq!(
            classify_page_replay_v1(Some((identity.clone(), [2; 32])), &identity, [2; 32],)
                .expect("exact replay"),
            MailPersonsSyncReplayDecisionV1::ExactReplay
        );
        let mut other_account = identity.clone();
        other_account.account_public_id = vec![4; 16];
        assert!(
            classify_page_replay_v1(Some((identity.clone(), [2; 32])), &other_account, [2; 32],)
                .is_err()
        );
        assert!(
            classify_page_replay_v1(Some((identity, [2; 32])), &other_account, [3; 32]).is_err()
        );
    }
}
