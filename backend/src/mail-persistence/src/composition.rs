//! Mail-owned drafts, templates and signatures with optimistic concurrency.

use makosh_mail_api::{
    composition::{
        MailCompositionCommandV1, MailCompositionEntityKindV1, MailCompositionModeV1,
        MailCompositionMutationReceiptV1, MailCompositionPageV1, MailCompositionQueryResponseV1,
        MailCompositionQueryV1, MailDraftInputV1, MailDraftV1, MailSignatureInputV1,
        MailSignatureV1, MailTemplateInputV1, MailTemplateV1, composition_command_connection_id,
        composition_command_operation_id, render_mail_template_preview,
        validate_composition_command, validate_composition_query, validate_composition_receipt,
        validate_composition_response,
    },
    composition_wire::encode_composition_command,
};
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Postgres, Row, Transaction, postgres::PgRow};

use crate::MailDurablePersistence;

pub const MAIL_SCHEMA_V11: &str = r#"
CREATE TABLE IF NOT EXISTS makosh_data.mail_composition_commands (
    operation_id TEXT PRIMARY KEY,
    connection_id TEXT NOT NULL,
    request_sha256 BYTEA NOT NULL CHECK (octet_length(request_sha256) = 32),
    entity_kind SMALLINT NOT NULL CHECK (entity_kind BETWEEN 1 AND 3),
    entity_id TEXT NOT NULL,
    entity_revision BIGINT NOT NULL CHECK (entity_revision >= 0),
    deleted BOOLEAN NOT NULL DEFAULT FALSE,
    completed_at_unix_seconds BIGINT
);

CREATE TABLE IF NOT EXISTS makosh_data.mail_drafts (
    sequence BIGSERIAL UNIQUE NOT NULL,
    connection_id TEXT NOT NULL,
    draft_id TEXT NOT NULL,
    revision BIGINT NOT NULL CHECK (revision > 0),
    mode SMALLINT NOT NULL CHECK (mode BETWEEN 1 AND 5),
    provider_conversation_id TEXT,
    in_reply_to_provider_message_id TEXT,
    to_recipients TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[],
    cc_recipients TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[],
    bcc_recipients TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[],
    subject TEXT NOT NULL,
    text_body TEXT NOT NULL,
    template_id TEXT,
    signature_id TEXT,
    created_at_unix_seconds BIGINT NOT NULL CHECK (created_at_unix_seconds > 0),
    updated_at_unix_seconds BIGINT NOT NULL CHECK (updated_at_unix_seconds > 0),
    PRIMARY KEY (connection_id, draft_id)
);

CREATE INDEX IF NOT EXISTS mail_drafts_connection_updated_idx
ON makosh_data.mail_drafts (
    connection_id,
    updated_at_unix_seconds DESC,
    sequence DESC
);

CREATE TABLE IF NOT EXISTS makosh_data.mail_templates (
    sequence BIGSERIAL UNIQUE NOT NULL,
    connection_id TEXT NOT NULL,
    template_id TEXT NOT NULL,
    revision BIGINT NOT NULL CHECK (revision > 0),
    name TEXT NOT NULL,
    subject_template TEXT NOT NULL,
    text_body_template TEXT NOT NULL,
    variables TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[],
    locale TEXT,
    created_at_unix_seconds BIGINT NOT NULL CHECK (created_at_unix_seconds > 0),
    updated_at_unix_seconds BIGINT NOT NULL CHECK (updated_at_unix_seconds > 0),
    PRIMARY KEY (connection_id, template_id)
);

CREATE INDEX IF NOT EXISTS mail_templates_connection_updated_idx
ON makosh_data.mail_templates (
    connection_id,
    updated_at_unix_seconds DESC,
    sequence DESC
);

CREATE TABLE IF NOT EXISTS makosh_data.mail_signatures (
    sequence BIGSERIAL UNIQUE NOT NULL,
    connection_id TEXT NOT NULL,
    signature_id TEXT NOT NULL,
    revision BIGINT NOT NULL CHECK (revision > 0),
    name TEXT NOT NULL,
    text_body TEXT NOT NULL,
    is_default BOOLEAN NOT NULL DEFAULT FALSE,
    created_at_unix_seconds BIGINT NOT NULL CHECK (created_at_unix_seconds > 0),
    updated_at_unix_seconds BIGINT NOT NULL CHECK (updated_at_unix_seconds > 0),
    PRIMARY KEY (connection_id, signature_id)
);

CREATE UNIQUE INDEX IF NOT EXISTS mail_signatures_one_default_idx
ON makosh_data.mail_signatures (connection_id)
WHERE is_default;

CREATE INDEX IF NOT EXISTS mail_signatures_connection_updated_idx
ON makosh_data.mail_signatures (
    connection_id,
    updated_at_unix_seconds DESC,
    sequence DESC
);
"#;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailCompositionPersistenceErrorV1 {
    Database,
    InvalidInput,
    ConflictingOperation,
    MissingEntity,
    StaleRevision,
    InvalidCursor,
    InvalidRow,
}

impl MailDurablePersistence {
    pub async fn execute_composition_command(
        &self,
        command: &MailCompositionCommandV1,
        canonical_command_bytes: &[u8],
        now_unix_seconds: i64,
    ) -> Result<MailCompositionMutationReceiptV1, MailCompositionPersistenceErrorV1> {
        if now_unix_seconds <= 0
            || validate_composition_command(command).is_err()
            || encode_composition_command(command)
                .map(|bytes| bytes != canonical_command_bytes)
                .unwrap_or(true)
        {
            return Err(MailCompositionPersistenceErrorV1::InvalidInput);
        }
        let request_sha256: [u8; 32] = Sha256::digest(canonical_command_bytes).into();
        let identity = command_identity(command);
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| MailCompositionPersistenceErrorV1::Database)?;
        let inserted = sqlx::query(
            r#"
            INSERT INTO makosh_data.mail_composition_commands (
                operation_id,
                connection_id,
                request_sha256,
                entity_kind,
                entity_id,
                entity_revision,
                deleted
            )
            VALUES ($1, $2, $3, $4, $5, 0, FALSE)
            ON CONFLICT (operation_id) DO NOTHING
            "#,
        )
        .bind(composition_command_operation_id(command))
        .bind(composition_command_connection_id(command))
        .bind(request_sha256.as_slice())
        .bind(entity_kind_id(identity.kind))
        .bind(identity.entity_id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| MailCompositionPersistenceErrorV1::Database)?
        .rows_affected();
        if inserted == 0 {
            let receipt = existing_receipt(
                &mut transaction,
                composition_command_operation_id(command),
                &request_sha256,
            )
            .await?;
            transaction
                .commit()
                .await
                .map_err(|_| MailCompositionPersistenceErrorV1::Database)?;
            return Ok(receipt);
        }

        let receipt = match command {
            MailCompositionCommandV1::UpsertDraft {
                operation_id,
                draft,
                expected_revision,
            } => {
                upsert_draft(
                    &mut transaction,
                    operation_id,
                    draft,
                    *expected_revision,
                    now_unix_seconds,
                )
                .await?
            }
            MailCompositionCommandV1::DeleteDraft {
                operation_id,
                connection_id,
                draft_id,
                expected_revision,
            } => {
                delete_entity(
                    &mut transaction,
                    DeleteEntityV1 {
                        operation_id,
                        connection_id,
                        entity_id: draft_id,
                        expected_revision: *expected_revision,
                        kind: MailCompositionEntityKindV1::Draft,
                    },
                )
                .await?
            }
            MailCompositionCommandV1::UpsertTemplate {
                operation_id,
                template,
                expected_revision,
            } => {
                upsert_template(
                    &mut transaction,
                    operation_id,
                    template,
                    *expected_revision,
                    now_unix_seconds,
                )
                .await?
            }
            MailCompositionCommandV1::DeleteTemplate {
                operation_id,
                connection_id,
                template_id,
                expected_revision,
            } => {
                delete_entity(
                    &mut transaction,
                    DeleteEntityV1 {
                        operation_id,
                        connection_id,
                        entity_id: template_id,
                        expected_revision: *expected_revision,
                        kind: MailCompositionEntityKindV1::Template,
                    },
                )
                .await?
            }
            MailCompositionCommandV1::UpsertSignature {
                operation_id,
                signature,
                expected_revision,
            } => {
                upsert_signature(
                    &mut transaction,
                    operation_id,
                    signature,
                    *expected_revision,
                    now_unix_seconds,
                )
                .await?
            }
            MailCompositionCommandV1::DeleteSignature {
                operation_id,
                connection_id,
                signature_id,
                expected_revision,
            } => {
                delete_entity(
                    &mut transaction,
                    DeleteEntityV1 {
                        operation_id,
                        connection_id,
                        entity_id: signature_id,
                        expected_revision: *expected_revision,
                        kind: MailCompositionEntityKindV1::Signature,
                    },
                )
                .await?
            }
        };
        validate_composition_receipt(&receipt)
            .map_err(|_| MailCompositionPersistenceErrorV1::InvalidRow)?;
        sqlx::query(
            r#"
            UPDATE makosh_data.mail_composition_commands
            SET entity_revision = $2,
                deleted = $3,
                completed_at_unix_seconds = $4
            WHERE operation_id = $1
            "#,
        )
        .bind(&receipt.operation_id)
        .bind(i64_from_u64(receipt.revision)?)
        .bind(receipt.deleted)
        .bind(now_unix_seconds)
        .execute(&mut *transaction)
        .await
        .map_err(|_| MailCompositionPersistenceErrorV1::Database)?;
        transaction
            .commit()
            .await
            .map_err(|_| MailCompositionPersistenceErrorV1::Database)?;
        Ok(receipt)
    }

    pub async fn execute_composition_query(
        &self,
        query: &MailCompositionQueryV1,
    ) -> Result<MailCompositionQueryResponseV1, MailCompositionPersistenceErrorV1> {
        validate_composition_query(query)
            .map_err(|_| MailCompositionPersistenceErrorV1::InvalidInput)?;
        let response = match query {
            MailCompositionQueryV1::ListDrafts {
                connection_id,
                cursor,
                limit,
            } => MailCompositionQueryResponseV1::Drafts(
                list_drafts(&self.pool, connection_id, cursor.as_deref(), *limit).await?,
            ),
            MailCompositionQueryV1::GetDraft {
                connection_id,
                draft_id,
            } => match get_draft(&self.pool, connection_id, draft_id).await? {
                Some(value) => MailCompositionQueryResponseV1::Draft(value),
                None => MailCompositionQueryResponseV1::NotFound,
            },
            MailCompositionQueryV1::ListTemplates {
                connection_id,
                cursor,
                limit,
            } => MailCompositionQueryResponseV1::Templates(
                list_templates(&self.pool, connection_id, cursor.as_deref(), *limit).await?,
            ),
            MailCompositionQueryV1::GetTemplate {
                connection_id,
                template_id,
            } => match get_template(&self.pool, connection_id, template_id).await? {
                Some(value) => MailCompositionQueryResponseV1::Template(value),
                None => MailCompositionQueryResponseV1::NotFound,
            },
            MailCompositionQueryV1::PreviewTemplate {
                connection_id,
                template_id,
                values,
            } => {
                let template = get_template(&self.pool, connection_id, template_id)
                    .await?
                    .ok_or(MailCompositionPersistenceErrorV1::MissingEntity)?;
                MailCompositionQueryResponseV1::TemplatePreview(
                    render_mail_template_preview(&template, values)
                        .map_err(|_| MailCompositionPersistenceErrorV1::InvalidInput)?,
                )
            }
            MailCompositionQueryV1::ListSignatures {
                connection_id,
                cursor,
                limit,
            } => MailCompositionQueryResponseV1::Signatures(
                list_signatures(&self.pool, connection_id, cursor.as_deref(), *limit).await?,
            ),
            MailCompositionQueryV1::GetSignature {
                connection_id,
                signature_id,
            } => match get_signature(&self.pool, connection_id, signature_id).await? {
                Some(value) => MailCompositionQueryResponseV1::Signature(value),
                None => MailCompositionQueryResponseV1::NotFound,
            },
        };
        validate_composition_response(&response)
            .map_err(|_| MailCompositionPersistenceErrorV1::InvalidRow)?;
        Ok(response)
    }
}

struct CommandIdentityV1<'a> {
    kind: MailCompositionEntityKindV1,
    entity_id: &'a str,
}

fn command_identity(command: &MailCompositionCommandV1) -> CommandIdentityV1<'_> {
    match command {
        MailCompositionCommandV1::UpsertDraft { draft, .. } => CommandIdentityV1 {
            kind: MailCompositionEntityKindV1::Draft,
            entity_id: &draft.draft_id,
        },
        MailCompositionCommandV1::DeleteDraft { draft_id, .. } => CommandIdentityV1 {
            kind: MailCompositionEntityKindV1::Draft,
            entity_id: draft_id,
        },
        MailCompositionCommandV1::UpsertTemplate { template, .. } => CommandIdentityV1 {
            kind: MailCompositionEntityKindV1::Template,
            entity_id: &template.template_id,
        },
        MailCompositionCommandV1::DeleteTemplate { template_id, .. } => CommandIdentityV1 {
            kind: MailCompositionEntityKindV1::Template,
            entity_id: template_id,
        },
        MailCompositionCommandV1::UpsertSignature { signature, .. } => CommandIdentityV1 {
            kind: MailCompositionEntityKindV1::Signature,
            entity_id: &signature.signature_id,
        },
        MailCompositionCommandV1::DeleteSignature { signature_id, .. } => CommandIdentityV1 {
            kind: MailCompositionEntityKindV1::Signature,
            entity_id: signature_id,
        },
    }
}

async fn existing_receipt(
    transaction: &mut Transaction<'_, Postgres>,
    operation_id: &str,
    request_sha256: &[u8; 32],
) -> Result<MailCompositionMutationReceiptV1, MailCompositionPersistenceErrorV1> {
    let row = sqlx::query(
        r#"
        SELECT connection_id,
               request_sha256,
               entity_kind,
               entity_id,
               entity_revision,
               deleted
        FROM makosh_data.mail_composition_commands
        WHERE operation_id = $1
        "#,
    )
    .bind(operation_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| MailCompositionPersistenceErrorV1::Database)?
    .ok_or(MailCompositionPersistenceErrorV1::InvalidRow)?;
    let stored_digest: Vec<u8> = row
        .try_get("request_sha256")
        .map_err(|_| MailCompositionPersistenceErrorV1::InvalidRow)?;
    if stored_digest.as_slice() != request_sha256 {
        return Err(MailCompositionPersistenceErrorV1::ConflictingOperation);
    }
    let revision = row_u64(&row, "entity_revision")?;
    if revision == 0 {
        return Err(MailCompositionPersistenceErrorV1::InvalidRow);
    }
    Ok(MailCompositionMutationReceiptV1 {
        operation_id: operation_id.to_owned(),
        connection_id: row_string(&row, "connection_id")?,
        entity_kind: entity_kind_from_id(row_i16(&row, "entity_kind")?)?,
        entity_id: row_string(&row, "entity_id")?,
        revision,
        deleted: row_bool(&row, "deleted")?,
    })
}

async fn upsert_draft(
    transaction: &mut Transaction<'_, Postgres>,
    operation_id: &str,
    draft: &MailDraftInputV1,
    expected_revision: Option<u64>,
    now: i64,
) -> Result<MailCompositionMutationReceiptV1, MailCompositionPersistenceErrorV1> {
    let row = if let Some(expected_revision) = expected_revision {
        sqlx::query(
            r#"
            UPDATE makosh_data.mail_drafts
            SET revision = revision + 1,
                mode = $4,
                provider_conversation_id = $5,
                in_reply_to_provider_message_id = $6,
                to_recipients = $7,
                cc_recipients = $8,
                bcc_recipients = $9,
                subject = $10,
                text_body = $11,
                template_id = $12,
                signature_id = $13,
                updated_at_unix_seconds = $14
            WHERE connection_id = $1
              AND draft_id = $2
              AND revision = $3
            RETURNING revision
            "#,
        )
        .bind(&draft.connection_id)
        .bind(&draft.draft_id)
        .bind(i64_from_u64(expected_revision)?)
        .bind(mode_id(draft.mode))
        .bind(draft.provider_conversation_id.as_deref())
        .bind(draft.in_reply_to_provider_message_id.as_deref())
        .bind(&draft.to_recipients)
        .bind(&draft.cc_recipients)
        .bind(&draft.bcc_recipients)
        .bind(&draft.subject)
        .bind(&draft.text_body)
        .bind(draft.template_id.as_deref())
        .bind(draft.signature_id.as_deref())
        .bind(now)
        .fetch_optional(&mut **transaction)
        .await
        .map_err(|_| MailCompositionPersistenceErrorV1::Database)?
        .ok_or(MailCompositionPersistenceErrorV1::StaleRevision)?
    } else {
        sqlx::query(
            r#"
            INSERT INTO makosh_data.mail_drafts (
                connection_id,
                draft_id,
                revision,
                mode,
                provider_conversation_id,
                in_reply_to_provider_message_id,
                to_recipients,
                cc_recipients,
                bcc_recipients,
                subject,
                text_body,
                template_id,
                signature_id,
                created_at_unix_seconds,
                updated_at_unix_seconds
            )
            VALUES ($1, $2, 1, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $13)
            ON CONFLICT (connection_id, draft_id) DO NOTHING
            RETURNING revision
            "#,
        )
        .bind(&draft.connection_id)
        .bind(&draft.draft_id)
        .bind(mode_id(draft.mode))
        .bind(draft.provider_conversation_id.as_deref())
        .bind(draft.in_reply_to_provider_message_id.as_deref())
        .bind(&draft.to_recipients)
        .bind(&draft.cc_recipients)
        .bind(&draft.bcc_recipients)
        .bind(&draft.subject)
        .bind(&draft.text_body)
        .bind(draft.template_id.as_deref())
        .bind(draft.signature_id.as_deref())
        .bind(now)
        .fetch_optional(&mut **transaction)
        .await
        .map_err(|_| MailCompositionPersistenceErrorV1::Database)?
        .ok_or(MailCompositionPersistenceErrorV1::StaleRevision)?
    };
    mutation_receipt(
        operation_id,
        &draft.connection_id,
        MailCompositionEntityKindV1::Draft,
        &draft.draft_id,
        row_u64(&row, "revision")?,
        false,
    )
}

async fn upsert_template(
    transaction: &mut Transaction<'_, Postgres>,
    operation_id: &str,
    template: &MailTemplateInputV1,
    expected_revision: Option<u64>,
    now: i64,
) -> Result<MailCompositionMutationReceiptV1, MailCompositionPersistenceErrorV1> {
    let row = if let Some(expected_revision) = expected_revision {
        sqlx::query(
            r#"
            UPDATE makosh_data.mail_templates
            SET revision = revision + 1,
                name = $4,
                subject_template = $5,
                text_body_template = $6,
                variables = $7,
                locale = $8,
                updated_at_unix_seconds = $9
            WHERE connection_id = $1
              AND template_id = $2
              AND revision = $3
            RETURNING revision
            "#,
        )
        .bind(&template.connection_id)
        .bind(&template.template_id)
        .bind(i64_from_u64(expected_revision)?)
        .bind(&template.name)
        .bind(&template.subject_template)
        .bind(&template.text_body_template)
        .bind(&template.variables)
        .bind(template.locale.as_deref())
        .bind(now)
        .fetch_optional(&mut **transaction)
        .await
        .map_err(|_| MailCompositionPersistenceErrorV1::Database)?
        .ok_or(MailCompositionPersistenceErrorV1::StaleRevision)?
    } else {
        sqlx::query(
            r#"
            INSERT INTO makosh_data.mail_templates (
                connection_id,
                template_id,
                revision,
                name,
                subject_template,
                text_body_template,
                variables,
                locale,
                created_at_unix_seconds,
                updated_at_unix_seconds
            )
            VALUES ($1, $2, 1, $3, $4, $5, $6, $7, $8, $8)
            ON CONFLICT (connection_id, template_id) DO NOTHING
            RETURNING revision
            "#,
        )
        .bind(&template.connection_id)
        .bind(&template.template_id)
        .bind(&template.name)
        .bind(&template.subject_template)
        .bind(&template.text_body_template)
        .bind(&template.variables)
        .bind(template.locale.as_deref())
        .bind(now)
        .fetch_optional(&mut **transaction)
        .await
        .map_err(|_| MailCompositionPersistenceErrorV1::Database)?
        .ok_or(MailCompositionPersistenceErrorV1::StaleRevision)?
    };
    mutation_receipt(
        operation_id,
        &template.connection_id,
        MailCompositionEntityKindV1::Template,
        &template.template_id,
        row_u64(&row, "revision")?,
        false,
    )
}

async fn upsert_signature(
    transaction: &mut Transaction<'_, Postgres>,
    operation_id: &str,
    signature: &MailSignatureInputV1,
    expected_revision: Option<u64>,
    now: i64,
) -> Result<MailCompositionMutationReceiptV1, MailCompositionPersistenceErrorV1> {
    if signature.is_default {
        sqlx::query(
            r#"
            UPDATE makosh_data.mail_signatures
            SET is_default = FALSE,
                revision = revision + 1,
                updated_at_unix_seconds = $2
            WHERE connection_id = $1
              AND is_default = TRUE
              AND signature_id <> $3
            "#,
        )
        .bind(&signature.connection_id)
        .bind(now)
        .bind(&signature.signature_id)
        .execute(&mut **transaction)
        .await
        .map_err(|_| MailCompositionPersistenceErrorV1::Database)?;
    }
    let row = if let Some(expected_revision) = expected_revision {
        sqlx::query(
            r#"
            UPDATE makosh_data.mail_signatures
            SET revision = revision + 1,
                name = $4,
                text_body = $5,
                is_default = $6,
                updated_at_unix_seconds = $7
            WHERE connection_id = $1
              AND signature_id = $2
              AND revision = $3
            RETURNING revision
            "#,
        )
        .bind(&signature.connection_id)
        .bind(&signature.signature_id)
        .bind(i64_from_u64(expected_revision)?)
        .bind(&signature.name)
        .bind(&signature.text_body)
        .bind(signature.is_default)
        .bind(now)
        .fetch_optional(&mut **transaction)
        .await
        .map_err(|_| MailCompositionPersistenceErrorV1::Database)?
        .ok_or(MailCompositionPersistenceErrorV1::StaleRevision)?
    } else {
        sqlx::query(
            r#"
            INSERT INTO makosh_data.mail_signatures (
                connection_id,
                signature_id,
                revision,
                name,
                text_body,
                is_default,
                created_at_unix_seconds,
                updated_at_unix_seconds
            )
            VALUES ($1, $2, 1, $3, $4, $5, $6, $6)
            ON CONFLICT (connection_id, signature_id) DO NOTHING
            RETURNING revision
            "#,
        )
        .bind(&signature.connection_id)
        .bind(&signature.signature_id)
        .bind(&signature.name)
        .bind(&signature.text_body)
        .bind(signature.is_default)
        .bind(now)
        .fetch_optional(&mut **transaction)
        .await
        .map_err(|_| MailCompositionPersistenceErrorV1::Database)?
        .ok_or(MailCompositionPersistenceErrorV1::StaleRevision)?
    };
    mutation_receipt(
        operation_id,
        &signature.connection_id,
        MailCompositionEntityKindV1::Signature,
        &signature.signature_id,
        row_u64(&row, "revision")?,
        false,
    )
}

struct DeleteEntityV1<'a> {
    operation_id: &'a str,
    connection_id: &'a str,
    entity_id: &'a str,
    expected_revision: u64,
    kind: MailCompositionEntityKindV1,
}

async fn delete_entity(
    transaction: &mut Transaction<'_, Postgres>,
    request: DeleteEntityV1<'_>,
) -> Result<MailCompositionMutationReceiptV1, MailCompositionPersistenceErrorV1> {
    let expected_revision = i64_from_u64(request.expected_revision)?;
    let row = match request.kind {
        MailCompositionEntityKindV1::Draft => {
            sqlx::query(
                r#"
            DELETE FROM makosh_data.mail_drafts
            WHERE connection_id = $1 AND draft_id = $2 AND revision = $3
            RETURNING revision
            "#,
            )
            .bind(request.connection_id)
            .bind(request.entity_id)
            .bind(expected_revision)
            .fetch_optional(&mut **transaction)
            .await
        }
        MailCompositionEntityKindV1::Template => {
            sqlx::query(
                r#"
            DELETE FROM makosh_data.mail_templates
            WHERE connection_id = $1 AND template_id = $2 AND revision = $3
            RETURNING revision
            "#,
            )
            .bind(request.connection_id)
            .bind(request.entity_id)
            .bind(expected_revision)
            .fetch_optional(&mut **transaction)
            .await
        }
        MailCompositionEntityKindV1::Signature => {
            sqlx::query(
                r#"
            DELETE FROM makosh_data.mail_signatures
            WHERE connection_id = $1 AND signature_id = $2 AND revision = $3
            RETURNING revision
            "#,
            )
            .bind(request.connection_id)
            .bind(request.entity_id)
            .bind(expected_revision)
            .fetch_optional(&mut **transaction)
            .await
        }
    }
    .map_err(|_| MailCompositionPersistenceErrorV1::Database)?
    .ok_or(MailCompositionPersistenceErrorV1::StaleRevision)?;
    mutation_receipt(
        request.operation_id,
        request.connection_id,
        request.kind,
        request.entity_id,
        row_u64(&row, "revision")?,
        true,
    )
}

fn mutation_receipt(
    operation_id: &str,
    connection_id: &str,
    entity_kind: MailCompositionEntityKindV1,
    entity_id: &str,
    revision: u64,
    deleted: bool,
) -> Result<MailCompositionMutationReceiptV1, MailCompositionPersistenceErrorV1> {
    let receipt = MailCompositionMutationReceiptV1 {
        operation_id: operation_id.to_owned(),
        connection_id: connection_id.to_owned(),
        entity_kind,
        entity_id: entity_id.to_owned(),
        revision,
        deleted,
    };
    validate_composition_receipt(&receipt)
        .map_err(|_| MailCompositionPersistenceErrorV1::InvalidRow)?;
    Ok(receipt)
}

async fn get_draft(
    pool: &PgPool,
    connection_id: &str,
    draft_id: &str,
) -> Result<Option<MailDraftV1>, MailCompositionPersistenceErrorV1> {
    sqlx::query(
        r#"
        SELECT connection_id,
               draft_id,
               revision,
               mode,
               provider_conversation_id,
               in_reply_to_provider_message_id,
               to_recipients,
               cc_recipients,
               bcc_recipients,
               subject,
               text_body,
               template_id,
               signature_id,
               created_at_unix_seconds,
               updated_at_unix_seconds
        FROM makosh_data.mail_drafts
        WHERE connection_id = $1 AND draft_id = $2
        "#,
    )
    .bind(connection_id)
    .bind(draft_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| MailCompositionPersistenceErrorV1::Database)?
    .map(draft_from_row)
    .transpose()
}

async fn get_template(
    pool: &PgPool,
    connection_id: &str,
    template_id: &str,
) -> Result<Option<MailTemplateV1>, MailCompositionPersistenceErrorV1> {
    sqlx::query(
        r#"
        SELECT connection_id,
               template_id,
               revision,
               name,
               subject_template,
               text_body_template,
               variables,
               locale,
               created_at_unix_seconds,
               updated_at_unix_seconds
        FROM makosh_data.mail_templates
        WHERE connection_id = $1 AND template_id = $2
        "#,
    )
    .bind(connection_id)
    .bind(template_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| MailCompositionPersistenceErrorV1::Database)?
    .map(template_from_row)
    .transpose()
}

async fn get_signature(
    pool: &PgPool,
    connection_id: &str,
    signature_id: &str,
) -> Result<Option<MailSignatureV1>, MailCompositionPersistenceErrorV1> {
    sqlx::query(
        r#"
        SELECT connection_id,
               signature_id,
               revision,
               name,
               text_body,
               is_default,
               created_at_unix_seconds,
               updated_at_unix_seconds
        FROM makosh_data.mail_signatures
        WHERE connection_id = $1 AND signature_id = $2
        "#,
    )
    .bind(connection_id)
    .bind(signature_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| MailCompositionPersistenceErrorV1::Database)?
    .map(signature_from_row)
    .transpose()
}

async fn list_drafts(
    pool: &PgPool,
    connection_id: &str,
    cursor: Option<&str>,
    limit: u32,
) -> Result<MailCompositionPageV1<MailDraftV1>, MailCompositionPersistenceErrorV1> {
    let anchor = decode_cursor(cursor, CursorKindV1::Draft, connection_id)?;
    let rows = sqlx::query(
        r#"
        SELECT connection_id,
               draft_id,
               revision,
               mode,
               provider_conversation_id,
               in_reply_to_provider_message_id,
               to_recipients,
               cc_recipients,
               bcc_recipients,
               subject,
               text_body,
               template_id,
               signature_id,
               created_at_unix_seconds,
               updated_at_unix_seconds,
               sequence
        FROM makosh_data.mail_drafts
        WHERE connection_id = $1
          AND (
              $2::BIGINT IS NULL
              OR (updated_at_unix_seconds, sequence) < ($2, $3)
          )
        ORDER BY updated_at_unix_seconds DESC, sequence DESC
        LIMIT $4
        "#,
    )
    .bind(connection_id)
    .bind(anchor.map(|value| value.updated_at))
    .bind(anchor.map(|value| value.sequence))
    .bind(i64::from(limit) + 1)
    .fetch_all(pool)
    .await
    .map_err(|_| MailCompositionPersistenceErrorV1::Database)?;
    page_from_rows(
        rows,
        limit,
        CursorKindV1::Draft,
        connection_id,
        draft_from_row,
    )
}

async fn list_templates(
    pool: &PgPool,
    connection_id: &str,
    cursor: Option<&str>,
    limit: u32,
) -> Result<MailCompositionPageV1<MailTemplateV1>, MailCompositionPersistenceErrorV1> {
    let anchor = decode_cursor(cursor, CursorKindV1::Template, connection_id)?;
    let rows = sqlx::query(
        r#"
        SELECT connection_id,
               template_id,
               revision,
               name,
               subject_template,
               text_body_template,
               variables,
               locale,
               created_at_unix_seconds,
               updated_at_unix_seconds,
               sequence
        FROM makosh_data.mail_templates
        WHERE connection_id = $1
          AND (
              $2::BIGINT IS NULL
              OR (updated_at_unix_seconds, sequence) < ($2, $3)
          )
        ORDER BY updated_at_unix_seconds DESC, sequence DESC
        LIMIT $4
        "#,
    )
    .bind(connection_id)
    .bind(anchor.map(|value| value.updated_at))
    .bind(anchor.map(|value| value.sequence))
    .bind(i64::from(limit) + 1)
    .fetch_all(pool)
    .await
    .map_err(|_| MailCompositionPersistenceErrorV1::Database)?;
    page_from_rows(
        rows,
        limit,
        CursorKindV1::Template,
        connection_id,
        template_from_row,
    )
}

async fn list_signatures(
    pool: &PgPool,
    connection_id: &str,
    cursor: Option<&str>,
    limit: u32,
) -> Result<MailCompositionPageV1<MailSignatureV1>, MailCompositionPersistenceErrorV1> {
    let anchor = decode_cursor(cursor, CursorKindV1::Signature, connection_id)?;
    let rows = sqlx::query(
        r#"
        SELECT connection_id,
               signature_id,
               revision,
               name,
               text_body,
               is_default,
               created_at_unix_seconds,
               updated_at_unix_seconds,
               sequence
        FROM makosh_data.mail_signatures
        WHERE connection_id = $1
          AND (
              $2::BIGINT IS NULL
              OR (updated_at_unix_seconds, sequence) < ($2, $3)
          )
        ORDER BY updated_at_unix_seconds DESC, sequence DESC
        LIMIT $4
        "#,
    )
    .bind(connection_id)
    .bind(anchor.map(|value| value.updated_at))
    .bind(anchor.map(|value| value.sequence))
    .bind(i64::from(limit) + 1)
    .fetch_all(pool)
    .await
    .map_err(|_| MailCompositionPersistenceErrorV1::Database)?;
    page_from_rows(
        rows,
        limit,
        CursorKindV1::Signature,
        connection_id,
        signature_from_row,
    )
}

fn page_from_rows<T>(
    mut rows: Vec<PgRow>,
    limit: u32,
    kind: CursorKindV1,
    connection_id: &str,
    from_row: impl Fn(PgRow) -> Result<T, MailCompositionPersistenceErrorV1>,
) -> Result<MailCompositionPageV1<T>, MailCompositionPersistenceErrorV1> {
    let has_more = rows.len() > limit as usize;
    if has_more {
        rows.pop();
    }
    let next_cursor = if has_more {
        let last = rows
            .last()
            .ok_or(MailCompositionPersistenceErrorV1::InvalidRow)?;
        Some(encode_cursor(
            kind,
            connection_id,
            CursorAnchorV1 {
                updated_at: row_i64(last, "updated_at_unix_seconds")?,
                sequence: row_i64(last, "sequence")?,
            },
        ))
    } else {
        None
    };
    let items = rows
        .into_iter()
        .map(from_row)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(MailCompositionPageV1 { items, next_cursor })
}

fn draft_from_row(row: PgRow) -> Result<MailDraftV1, MailCompositionPersistenceErrorV1> {
    Ok(MailDraftV1 {
        connection_id: row_string(&row, "connection_id")?,
        draft_id: row_string(&row, "draft_id")?,
        revision: row_u64(&row, "revision")?,
        mode: mode_from_id(row_i16(&row, "mode")?)?,
        provider_conversation_id: row_optional_string(&row, "provider_conversation_id")?,
        in_reply_to_provider_message_id: row_optional_string(
            &row,
            "in_reply_to_provider_message_id",
        )?,
        to_recipients: row_strings(&row, "to_recipients")?,
        cc_recipients: row_strings(&row, "cc_recipients")?,
        bcc_recipients: row_strings(&row, "bcc_recipients")?,
        subject: row_string(&row, "subject")?,
        text_body: row_string(&row, "text_body")?,
        template_id: row_optional_string(&row, "template_id")?,
        signature_id: row_optional_string(&row, "signature_id")?,
        created_at_unix_seconds: row_i64(&row, "created_at_unix_seconds")?,
        updated_at_unix_seconds: row_i64(&row, "updated_at_unix_seconds")?,
    })
}

fn template_from_row(row: PgRow) -> Result<MailTemplateV1, MailCompositionPersistenceErrorV1> {
    Ok(MailTemplateV1 {
        connection_id: row_string(&row, "connection_id")?,
        template_id: row_string(&row, "template_id")?,
        revision: row_u64(&row, "revision")?,
        name: row_string(&row, "name")?,
        subject_template: row_string(&row, "subject_template")?,
        text_body_template: row_string(&row, "text_body_template")?,
        variables: row_strings(&row, "variables")?,
        locale: row_optional_string(&row, "locale")?,
        created_at_unix_seconds: row_i64(&row, "created_at_unix_seconds")?,
        updated_at_unix_seconds: row_i64(&row, "updated_at_unix_seconds")?,
    })
}

fn signature_from_row(row: PgRow) -> Result<MailSignatureV1, MailCompositionPersistenceErrorV1> {
    Ok(MailSignatureV1 {
        connection_id: row_string(&row, "connection_id")?,
        signature_id: row_string(&row, "signature_id")?,
        revision: row_u64(&row, "revision")?,
        name: row_string(&row, "name")?,
        text_body: row_string(&row, "text_body")?,
        is_default: row_bool(&row, "is_default")?,
        created_at_unix_seconds: row_i64(&row, "created_at_unix_seconds")?,
        updated_at_unix_seconds: row_i64(&row, "updated_at_unix_seconds")?,
    })
}

#[derive(Clone, Copy)]
enum CursorKindV1 {
    Draft,
    Template,
    Signature,
}

impl CursorKindV1 {
    const fn label(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Template => "template",
            Self::Signature => "signature",
        }
    }
}

#[derive(Clone, Copy)]
struct CursorAnchorV1 {
    updated_at: i64,
    sequence: i64,
}

fn encode_cursor(kind: CursorKindV1, connection_id: &str, anchor: CursorAnchorV1) -> String {
    let scope = cursor_scope(kind, connection_id);
    let mut bytes = Vec::with_capacity(48);
    bytes.extend_from_slice(&scope);
    bytes.extend_from_slice(&anchor.updated_at.to_be_bytes());
    bytes.extend_from_slice(&anchor.sequence.to_be_bytes());
    hex_encode(&bytes)
}

fn decode_cursor(
    cursor: Option<&str>,
    kind: CursorKindV1,
    connection_id: &str,
) -> Result<Option<CursorAnchorV1>, MailCompositionPersistenceErrorV1> {
    let Some(cursor) = cursor else {
        return Ok(None);
    };
    let bytes = hex_decode(cursor)?;
    if bytes.len() != 48 || bytes[..32] != cursor_scope(kind, connection_id) {
        return Err(MailCompositionPersistenceErrorV1::InvalidCursor);
    }
    let updated_at = i64::from_be_bytes(
        bytes[32..40]
            .try_into()
            .map_err(|_| MailCompositionPersistenceErrorV1::InvalidCursor)?,
    );
    let sequence = i64::from_be_bytes(
        bytes[40..48]
            .try_into()
            .map_err(|_| MailCompositionPersistenceErrorV1::InvalidCursor)?,
    );
    if updated_at <= 0 || sequence <= 0 {
        return Err(MailCompositionPersistenceErrorV1::InvalidCursor);
    }
    Ok(Some(CursorAnchorV1 {
        updated_at,
        sequence,
    }))
}

fn cursor_scope(kind: CursorKindV1, connection_id: &str) -> [u8; 32] {
    Sha256::digest(
        [
            b"mail-composition-cursor-v1".as_slice(),
            kind.label().as_bytes(),
            connection_id.as_bytes(),
        ]
        .join(&0),
    )
    .into()
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(char::from(HEX[(byte >> 4) as usize]));
        encoded.push(char::from(HEX[(byte & 0x0f) as usize]));
    }
    encoded
}

fn hex_decode(value: &str) -> Result<Vec<u8>, MailCompositionPersistenceErrorV1> {
    if !value.len().is_multiple_of(2) {
        return Err(MailCompositionPersistenceErrorV1::InvalidCursor);
    }
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let high = hex_nibble(pair[0])?;
            let low = hex_nibble(pair[1])?;
            Ok((high << 4) | low)
        })
        .collect()
}

fn hex_nibble(value: u8) -> Result<u8, MailCompositionPersistenceErrorV1> {
    match value {
        b'0'..=b'9' => Ok(value - b'0'),
        b'a'..=b'f' => Ok(value - b'a' + 10),
        _ => Err(MailCompositionPersistenceErrorV1::InvalidCursor),
    }
}

const fn mode_id(value: MailCompositionModeV1) -> i16 {
    match value {
        MailCompositionModeV1::New => 1,
        MailCompositionModeV1::Reply => 2,
        MailCompositionModeV1::ReplyAll => 3,
        MailCompositionModeV1::Forward => 4,
        MailCompositionModeV1::Redirect => 5,
    }
}

fn mode_from_id(value: i16) -> Result<MailCompositionModeV1, MailCompositionPersistenceErrorV1> {
    match value {
        1 => Ok(MailCompositionModeV1::New),
        2 => Ok(MailCompositionModeV1::Reply),
        3 => Ok(MailCompositionModeV1::ReplyAll),
        4 => Ok(MailCompositionModeV1::Forward),
        5 => Ok(MailCompositionModeV1::Redirect),
        _ => Err(MailCompositionPersistenceErrorV1::InvalidRow),
    }
}

const fn entity_kind_id(value: MailCompositionEntityKindV1) -> i16 {
    match value {
        MailCompositionEntityKindV1::Draft => 1,
        MailCompositionEntityKindV1::Template => 2,
        MailCompositionEntityKindV1::Signature => 3,
    }
}

fn entity_kind_from_id(
    value: i16,
) -> Result<MailCompositionEntityKindV1, MailCompositionPersistenceErrorV1> {
    match value {
        1 => Ok(MailCompositionEntityKindV1::Draft),
        2 => Ok(MailCompositionEntityKindV1::Template),
        3 => Ok(MailCompositionEntityKindV1::Signature),
        _ => Err(MailCompositionPersistenceErrorV1::InvalidRow),
    }
}

fn i64_from_u64(value: u64) -> Result<i64, MailCompositionPersistenceErrorV1> {
    i64::try_from(value).map_err(|_| MailCompositionPersistenceErrorV1::InvalidInput)
}

fn row_string(row: &PgRow, column: &str) -> Result<String, MailCompositionPersistenceErrorV1> {
    row.try_get(column)
        .map_err(|_| MailCompositionPersistenceErrorV1::InvalidRow)
}

fn row_optional_string(
    row: &PgRow,
    column: &str,
) -> Result<Option<String>, MailCompositionPersistenceErrorV1> {
    row.try_get(column)
        .map_err(|_| MailCompositionPersistenceErrorV1::InvalidRow)
}

fn row_strings(
    row: &PgRow,
    column: &str,
) -> Result<Vec<String>, MailCompositionPersistenceErrorV1> {
    row.try_get(column)
        .map_err(|_| MailCompositionPersistenceErrorV1::InvalidRow)
}

fn row_i16(row: &PgRow, column: &str) -> Result<i16, MailCompositionPersistenceErrorV1> {
    row.try_get(column)
        .map_err(|_| MailCompositionPersistenceErrorV1::InvalidRow)
}

fn row_i64(row: &PgRow, column: &str) -> Result<i64, MailCompositionPersistenceErrorV1> {
    row.try_get(column)
        .map_err(|_| MailCompositionPersistenceErrorV1::InvalidRow)
}

fn row_u64(row: &PgRow, column: &str) -> Result<u64, MailCompositionPersistenceErrorV1> {
    let value = row_i64(row, column)?;
    u64::try_from(value).map_err(|_| MailCompositionPersistenceErrorV1::InvalidRow)
}

fn row_bool(row: &PgRow, column: &str) -> Result<bool, MailCompositionPersistenceErrorV1> {
    row.try_get(column)
        .map_err(|_| MailCompositionPersistenceErrorV1::InvalidRow)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_is_mail_owned_and_has_no_cross_owner_foreign_keys() {
        assert_eq!(
            MAIL_SCHEMA_V11
                .matches("CREATE TABLE IF NOT EXISTS makosh_data.")
                .count(),
            4
        );
        assert!(MAIL_SCHEMA_V11.contains("mail_composition_commands"));
        assert!(MAIL_SCHEMA_V11.contains("mail_drafts"));
        assert!(MAIL_SCHEMA_V11.contains("mail_templates"));
        assert!(MAIL_SCHEMA_V11.contains("mail_signatures"));
        assert!(!MAIL_SCHEMA_V11.contains("communications"));
        assert!(!MAIL_SCHEMA_V11.contains("REFERENCES"));
    }

    #[test]
    fn cursors_are_scoped_and_tamper_evident() {
        let cursor = encode_cursor(
            CursorKindV1::Draft,
            "account-1",
            CursorAnchorV1 {
                updated_at: 100,
                sequence: 7,
            },
        );
        let decoded = decode_cursor(Some(&cursor), CursorKindV1::Draft, "account-1")
            .expect("decode")
            .expect("anchor");
        assert_eq!(decoded.updated_at, 100);
        assert_eq!(decoded.sequence, 7);
        assert_eq!(
            decode_cursor(Some(&cursor), CursorKindV1::Template, "account-1").err(),
            Some(MailCompositionPersistenceErrorV1::InvalidCursor)
        );
        assert_eq!(
            decode_cursor(Some(&cursor), CursorKindV1::Draft, "account-2").err(),
            Some(MailCompositionPersistenceErrorV1::InvalidCursor)
        );
    }
}
