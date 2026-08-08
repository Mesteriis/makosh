use makosh_signal_hub_api::policies::{SignalPolicy, SignalPolicyMode};
use uuid::Uuid;

use super::{SignalHubError, SignalHubStore, SignalProfilePolicy};

impl SignalHubStore {
    pub async fn create_policy(&self, policy: &SignalPolicy) -> Result<Uuid, SignalHubError> {
        let id = Uuid::now_v7();
        let connection_id = policy_connection_id(policy.connection_id.as_deref())?;

        sqlx::query(
            r#"
            INSERT INTO signal_policies (
                id,
                scope,
                source_code,
                connection_id,
                event_pattern,
                mode,
                reason,
                created_by,
                expires_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, 'makosh-frontend', $8)
            "#,
        )
        .bind(id)
        .bind(policy.scope.as_str())
        .bind(&policy.source_code)
        .bind(connection_id)
        .bind(&policy.event_pattern)
        .bind(policy.mode.as_str())
        .bind(&policy.reason)
        .bind(policy.expires_at)
        .execute(&self.pool)
        .await?;

        Ok(id)
    }

    pub async fn expire_matching_policies(
        &self,
        policy: &SignalPolicy,
        modes: &[SignalPolicyMode],
    ) -> Result<u64, SignalHubError> {
        let connection_id = optional_policy_connection_id(policy.connection_id.as_deref())?;
        let mode_values: Vec<&str> = modes.iter().map(SignalPolicyMode::as_str).collect();
        let result = sqlx::query(
            r#"
            UPDATE signal_policies
            SET expires_at = now()
            WHERE (expires_at IS NULL OR expires_at > now())
              AND mode = ANY($1)
              AND scope = $2
              AND (
                ($3::text IS NULL AND source_code IS NULL)
                OR source_code = $3
              )
              AND (
                ($4::uuid IS NULL AND connection_id IS NULL)
                OR connection_id = $4
              )
              AND (
                ($5::text IS NULL AND event_pattern IS NULL)
                OR event_pattern = $5
              )
            "#,
        )
        .bind(mode_values)
        .bind(policy.scope.as_str())
        .bind(&policy.source_code)
        .bind(connection_id)
        .bind(&policy.event_pattern)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    pub async fn create_profile_managed_policy(
        &self,
        profile_code: &str,
        policy: &SignalProfilePolicy,
    ) -> Result<Uuid, SignalHubError> {
        let id = Uuid::now_v7();
        let connection_id = policy_connection_id(policy.connection_id.as_deref())?;

        sqlx::query(
            r#"
            INSERT INTO signal_policies (
                id,
                scope,
                source_code,
                connection_id,
                event_pattern,
                mode,
                reason,
                created_by,
                expires_at,
                metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NULL, $9)
            "#,
        )
        .bind(id)
        .bind(policy.scope.as_str())
        .bind(&policy.source_code)
        .bind(connection_id)
        .bind(&policy.event_pattern)
        .bind(policy.mode.as_str())
        .bind(&policy.reason)
        .bind(format!("signal_profile:{profile_code}"))
        .bind(serde_json::json!({
            "managed_by": "signal_profile",
            "profile_code": profile_code,
        }))
        .execute(&self.pool)
        .await?;

        Ok(id)
    }

    pub async fn expire_managed_profile_policies(&self) -> Result<u64, SignalHubError> {
        let result = sqlx::query(
            r#"
            UPDATE signal_policies
            SET expires_at = now()
            WHERE (metadata->>'managed_by') = 'signal_profile'
              AND (expires_at IS NULL OR expires_at > now())
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }
}

fn policy_connection_id(value: Option<&str>) -> Result<Option<Uuid>, SignalHubError> {
    value
        .map(|value| {
            Uuid::parse_str(value)
                .map_err(|_| SignalHubError::InvalidConnectionId(value.to_owned()))
        })
        .transpose()
}

fn optional_policy_connection_id(value: Option<&str>) -> Result<Option<Uuid>, SignalHubError> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| {
            Uuid::parse_str(value)
                .map_err(|_| SignalHubError::InvalidConnectionId(value.to_owned()))
        })
        .transpose()
}
