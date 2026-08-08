use chrono::Utc;
use serde_json::{Value, json};

use super::MessageProjectionStore;
use crate::domains::communications::messages::errors::MessageProjectionError;
use crate::domains::communications::messages::ids::message_id;
use crate::domains::communications::messages::models::{NewProjectedMessage, ProjectedMessage};
use crate::domains::communications::messages::rows::row_to_projected_message;
use makosh_communications_postgres::provider_commands::CommunicationProviderCommandStore;

impl MessageProjectionStore {
    pub async fn upsert_email_message(
        &self,
        message: &NewProjectedMessage,
    ) -> Result<ProjectedMessage, MessageProjectionError> {
        message.validate()?;
        let canonical_message_id = message_id(&message.account_id, &message.provider_record_id);
        let mut transaction = self.pool.begin().await?;

        let row = sqlx::query(
            r#"
            INSERT INTO communication_messages (
                message_id,
                raw_record_id,
                observation_id,
                account_id,
                provider_record_id,
                subject,
                sender,
                recipients,
                body_text,
                occurred_at,
                channel_kind,
                conversation_id,
                sender_display_name,
                delivery_state,
                message_metadata,
                is_read,
                read_changed_at,
                read_origin
            )
            SELECT
                $1,
                raw_record_id,
                observation_id,
                account_id,
                $4,
                $5,
                $6,
                $7,
                $8,
                $9,
                'email',
                NULL,
                $6,
                CASE
                    WHEN $11->>'transport' = 'imap'
                         AND EXISTS (
                            SELECT 1
                            FROM communication_mail_provider_resources AS resource
                            WHERE resource.account_id = $3
                              AND resource.resource_kind = 'folder'
                              AND resource.provider_resource_id = $11->>'mailbox'
                              AND resource.semantic_role = 'sent'
                        )
                    THEN 'sent'
                    ELSE $10
                END,
                $11 || COALESCE(
                        (
                            SELECT jsonb_build_object(
                                'outbox_id', outbox.outbox_id,
                                'outbox_status', outbox.status
                            )
                            FROM communication_outbox AS outbox
                            WHERE outbox.account_id = $3
                              AND outbox.status = 'sent'
                              AND (
                                  outbox.provider_message_id = $4
                                  OR (
                                      $11->>'transport' = 'imap'
                                      AND outbox.metadata->>'rfc822_message_id'
                                          = $11->>'rfc822_message_id'
                                  )
                              )
                            ORDER BY outbox.sent_at DESC NULLS LAST, outbox.outbox_id ASC
                            LIMIT 1
                        ),
                        '{}'::jsonb
                    ),
                CASE
                    WHEN COALESCE($11->'label_ids', '[]'::jsonb) ? 'UNREAD' THEN false
                    WHEN jsonb_typeof($11->'label_ids') = 'array' THEN true
                    ELSE COALESCE(($11->>'is_read')::boolean, false)
                END,
                now(),
                'provider_observed'
            FROM communication_raw_records
            WHERE raw_record_id = $2
              AND account_id = $3
              AND record_kind = 'email_message'
            ON CONFLICT (account_id, provider_record_id)
            DO UPDATE SET
                raw_record_id = EXCLUDED.raw_record_id,
                observation_id = EXCLUDED.observation_id,
                subject = EXCLUDED.subject,
                sender = EXCLUDED.sender,
                recipients = EXCLUDED.recipients,
                body_text = EXCLUDED.body_text,
                occurred_at = EXCLUDED.occurred_at,
                channel_kind = EXCLUDED.channel_kind,
                conversation_id = EXCLUDED.conversation_id,
                sender_display_name = EXCLUDED.sender_display_name,
                delivery_state = EXCLUDED.delivery_state,
                message_metadata = CASE
                    WHEN communication_messages.message_metadata->>'starred_origin' = 'local_user'
                    THEN EXCLUDED.message_metadata || jsonb_build_object(
                        'starred',
                        COALESCE(communication_messages.message_metadata->'starred', 'false'::jsonb),
                        'starred_origin',
                        'local_user'
                    )
                    ELSE EXCLUDED.message_metadata
                END,
                is_read = CASE
                    WHEN communication_messages.read_origin = 'local_user'
                        THEN communication_messages.is_read
                    ELSE EXCLUDED.is_read
                END,
                read_changed_at = CASE
                    WHEN communication_messages.read_origin = 'local_user'
                        THEN communication_messages.read_changed_at
                    ELSE EXCLUDED.read_changed_at
                END,
                read_origin = CASE
                    WHEN communication_messages.read_origin = 'local_user'
                        THEN communication_messages.read_origin
                    ELSE EXCLUDED.read_origin
                END,
                projected_at = now()
            RETURNING
                message_id,
                raw_record_id,
                observation_id,
                account_id,
                provider_record_id,
                subject,
                sender,
                recipients,
                body_text,
                occurred_at,
                projected_at,
                channel_kind,
                conversation_id,
                sender_display_name,
                delivery_state,
                message_metadata,
                workflow_state,
                importance_score,
                ai_category,
                ai_summary,
                ai_summary_generated_at,
                (SELECT s.ai_state FROM communication_ai_states s WHERE s.message_id = communication_messages.message_id) AS ai_state,
                local_state,
                local_state_changed_at,
                local_state_reason,
                is_read,
                read_changed_at,
                read_origin
            "#,
        )
        .bind(&canonical_message_id)
        .bind(&message.raw_record_id)
        .bind(&message.account_id)
        .bind(&message.provider_record_id)
        .bind(&message.subject)
        .bind(&message.sender)
        .bind(json!(message.recipients))
        .bind(&message.body_text)
        .bind(message.occurred_at)
        .bind(&message.delivery_state)
        .bind(&message.message_metadata)
        .fetch_optional(&mut *transaction)
        .await?;

        let Some(row) = row else {
            return Err(MessageProjectionError::RawRecordTupleMismatch {
                raw_record_id: message.raw_record_id.clone(),
                account_id: message.account_id.clone(),
                provider_record_id: message.provider_record_id.clone(),
            });
        };

        let projected = row_to_projected_message(row)?;
        reconcile_provider_folder_memberships_in_transaction(
            &mut transaction,
            &projected,
            &message.message_metadata,
        )
        .await?;
        transaction.commit().await?;
        let read_reconciled = self
            .reconcile_observed_read_state(
                &projected,
                provider_read_state(&message.message_metadata),
            )
            .await?;
        let starred_reconciled = self
            .reconcile_observed_starred_state(
                &projected,
                provider_starred_state(&message.message_metadata),
            )
            .await?;
        if read_reconciled || starred_reconciled {
            return self
                .message(&projected.message_id)
                .await?
                .ok_or(MessageProjectionError::MessageNotFound);
        }
        Ok(projected)
    }

    async fn reconcile_observed_read_state(
        &self,
        message: &ProjectedMessage,
        provider_is_read: bool,
    ) -> Result<bool, MessageProjectionError> {
        // A conflicting observation must not acknowledge a newer local intent.
        if message.is_read != provider_is_read {
            return Ok(false);
        }
        let command_kinds = if provider_is_read {
            ["mark_read"]
        } else {
            ["mark_unread"]
        };
        let observed = CommunicationProviderCommandStore::new(self.pool.clone())
            .mark_observed_by_provider_message(
                &message.account_id,
                "mail",
                &message.provider_record_id,
                &command_kinds,
                Utc::now(),
                json!({ "is_read": provider_is_read }),
            )
            .await?;
        if observed.is_empty() {
            return Ok(false);
        }
        let updated = sqlx::query(
            r#"
            UPDATE communication_messages
            SET read_origin = 'provider_observed',
                read_changed_at = now(),
                projected_at = now()
            WHERE message_id = $1
              AND is_read = $2
              AND read_origin = 'local_user'
            "#,
        )
        .bind(&message.message_id)
        .bind(provider_is_read)
        .execute(&self.pool)
        .await?;
        Ok(updated.rows_affected() > 0)
    }

    async fn reconcile_observed_starred_state(
        &self,
        message: &ProjectedMessage,
        provider_starred: Option<bool>,
    ) -> Result<bool, MessageProjectionError> {
        let Some(provider_starred) = provider_starred else {
            return Ok(false);
        };
        if message
            .message_metadata
            .get("starred")
            .and_then(Value::as_bool)
            != Some(provider_starred)
        {
            return Ok(false);
        }
        let observed = CommunicationProviderCommandStore::new(self.pool.clone())
            .mark_observed_by_provider_message(
                &message.account_id,
                "mail",
                &message.provider_record_id,
                &["star", "unstar"],
                Utc::now(),
                json!({ "starred": provider_starred }),
            )
            .await?;
        if observed.is_empty() {
            return Ok(false);
        }
        let updated = sqlx::query(
            r#"
            UPDATE communication_messages
            SET message_metadata = jsonb_set(
                    COALESCE(message_metadata, '{}'::jsonb),
                    '{starred_origin}',
                    '"provider_observed"'::jsonb,
                    true
                ),
                projected_at = now()
            WHERE message_id = $1
              AND message_metadata->>'starred_origin' = 'local_user'
              AND (message_metadata->>'starred')::boolean = $2
            "#,
        )
        .bind(&message.message_id)
        .bind(provider_starred)
        .execute(&self.pool)
        .await?;
        Ok(updated.rows_affected() > 0)
    }
}

async fn reconcile_provider_folder_memberships_in_transaction(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    message: &ProjectedMessage,
    metadata: &Value,
) -> Result<(), MessageProjectionError> {
    sqlx::query(
        r#"
        DELETE FROM communication_folder_messages AS membership
        WHERE membership.message_id = $1
          AND membership.metadata->>'source' = 'provider_resource_mapping'
          AND NOT EXISTS (
              SELECT 1
              FROM communication_mail_provider_resources AS resource
              WHERE resource.account_id = $2
                AND resource.local_folder_id = membership.folder_id
                AND (
                    (
                        $3::jsonb->>'provider' = 'gmail'
                        AND resource.resource_kind = 'label'
                        AND COALESCE($3::jsonb->'label_ids', '[]'::jsonb)
                            ? resource.provider_resource_id
                    )
                    OR (
                        $3::jsonb->>'transport' = 'imap'
                        AND resource.resource_kind = 'folder'
                        AND resource.provider_resource_id = $3::jsonb->>'mailbox'
                    )
                )
          )
        "#,
    )
    .bind(&message.message_id)
    .bind(&message.account_id)
    .bind(metadata)
    .execute(&mut **transaction)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO communication_folder_messages (
            folder_id, message_id, added_at, last_operation, metadata
        )
        SELECT
            resource.local_folder_id,
            $1,
            now(),
            'copy',
            jsonb_build_object(
                'source', 'provider_resource_mapping',
                'mapping_id', resource.mapping_id
            )
        FROM communication_mail_provider_resources AS resource
        WHERE resource.account_id = $2
          AND resource.local_folder_id IS NOT NULL
          AND (
              (
                  $3::jsonb->>'provider' = 'gmail'
                  AND resource.resource_kind = 'label'
                  AND COALESCE($3::jsonb->'label_ids', '[]'::jsonb)
                      ? resource.provider_resource_id
              )
              OR (
                  $3::jsonb->>'transport' = 'imap'
                  AND resource.resource_kind = 'folder'
                  AND resource.provider_resource_id = $3::jsonb->>'mailbox'
              )
          )
        ON CONFLICT (folder_id, message_id)
        DO UPDATE SET metadata = CASE
                WHEN communication_folder_messages.metadata->>'source' = 'provider_resource_mapping'
                THEN EXCLUDED.metadata
                ELSE communication_folder_messages.metadata
            END
        "#,
    )
    .bind(&message.message_id)
    .bind(&message.account_id)
    .bind(metadata)
    .execute(&mut **transaction)
    .await?;
    Ok(())
}

fn provider_read_state(metadata: &Value) -> bool {
    makosh_communications_api::provider_state::observed_read_state(metadata)
}

fn provider_starred_state(metadata: &Value) -> Option<bool> {
    makosh_communications_api::provider_state::observed_starred_state(metadata)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{provider_read_state, provider_starred_state};

    #[test]
    fn provider_read_state_uses_gmail_unread_label_as_authoritative() {
        assert!(provider_read_state(&json!({ "label_ids": ["INBOX"] })));
        assert!(!provider_read_state(
            &json!({ "label_ids": ["INBOX", "UNREAD"] })
        ));
    }

    #[test]
    fn provider_read_state_uses_imap_seen_projection_without_labels() {
        assert!(provider_read_state(&json!({ "is_read": true })));
        assert!(!provider_read_state(&json!({ "is_read": false })));
    }

    #[test]
    fn provider_starred_state_requires_an_explicit_provider_flag() {
        assert_eq!(
            provider_starred_state(&json!({ "starred": true })),
            Some(true)
        );
        assert_eq!(provider_starred_state(&json!({})), None);
    }
}
