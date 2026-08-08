use super::*;

pub(super) async fn due_account_ids(pool: &PgPool) -> Result<Vec<String>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        WITH latest AS (
            SELECT DISTINCT ON (account_id)
                account_id,
                status,
                completed_at,
                next_run_at
            FROM communication_address_book_sync_runs
            ORDER BY account_id, started_at DESC
        )
        SELECT a.account_id
        FROM communication_provider_accounts a
        LEFT JOIN latest ON latest.account_id = a.account_id
        WHERE a.provider_kind IN ('gmail', 'icloud')
          AND COALESCE(a.config->>'auth_state', '') <> 'deleted'
          AND NOT (a.config ? 'deleted_at')
          AND (a.config->'connected_services') ? 'contacts'
          AND COALESCE(
              CASE
                  WHEN jsonb_typeof(a.config->'address_book_sync_enabled') = 'boolean'
                  THEN (a.config->>'address_book_sync_enabled')::boolean
              END,
              true
          )
          AND NOT EXISTS (
              SELECT 1
              FROM communication_address_book_sync_runs active
              WHERE active.account_id = a.account_id
                AND active.status = 'running'
          )
          AND (
              COALESCE(
                  latest.next_run_at,
                  latest.completed_at + interval '1 hour',
                  now()
              ) <= now()
          )
        ORDER BY latest.completed_at ASC NULLS FIRST, a.account_id ASC
        LIMIT 10
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .filter_map(|row| row.try_get::<String, _>("account_id").ok())
        .collect())
}

pub(super) async fn start_run(
    pool: &PgPool,
    run_id: &str,
    account_id: &str,
    trigger: AddressBookSyncTrigger,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO communication_address_book_sync_runs (
            run_id,
            account_id,
            status,
            trigger
        )
        VALUES ($1, $2, 'running', $3)
        "#,
    )
    .bind(run_id)
    .bind(account_id)
    .bind(trigger.as_str())
    .execute(pool)
    .await?;
    Ok(())
}

pub(super) async fn finish_run(
    pool: &PgPool,
    run_id: &str,
    report: &AddressBookSyncAccountReport,
    error: Option<(&str, &str)>,
) -> Result<(), sqlx::Error> {
    let (error_code, error_message) = error
        .map(|(code, message)| (Some(code), Some(message)))
        .unwrap_or((None, None));
    sqlx::query(
        r#"
        UPDATE communication_address_book_sync_runs
        SET
            status = $2,
            completed_at = now(),
            provider_entries_seen = $3,
            provider_entries_upserted = $4,
            provider_entries_skipped = $5,
            local_entries_seen = $6,
            local_entries_pushed = $7,
            local_entries_blocked = $8,
            error_code = $9,
            error_message = $10,
            next_run_at = now() + ($11::text || ' seconds')::interval
        WHERE run_id = $1
        "#,
    )
    .bind(run_id)
    .bind(report.status.as_str())
    .bind(report.provider_entries_seen)
    .bind(report.provider_entries_upserted)
    .bind(report.provider_entries_skipped)
    .bind(report.local_entries_seen)
    .bind(report.local_entries_pushed)
    .bind(report.local_entries_blocked)
    .bind(error_code)
    .bind(error_message)
    .bind(ADDRESS_BOOK_SYNC_POLL_INTERVAL_SECONDS)
    .execute(pool)
    .await?;
    Ok(())
}

pub(super) async fn upsert_provider_address_book_entry_link(
    pool: &PgPool,
    account: &ProviderAccount,
    persona_id: &str,
    provider_entry: &AddressBookProviderEntry,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO communication_provider_address_book_links (
            account_id,
            persona_id,
            provider_address_book_entry_id,
            provider_etag,
            last_provider_seen_at,
            metadata
        )
        VALUES ($1, $2, $3, $4, now(), $5)
        ON CONFLICT (account_id, provider_address_book_entry_id)
        DO UPDATE SET
            persona_id = EXCLUDED.persona_id,
            provider_address_book_entry_id = EXCLUDED.provider_address_book_entry_id,
            provider_etag = EXCLUDED.provider_etag,
            last_provider_seen_at = EXCLUDED.last_provider_seen_at,
            last_synced_at = now(),
            sync_state = 'linked',
            metadata = communication_provider_address_book_links.metadata || EXCLUDED.metadata,
            updated_at = now()
        "#,
    )
    .bind(&account.account_id)
    .bind(persona_id)
    .bind(&provider_entry.provider_address_book_entry_id)
    .bind(&provider_entry.etag)
    .bind(serde_json::json!({
        "provider_kind": account.provider_kind.as_str(),
    }))
    .execute(pool)
    .await?;
    Ok(())
}

pub(super) async fn mark_provider_address_book_link_pushed(
    pool: &PgPool,
    account_id: &str,
    persona_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE communication_provider_address_book_links
        SET last_local_pushed_at = now(), updated_at = now()
        WHERE account_id = $1 AND persona_id = $2
        "#,
    )
    .bind(account_id)
    .bind(persona_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub(super) async fn mark_provider_address_book_link_blocked(
    pool: &PgPool,
    account_id: &str,
    persona_id: &str,
    reason: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE communication_provider_address_book_links
        SET
            sync_state = 'blocked',
            metadata = metadata || jsonb_build_object('blocked_reason', $3),
            updated_at = now()
        WHERE account_id = $1 AND persona_id = $2
        "#,
    )
    .bind(account_id)
    .bind(persona_id)
    .bind(reason)
    .execute(pool)
    .await?;
    Ok(())
}

pub(super) async fn local_address_book_entries_due_for_provider_sync(
    pool: &PgPool,
    account_id: &str,
) -> Result<Vec<LocalAddressBookEntry>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT
            p.persona_id,
            p.display_name,
            p.email_address,
            COALESCE(
                array_agg(identity.identity_value ORDER BY identity.identity_value)
                    FILTER (WHERE identity.identity_value IS NOT NULL),
                ARRAY[]::text[]
            ) AS phone_numbers,
            link.provider_address_book_entry_id,
            link.provider_etag
        FROM personas p
        LEFT JOIN persona_identities identity
          ON identity.persona_id = p.persona_id
         AND identity.identity_type = 'phone'
         AND identity.status = 'active'
        LEFT JOIN communication_provider_address_book_links link
          ON link.account_id = $1
         AND link.persona_id = p.persona_id
        WHERE p.is_address_book = true
          AND (
              p.email_address IS NULL
              OR p.email_address NOT LIKE '%@makosh.invalid'
          )
          AND (
              link.persona_id IS NULL
              OR p.updated_at > COALESCE(link.last_synced_at, link.created_at)
          )
        GROUP BY
            p.persona_id,
            p.display_name,
            p.email_address,
            link.provider_address_book_entry_id,
            link.provider_etag,
            link.persona_id,
            link.last_synced_at,
            link.created_at
        HAVING p.email_address IS NOT NULL
            OR COUNT(identity.identity_value) > 0
            OR length(trim(p.display_name)) > 0
        ORDER BY p.updated_at DESC, p.persona_id
        LIMIT 100
        "#,
    )
    .bind(account_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| LocalAddressBookEntry {
            persona_id: row.try_get("persona_id").unwrap_or_default(),
            display_name: row.try_get("display_name").unwrap_or_default(),
            email_address: row.try_get("email_address").ok(),
            phone_numbers: row.try_get("phone_numbers").unwrap_or_default(),
            provider_address_book_entry_id: row.try_get("provider_address_book_entry_id").ok(),
            provider_etag: row.try_get("provider_etag").ok(),
        })
        .collect())
}
