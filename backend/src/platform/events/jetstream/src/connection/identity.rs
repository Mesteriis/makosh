//! NATS identities and local publish permits.

use std::collections::BTreeSet;
use std::fmt;

use makosh_runtime_protocol::v1::ContractReferenceV1;

use crate::{subjects::DurableSubjectV1, topology::ConsumerSpecV1};

/// Runtime identity fenced to one generation and grant epoch.
pub struct RuntimeNatsIdentity {
    runtime_id: String,
    runtime_generation: u64,
    grant_epoch: u64,
}

/// Password authentication material for one broker connection.
///
/// This is a transitional adapter for the existing authenticated contour. It is
/// intentionally separate from runtime and Event Hub identity so JWT credentials
/// can replace it without changing fencing semantics.
pub struct NatsPasswordCredentialV1 {
    username: String,
    password: zeroize::Zeroizing<String>,
}

/// Kernel-resolved publish fence for one runtime generation and grant epoch.
pub struct RuntimePublishPermitV1 {
    registration_id: String,
    runtime_id: String,
    runtime_generation: u64,
    grant_epoch: u64,
    subjects: BTreeSet<String>,
}

/// Kernel-resolved pull-consumer fence for one runtime generation and grant epoch.
#[derive(Clone)]
pub struct RuntimeSubscribePermitV1 {
    registration_id: String,
    runtime_id: String,
    runtime_generation: u64,
    grant_epoch: u64,
    consumer: ConsumerSpecV1,
    contract: Option<ContractReferenceV1>,
}

impl NatsPasswordCredentialV1 {
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Result<Self, String> {
        let username = username.into();
        let password = password.into();
        (valid_credential_id(&username) && !password.is_empty() && password.len() <= 512)
            .then_some(Self {
                username,
                password: zeroize::Zeroizing::new(password),
            })
            .ok_or_else(|| "NATS password credential is invalid".to_owned())
    }

    pub(super) fn credentials(&self) -> (&str, &str) {
        (&self.username, self.password.as_str())
    }
}

impl RuntimeNatsIdentity {
    pub fn new(
        runtime_id: impl Into<String>,
        runtime_generation: u64,
        grant_epoch: u64,
    ) -> Result<Self, String> {
        let runtime_id = runtime_id.into();
        (valid_runtime_id(&runtime_id) && runtime_generation > 0 && grant_epoch > 0)
            .then_some(Self {
                runtime_id,
                runtime_generation,
                grant_epoch,
            })
            .ok_or_else(|| "NATS runtime identity is invalid".to_owned())
    }

    #[must_use]
    pub fn runtime_id(&self) -> &str {
        &self.runtime_id
    }

    #[must_use]
    pub const fn runtime_generation(&self) -> u64 {
        self.runtime_generation
    }

    #[must_use]
    pub const fn grant_epoch(&self) -> u64 {
        self.grant_epoch
    }
}

impl RuntimePublishPermitV1 {
    pub fn new(
        registration_id: impl Into<String>,
        runtime_id: impl Into<String>,
        runtime_generation: u64,
        grant_epoch: u64,
        subjects: Vec<DurableSubjectV1>,
    ) -> Result<Self, String> {
        let registration_id = registration_id.into();
        let runtime_id = runtime_id.into();
        let subjects: BTreeSet<String> = subjects
            .into_iter()
            .map(|subject| subject.as_str())
            .collect();
        (valid_runtime_id(&registration_id)
            && valid_runtime_id(&runtime_id)
            && runtime_generation > 0
            && grant_epoch > 0
            && !subjects.is_empty())
        .then_some(Self {
            registration_id,
            runtime_id,
            runtime_generation,
            grant_epoch,
            subjects,
        })
        .ok_or_else(|| "NATS runtime publish permit is invalid".to_owned())
    }

    pub(super) fn permits(&self, identity: &RuntimeNatsIdentity, subject: &str) -> bool {
        self.runtime_id == identity.runtime_id
            && self.runtime_generation == identity.runtime_generation
            && self.grant_epoch == identity.grant_epoch
            && self.subjects.contains(subject)
    }

    /// Returns whether this fenced permit includes one exact durable subject.
    #[must_use]
    pub fn permits_subject(&self, subject: &DurableSubjectV1) -> bool {
        self.subjects.contains(&subject.as_str())
    }

    /// Returns whether this permit contains exactly the supplied non-wildcard subjects.
    #[must_use]
    pub fn permits_exact_subjects(&self, subjects: &[DurableSubjectV1]) -> bool {
        let expected = subjects
            .iter()
            .map(DurableSubjectV1::as_str)
            .collect::<BTreeSet<_>>();
        self.subjects == expected
    }
}

impl RuntimeSubscribePermitV1 {
    pub fn new(
        registration_id: impl Into<String>,
        runtime_id: impl Into<String>,
        runtime_generation: u64,
        grant_epoch: u64,
        consumer: ConsumerSpecV1,
    ) -> Result<Self, String> {
        let registration_id = registration_id.into();
        let runtime_id = runtime_id.into();
        (valid_runtime_id(&registration_id)
            && valid_runtime_id(&runtime_id)
            && runtime_generation > 0
            && grant_epoch > 0)
            .then_some(Self {
                registration_id,
                runtime_id,
                runtime_generation,
                grant_epoch,
                consumer,
                contract: None,
            })
            .ok_or_else(|| "NATS runtime subscribe permit is invalid".to_owned())
    }

    #[must_use]
    pub fn consumer(&self) -> &ConsumerSpecV1 {
        &self.consumer
    }

    pub fn new_bound(
        registration_id: impl Into<String>,
        runtime_id: impl Into<String>,
        runtime_generation: u64,
        grant_epoch: u64,
        consumer: ConsumerSpecV1,
        contract: ContractReferenceV1,
    ) -> Result<Self, String> {
        (valid_contract_reference(&contract) && contract_subject_matches(&consumer, &contract))
            .then_some(())
            .ok_or_else(|| "NATS runtime subscription contract is invalid".to_owned())?;
        let mut permit = Self::new(
            registration_id,
            runtime_id,
            runtime_generation,
            grant_epoch,
            consumer,
        )?;
        permit.contract = Some(contract);
        Ok(permit)
    }

    #[must_use]
    pub fn contract(&self) -> Option<&ContractReferenceV1> {
        self.contract.as_ref()
    }

    pub(super) fn permits(&self, identity: &RuntimeNatsIdentity) -> bool {
        self.runtime_id == identity.runtime_id
            && self.runtime_generation == identity.runtime_generation
            && self.grant_epoch == identity.grant_epoch
    }
}

impl fmt::Debug for NatsPasswordCredentialV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("NatsPasswordCredentialV1")
            .field("username", &"[redacted]")
            .field("password", &"[redacted]")
            .finish()
    }
}

impl fmt::Debug for RuntimeNatsIdentity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RuntimeNatsIdentity")
            .field("runtime_id", &self.runtime_id)
            .field("runtime_generation", &self.runtime_generation)
            .field("grant_epoch", &self.grant_epoch)
            .finish()
    }
}

impl fmt::Debug for RuntimePublishPermitV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RuntimePublishPermitV1")
            .field("registration_id", &self.registration_id)
            .field("runtime_id", &self.runtime_id)
            .field("runtime_generation", &self.runtime_generation)
            .field("grant_epoch", &self.grant_epoch)
            .field("subject_count", &self.subjects.len())
            .finish()
    }
}

impl fmt::Debug for RuntimeSubscribePermitV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RuntimeSubscribePermitV1")
            .field("registration_id", &self.registration_id)
            .field("runtime_id", &self.runtime_id)
            .field("runtime_generation", &self.runtime_generation)
            .field("grant_epoch", &self.grant_epoch)
            .field("stream_kind", &self.consumer.stream_kind())
            .field("durable_name", &self.consumer.durable_name())
            .field("filter_subject", &self.consumer.filter_subject())
            .finish()
    }
}

fn valid_runtime_id(value: &str) -> bool {
    valid_credential_id(value)
}

fn valid_credential_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        })
}

fn valid_contract_reference(value: &ContractReferenceV1) -> bool {
    valid_credential_id(&value.owner)
        && valid_credential_id(&value.name)
        && value.major > 0
        && value.revision > 0
        && value.schema_sha256.len() == 32
}

fn contract_subject_matches(consumer: &ConsumerSpecV1, contract: &ContractReferenceV1) -> bool {
    consumer.filter_subject()
        == format!(
            "makosh.{}.v1.{}.{}.v{}",
            consumer.stream_kind().subject_token(),
            contract.owner,
            contract.name,
            contract.major
        )
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use makosh_runtime_protocol::v1::ContractReferenceV1;

    use super::*;
    use crate::topology::{ConsumerBudgetV1, StreamKindV1};

    #[test]
    fn bound_permit_rejects_a_contract_that_does_not_match_its_subject() {
        let consumer = ConsumerSpecV1::new(
            StreamKindV1::Observation,
            "consumer",
            "makosh.observation.v1.communications.communication_observed.v1",
            ConsumerBudgetV1::new(1, 1, Duration::from_secs(1)).expect("budget"),
        )
        .expect("consumer");
        let matching = ContractReferenceV1 {
            owner: "communications".to_owned(),
            name: "communication_observed".to_owned(),
            major: 1,
            revision: 1,
            schema_sha256: vec![7; 32],
        };
        assert!(
            RuntimeSubscribePermitV1::new_bound(
                "registration",
                "runtime",
                1,
                1,
                consumer.clone(),
                matching,
            )
            .is_ok()
        );
        assert!(
            RuntimeSubscribePermitV1::new_bound(
                "registration",
                "runtime",
                1,
                1,
                consumer,
                ContractReferenceV1 {
                    owner: "communications".to_owned(),
                    name: "other".to_owned(),
                    major: 1,
                    revision: 1,
                    schema_sha256: vec![7; 32],
                },
            )
            .is_err()
        );
    }

    #[test]
    fn publish_permit_exact_subject_check_rejects_missing_and_extra_authority() {
        let first = DurableSubjectV1::new(StreamKindV1::Result, "persons", "succeeded", 1)
            .expect("first subject");
        let second = DurableSubjectV1::new(StreamKindV1::Event, "persons", "changed", 1)
            .expect("second subject");
        let permit = RuntimePublishPermitV1::new(
            "registration",
            "runtime",
            1,
            1,
            vec![first.clone(), second.clone()],
        )
        .expect("permit");
        assert!(permit.permits_exact_subjects(&[first.clone(), second.clone()]));
        assert!(!permit.permits_exact_subjects(std::slice::from_ref(&first)));
        let extra = DurableSubjectV1::new(StreamKindV1::Event, "persons", "extra", 1)
            .expect("extra subject");
        assert!(!permit.permits_exact_subjects(&[first, second, extra]));
    }
}
