use super::*;

pub(crate) async fn capture_outbox_transition_observation(
    transaction: &mut Transaction<'_, Postgres>,
    item: &CommunicationOutboxItem,
    operation: &str,
    metadata: Value,
) -> Result<(), CommunicationOutboxError> {
    let observation = ObservationStore::capture_in_transaction(
        transaction,
        &NewObservation::new(
            "COMMUNICATION_OUTBOX",
            ObservationOriginKind::LocalRuntime,
            Utc::now(),
            json!({
                "outbox_id": item.outbox_id,
                "account_id": item.account_id,
                "status": item.status.as_str(),
                "scheduled_send_at": item.scheduled_send_at,
                "undo_deadline_at": item.undo_deadline_at,
                "send_attempts": item.send_attempts,
                "provider_message_id": item.provider_message_id,
                "last_error": item.last_error,
                "operation": operation,
            }),
            format!("outbox://{}/{}", item.outbox_id, operation),
        )
        .provenance(json!({
            "captured_by": "mail.outbox",
            "operation": operation,
        })),
    )
    .await?;
    link_mail_entity_in_transaction(
        transaction,
        &observation.observation_id,
        "outbox_item",
        item.outbox_id.clone(),
        "outbox_status_transition",
        metadata,
        None,
    )
    .await?;
    Ok(())
}

pub(crate) async fn existing_draft_id_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    draft_id: Option<&str>,
    account_id: &str,
) -> Result<Option<String>, CommunicationOutboxError> {
    let Some(draft_id) = draft_id.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM communication_drafts WHERE draft_id = $1 AND account_id = $2)",
    )
    .bind(draft_id)
    .bind(account_id)
    .fetch_one(&mut **transaction)
    .await?;

    if !exists {
        return Err(CommunicationOutboxError::Invalid(
            "draft_id is unavailable for this account",
        ));
    }
    Ok(Some(draft_id.to_owned()))
}

pub(crate) async fn copy_draft_attachments_to_outbox_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    outbox_id: &str,
    draft_id: &str,
) -> Result<(), CommunicationOutboxError> {
    sqlx::query(
        r#"
        INSERT INTO communication_outbox_attachments (
            outbox_id, attachment_id, disposition, content_id, sort_order
        )
        SELECT $1, attachment_id, disposition, content_id, sort_order
        FROM communication_draft_attachments
        WHERE draft_id = $2
        ORDER BY sort_order
        "#,
    )
    .bind(outbox_id)
    .bind(draft_id)
    .execute(&mut **transaction)
    .await?;
    Ok(())
}

pub(crate) async fn ensure_canonical_account_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: Option<&str>,
) -> Result<(), CommunicationOutboxError> {
    let Some(account_id) = account_id else {
        return Ok(());
    };

    let result = sqlx::query(
        r#"
        INSERT INTO communication_accounts (
            account_id, provider_kind, display_name, external_account_id, config, metadata, created_at, updated_at
        )
        SELECT
            account_id,
            provider_kind,
            display_name,
            external_account_id,
            config,
            '{}'::jsonb,
            created_at,
            updated_at
        FROM communication_provider_accounts
        WHERE account_id = $1
        ON CONFLICT (account_id)
        DO UPDATE SET
            provider_kind = EXCLUDED.provider_kind,
            display_name = EXCLUDED.display_name,
            external_account_id = EXCLUDED.external_account_id,
            config = EXCLUDED.config,
            updated_at = EXCLUDED.updated_at
        "#,
    )
    .bind(account_id.trim())
    .execute(&mut **transaction)
    .await?;
    if result.rows_affected() == 0 {
        return Err(CommunicationOutboxError::Invalid("account_id"));
    }

    Ok(())
}

pub(crate) fn outbox_returning_query(prefix: &str, qualifier: &str) -> String {
    format!(
        r#"{prefix}
        RETURNING
            {qualifier}.outbox_id,
            {qualifier}.account_id,
            {qualifier}.draft_id,
            {qualifier}.to_participants AS to_recipients,
            {qualifier}.cc_participants AS cc_recipients,
            {qualifier}.bcc_participants AS bcc_recipients,
            {qualifier}.subject,
            {qualifier}.body_text,
            {qualifier}.body_html,
            {qualifier}.status,
            {qualifier}.scheduled_send_at,
            {qualifier}.undo_deadline_at,
            {qualifier}.send_attempts,
            {qualifier}.claimed_at,
            {qualifier}.sent_at,
            {qualifier}.provider_message_id,
            {qualifier}.last_error,
            {qualifier}.metadata,
            {qualifier}.created_at,
            {qualifier}.updated_at
        "#
    )
}

pub(crate) fn row_to_outbox_item(
    row: PgRow,
) -> Result<CommunicationOutboxItem, CommunicationOutboxError> {
    let status: String = row.try_get("status")?;
    Ok(CommunicationOutboxItem {
        outbox_id: row.try_get("outbox_id")?,
        account_id: row.try_get("account_id")?,
        draft_id: row.try_get("draft_id")?,
        to_recipients: string_array(row.try_get("to_recipients")?)?,
        cc_recipients: string_array(row.try_get("cc_recipients")?)?,
        bcc_recipients: string_array(row.try_get("bcc_recipients")?)?,
        subject: row.try_get("subject")?,
        body_text: row.try_get("body_text")?,
        body_html: row.try_get("body_html")?,
        status: CommunicationOutboxStatus::parse(&status)
            .unwrap_or(CommunicationOutboxStatus::Queued),
        scheduled_send_at: row.try_get("scheduled_send_at")?,
        undo_deadline_at: row.try_get("undo_deadline_at")?,
        send_attempts: row.try_get("send_attempts")?,
        claimed_at: row.try_get("claimed_at")?,
        sent_at: row.try_get("sent_at")?,
        provider_message_id: row.try_get("provider_message_id")?,
        last_error: row.try_get("last_error")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub(crate) fn string_array(value: Value) -> Result<Vec<String>, CommunicationOutboxError> {
    serde_json::from_value(value).map_err(CommunicationOutboxError::Serde)
}

pub(crate) fn validate_non_empty(
    field_name: &'static str,
    value: &str,
) -> Result<(), CommunicationOutboxError> {
    if value.trim().is_empty() {
        return Err(CommunicationOutboxError::Invalid(field_name));
    }

    Ok(())
}

pub(crate) fn validate_limit(limit: i64) -> Result<i64, CommunicationOutboxError> {
    if !(1..=500).contains(&limit) {
        return Err(CommunicationOutboxError::Invalid(
            "limit must be between 1 and 500",
        ));
    }

    Ok(limit)
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct OutboxListCursor {
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) outbox_id: String,
}

pub(crate) fn encode_outbox_list_cursor(
    item: &CommunicationOutboxItem,
) -> Result<String, CommunicationOutboxError> {
    let cursor = OutboxListCursor {
        created_at: item.created_at,
        outbox_id: item.outbox_id.clone(),
    };
    let bytes = serde_json::to_vec(&cursor).map_err(|_| CommunicationOutboxError::InvalidCursor)?;

    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

pub(crate) fn decode_outbox_list_cursor(
    cursor: &str,
) -> Result<OutboxListCursor, CommunicationOutboxError> {
    let bytes = URL_SAFE_NO_PAD
        .decode(cursor)
        .map_err(|_| CommunicationOutboxError::InvalidCursor)?;
    let cursor: OutboxListCursor =
        serde_json::from_slice(&bytes).map_err(|_| CommunicationOutboxError::InvalidCursor)?;
    if cursor.outbox_id.trim().is_empty() {
        return Err(CommunicationOutboxError::InvalidCursor);
    }

    Ok(cursor)
}

pub(crate) fn outbox_delivery_event(
    event_type: &str,
    item: &CommunicationOutboxItem,
) -> Result<NewEventEnvelope, CommunicationOutboxError> {
    let recipient_count =
        item.to_recipients.len() + item.cc_recipients.len() + item.bcc_recipients.len();
    Ok(NewEventEnvelope::builder(
        generate_outbox_event_id(event_type, &item.outbox_id),
        event_type,
        Utc::now(),
        json!({ "kind": "mail_outbox_worker" }),
        json!({
            "kind": "email_outbox",
            "id": item.outbox_id,
            "account_id": item.account_id,
            "status": item.status.as_str(),
        }),
    )
    .actor(json!({ "actor_id": "makosh-outbox-worker" }))
    .payload(json!({
        "outbox_id": item.outbox_id,
        "account_id": item.account_id,
        "status": item.status.as_str(),
        "provider_message_id": item.provider_message_id,
        "last_error": item.last_error,
        "send_attempts": item.send_attempts,
        "scheduled_send_at": item.scheduled_send_at,
        "undo_deadline_at": item.undo_deadline_at,
        "sent_at": item.sent_at,
        "recipient_count": recipient_count,
    }))
    .provenance(json!({
        "source_kind": "local_outbox",
        "source_id": item.outbox_id,
    }))
    .correlation_id(item.outbox_id.clone())
    .build()?)
}

pub(crate) fn generate_outbox_event_id(event_type: &str, outbox_id: &str) -> String {
    format!(
        "mail_outbox_event:{event_type}:{outbox_id}:{:x}",
        Utc::now().timestamp_nanos_opt().unwrap_or_default()
    )
}
