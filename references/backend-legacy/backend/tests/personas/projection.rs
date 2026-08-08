use makosh_hub_backend::domains::personas::api::{
    errors::PersonaProjectionError, models::PersonaType,
    participants::upsert_personas_from_message_participants, store::PersonaProjectionStore,
};
use makosh_hub_backend::domains::personas::command_service::{
    PersonaCommandService, ProviderAddressBookEntryPersonaCommand,
};

use super::support::{
    disconnected_personas_store, live_personas_pool, live_personas_store, unique_suffix,
};

#[tokio::test]
async fn personas_projection_upserts_email_identities_against_postgres() {
    let Some(store) = live_personas_store("personas projection").await else {
        return;
    };
    let suffix = unique_suffix();

    let personas = upsert_personas_from_message_participants(
        &store,
        &[
            format!("alice-{suffix}@example.com"),
            format!("bob-{suffix}@example.com"),
        ],
    )
    .await
    .expect("upsert personas");

    assert_eq!(personas.len(), 2);
    let alice_email = format!("alice-{suffix}@example.com");
    let bob_email = format!("bob-{suffix}@example.com");
    assert!(
        personas
            .iter()
            .any(|p| p.email_address.as_deref() == Some(alice_email.as_str()))
    );
    assert!(
        personas
            .iter()
            .any(|p| p.email_address.as_deref() == Some(bob_email.as_str()))
    );
}

#[tokio::test]
async fn personas_projection_normalizes_and_deduplicates_participants_against_postgres() {
    let Some(store) = live_personas_store("personas normalization and deduplication").await else {
        return;
    };
    let suffix = unique_suffix();
    let normalized_email = format!("alice-{suffix}@example.com");

    let personas = upsert_personas_from_message_participants(
        &store,
        &[
            format!(" Alice-{suffix}@Example.com "),
            format!("alice-{suffix}@example.com"),
        ],
    )
    .await
    .expect("upsert normalized personas");

    assert_eq!(personas.len(), 1);
    assert_eq!(
        personas[0].email_address.as_deref(),
        Some(normalized_email.as_str())
    );
    assert_eq!(personas[0].display_name, normalized_email);
}

#[tokio::test]
async fn personas_projection_rejects_blank_email_participant() {
    let store = disconnected_personas_store();

    let error = upsert_personas_from_message_participants(&store, &[String::from("   ")])
        .await
        .expect_err("blank email input must fail");

    assert!(
        matches!(error, PersonaProjectionError::EmptyEmailAddress),
        "expected EmptyEmailAddress, got {error:?}"
    );
}

#[tokio::test]
async fn personas_projection_rejects_invalid_batch_before_writing_against_postgres() {
    let Some(pool) = live_personas_pool("personas invalid batch atomicity").await else {
        return;
    };
    let store = PersonaProjectionStore::new(pool.clone());
    let suffix = unique_suffix();
    let valid_email = format!("valid-before-blank-{suffix}@example.com");

    let error = upsert_personas_from_message_participants(
        &store,
        &[valid_email.clone(), String::from("   ")],
    )
    .await
    .expect_err("invalid participant batch must fail");

    assert!(
        matches!(error, PersonaProjectionError::EmptyEmailAddress),
        "expected EmptyEmailAddress, got {error:?}"
    );

    let count =
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM personas WHERE email_address = $1")
            .bind(&valid_email)
            .fetch_one(&pool)
            .await
            .expect("person count after rejected batch");
    assert_eq!(count, 0);
}

#[tokio::test]
async fn personas_projection_distinguishes_delimiter_bearing_email_identities_against_postgres() {
    let Some(store) = live_personas_store("delimiter-bearing person identities").await else {
        return;
    };
    let suffix = unique_suffix();

    let left = store
        .upsert_email_persona(&format!("person:{suffix}@example.com"))
        .await
        .expect("upsert delimiter-bearing person");
    let right = store
        .upsert_email_persona(&format!("person-{suffix}@example.com"))
        .await
        .expect("upsert non-delimiter person");

    assert_ne!(left.persona_id, right.persona_id);
    assert!(left.persona_id.starts_with("person:v1:email:"));
    assert!(right.persona_id.starts_with("person:v1:email:"));
}

#[tokio::test]
async fn address_book_persona_can_be_phone_only_and_merge_by_phone_against_postgres() {
    let Some(pool) = live_personas_pool("phone-only address book persona").await else {
        return;
    };
    let service = PersonaCommandService::new(pool);
    let suffix = unique_suffix();
    let phone_number = format!("+1555{}", suffix % 1_000_000_000);
    let source_account_id = format!("account-phone-only-{suffix}");

    let first = service
        .upsert_persona_from_address_book_entry(ProviderAddressBookEntryPersonaCommand {
            source_account_id: source_account_id.clone(),
            provider_address_book_entry_id: "people/phone-only-a".to_owned(),
            display_name: Some("Phone Only Persona".to_owned()),
            primary_email: None,
            additional_emails: Vec::new(),
            phone_numbers: vec![phone_number.clone()],
        })
        .await
        .expect("upsert phone-only address book persona");

    assert_eq!(first.email_address, None);
    assert!(first.is_address_book);
    assert!(
        first
            .persona_id
            .starts_with("persona:v1:provider_address_book_entry:"),
        "new provider-address-book-only personas should use persona-native ids, got {}",
        first.persona_id
    );

    let second = service
        .upsert_persona_from_address_book_entry(ProviderAddressBookEntryPersonaCommand {
            source_account_id,
            provider_address_book_entry_id: "people/phone-only-b".to_owned(),
            display_name: Some("Phone Only Persona Duplicate".to_owned()),
            primary_email: None,
            additional_emails: Vec::new(),
            phone_numbers: vec![phone_number],
        })
        .await
        .expect("merge phone-only address book persona by phone identity");

    assert_eq!(second.persona_id, first.persona_id);
    assert_eq!(second.email_address, None);
    assert!(second.is_address_book);

    let third = service
        .upsert_persona_from_address_book_entry(ProviderAddressBookEntryPersonaCommand {
            source_account_id: format!("account-phone-email-{suffix}"),
            provider_address_book_entry_id: "people/phone-only-c".to_owned(),
            display_name: Some("Phone Email Persona".to_owned()),
            primary_email: Some(format!("phone-email-{suffix}@example.com")),
            additional_emails: Vec::new(),
            phone_numbers: vec![format!("+1 (555) {}", suffix % 1_000_000_000)],
        })
        .await
        .expect("merge later email-bearing address book persona by phone identity");

    assert_eq!(third.persona_id, first.persona_id);
    let expected_email = format!("phone-email-{suffix}@example.com");
    assert_eq!(
        third.email_address.as_deref(),
        Some(expected_email.as_str())
    );
    assert!(third.is_address_book);
}

#[tokio::test]
async fn address_book_persona_without_email_reuses_provider_link_against_postgres() {
    let Some(pool) = live_personas_pool("name-only address book persona link reuse").await else {
        return;
    };
    let service = PersonaCommandService::new(pool.clone());
    let suffix = unique_suffix();
    let source_account_id = format!("account-name-only-{suffix}");
    let provider_entry_id = "people/name-only".to_owned();

    sqlx::query(
        r#"
        INSERT INTO communication_provider_accounts (
            account_id,
            provider_kind,
            display_name,
            external_account_id,
            config
        )
        VALUES ($1, 'gmail', 'Name-only contacts', $2, '{}'::jsonb)
        "#,
    )
    .bind(&source_account_id)
    .bind(format!("name-only-{suffix}@example.com"))
    .execute(&pool)
    .await
    .expect("insert provider account");

    let first = service
        .upsert_persona_from_address_book_entry(ProviderAddressBookEntryPersonaCommand {
            source_account_id: source_account_id.clone(),
            provider_address_book_entry_id: provider_entry_id.clone(),
            display_name: Some("Name Only Persona".to_owned()),
            primary_email: None,
            additional_emails: Vec::new(),
            phone_numbers: Vec::new(),
        })
        .await
        .expect("upsert name-only address book persona");

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
        VALUES ($1, $2, $3, 'etag-1', now(), '{}'::jsonb)
        "#,
    )
    .bind(&source_account_id)
    .bind(&first.persona_id)
    .bind(&provider_entry_id)
    .execute(&pool)
    .await
    .expect("insert provider address book link");

    let second = service
        .upsert_persona_from_address_book_entry(ProviderAddressBookEntryPersonaCommand {
            source_account_id,
            provider_address_book_entry_id: provider_entry_id,
            display_name: Some("Name Only Persona Updated".to_owned()),
            primary_email: None,
            additional_emails: Vec::new(),
            phone_numbers: Vec::new(),
        })
        .await
        .expect("reuse provider link for name-only address book persona");

    assert_eq!(second.persona_id, first.persona_id);
    assert_eq!(second.email_address, None);
    assert!(second.is_address_book);
}

#[tokio::test]
async fn personas_projection_defaults_to_human_non_owner_persona_against_postgres() {
    let Some(store) = live_personas_store("persona defaults").await else {
        return;
    };
    let suffix = unique_suffix();

    let person = store
        .upsert_email_persona(&format!("persona-default-{suffix}@example.com"))
        .await
        .expect("upsert default persona");

    assert_eq!(person.persona_type, PersonaType::Human);
    assert!(!person.is_self);
}

#[tokio::test]
async fn personas_projection_tracks_single_owner_persona_against_postgres() {
    let Some(store) = live_personas_store("single owner persona").await else {
        return;
    };
    let suffix = unique_suffix();
    let first = store
        .upsert_email_persona(&format!("owner-first-{suffix}@example.com"))
        .await
        .expect("upsert first owner candidate");
    let second = store
        .upsert_email_persona(&format!("owner-second-{suffix}@example.com"))
        .await
        .expect("upsert second owner candidate");

    let first_owner = store
        .set_owner_persona(&first.persona_id)
        .await
        .expect("set first owner persona");
    assert!(first_owner.is_self);

    let second_owner = store
        .set_owner_persona(&second.persona_id)
        .await
        .expect("move owner persona");
    assert!(second_owner.is_self);

    let owner = store
        .owner_persona()
        .await
        .expect("load owner persona")
        .expect("owner persona exists");
    assert_eq!(owner.persona_id, second.persona_id);
}

#[tokio::test]
async fn personas_projection_sets_supported_persona_type_against_postgres() {
    let Some(store) = live_personas_store("persona type update").await else {
        return;
    };
    let suffix = unique_suffix();
    let person = store
        .upsert_email_persona(&format!("ai-agent-{suffix}@example.com"))
        .await
        .expect("upsert persona");

    let updated = store
        .set_persona_type(&person.persona_id, PersonaType::AiAgent)
        .await
        .expect("set persona type");

    assert_eq!(updated.persona_type, PersonaType::AiAgent);
    assert!(!updated.is_self);
}

#[tokio::test]
async fn personas_schema_rejects_invalid_persona_type_against_postgres() {
    let Some(pool) = live_personas_pool("invalid persona type").await else {
        return;
    };
    let store = PersonaProjectionStore::new(pool.clone());
    let suffix = unique_suffix();
    let person = store
        .upsert_email_persona(&format!("invalid-type-{suffix}@example.com"))
        .await
        .expect("upsert persona");

    let error = sqlx::query("UPDATE personas SET person_type = 'lead' WHERE persona_id = $1")
        .bind(&person.persona_id)
        .execute(&pool)
        .await
        .expect_err("invalid persona type must violate the check constraint");

    assert!(
        error.to_string().contains("personas_person_type_check"),
        "expected personas_person_type_check violation, got {error}"
    );
}
