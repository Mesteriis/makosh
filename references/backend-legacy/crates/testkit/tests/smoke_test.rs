//! Smoke test for Testcontainers infrastructure in crates/testkit.
//!
//! Verifies that:
//! - AC1: Testcontainers PostgreSQL контейнер поднимается и проходит health check
//! - Migration runner works against the container
//! - Entity factories can create records
//!
//! This is a characterization test — it captures CURRENT behavior without
//! modifying it. Any change to the infrastructure will cause these tests to
//! fail, alerting the developer to review compatibility.

use makosh_backend_testkit::context::TestContext;

/// AC1: Testcontainers PostgreSQL container starts and passes health check.
#[tokio::test]
async fn test_context_creates_isolated_database() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool();

    // Verify basic query works (container is reachable, DB is responsive)
    let result: i32 = sqlx::query_scalar("SELECT 1")
        .fetch_one(pool)
        .await
        .expect("basic query against test database must succeed");
    assert_eq!(result, 1, "pgvector container must respond to queries");
}

/// Verify migrations were applied successfully.
#[tokio::test]
async fn test_context_runs_migrations() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool();

    // Check that _sqlx_migrations table exists (proof migrations ran)
    let migration_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations WHERE success = true")
            .fetch_one(pool)
            .await
            .expect("migrations table must exist and be queryable");
    assert!(
        migration_count > 0,
        "at least one migration must have been applied (got {migration_count})"
    );
}

/// Verify Persona identifier columns and indexes use their canonical names.
#[tokio::test]
async fn test_context_runs_persona_identifier_migration() {
    let ctx = TestContext::new().await;
    let (
        has_conversation_persona_id,
        has_conversation_person_id,
        has_organization_persona_index,
        has_legacy_organization_person_index,
        has_message_persona_index,
        has_legacy_message_person_index,
        has_interaction_context_persona_index,
        has_legacy_interaction_context_person_index,
    ): (bool, bool, bool, bool, bool, bool, bool, bool) = sqlx::query_as(
        r#"
        SELECT
            EXISTS (
                SELECT 1
                FROM information_schema.columns
                WHERE table_schema = 'public'
                  AND table_name = 'communication_conversation_participants'
                  AND column_name = 'persona_id'
            ),
            EXISTS (
                SELECT 1
                FROM information_schema.columns
                WHERE table_schema = 'public'
                  AND table_name = 'communication_conversation_participants'
                  AND column_name = 'person_id'
            ),
            to_regclass('public.organization_persona_links_persona_id_idx') IS NOT NULL,
            to_regclass('public.org_persona_links_person_id_idx') IS NOT NULL,
            to_regclass('public.communication_message_participants_persona_id_idx') IS NOT NULL,
            to_regclass('public.communication_message_participants_person_idx') IS NOT NULL,
            to_regclass('public.persona_interaction_contexts_source_persona_id_idx') IS NOT NULL,
            to_regclass('public.persona_interaction_contexts_person_id_idx') IS NOT NULL
        "#,
    )
    .fetch_one(ctx.pool())
    .await
    .expect("persona identifier migration schema must be queryable");

    assert!(
        has_conversation_persona_id,
        "communication conversation participants must expose persona_id"
    );
    assert!(
        !has_conversation_person_id,
        "communication conversation participants must retire person_id"
    );
    assert!(
        has_organization_persona_index,
        "organization-persona links must expose the canonical persona index"
    );
    assert!(
        !has_legacy_organization_person_index,
        "organization-persona links must retire the legacy person index"
    );
    assert!(
        has_message_persona_index,
        "communication message participants must expose the canonical persona index"
    );
    assert!(
        !has_legacy_message_person_index,
        "communication message participants must retire the legacy person index"
    );
    assert!(
        has_interaction_context_persona_index,
        "Persona interaction contexts must expose the canonical source-Persona index"
    );
    assert!(
        !has_legacy_interaction_context_person_index,
        "Persona interaction contexts must retire the legacy person index"
    );
}

/// Verify each test gets a unique, isolated database.
#[tokio::test]
async fn test_context_databases_are_isolated() {
    let ctx_a = TestContext::new().await;
    let ctx_b = TestContext::new().await;

    let a_conn = ctx_a.connection_string();
    let b_conn = ctx_b.connection_string();

    assert_ne!(
        a_conn, b_conn,
        "each TestContext must produce a unique database connection string"
    );
}

/// Verify the PersonaFactory works against a real container.
#[tokio::test]
async fn testkit_persona_factory_creates_persona() {
    let ctx = TestContext::new().await;
    let factory = makosh_backend_testkit::factories::persona::PersonaFactory::new(ctx.pool());

    let persona_id = factory
        .with_name("Characterization Test Persona")
        .with_email("char-test@example.com")
        .create()
        .await
        .expect("PersonaFactory must create a persona against the test container");

    assert!(
        !persona_id.is_empty(),
        "PersonaFactory must return a non-empty persona record ID"
    );

    // Verify the persona record actually exists in the database.
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM personas WHERE persona_id = $1)")
            .bind(&persona_id)
            .fetch_one(ctx.pool())
            .await
            .expect("query must succeed");
    assert!(exists, "persona created by PersonaFactory must exist in DB");
}

/// Verify the EmailFactory works against a real container.
#[tokio::test]
async fn testkit_email_factory_creates_email() {
    let ctx = TestContext::new().await;
    let factory = makosh_backend_testkit::factories::email::EmailFactory::new(ctx.pool());

    let (_account, raw) = factory
        .with_subject("Characterization Test Email")
        .with_from("sender@example.com")
        .with_body("This is a characterization test email body.")
        .create()
        .await
        .expect("EmailFactory must create an email record against the test container");

    assert!(
        !raw.raw_record_id.is_empty(),
        "EmailFactory must return a non-empty raw_record_id"
    );
}
