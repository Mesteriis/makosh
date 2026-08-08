use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::{Postgres, Transaction};

use super::WhatsappWebStore;
use super::evidence::link_whatsapp_entity_in_transaction;
use crate::integrations::whatsapp::client::errors::WhatsappWebError;
use crate::integrations::whatsapp::client::models::{NewWhatsappWebSession, WhatsappWebSession};
use crate::integrations::whatsapp::client::rows::row_to_whatsapp_web_session;
use crate::integrations::whatsapp::client::validation::validate_limit;
use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::store::ObservationStore;

async fn capture_whatsapp_session_observation(
    transaction: &mut Transaction<'_, Postgres>,
    session: &WhatsappWebSession,
    relationship_kind: &str,
    actor: &str,
    observed_at: DateTime<Utc>,
) -> Result<(), WhatsappWebError> {
    let observation = ObservationStore::capture_in_transaction(
        transaction,
        &NewObservation::new(
            "WHATSAPP_WEB_SESSION",
            ObservationOriginKind::LocalRuntime,
            observed_at,
            json!({
                "session_id": session.session_id,
                "account_id": session.account_id,
                "device_name": session.device_name,
                "companion_runtime": session.companion_runtime,
                "link_state": session.link_state,
                "local_state_path": session.local_state_path,
                "last_sync_at": session.last_sync_at,
                "metadata": session.metadata,
                "operation": relationship_kind,
            }),
            format!(
                "whatsapp-web-session://{}/{}",
                session.session_id, relationship_kind
            ),
        )
        .provenance(json!({
            "captured_by": actor,
            "operation": relationship_kind,
        })),
    )
    .await?;
    link_whatsapp_entity_in_transaction(
        transaction,
        &observation.observation_id,
        "whatsapp_web_session",
        session.session_id.clone(),
        relationship_kind,
        json!({
            "account_id": session.account_id,
            "link_state": session.link_state,
            "companion_runtime": session.companion_runtime,
        }),
    )
    .await?;
    Ok(())
}

impl WhatsappWebStore {
    pub async fn update_projection_link_state(
        pool: &sqlx::PgPool,
        account_id: &str,
        link_state: &str,
        observed_at: DateTime<Utc>,
    ) -> Result<(), WhatsappWebError> {
        sqlx::query("UPDATE whatsapp_web_sessions SET link_state = $2, last_sync_at = COALESCE($3, last_sync_at), updated_at = now() WHERE account_id = $1")
            .bind(account_id)
            .bind(link_state)
            .bind(observed_at)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn upsert_session(
        &self,
        session: &NewWhatsappWebSession,
    ) -> Result<WhatsappWebSession, WhatsappWebError> {
        session.validate()?;
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            INSERT INTO whatsapp_web_sessions (
                session_id, account_id, device_name, companion_runtime,
                link_state, local_state_path, last_sync_at, metadata, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, now())
            ON CONFLICT (account_id)
            DO UPDATE SET
                device_name = EXCLUDED.device_name,
                companion_runtime = EXCLUDED.companion_runtime,
                link_state = EXCLUDED.link_state,
                local_state_path = EXCLUDED.local_state_path,
                last_sync_at = EXCLUDED.last_sync_at,
                metadata = EXCLUDED.metadata,
                updated_at = now()
            RETURNING
                session_id, account_id, device_name, companion_runtime,
                link_state, local_state_path, last_sync_at, metadata,
                created_at, updated_at
            "#,
        )
        .bind(session.session_id.trim())
        .bind(session.account_id.trim())
        .bind(session.device_name.trim())
        .bind(session.companion_runtime.as_str())
        .bind(session.link_state.as_str())
        .bind(session.local_state_path.trim())
        .bind(session.last_sync_at)
        .bind(&session.metadata)
        .fetch_one(&mut *transaction)
        .await?;

        let stored = row_to_whatsapp_web_session(row)?;
        capture_whatsapp_session_observation(
            &mut transaction,
            &stored,
            "upsert",
            "whatsapp.client.store.upsert_session",
            stored.updated_at,
        )
        .await?;
        transaction.commit().await?;
        Ok(stored)
    }

    pub async fn list_sessions(
        &self,
        account_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<WhatsappWebSession>, WhatsappWebError> {
        let limit = validate_limit(limit)?;
        let account_id = account_id.map(str::trim).filter(|value| !value.is_empty());
        let rows = sqlx::query(
            r#"
            SELECT
                session_id, account_id, device_name, companion_runtime,
                link_state, local_state_path, last_sync_at, metadata,
                created_at, updated_at
            FROM whatsapp_web_sessions
            WHERE ($1::text IS NULL OR account_id = $1)
            ORDER BY updated_at DESC, session_id ASC
            LIMIT $2
            "#,
        )
        .bind(account_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_whatsapp_web_session).collect()
    }

    pub(in crate::integrations::whatsapp::client::store) async fn update_session_last_sync(
        &self,
        account_id: &str,
        last_sync_at: DateTime<Utc>,
    ) -> Result<(), WhatsappWebError> {
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            UPDATE whatsapp_web_sessions
            SET last_sync_at = GREATEST(COALESCE(last_sync_at, $2), $2),
                updated_at = now()
            WHERE account_id = $1
            RETURNING
                session_id, account_id, device_name, companion_runtime,
                link_state, local_state_path, last_sync_at, metadata,
                created_at, updated_at
            "#,
        )
        .bind(account_id.trim())
        .bind(last_sync_at)
        .fetch_optional(&mut *transaction)
        .await?;
        if let Some(row) = row {
            let session = row_to_whatsapp_web_session(row)?;
            capture_whatsapp_session_observation(
                &mut transaction,
                &session,
                "sync_progress",
                "whatsapp.client.store.update_session_last_sync",
                session.updated_at,
            )
            .await?;
        }
        transaction.commit().await?;
        Ok(())
    }

    pub(in crate::integrations::whatsapp) async fn update_session_link_state(
        &self,
        account_id: &str,
        link_state: &str,
        actor: &str,
    ) -> Result<(), WhatsappWebError> {
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            UPDATE whatsapp_web_sessions
            SET link_state = $2,
                updated_at = now()
            WHERE account_id = $1
            RETURNING
                session_id, account_id, device_name, companion_runtime,
                link_state, local_state_path, last_sync_at, metadata,
                created_at, updated_at
            "#,
        )
        .bind(account_id.trim())
        .bind(link_state.trim())
        .fetch_optional(&mut *transaction)
        .await?;
        if let Some(row) = row {
            let session = row_to_whatsapp_web_session(row)?;
            capture_whatsapp_session_observation(
                &mut transaction,
                &session,
                "link_state_update",
                actor,
                session.updated_at,
            )
            .await?;
        }
        transaction.commit().await?;
        Ok(())
    }
}
