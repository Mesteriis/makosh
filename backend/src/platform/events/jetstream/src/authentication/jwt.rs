//! Short-lived NATS user JWT issuance fenced to one runtime generation.

use std::collections::BTreeSet;
use std::fmt;

use makosh_events_protocol::RuntimeNatsJwtCredentialV1;
use nats_jwt::{KeyPair, Token};
use zeroize::Zeroizing;

use crate::vault::NatsRuntimeCredentialFenceV1;
use crate::{subjects::DurableSubjectV1, topology::StreamKindV1};

const MAX_TTL_SECONDS: u64 = 600;
const MAX_SUBJECTS_PER_DIRECTION: usize = 64;
const MAX_NATS_PAYLOAD_BYTES: i64 = 262_144;
const MAX_NATS_DATA_BYTES: i64 = 524_288;
const RESPONSE_INBOX_SUBJECT: &str = "_INBOX.>";
const MAX_CONSUMER_GRANTS: usize = 64;
// A pull fetch holds its delivery inbox while the JetStream request receives its
// acknowledgement. No other runtime subscriptions are admitted.
const MAX_RUNTIME_RESPONSE_SUBSCRIPTIONS: i64 = 2;

/// Exact declared publish and subscribe permissions for one runtime JWT.
pub struct NatsJwtPermissionSetV1 {
    publish_subjects: BTreeSet<String>,
    consumer_grants: BTreeSet<NatsJwtConsumerGrantV1>,
}

/// Exact pull-consumer operation grant, including its JetStream durable name.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct NatsJwtConsumerGrantV1 {
    stream_kind: StreamKindV1,
    durable_name: String,
    filter_subject: String,
}

/// Account signing authority that issues short-lived runtime user JWTs.
pub struct RuntimeNatsJwtIssuerV1 {
    account_public_key: String,
    account_signing_key: KeyPair,
}

impl NatsJwtPermissionSetV1 {
    pub fn new(
        publish_subjects: Vec<DurableSubjectV1>,
        consumer_grants: Vec<NatsJwtConsumerGrantV1>,
    ) -> Result<Self, NatsJwtIssueErrorV1> {
        let publish_subjects = canonical_subjects(publish_subjects)?;
        let grant_count = consumer_grants.len();
        let consumer_grants: BTreeSet<_> = consumer_grants.into_iter().collect();
        (grant_count == consumer_grants.len()
            && grant_count <= MAX_CONSUMER_GRANTS
            && (!publish_subjects.is_empty() || !consumer_grants.is_empty()))
        .then_some(Self {
            publish_subjects,
            consumer_grants,
        })
        .ok_or(NatsJwtIssueErrorV1::InvalidPermissions)
    }

    #[must_use]
    pub fn publish_count(&self) -> usize {
        self.publish_subjects.len()
    }

    #[must_use]
    pub fn consumer_count(&self) -> usize {
        self.consumer_grants.len()
    }
}

impl NatsJwtConsumerGrantV1 {
    pub fn new(
        filter_subject: DurableSubjectV1,
        durable_name: impl Into<String>,
    ) -> Result<Self, NatsJwtIssueErrorV1> {
        let durable_name = durable_name.into();
        valid_durable_name(&durable_name)
            .then_some(Self {
                stream_kind: filter_subject.kind(),
                durable_name,
                filter_subject: filter_subject.as_str(),
            })
            .ok_or(NatsJwtIssueErrorV1::InvalidPermissions)
    }

    fn jetstream_publish_subjects(&self) -> [String; 4] {
        let stream = self.stream_kind.stream_name();
        [
            format!("$JS.API.STREAM.INFO.{stream}"),
            format!("$JS.API.CONSUMER.INFO.{stream}.{}", self.durable_name),
            format!("$JS.API.CONSUMER.MSG.NEXT.{stream}.{}", self.durable_name),
            format!("$JS.ACK.{stream}.{}.>", self.durable_name),
        ]
    }
}

impl RuntimeNatsJwtIssuerV1 {
    pub fn from_account_signing_seed(
        account_public_key: impl Into<String>,
        account_signing_seed: impl Into<String>,
    ) -> Result<Self, NatsJwtIssueErrorV1> {
        let account_public_key = account_public_key.into();
        let account_signing_seed = Zeroizing::new(account_signing_seed.into());
        let account_signing_key = KeyPair::from_seed(account_signing_seed.as_str())
            .map_err(|_| NatsJwtIssueErrorV1::InvalidIssuer)?;
        (is_account_key(&account_public_key) && is_account_key(&account_signing_key.public_key()))
            .then_some(Self {
                account_public_key,
                account_signing_key,
            })
            .ok_or(NatsJwtIssueErrorV1::InvalidIssuer)
    }

    pub fn issue_runtime_credential(
        &self,
        fence: &NatsRuntimeCredentialFenceV1,
        permissions: NatsJwtPermissionSetV1,
        now_unix_seconds: u64,
        ttl_seconds: u64,
    ) -> Result<RuntimeNatsJwtCredentialV1, NatsJwtIssueErrorV1> {
        let expires_at_unix_seconds = bounded_expiry(now_unix_seconds, ttl_seconds)?;
        let user_key = KeyPair::new_user();
        let user_public_key = user_key.public_key();
        let user_seed = Zeroizing::new(
            user_key
                .seed()
                .map_err(|_| NatsJwtIssueErrorV1::KeyMaterialUnavailable)?,
        );
        let token = configured_user_token(
            &self.account_public_key,
            &self.account_signing_key,
            fence,
            permissions,
            &user_public_key,
            expires_at_unix_seconds,
        );
        RuntimeNatsJwtCredentialV1::new(
            token,
            user_seed.to_string(),
            user_public_key,
            expires_at_unix_seconds,
        )
        .map_err(|_| NatsJwtIssueErrorV1::KeyMaterialUnavailable)
    }
}

impl fmt::Debug for NatsJwtPermissionSetV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("NatsJwtPermissionSetV1")
            .field("publish_count", &self.publish_subjects.len())
            .field("consumer_count", &self.consumer_grants.len())
            .finish()
    }
}

impl fmt::Debug for RuntimeNatsJwtIssuerV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RuntimeNatsJwtIssuerV1")
            .field("account_public_key", &"[redacted]")
            .field("account_signing_key", &"[redacted]")
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NatsJwtIssueErrorV1 {
    InvalidIssuer,
    InvalidPermissions,
    InvalidTtl,
    KeyMaterialUnavailable,
    ClockUnavailable,
    Expired,
}

fn canonical_subjects(
    subjects: Vec<DurableSubjectV1>,
) -> Result<BTreeSet<String>, NatsJwtIssueErrorV1> {
    let subject_count = subjects.len();
    let subjects: BTreeSet<String> = subjects
        .into_iter()
        .map(|subject| subject.as_str())
        .collect();
    (subject_count == subjects.len() && subject_count <= MAX_SUBJECTS_PER_DIRECTION)
        .then_some(subjects)
        .ok_or(NatsJwtIssueErrorV1::InvalidPermissions)
}

fn bounded_expiry(now_unix_seconds: u64, ttl_seconds: u64) -> Result<u64, NatsJwtIssueErrorV1> {
    (ttl_seconds > 0 && ttl_seconds <= MAX_TTL_SECONDS)
        .then_some(())
        .ok_or(NatsJwtIssueErrorV1::InvalidTtl)?;
    let expires_at_unix_seconds = now_unix_seconds
        .checked_add(ttl_seconds)
        .ok_or(NatsJwtIssueErrorV1::InvalidTtl)?;
    (expires_at_unix_seconds <= i64::MAX as u64)
        .then_some(expires_at_unix_seconds)
        .ok_or(NatsJwtIssueErrorV1::InvalidTtl)
}

fn configured_user_token(
    account_public_key: &str,
    account_signing_key: &KeyPair,
    fence: &NatsRuntimeCredentialFenceV1,
    permissions: NatsJwtPermissionSetV1,
    user_public_key: &str,
    expires_at_unix_seconds: u64,
) -> String {
    let name = format!(
        "makosh-{}-{}-g{}-e{}",
        fence.registration_id(),
        fence.runtime_instance_id(),
        fence.runtime_generation(),
        fence.grant_epoch(),
    );
    let mut token = Token::new_user(account_public_key, user_public_key)
        .name(name)
        .bearer_token(false)
        .max_subscriptions(MAX_RUNTIME_RESPONSE_SUBSCRIPTIONS)
        .max_payload(MAX_NATS_PAYLOAD_BYTES)
        .max_data(MAX_NATS_DATA_BYTES)
        .expires(expires_at_unix_seconds as i64);
    for subject in permissions.publish_subjects {
        token = token.allow_publish(subject);
    }
    for grant in permissions.consumer_grants {
        for subject in grant.jetstream_publish_subjects() {
            token = token.allow_publish(subject);
        }
    }
    token
        .allow_subscribe(RESPONSE_INBOX_SUBJECT)
        .sign(account_signing_key)
}

fn valid_durable_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        })
}

fn is_account_key(value: &str) -> bool {
    value.len() == 56
        && value.starts_with('A')
        && value
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
}
