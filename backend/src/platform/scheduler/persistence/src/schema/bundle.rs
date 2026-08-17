use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

const INITIAL_SCHEMA: &[u8] = include_bytes!("../../migrations/0001_scheduler_state.sql");
const PENDING_FIRES_SCHEMA: &[u8] =
    include_bytes!("../../migrations/0002_scheduler_pending_fires.sql");
const RUN_RETRIES_SCHEMA: &[u8] = include_bytes!("../../migrations/0003_scheduler_run_retries.sql");
const DISPATCHES_SCHEMA: &[u8] = include_bytes!("../../migrations/0004_scheduler_dispatches.sql");
const RUN_ACCEPTANCES_SCHEMA: &[u8] =
    include_bytes!("../../migrations/0005_scheduler_run_acceptances.sql");
const RUN_RESULTS_SCHEMA: &[u8] = include_bytes!("../../migrations/0006_scheduler_run_results.sql");
const JOB_CONTRACT_REVISION_SCHEMA: &[u8] =
    include_bytes!("../../migrations/0007_scheduler_job_contract_revision.sql");
const SCHEDULE_CONTROL_SCHEMA: &[u8] =
    include_bytes!("../../migrations/0008_scheduler_schedule_control.sql");
const SCHEDULE_CONTROL_AUTHORITY_SCHEMA: &[u8] =
    include_bytes!("../../migrations/0009_scheduler_schedule_control_authority.sql");
const PENDING_FIRES_SCHEDULE_SCHEMA: &[u8] =
    include_bytes!("../../migrations/0010_scheduler_pending_fires_schedule.sql");

/// Canonical Scheduler state schema admitted through the existing Storage bundle path.
#[must_use]
pub fn scheduler_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: 10,
        bundle_id: "scheduler_state".to_owned(),
        owner_id: "scheduler".to_owned(),
        steps: vec![
            step(1, "scheduler_state_initial", INITIAL_SCHEMA),
            step(2, "scheduler_pending_fires", PENDING_FIRES_SCHEMA),
            step(3, "scheduler_run_retries", RUN_RETRIES_SCHEMA),
            step(4, "scheduler_dispatches", DISPATCHES_SCHEMA),
            step(5, "scheduler_run_acceptances", RUN_ACCEPTANCES_SCHEMA),
            step(6, "scheduler_run_results", RUN_RESULTS_SCHEMA),
            step(
                7,
                "scheduler_job_contract_revision",
                JOB_CONTRACT_REVISION_SCHEMA,
            ),
            step(8, "scheduler_schedule_control", SCHEDULE_CONTROL_SCHEMA),
            step(
                9,
                "scheduler_schedule_control_authority",
                SCHEDULE_CONTROL_AUTHORITY_SCHEMA,
            ),
            step(
                10,
                "scheduler_pending_fires_schedule",
                PENDING_FIRES_SCHEDULE_SCHEMA,
            ),
        ],
    }
}

fn step(revision: u32, migration_id: &str, sql: &[u8]) -> StorageMigrationStepV1 {
    StorageMigrationStepV1 {
        revision,
        migration_id: migration_id.to_owned(),
        forward_sql_utf8: sql.to_vec(),
        sha256: Sha256::digest(sql).to_vec(),
    }
}
