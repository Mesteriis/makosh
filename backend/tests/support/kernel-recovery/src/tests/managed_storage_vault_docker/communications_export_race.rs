use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU64, Ordering},
};

use makosh_communications_runtime::admission::COMMUNICATIONS_EXPORT_SOURCE_BLOB_CAPABILITY_ID;
use makosh_runtime_protocol::v1::{
    BlobDataOperationV1, ManagedRuntimeBlobCustodyDelegationDeliveryV1,
    ManagedRuntimeBlobCustodyDelegationRequestV1, ManagedRuntimeBlobSessionDeliveryV1,
    ManagedRuntimeBlobSessionRequestV1,
};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgSslMode};
use zeroize::Zeroizing;

use super::communications_setup::COMMUNICATIONS_REGISTRATION;
use super::*;

struct ArmedRevisionMutationV1 {
    database_id: String,
    message_id: [u8; 16],
}

pub(super) struct CommunicationsExportRevisionRaceV1 {
    armed: Mutex<Option<ArmedRevisionMutationV1>>,
    fired_revision: AtomicU64,
}

impl CommunicationsExportRevisionRaceV1 {
    pub(super) const fn new() -> Self {
        Self {
            armed: Mutex::new(None),
            fired_revision: AtomicU64::new(0),
        }
    }

    pub(super) fn arm(&self, database_id: &str, message_id: &[u8]) {
        let message_id: [u8; 16] = message_id
            .try_into()
            .expect("race fixture canonical message ID");
        let mut armed = self.armed.lock().expect("lock export revision race");
        assert!(armed.is_none(), "export revision race is already armed");
        self.fired_revision.store(0, Ordering::Release);
        *armed = Some(ArmedRevisionMutationV1 {
            database_id: database_id.to_owned(),
            message_id,
        });
    }

    pub(super) fn fired_revision(&self) -> u64 {
        self.fired_revision.load(Ordering::Acquire)
    }

    fn mutate_if_armed(&self) -> Result<(), String> {
        let mutation = self
            .armed
            .lock()
            .map_err(|_| "export revision race state is unavailable".to_owned())?
            .take();
        let Some(mutation) = mutation else {
            return Ok(());
        };
        let revision = force_canonical_revision_change(mutation)?;
        self.fired_revision.store(revision, Ordering::Release);
        Ok(())
    }
}

pub(super) struct CommunicationsExportRaceBlobSessionHandlerV1 {
    inner: BlobSessionHandlerV1,
    race: Arc<CommunicationsExportRevisionRaceV1>,
}

impl CommunicationsExportRaceBlobSessionHandlerV1 {
    pub(super) fn new(
        store: Arc<SqliteControlStore>,
        relay: crate::runtime::lifecycle::supervisor::ManagedRuntimeRelayPort,
        data_dir: PathBuf,
        race: Arc<CommunicationsExportRevisionRaceV1>,
    ) -> Self {
        Self {
            inner: BlobSessionHandlerV1::new(store, relay, data_dir),
            race,
        }
    }
}

impl ManagedRuntimeBlobSessionHandler for CommunicationsExportRaceBlobSessionHandlerV1 {
    fn issue_blob_session(
        &self,
        expectation: &ManagedRuntimeExpectation,
        request: ManagedRuntimeBlobSessionRequestV1,
    ) -> Result<ManagedRuntimeBlobSessionDeliveryV1, String> {
        if expectation.registration_id() == COMMUNICATIONS_REGISTRATION
            && request.capability_id == COMMUNICATIONS_EXPORT_SOURCE_BLOB_CAPABILITY_ID
            && request.operation == BlobDataOperationV1::BlobDataOperationWriteV1 as u32
        {
            self.race.mutate_if_armed()?;
        }
        self.inner.issue_blob_session(expectation, request)
    }

    fn delegate_blob_custody(
        &self,
        expectation: &ManagedRuntimeExpectation,
        request: ManagedRuntimeBlobCustodyDelegationRequestV1,
    ) -> Result<ManagedRuntimeBlobCustodyDelegationDeliveryV1, String> {
        self.inner.delegate_blob_custody(expectation, request)
    }
}

pub(super) fn communications_export_rejection_code(database_id: &str, export_id: &[u8; 16]) -> u16 {
    let runtime = tokio::runtime::Runtime::new().expect("export race query runtime");
    runtime.block_on(async {
        let pool = admin_pool(database_id)
            .await
            .expect("connect export race conformance database");
        let code = sqlx::query_scalar::<_, i16>(
            "SELECT rejection_code
             FROM makosh_data.communications_export_jobs
             WHERE export_id = $1",
        )
        .bind(export_id.as_slice())
        .fetch_one(&pool)
        .await
        .expect("read terminal export rejection code");
        pool.close().await;
        u16::try_from(code).expect("positive export rejection code")
    })
}

fn force_canonical_revision_change(mutation: ArmedRevisionMutationV1) -> Result<u64, String> {
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|_| "export revision race runtime is unavailable".to_owned())?;
        runtime.block_on(async move {
            let pool = admin_pool(&mutation.database_id).await?;
            let revision = sqlx::query_scalar::<_, i64>(
                "UPDATE makosh_data.communications_messages
                 SET canonical_revision = canonical_revision + 1
                 WHERE message_id = $1 AND lifecycle_state = 1
                 RETURNING canonical_revision",
            )
            .bind(mutation.message_id.as_slice())
            .fetch_one(&pool)
            .await
            .map_err(|_| "export revision race mutation failed".to_owned())?;
            pool.close().await;
            u64::try_from(revision)
                .ok()
                .filter(|value| *value > 1)
                .ok_or_else(|| "export revision race produced an invalid revision".to_owned())
        })
    })
    .join()
    .map_err(|_| "export revision race worker failed".to_owned())?
}

async fn admin_pool(database_id: &str) -> Result<sqlx::PgPool, String> {
    let password = Zeroizing::new(
        std::fs::read_to_string(required(
            "MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_PASSWORD_FILE",
        ))
        .map_err(|_| "disposable PostgreSQL credential is unavailable".to_owned())?
        .trim()
        .to_owned(),
    );
    let port = required("MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_PORT")
        .parse()
        .map_err(|_| "disposable PostgreSQL port is invalid".to_owned())?;
    let options = PgConnectOptions::new()
        .host(&required("MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_HOST"))
        .port(port)
        .username("makosh_postgres_admin")
        .password(password.as_str())
        .database(database_id)
        .ssl_mode(PgSslMode::Disable);
    PgPoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .map_err(|_| "disposable PostgreSQL connection is unavailable".to_owned())
}
