use makosh_scheduler_protocol::{JobKindV1, JobRunIdV1, OpaqueOwnerJobScopeV1};
use sha2::{Digest, Sha256};

pub const TELEGRAM_CALLS_REALTIME_BACKFILL_JOB_OWNER_V1: &str = "telegram";
pub const TELEGRAM_CALLS_REALTIME_BACKFILL_JOB_NAME_V1: &str = "calls_realtime_backfill";
pub const TELEGRAM_CALLS_REALTIME_BACKFILL_JOB_MAJOR_V1: u16 = 1;
pub const TELEGRAM_CALLS_REALTIME_BACKFILL_SCOPE_V1: &str = "owner";
pub const TELEGRAM_CALLS_REALTIME_BACKFILL_BATCH_SIZE_V1: u32 = 256;
pub const TELEGRAM_CALLS_REALTIME_BACKFILL_MAX_BATCHES_PER_BOOT_V1: u32 = 4_096;
pub const TELEGRAM_CALLS_REALTIME_BACKFILL_LEASE_TTL_MILLIS_V1: i64 = 60_000;
const _: () = assert!(TELEGRAM_CALLS_REALTIME_BACKFILL_BATCH_SIZE_V1 <= 256);
const _: () = assert!(TELEGRAM_CALLS_REALTIME_BACKFILL_MAX_BATCHES_PER_BOOT_V1 <= 4_096);

const RUN_ID_LABEL: &[u8] = b"makosh.telegram.calls.realtime-backfill.run.v1";
const MESSAGE_ID_LABEL: &[u8] = b"makosh.telegram.calls.realtime-backfill.message.v1";
const IDEMPOTENCY_KEY_LABEL: &[u8] = b"makosh.telegram.calls.realtime-backfill.idempotency.v1";

#[must_use]
pub fn telegram_calls_realtime_backfill_job_kind_v1() -> JobKindV1 {
    JobKindV1::new(
        TELEGRAM_CALLS_REALTIME_BACKFILL_JOB_OWNER_V1.to_owned(),
        TELEGRAM_CALLS_REALTIME_BACKFILL_JOB_NAME_V1.to_owned(),
        TELEGRAM_CALLS_REALTIME_BACKFILL_JOB_MAJOR_V1,
    )
    .expect("Telegram Calls backfill JobKind is a validated constant")
}

#[must_use]
pub fn telegram_calls_realtime_backfill_scope_v1() -> OpaqueOwnerJobScopeV1 {
    OpaqueOwnerJobScopeV1::new(TELEGRAM_CALLS_REALTIME_BACKFILL_SCOPE_V1.to_owned())
        .expect("Telegram Calls backfill scope is a validated constant")
}

#[must_use]
pub fn telegram_calls_realtime_backfill_run_id_v1() -> JobRunIdV1 {
    JobRunIdV1::new(identifier(RUN_ID_LABEL))
        .expect("Telegram Calls backfill run identity is non-zero")
}

#[must_use]
pub fn telegram_calls_realtime_backfill_message_id_v1() -> [u8; 16] {
    identifier(MESSAGE_ID_LABEL)
}

#[must_use]
pub fn telegram_calls_realtime_backfill_idempotency_key_v1() -> [u8; 32] {
    Sha256::digest(IDEMPOTENCY_KEY_LABEL).into()
}

#[must_use]
pub fn telegram_calls_realtime_backfill_lease_expiry_v1(
    accepted_at_unix_millis: i64,
) -> Option<i64> {
    accepted_at_unix_millis
        .checked_add(TELEGRAM_CALLS_REALTIME_BACKFILL_LEASE_TTL_MILLIS_V1)
        .filter(|expiry| *expiry > accepted_at_unix_millis)
}

fn identifier(label: &[u8]) -> [u8; 16] {
    let digest = Sha256::digest(label);
    let mut identifier = [0_u8; 16];
    identifier.copy_from_slice(&digest[..16]);
    identifier
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backfill_identity_and_policy_are_stable_and_bounded() {
        let job = telegram_calls_realtime_backfill_job_kind_v1();
        assert_eq!(job.owner(), "telegram");
        assert_eq!(job.name(), "calls_realtime_backfill");
        assert_eq!(job.major(), 1);
        assert_eq!(telegram_calls_realtime_backfill_scope_v1().value(), "owner");
        assert_eq!(
            telegram_calls_realtime_backfill_lease_expiry_v1(1_000),
            Some(61_000)
        );
    }

    #[test]
    fn backfill_durable_identifiers_are_exact_and_non_zero() {
        let run_id = telegram_calls_realtime_backfill_run_id_v1().bytes();
        let message_id = telegram_calls_realtime_backfill_message_id_v1();
        let idempotency_key = telegram_calls_realtime_backfill_idempotency_key_v1();
        assert!(run_id.iter().any(|byte| *byte != 0));
        assert!(message_id.iter().any(|byte| *byte != 0));
        assert!(idempotency_key.iter().any(|byte| *byte != 0));
        assert_ne!(run_id, message_id);
    }
}
