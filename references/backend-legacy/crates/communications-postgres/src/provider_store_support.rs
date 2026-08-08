use super::*;

pub(super) struct VaultOwnedEntityLink {
    pub(super) observation_id: String,
    pub(super) domain: &'static str,
    pub(super) entity_kind: &'static str,
    pub(super) entity_id: String,
    pub(super) relationship_kind: &'static str,
    pub(super) base_metadata: serde_json::Value,
    pub(super) extra_metadata: Option<serde_json::Value>,
}

pub(super) async fn link_vault_owned_entity_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    request: VaultOwnedEntityLink,
) -> Result<(), makosh_observations_postgres::errors::ObservationStoreError> {
    let metadata = match request.extra_metadata {
        Some(extra) if request.base_metadata.is_object() && extra.is_object() => {
            let mut merged = request.base_metadata;
            if let (Some(base), Some(extra)) = (merged.as_object_mut(), extra.as_object()) {
                for (key, value) in extra {
                    base.insert(key.clone(), value.clone());
                }
            }
            merged
        }
        Some(extra) => extra,
        None => request.base_metadata,
    };

    link_domain_entity_in_transaction(
        transaction,
        &request.observation_id,
        request.domain,
        request.entity_kind,
        request.entity_id,
        Some(request.relationship_kind),
        None,
        Some(metadata),
    )
    .await
}

pub(super) async fn ensure_canonical_provider_account_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: &str,
) -> Result<(), CommunicationIngestionError> {
    sqlx::query(
        r#"
        INSERT INTO communication_accounts (
            account_id,
            provider_kind,
            display_name,
            external_account_id,
            config,
            metadata,
            created_at,
            updated_at
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
    .bind(account_id)
    .execute(&mut **transaction)
    .await?;

    Ok(())
}

pub(super) fn row_to_provider_account(
    row: PgRow,
) -> Result<ProviderAccount, CommunicationIngestionError> {
    Ok(ProviderAccount {
        account_id: row.try_get("account_id")?,
        provider_kind: row
            .try_get::<String, _>("provider_kind")?
            .as_str()
            .try_into()?,
        display_name: row.try_get("display_name")?,
        external_account_id: row.try_get("external_account_id")?,
        config: row.try_get("config")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub(super) fn row_ref_to_provider_account(
    row: &PgRow,
) -> Result<ProviderAccount, CommunicationIngestionError> {
    Ok(ProviderAccount {
        account_id: row.try_get("account_id")?,
        provider_kind: row
            .try_get::<String, _>("provider_kind")?
            .as_str()
            .try_into()?,
        display_name: row.try_get("display_name")?,
        external_account_id: row.try_get("external_account_id")?,
        config: row.try_get("config")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub(super) fn row_to_provider_secret_binding(
    row: PgRow,
) -> Result<ProviderAccountSecretBinding, CommunicationIngestionError> {
    Ok(ProviderAccountSecretBinding {
        account_id: row.try_get("account_id")?,
        secret_purpose: ProviderAccountSecretPurpose::try_from(
            row.try_get::<String, _>("secret_purpose")?.as_str(),
        )?,
        secret_ref: row.try_get("secret_ref")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub(super) fn validate_provider_account(
    account: &NewProviderAccount,
) -> Result<(), CommunicationIngestionError> {
    account.validate()?;
    Ok(())
}

pub(super) fn validate_provider_secret_binding(
    binding: &NewProviderAccountSecretBinding,
) -> Result<(), CommunicationIngestionError> {
    binding.validate()?;
    Ok(())
}

pub(super) fn validate_non_empty_field(
    field: &'static str,
    value: &str,
) -> Result<(), CommunicationIngestionError> {
    if value.trim().is_empty() {
        return Err(CommunicationIngestionError::EmptyField(field));
    }
    Ok(())
}
