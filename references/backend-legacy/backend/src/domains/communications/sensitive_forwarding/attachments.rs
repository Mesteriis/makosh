use super::*;

pub(super) async fn plan_forwardable_attachments(
    transaction: &mut Transaction<'_, sqlx::Postgres>,
    message_id: &str,
) -> Result<AttachmentForwardingPlan, SensitiveForwardingError> {
    let rows = sqlx::query(
        r#"
        SELECT
            attachment_id,
            blob_id,
            filename,
            content_type,
            size_bytes,
            sha256,
            scan_status,
            scan_engine,
            scan_checked_at,
            scan_summary,
            scan_metadata
        FROM communication_attachments
        WHERE message_id = $1
        ORDER BY created_at ASC, attachment_id ASC
        FOR UPDATE
        "#,
    )
    .bind(message_id)
    .fetch_all(&mut **transaction)
    .await?;

    let mut plan = AttachmentForwardingPlan::default();
    let mut total_bytes = 0_i64;
    for row in rows {
        let scan_status: String = row.try_get("scan_status")?;
        if scan_status != "clean" {
            plan.unsafe_withheld += 1;
            continue;
        }
        let size_bytes: i64 = row.try_get("size_bytes")?;
        if plan.attachments.len() >= MAX_SENSITIVE_FORWARDING_ATTACHMENTS
            || total_bytes.saturating_add(size_bytes) > MAX_SENSITIVE_FORWARDING_ATTACHMENT_BYTES
        {
            plan.delivery_limit_withheld += 1;
            continue;
        }
        total_bytes = total_bytes.saturating_add(size_bytes);
        plan.attachments.push(ForwardableAttachment {
            attachment_id: row.try_get("attachment_id")?,
            blob_id: row.try_get("blob_id")?,
            filename: row.try_get("filename")?,
            content_type: row.try_get("content_type")?,
            size_bytes,
            sha256: row.try_get("sha256")?,
            scan_engine: row.try_get("scan_engine")?,
            scan_checked_at: row.try_get("scan_checked_at")?,
            scan_summary: row.try_get("scan_summary")?,
            scan_metadata: row.try_get("scan_metadata")?,
        });
    }
    Ok(plan)
}

pub(super) async fn copy_forwardable_attachments_to_outbox(
    transaction: &mut Transaction<'_, sqlx::Postgres>,
    outbox_id: &str,
    delivery_account_id: &str,
    source_message_id: &str,
    policy_id: &str,
    plan: &AttachmentForwardingPlan,
) -> Result<(), SensitiveForwardingError> {
    for (sort_order, source) in plan.attachments.iter().enumerate() {
        let attachment_id = format!(
            "sensitive-forwarding-attachment:{outbox_id}:{}",
            source.attachment_id
        );
        sqlx::query(
            r#"
            INSERT INTO communication_attachment_imports (
                attachment_id,
                account_id,
                channel_kind,
                blob_id,
                filename,
                content_type,
                size_bytes,
                sha256,
                source_kind,
                imported_by,
                scan_status,
                scan_engine,
                scan_checked_at,
                scan_summary,
                scan_metadata,
                metadata,
                updated_at
            )
            VALUES (
                $1, $2, 'mail', $3, $4, $5, $6, $7,
                'sensitive_forwarding', 'makosh-mail-automation',
                'clean', $8, $9, $10, $11, $12, now()
            )
            ON CONFLICT (attachment_id)
            DO UPDATE SET
                account_id = EXCLUDED.account_id,
                channel_kind = EXCLUDED.channel_kind,
                blob_id = EXCLUDED.blob_id,
                filename = EXCLUDED.filename,
                content_type = EXCLUDED.content_type,
                size_bytes = EXCLUDED.size_bytes,
                sha256 = EXCLUDED.sha256,
                source_kind = EXCLUDED.source_kind,
                imported_by = EXCLUDED.imported_by,
                scan_status = EXCLUDED.scan_status,
                scan_engine = EXCLUDED.scan_engine,
                scan_checked_at = EXCLUDED.scan_checked_at,
                scan_summary = EXCLUDED.scan_summary,
                scan_metadata = EXCLUDED.scan_metadata,
                metadata = EXCLUDED.metadata,
                updated_at = now()
            "#,
        )
        .bind(&attachment_id)
        .bind(delivery_account_id)
        .bind(&source.blob_id)
        .bind(&source.filename)
        .bind(&source.content_type)
        .bind(source.size_bytes)
        .bind(&source.sha256)
        .bind(&source.scan_engine)
        .bind(source.scan_checked_at)
        .bind(&source.scan_summary)
        .bind(&source.scan_metadata)
        .bind(json!({
            "automation": { "kind": "sensitive_forwarding", "policy_id": policy_id },
            "source": {
                "message_id": source_message_id,
                "attachment_id": source.attachment_id,
            },
        }))
        .execute(&mut **transaction)
        .await?;
        sqlx::query(
            r#"
            INSERT INTO communication_outbox_attachments (
                outbox_id, attachment_id, disposition, content_id, sort_order
            )
            VALUES ($1, $2, 'attachment', NULL, $3)
            ON CONFLICT (outbox_id, attachment_id) DO NOTHING
            "#,
        )
        .bind(outbox_id)
        .bind(&attachment_id)
        .bind(i32::try_from(sort_order).map_err(|_| SensitiveForwardingError::Invalid)?)
        .execute(&mut **transaction)
        .await?;
    }
    Ok(())
}
