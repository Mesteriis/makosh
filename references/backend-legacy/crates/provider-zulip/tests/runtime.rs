use chrono::Utc;
use makosh_provider_api::{
    CredentialLease, ProviderCommandEnvelope, ProviderCommandInput, ProviderId, ProviderRuntimePort,
};
use makosh_provider_zulip::runtime::{ZulipInProcessRuntime, ZulipRuntimeConfig};
use serde_json::json;

fn command(account_id: &str, lease_epoch: u64) -> ProviderCommandEnvelope {
    let issued_at = Utc::now();
    ProviderCommandEnvelope::try_from(ProviderCommandInput::new(
        "command-1",
        "idempotency-1",
        ProviderId::parse("zulip").expect("provider id"),
        account_id,
        issued_at,
        issued_at + chrono::Duration::minutes(5),
        0,
        lease_epoch,
        json!({
            "command_kind": "send_stream_message",
            "payload": {"stream": "general", "topic": "test", "content": "hello"}
        }),
    ))
    .expect("command")
}

fn lease(account_id: &str, epoch: u64) -> CredentialLease {
    let issued_at = Utc::now();
    CredentialLease::new(
        "zulip",
        account_id,
        "command_execution",
        epoch,
        issued_at,
        issued_at + chrono::Duration::minutes(5),
        b"api-key",
    )
    .expect("lease")
}

fn runtime() -> ZulipInProcessRuntime {
    ZulipInProcessRuntime::new(
        ZulipRuntimeConfig::new(
            "account-1",
            "https://zulip.example.test",
            "bot@example.test",
        )
        .expect("config"),
    )
}

#[tokio::test]
async fn runtime_rejects_an_account_mismatch_before_provider_call() {
    let error = runtime()
        .execute(&command("account-2", 7), lease("account-2", 7))
        .await
        .expect_err("account mismatch must fail before transport setup");

    assert_eq!(error.code, "account_mismatch");
    assert!(!error.retryable);
}

#[tokio::test]
async fn runtime_rejects_a_stale_credential_lease_before_provider_call() {
    let error = runtime()
        .execute(&command("account-1", 8), lease("account-1", 7))
        .await
        .expect_err("stale lease must fail before transport setup");

    assert_eq!(error.code, "credential_lease_mismatch");
    assert!(!error.retryable);
}

#[tokio::test]
async fn runtime_rejects_an_expired_command_before_provider_call() {
    let issued_at = Utc::now() - chrono::Duration::minutes(2);
    let command = ProviderCommandEnvelope::try_from(ProviderCommandInput::new(
        "command-expired",
        "idempotency-expired",
        ProviderId::parse("zulip").expect("provider id"),
        "account-1",
        issued_at,
        issued_at + chrono::Duration::minutes(1),
        0,
        7,
        json!({
            "command_kind": "send_stream_message",
            "payload": {"stream": "general", "topic": "test", "content": "hello"}
        }),
    ))
    .expect("expired command envelope is structurally valid");

    let error = runtime()
        .execute(&command, lease("account-1", 7))
        .await
        .expect_err("expired command must fail before transport setup");

    assert_eq!(error.code, "command_deadline_expired");
    assert!(!error.retryable);
}
