use makosh_telegram_automation_core::{
    AutomationError, AutomationPolicy, AutomationPolicyDraft, AutomationPreviewReceipt,
    AutomationPreviewRequest, AutomationTemplate, AutomationTemplateDraft, render_preview,
    validate_identifier,
};
use sqlx::{PgPool, Postgres, Row, Transaction};

#[derive(Clone)]
pub struct TelegramAutomationPersistence {
    pool: PgPool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TelegramAutomationPersistenceError {
    Database,
    InvalidRow,
    MissingTemplate,
    MissingAccount,
    MissingPolicy,
    RevisionConflict,
    IdempotencyConflict,
    Core(AutomationError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PersistedMutation<T> {
    Applied { value: T, response_payload: Vec<u8> },
    Replayed { response_payload: Vec<u8> },
}

impl TelegramAutomationPersistence {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[cfg(any(test, feature = "conformance-test-support"))]
    pub async fn apply_schema_for_conformance(
        &self,
    ) -> Result<(), TelegramAutomationPersistenceError> {
        sqlx::raw_sql(crate::schema::TELEGRAM_AUTOMATION_SCHEMA_V1)
            .execute(&self.pool)
            .await
            .map_err(|_| TelegramAutomationPersistenceError::Database)?;
        Ok(())
    }

    pub async fn template(
        &self,
        template_id: &str,
    ) -> Result<Option<AutomationTemplate>, TelegramAutomationPersistenceError> {
        load_template(&self.pool, template_id).await
    }

    pub async fn policy(
        &self,
        policy_id: &str,
    ) -> Result<Option<AutomationPolicy>, TelegramAutomationPersistenceError> {
        load_policy(&self.pool, policy_id).await
    }

    pub async fn preview_receipt(
        &self,
        preview_id: &str,
    ) -> Result<Option<AutomationPreviewReceipt>, TelegramAutomationPersistenceError> {
        let row = sqlx::query(
            "SELECT preview_id, policy_id, policy_revision, template_id, template_revision, \
             account_id, provider_chat_id, rendered_text, rendered_sha256, \
             created_at_unix_seconds \
             FROM makosh_data.telegram_automation_preview_receipts WHERE preview_id = $1",
        )
        .bind(preview_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| TelegramAutomationPersistenceError::Database)?;
        row.map(|row| preview_from_row(&row)).transpose()
    }

    pub async fn list_templates(
        &self,
        after_template_id: &str,
        limit: u32,
    ) -> Result<Vec<AutomationTemplate>, TelegramAutomationPersistenceError> {
        let rows = sqlx::query(
            "SELECT template_id FROM makosh_data.telegram_automation_templates \
             WHERE template_id > $1 ORDER BY template_id ASC LIMIT $2",
        )
        .bind(after_template_id)
        .bind(i64::from(limit))
        .fetch_all(&self.pool)
        .await
        .map_err(|_| TelegramAutomationPersistenceError::Database)?;
        let mut templates = Vec::with_capacity(rows.len());
        for row in rows {
            let template_id: String = row
                .try_get("template_id")
                .map_err(|_| TelegramAutomationPersistenceError::InvalidRow)?;
            templates.push(
                self.template(&template_id)
                    .await?
                    .ok_or(TelegramAutomationPersistenceError::InvalidRow)?,
            );
        }
        Ok(templates)
    }

    pub async fn list_policies(
        &self,
        after_policy_id: &str,
        limit: u32,
    ) -> Result<Vec<AutomationPolicy>, TelegramAutomationPersistenceError> {
        let rows = sqlx::query(
            "SELECT policy_id FROM makosh_data.telegram_automation_policies \
             WHERE policy_id > $1 ORDER BY policy_id ASC LIMIT $2",
        )
        .bind(after_policy_id)
        .bind(i64::from(limit))
        .fetch_all(&self.pool)
        .await
        .map_err(|_| TelegramAutomationPersistenceError::Database)?;
        let mut policies = Vec::with_capacity(rows.len());
        for row in rows {
            let policy_id: String = row
                .try_get("policy_id")
                .map_err(|_| TelegramAutomationPersistenceError::InvalidRow)?;
            policies.push(
                self.policy(&policy_id)
                    .await?
                    .ok_or(TelegramAutomationPersistenceError::InvalidRow)?,
            );
        }
        Ok(policies)
    }

    pub async fn upsert_template<F>(
        &self,
        mutation_id: &str,
        request_sha256: &[u8; 32],
        expected_revision: u64,
        draft: &AutomationTemplateDraft,
        now_unix_seconds: u64,
        encode_response: F,
    ) -> Result<PersistedMutation<AutomationTemplate>, TelegramAutomationPersistenceError>
    where
        F: FnOnce(&AutomationTemplate) -> Vec<u8>,
    {
        validate_identifier("mutation_id", mutation_id)
            .map_err(TelegramAutomationPersistenceError::Core)?;
        draft
            .validate()
            .map_err(TelegramAutomationPersistenceError::Core)?;
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| TelegramAutomationPersistenceError::Database)?;
        if let Some(response_payload) = replay_mutation(
            &mut transaction,
            mutation_id,
            "upsert_template",
            request_sha256,
        )
        .await?
        {
            transaction
                .commit()
                .await
                .map_err(|_| TelegramAutomationPersistenceError::Database)?;
            return Ok(PersistedMutation::Replayed { response_payload });
        }

        let current = sqlx::query(
            "SELECT revision, created_at_unix_seconds \
             FROM makosh_data.telegram_automation_templates \
             WHERE template_id = $1 FOR UPDATE",
        )
        .bind(&draft.template_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| TelegramAutomationPersistenceError::Database)?;
        let (revision, created_at_unix_seconds) =
            next_revision(current.as_ref(), expected_revision, now_unix_seconds)?;
        let template = AutomationTemplate {
            template_id: draft.template_id.clone(),
            name: draft.name.clone(),
            body_template: draft.body_template.clone(),
            required_variables: draft.required_variables.clone(),
            revision,
            created_at_unix_seconds,
            updated_at_unix_seconds: now_unix_seconds,
        };
        sqlx::query(
            "INSERT INTO makosh_data.telegram_automation_templates \
             (template_id, revision, name, body_template, created_at_unix_seconds, updated_at_unix_seconds) \
             VALUES ($1, $2, $3, $4, $5, $6) \
             ON CONFLICT (template_id) DO UPDATE SET revision = EXCLUDED.revision, \
             name = EXCLUDED.name, body_template = EXCLUDED.body_template, \
             updated_at_unix_seconds = EXCLUDED.updated_at_unix_seconds",
        )
        .bind(&template.template_id)
        .bind(as_i64(template.revision)?)
        .bind(&template.name)
        .bind(&template.body_template)
        .bind(as_i64(template.created_at_unix_seconds)?)
        .bind(as_i64(template.updated_at_unix_seconds)?)
        .execute(&mut *transaction)
        .await
        .map_err(|_| TelegramAutomationPersistenceError::Database)?;
        sqlx::query(
            "DELETE FROM makosh_data.telegram_automation_template_variables WHERE template_id = $1",
        )
        .bind(&template.template_id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| TelegramAutomationPersistenceError::Database)?;
        for (ordinal, variable_name) in template.required_variables.iter().enumerate() {
            sqlx::query(
                "INSERT INTO makosh_data.telegram_automation_template_variables \
                 (template_id, ordinal, variable_name) VALUES ($1, $2, $3)",
            )
            .bind(&template.template_id)
            .bind(
                i32::try_from(ordinal)
                    .map_err(|_| TelegramAutomationPersistenceError::InvalidRow)?,
            )
            .bind(variable_name)
            .execute(&mut *transaction)
            .await
            .map_err(|_| TelegramAutomationPersistenceError::Database)?;
        }
        let response_payload = encode_response(&template);
        persist_mutation(
            &mut transaction,
            mutation_id,
            "upsert_template",
            request_sha256,
            &response_payload,
            now_unix_seconds,
        )
        .await?;
        transaction
            .commit()
            .await
            .map_err(|_| TelegramAutomationPersistenceError::Database)?;
        Ok(PersistedMutation::Applied {
            value: template,
            response_payload,
        })
    }

    pub async fn upsert_policy<F>(
        &self,
        mutation_id: &str,
        request_sha256: &[u8; 32],
        expected_revision: u64,
        draft: &AutomationPolicyDraft,
        now_unix_seconds: u64,
        encode_response: F,
    ) -> Result<PersistedMutation<AutomationPolicy>, TelegramAutomationPersistenceError>
    where
        F: FnOnce(&AutomationPolicy) -> Vec<u8>,
    {
        validate_identifier("mutation_id", mutation_id)
            .map_err(TelegramAutomationPersistenceError::Core)?;
        draft
            .validate()
            .map_err(TelegramAutomationPersistenceError::Core)?;
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| TelegramAutomationPersistenceError::Database)?;
        if let Some(response_payload) = replay_mutation(
            &mut transaction,
            mutation_id,
            "upsert_policy",
            request_sha256,
        )
        .await?
        {
            transaction
                .commit()
                .await
                .map_err(|_| TelegramAutomationPersistenceError::Database)?;
            return Ok(PersistedMutation::Replayed { response_payload });
        }
        let template_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM makosh_data.telegram_automation_templates \
             WHERE template_id = $1)",
        )
        .bind(&draft.template_id)
        .fetch_one(&mut *transaction)
        .await
        .map_err(|_| TelegramAutomationPersistenceError::Database)?;
        if !template_exists {
            return Err(TelegramAutomationPersistenceError::MissingTemplate);
        }
        let account_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM makosh_data.telegram_accounts WHERE account_id = $1)",
        )
        .bind(&draft.account_id)
        .fetch_one(&mut *transaction)
        .await
        .map_err(|_| TelegramAutomationPersistenceError::Database)?;
        if !account_exists {
            return Err(TelegramAutomationPersistenceError::MissingAccount);
        }
        let current = sqlx::query(
            "SELECT revision, created_at_unix_seconds \
             FROM makosh_data.telegram_automation_policies \
             WHERE policy_id = $1 FOR UPDATE",
        )
        .bind(&draft.policy_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| TelegramAutomationPersistenceError::Database)?;
        let (revision, created_at_unix_seconds) =
            next_revision(current.as_ref(), expected_revision, now_unix_seconds)?;
        let policy = AutomationPolicy {
            policy_id: draft.policy_id.clone(),
            template_id: draft.template_id.clone(),
            name: draft.name.clone(),
            enabled: draft.enabled,
            account_id: draft.account_id.clone(),
            provider_chat_ids: draft.provider_chat_ids.clone(),
            expires_at_unix_seconds: draft.expires_at_unix_seconds,
            revision,
            created_at_unix_seconds,
            updated_at_unix_seconds: now_unix_seconds,
        };
        sqlx::query(
            "INSERT INTO makosh_data.telegram_automation_policies \
             (policy_id, template_id, revision, name, enabled, account_id, \
              expires_at_unix_seconds, created_at_unix_seconds, updated_at_unix_seconds) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) \
             ON CONFLICT (policy_id) DO UPDATE SET template_id = EXCLUDED.template_id, \
             revision = EXCLUDED.revision, name = EXCLUDED.name, enabled = EXCLUDED.enabled, \
             account_id = EXCLUDED.account_id, expires_at_unix_seconds = EXCLUDED.expires_at_unix_seconds, \
             updated_at_unix_seconds = EXCLUDED.updated_at_unix_seconds",
        )
        .bind(&policy.policy_id)
        .bind(&policy.template_id)
        .bind(as_i64(policy.revision)?)
        .bind(&policy.name)
        .bind(policy.enabled)
        .bind(&policy.account_id)
        .bind(policy.expires_at_unix_seconds.map(as_i64).transpose()?)
        .bind(as_i64(policy.created_at_unix_seconds)?)
        .bind(as_i64(policy.updated_at_unix_seconds)?)
        .execute(&mut *transaction)
        .await
        .map_err(|_| TelegramAutomationPersistenceError::Database)?;
        sqlx::query(
            "DELETE FROM makosh_data.telegram_automation_policy_chat_scopes WHERE policy_id = $1",
        )
        .bind(&policy.policy_id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| TelegramAutomationPersistenceError::Database)?;
        for provider_chat_id in &policy.provider_chat_ids {
            sqlx::query(
                "INSERT INTO makosh_data.telegram_automation_policy_chat_scopes \
                 (policy_id, provider_chat_id) VALUES ($1, $2)",
            )
            .bind(&policy.policy_id)
            .bind(provider_chat_id)
            .execute(&mut *transaction)
            .await
            .map_err(|_| TelegramAutomationPersistenceError::Database)?;
        }
        let response_payload = encode_response(&policy);
        persist_mutation(
            &mut transaction,
            mutation_id,
            "upsert_policy",
            request_sha256,
            &response_payload,
            now_unix_seconds,
        )
        .await?;
        transaction
            .commit()
            .await
            .map_err(|_| TelegramAutomationPersistenceError::Database)?;
        Ok(PersistedMutation::Applied {
            value: policy,
            response_payload,
        })
    }

    pub async fn preview_policy<F>(
        &self,
        request_sha256: &[u8; 32],
        request: &AutomationPreviewRequest,
        now_unix_seconds: u64,
        encode_response: F,
    ) -> Result<PersistedMutation<AutomationPreviewReceipt>, TelegramAutomationPersistenceError>
    where
        F: FnOnce(&AutomationPreviewReceipt) -> Vec<u8>,
    {
        request
            .validate()
            .map_err(TelegramAutomationPersistenceError::Core)?;
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| TelegramAutomationPersistenceError::Database)?;
        if let Some((stored_hash, response_payload)) =
            preview_replay(&mut transaction, &request.preview_id).await?
        {
            if stored_hash != *request_sha256 {
                return Err(TelegramAutomationPersistenceError::IdempotencyConflict);
            }
            transaction
                .commit()
                .await
                .map_err(|_| TelegramAutomationPersistenceError::Database)?;
            return Ok(PersistedMutation::Replayed { response_payload });
        }
        let policy = load_policy_in_transaction(&mut transaction, &request.policy_id, true)
            .await?
            .ok_or(TelegramAutomationPersistenceError::MissingPolicy)?;
        let template = load_template_in_transaction(&mut transaction, &policy.template_id, true)
            .await?
            .ok_or(TelegramAutomationPersistenceError::MissingTemplate)?;
        let receipt = render_preview(&policy, &template, request, now_unix_seconds)
            .map_err(TelegramAutomationPersistenceError::Core)?;
        let response_payload = encode_response(&receipt);
        sqlx::query(
            "INSERT INTO makosh_data.telegram_automation_preview_receipts \
             (preview_id, request_sha256, policy_id, policy_revision, template_id, \
              template_revision, account_id, provider_chat_id, rendered_text, \
              rendered_sha256, response_payload, created_at_unix_seconds) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
        )
        .bind(&receipt.preview_id)
        .bind(request_sha256.as_slice())
        .bind(&receipt.policy_id)
        .bind(as_i64(receipt.policy_revision)?)
        .bind(&receipt.template_id)
        .bind(as_i64(receipt.template_revision)?)
        .bind(&receipt.account_id)
        .bind(&receipt.provider_chat_id)
        .bind(&receipt.rendered_text)
        .bind(receipt.rendered_sha256.as_slice())
        .bind(&response_payload)
        .bind(as_i64(receipt.created_at_unix_seconds)?)
        .execute(&mut *transaction)
        .await
        .map_err(|_| TelegramAutomationPersistenceError::Database)?;
        transaction
            .commit()
            .await
            .map_err(|_| TelegramAutomationPersistenceError::Database)?;
        Ok(PersistedMutation::Applied {
            value: receipt,
            response_payload,
        })
    }
}

async fn replay_mutation(
    transaction: &mut Transaction<'_, Postgres>,
    mutation_id: &str,
    mutation_kind: &str,
    request_sha256: &[u8; 32],
) -> Result<Option<Vec<u8>>, TelegramAutomationPersistenceError> {
    let row = sqlx::query(
        "SELECT mutation_kind, request_sha256, response_payload \
         FROM makosh_data.telegram_automation_mutation_receipts WHERE mutation_id = $1 FOR UPDATE",
    )
    .bind(mutation_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| TelegramAutomationPersistenceError::Database)?;
    let Some(row) = row else {
        return Ok(None);
    };
    let stored_kind: String = row
        .try_get("mutation_kind")
        .map_err(|_| TelegramAutomationPersistenceError::InvalidRow)?;
    let stored_hash: Vec<u8> = row
        .try_get("request_sha256")
        .map_err(|_| TelegramAutomationPersistenceError::InvalidRow)?;
    let response_payload: Vec<u8> = row
        .try_get("response_payload")
        .map_err(|_| TelegramAutomationPersistenceError::InvalidRow)?;
    if stored_kind != mutation_kind || stored_hash.as_slice() != request_sha256 {
        return Err(TelegramAutomationPersistenceError::IdempotencyConflict);
    }
    Ok(Some(response_payload))
}

async fn persist_mutation(
    transaction: &mut Transaction<'_, Postgres>,
    mutation_id: &str,
    mutation_kind: &str,
    request_sha256: &[u8; 32],
    response_payload: &[u8],
    now_unix_seconds: u64,
) -> Result<(), TelegramAutomationPersistenceError> {
    sqlx::query(
        "INSERT INTO makosh_data.telegram_automation_mutation_receipts \
         (mutation_id, mutation_kind, request_sha256, response_payload, created_at_unix_seconds) \
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(mutation_id)
    .bind(mutation_kind)
    .bind(request_sha256.as_slice())
    .bind(response_payload)
    .bind(as_i64(now_unix_seconds)?)
    .execute(&mut **transaction)
    .await
    .map_err(|_| TelegramAutomationPersistenceError::Database)?;
    Ok(())
}

async fn preview_replay(
    transaction: &mut Transaction<'_, Postgres>,
    preview_id: &str,
) -> Result<Option<([u8; 32], Vec<u8>)>, TelegramAutomationPersistenceError> {
    let row = sqlx::query(
        "SELECT request_sha256, response_payload \
         FROM makosh_data.telegram_automation_preview_receipts WHERE preview_id = $1 FOR UPDATE",
    )
    .bind(preview_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| TelegramAutomationPersistenceError::Database)?;
    row.map(|row| {
        let hash: Vec<u8> = row
            .try_get("request_sha256")
            .map_err(|_| TelegramAutomationPersistenceError::InvalidRow)?;
        let hash = hash
            .as_slice()
            .try_into()
            .map_err(|_| TelegramAutomationPersistenceError::InvalidRow)?;
        let response_payload = row
            .try_get("response_payload")
            .map_err(|_| TelegramAutomationPersistenceError::InvalidRow)?;
        Ok((hash, response_payload))
    })
    .transpose()
}

fn next_revision(
    current: Option<&sqlx::postgres::PgRow>,
    expected_revision: u64,
    now_unix_seconds: u64,
) -> Result<(u64, u64), TelegramAutomationPersistenceError> {
    if now_unix_seconds == 0 {
        return Err(TelegramAutomationPersistenceError::InvalidRow);
    }
    let Some(current) = current else {
        if expected_revision != 0 {
            return Err(TelegramAutomationPersistenceError::RevisionConflict);
        }
        return Ok((1, now_unix_seconds));
    };
    let revision = from_i64(
        current
            .try_get("revision")
            .map_err(|_| TelegramAutomationPersistenceError::InvalidRow)?,
    )?;
    if revision != expected_revision {
        return Err(TelegramAutomationPersistenceError::RevisionConflict);
    }
    let created_at = from_i64(
        current
            .try_get("created_at_unix_seconds")
            .map_err(|_| TelegramAutomationPersistenceError::InvalidRow)?,
    )?;
    Ok((
        revision
            .checked_add(1)
            .ok_or(TelegramAutomationPersistenceError::InvalidRow)?,
        created_at,
    ))
}

async fn load_template(
    pool: &PgPool,
    template_id: &str,
) -> Result<Option<AutomationTemplate>, TelegramAutomationPersistenceError> {
    let mut transaction = pool
        .begin()
        .await
        .map_err(|_| TelegramAutomationPersistenceError::Database)?;
    let result = load_template_in_transaction(&mut transaction, template_id, false).await?;
    transaction
        .commit()
        .await
        .map_err(|_| TelegramAutomationPersistenceError::Database)?;
    Ok(result)
}

async fn load_template_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    template_id: &str,
    lock: bool,
) -> Result<Option<AutomationTemplate>, TelegramAutomationPersistenceError> {
    let query = if lock {
        "SELECT template_id, revision, name, body_template, created_at_unix_seconds, \
         updated_at_unix_seconds FROM makosh_data.telegram_automation_templates \
         WHERE template_id = $1 FOR SHARE"
    } else {
        "SELECT template_id, revision, name, body_template, created_at_unix_seconds, \
         updated_at_unix_seconds FROM makosh_data.telegram_automation_templates \
         WHERE template_id = $1"
    };
    let row = sqlx::query(query)
        .bind(template_id)
        .fetch_optional(&mut **transaction)
        .await
        .map_err(|_| TelegramAutomationPersistenceError::Database)?;
    let Some(row) = row else {
        return Ok(None);
    };
    let variable_rows = sqlx::query(
        "SELECT variable_name FROM makosh_data.telegram_automation_template_variables \
         WHERE template_id = $1 ORDER BY ordinal ASC",
    )
    .bind(template_id)
    .fetch_all(&mut **transaction)
    .await
    .map_err(|_| TelegramAutomationPersistenceError::Database)?;
    Ok(Some(AutomationTemplate {
        template_id: row
            .try_get("template_id")
            .map_err(|_| TelegramAutomationPersistenceError::InvalidRow)?,
        revision: from_i64(
            row.try_get("revision")
                .map_err(|_| TelegramAutomationPersistenceError::InvalidRow)?,
        )?,
        name: row
            .try_get("name")
            .map_err(|_| TelegramAutomationPersistenceError::InvalidRow)?,
        body_template: row
            .try_get("body_template")
            .map_err(|_| TelegramAutomationPersistenceError::InvalidRow)?,
        required_variables: variable_rows
            .into_iter()
            .map(|row| {
                row.try_get("variable_name")
                    .map_err(|_| TelegramAutomationPersistenceError::InvalidRow)
            })
            .collect::<Result<_, _>>()?,
        created_at_unix_seconds: from_i64(
            row.try_get("created_at_unix_seconds")
                .map_err(|_| TelegramAutomationPersistenceError::InvalidRow)?,
        )?,
        updated_at_unix_seconds: from_i64(
            row.try_get("updated_at_unix_seconds")
                .map_err(|_| TelegramAutomationPersistenceError::InvalidRow)?,
        )?,
    }))
}

async fn load_policy(
    pool: &PgPool,
    policy_id: &str,
) -> Result<Option<AutomationPolicy>, TelegramAutomationPersistenceError> {
    let mut transaction = pool
        .begin()
        .await
        .map_err(|_| TelegramAutomationPersistenceError::Database)?;
    let result = load_policy_in_transaction(&mut transaction, policy_id, false).await?;
    transaction
        .commit()
        .await
        .map_err(|_| TelegramAutomationPersistenceError::Database)?;
    Ok(result)
}

async fn load_policy_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    policy_id: &str,
    lock: bool,
) -> Result<Option<AutomationPolicy>, TelegramAutomationPersistenceError> {
    let query = if lock {
        "SELECT policy_id, template_id, revision, name, enabled, account_id, \
         expires_at_unix_seconds, created_at_unix_seconds, updated_at_unix_seconds \
         FROM makosh_data.telegram_automation_policies WHERE policy_id = $1 FOR SHARE"
    } else {
        "SELECT policy_id, template_id, revision, name, enabled, account_id, \
         expires_at_unix_seconds, created_at_unix_seconds, updated_at_unix_seconds \
         FROM makosh_data.telegram_automation_policies WHERE policy_id = $1"
    };
    let row = sqlx::query(query)
        .bind(policy_id)
        .fetch_optional(&mut **transaction)
        .await
        .map_err(|_| TelegramAutomationPersistenceError::Database)?;
    let Some(row) = row else {
        return Ok(None);
    };
    let scope_rows = sqlx::query(
        "SELECT provider_chat_id FROM makosh_data.telegram_automation_policy_chat_scopes \
         WHERE policy_id = $1 ORDER BY provider_chat_id ASC",
    )
    .bind(policy_id)
    .fetch_all(&mut **transaction)
    .await
    .map_err(|_| TelegramAutomationPersistenceError::Database)?;
    Ok(Some(AutomationPolicy {
        policy_id: row
            .try_get("policy_id")
            .map_err(|_| TelegramAutomationPersistenceError::InvalidRow)?,
        template_id: row
            .try_get("template_id")
            .map_err(|_| TelegramAutomationPersistenceError::InvalidRow)?,
        revision: from_i64(
            row.try_get("revision")
                .map_err(|_| TelegramAutomationPersistenceError::InvalidRow)?,
        )?,
        name: row
            .try_get("name")
            .map_err(|_| TelegramAutomationPersistenceError::InvalidRow)?,
        enabled: row
            .try_get("enabled")
            .map_err(|_| TelegramAutomationPersistenceError::InvalidRow)?,
        account_id: row
            .try_get("account_id")
            .map_err(|_| TelegramAutomationPersistenceError::InvalidRow)?,
        provider_chat_ids: scope_rows
            .into_iter()
            .map(|row| {
                row.try_get("provider_chat_id")
                    .map_err(|_| TelegramAutomationPersistenceError::InvalidRow)
            })
            .collect::<Result<_, _>>()?,
        expires_at_unix_seconds: row
            .try_get::<Option<i64>, _>("expires_at_unix_seconds")
            .map_err(|_| TelegramAutomationPersistenceError::InvalidRow)?
            .map(from_i64)
            .transpose()?,
        created_at_unix_seconds: from_i64(
            row.try_get("created_at_unix_seconds")
                .map_err(|_| TelegramAutomationPersistenceError::InvalidRow)?,
        )?,
        updated_at_unix_seconds: from_i64(
            row.try_get("updated_at_unix_seconds")
                .map_err(|_| TelegramAutomationPersistenceError::InvalidRow)?,
        )?,
    }))
}

fn preview_from_row(
    row: &sqlx::postgres::PgRow,
) -> Result<AutomationPreviewReceipt, TelegramAutomationPersistenceError> {
    let rendered_sha256: Vec<u8> = row
        .try_get("rendered_sha256")
        .map_err(|_| TelegramAutomationPersistenceError::InvalidRow)?;
    Ok(AutomationPreviewReceipt {
        preview_id: row
            .try_get("preview_id")
            .map_err(|_| TelegramAutomationPersistenceError::InvalidRow)?,
        policy_id: row
            .try_get("policy_id")
            .map_err(|_| TelegramAutomationPersistenceError::InvalidRow)?,
        policy_revision: from_i64(
            row.try_get("policy_revision")
                .map_err(|_| TelegramAutomationPersistenceError::InvalidRow)?,
        )?,
        template_id: row
            .try_get("template_id")
            .map_err(|_| TelegramAutomationPersistenceError::InvalidRow)?,
        template_revision: from_i64(
            row.try_get("template_revision")
                .map_err(|_| TelegramAutomationPersistenceError::InvalidRow)?,
        )?,
        account_id: row
            .try_get("account_id")
            .map_err(|_| TelegramAutomationPersistenceError::InvalidRow)?,
        provider_chat_id: row
            .try_get("provider_chat_id")
            .map_err(|_| TelegramAutomationPersistenceError::InvalidRow)?,
        rendered_text: row
            .try_get("rendered_text")
            .map_err(|_| TelegramAutomationPersistenceError::InvalidRow)?,
        rendered_sha256: rendered_sha256
            .as_slice()
            .try_into()
            .map_err(|_| TelegramAutomationPersistenceError::InvalidRow)?,
        created_at_unix_seconds: from_i64(
            row.try_get("created_at_unix_seconds")
                .map_err(|_| TelegramAutomationPersistenceError::InvalidRow)?,
        )?,
    })
}

fn as_i64(value: u64) -> Result<i64, TelegramAutomationPersistenceError> {
    i64::try_from(value).map_err(|_| TelegramAutomationPersistenceError::InvalidRow)
}

fn from_i64(value: i64) -> Result<u64, TelegramAutomationPersistenceError> {
    u64::try_from(value).map_err(|_| TelegramAutomationPersistenceError::InvalidRow)
}
