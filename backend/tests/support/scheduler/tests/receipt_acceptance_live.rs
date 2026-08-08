//! Disposable PostgreSQL proof that only the current published Scheduler dispatch can start a run.

#[path = "support/receipt_fixture.rs"]
mod fixture;

use makosh_scheduler_persistence::{
    SchedulerRunAcceptanceOutcomeV1, SchedulerRunAcceptanceV1, SchedulerRunClaimErrorV1,
    SchedulerRunTerminalResultOutcomeV1, SchedulerRunTerminalResultV1,
};
use makosh_scheduler_protocol::v1::JobRunOutcomeV1;

use fixture::{
    CLAIMED_AT, acceptance_count, active_runs, failed_result, pending_published_dispatch, receipt,
    retry_due_at, retryable_failure_result, run_state, terminal_result,
};

#[tokio::test]
#[ignore = "requires the disposable Scheduler PostgreSQL contour"]
async fn matching_published_acceptance_starts_the_run_once() {
    let (pool, store, claim) = pending_published_dispatch().await;
    let acceptance = SchedulerRunAcceptanceV1::try_from(&receipt(&claim)).expect("acceptance");

    assert_eq!(
        store.accept_receipt(&acceptance).await,
        Ok(SchedulerRunAcceptanceOutcomeV1::Applied)
    );
    assert_eq!(
        store.accept_receipt(&acceptance).await,
        Ok(SchedulerRunAcceptanceOutcomeV1::AlreadyApplied)
    );
    assert_eq!(run_state(&pool).await, "running");
    assert_eq!(acceptance_count(&pool).await, 1);
}

#[tokio::test]
#[ignore = "requires the disposable Scheduler PostgreSQL contour"]
async fn foreign_or_stale_acceptance_cannot_start_the_run() {
    let (pool, store, claim) = pending_published_dispatch().await;
    let mut foreign_receipt = receipt(&claim);
    foreign_receipt.command_message_id = vec![9; 16];
    let foreign = SchedulerRunAcceptanceV1::try_from(&foreign_receipt).expect("foreign receipt");

    assert_eq!(
        store.accept_receipt(&foreign).await,
        Err(SchedulerRunClaimErrorV1::Denied)
    );
    let mut stale_receipt = receipt(&claim);
    stale_receipt.lease.as_mut().expect("lease").epoch = 2;
    let stale = SchedulerRunAcceptanceV1::try_from(&stale_receipt).expect("stale receipt");

    assert_eq!(
        store.accept_receipt(&stale).await,
        Err(SchedulerRunClaimErrorV1::Denied)
    );
    assert_eq!(run_state(&pool).await, "dispatched");
    assert_eq!(acceptance_count(&pool).await, 0);
}

#[tokio::test]
#[ignore = "requires the disposable Scheduler PostgreSQL contour"]
async fn accepted_terminal_result_releases_its_slot_once() {
    let (pool, store, claim) = pending_published_dispatch().await;
    store
        .accept_receipt(&SchedulerRunAcceptanceV1::try_from(&receipt(&claim)).expect("acceptance"))
        .await
        .expect("acceptance");
    let result = terminal_result(&claim);

    assert_eq!(
        store.finish_receipt(&result).await,
        Ok(SchedulerRunTerminalResultOutcomeV1::Applied)
    );
    assert_eq!(
        store.finish_receipt(&result).await,
        Ok(SchedulerRunTerminalResultOutcomeV1::AlreadyApplied)
    );
    assert_eq!(run_state(&pool).await, "finished");
    assert_eq!(active_runs(&pool).await, 0);
}

#[tokio::test]
#[ignore = "requires the disposable Scheduler PostgreSQL contour"]
async fn terminal_before_acceptance_waits_for_cross_stream_predecessor() {
    let (pool, store, claim) = pending_published_dispatch().await;
    let result = terminal_result(&claim);

    assert_eq!(
        store.finish_receipt(&result).await,
        Err(SchedulerRunClaimErrorV1::PendingMissing)
    );
    assert_eq!(run_state(&pool).await, "dispatched");
    store
        .accept_receipt(&SchedulerRunAcceptanceV1::try_from(&receipt(&claim)).expect("acceptance"))
        .await
        .expect("acceptance");
    assert_eq!(
        store.finish_receipt(&result).await,
        Ok(SchedulerRunTerminalResultOutcomeV1::Applied)
    );
    assert_eq!(run_state(&pool).await, "finished");
    assert_eq!(active_runs(&pool).await, 0);
}

#[tokio::test]
#[ignore = "requires the disposable Scheduler PostgreSQL contour"]
async fn retryable_failure_receipt_enters_retry_wait_and_releases_its_slot_once() {
    let (pool, store, claim) = pending_published_dispatch().await;
    store
        .accept_receipt(&SchedulerRunAcceptanceV1::try_from(&receipt(&claim)).expect("acceptance"))
        .await
        .expect("acceptance");
    let result = retryable_failure_result(&claim);

    assert_eq!(
        store.finish_receipt(&result).await,
        Ok(SchedulerRunTerminalResultOutcomeV1::Applied)
    );
    assert_eq!(
        store.finish_receipt(&result).await,
        Ok(SchedulerRunTerminalResultOutcomeV1::AlreadyApplied)
    );
    assert_eq!(run_state(&pool).await, "retry_wait");
    assert_eq!(retry_due_at(&pool).await, Some(CLAIMED_AT + 1_001));
    assert_eq!(active_runs(&pool).await, 0);
}

#[tokio::test]
#[ignore = "requires the disposable Scheduler PostgreSQL contour"]
async fn non_retryable_failure_receipt_is_terminal_and_releases_its_slot_once() {
    let (pool, store, claim) = pending_published_dispatch().await;
    store
        .accept_receipt(&SchedulerRunAcceptanceV1::try_from(&receipt(&claim)).expect("acceptance"))
        .await
        .expect("acceptance");
    let result = failed_result(&claim);

    assert_eq!(
        store.finish_receipt(&result).await,
        Ok(SchedulerRunTerminalResultOutcomeV1::Applied)
    );
    assert_eq!(
        store.finish_receipt(&result).await,
        Ok(SchedulerRunTerminalResultOutcomeV1::AlreadyApplied)
    );
    assert_eq!(run_state(&pool).await, "failed");
    assert_eq!(retry_due_at(&pool).await, None);
    assert_eq!(active_runs(&pool).await, 0);
}

#[tokio::test]
#[ignore = "requires the disposable Scheduler PostgreSQL contour"]
async fn stale_terminal_result_cannot_release_the_current_slot() {
    let (pool, store, claim) = pending_published_dispatch().await;
    store
        .accept_receipt(&SchedulerRunAcceptanceV1::try_from(&receipt(&claim)).expect("acceptance"))
        .await
        .expect("acceptance");
    let mut value = receipt(&claim);
    value.outcome = JobRunOutcomeV1::Succeeded as i32;
    value.lease.as_mut().expect("lease").epoch = 2;
    let stale = SchedulerRunTerminalResultV1::try_from(&value).expect("stale result");

    assert_eq!(
        store.finish_receipt(&stale).await,
        Err(SchedulerRunClaimErrorV1::Denied)
    );
    assert_eq!(run_state(&pool).await, "running");
    assert_eq!(active_runs(&pool).await, 1);
}
