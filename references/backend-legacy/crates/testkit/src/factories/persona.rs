use makosh_hub_backend::domains::personas::core::errors::PersonaCoreError;
use makosh_hub_backend::domains::personas::core::identities::PersonaIdentityStore;
use makosh_hub_backend::domains::personas::core::interaction_contexts::{
    NewPersonaInteractionContext, PersonaInteractionContextStore,
};
use sqlx::postgres::PgPool;
use uuid::Uuid;

pub struct PersonaFactory<'a> {
    pool: &'a PgPool,
    display_name: String,
    email: Option<String>,
    persona_id: Option<String>,
}

impl<'a> PersonaFactory<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self {
            pool,
            display_name: format!(
                "Test Persona {}",
                Uuid::new_v4()
                    .to_string()
                    .chars()
                    .take(8)
                    .collect::<String>()
            ),
            email: None,
            persona_id: None,
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = name.into();
        self
    }

    pub fn with_email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    pub fn with_persona_id(mut self, id: impl Into<String>) -> Self {
        self.persona_id = Some(id.into());
        self
    }

    #[deprecated(note = "use with_persona_id; persona_id is only the current storage column name")]
    pub fn with_person_id(self, id: impl Into<String>) -> Self {
        self.with_persona_id(id)
    }

    /// Create a persona record, email identity, and default interaction context.
    ///
    /// Returns the current schema's stable persona record identifier (`persona_id`).
    pub async fn create(self) -> Result<String, PersonaCoreError> {
        let identity_store = PersonaIdentityStore::new(self.pool.clone());
        let persona_store = PersonaInteractionContextStore::new(self.pool.clone());

        let persona_id = self
            .persona_id
            .unwrap_or_else(|| format!("persona:{}", Uuid::new_v4()));
        let email = self
            .email
            .unwrap_or_else(|| format!("{}@example.test", Uuid::new_v4()));

        sqlx::query(
            r#"
            INSERT INTO personas (
                persona_id,
                display_name,
                email_address
            )
            VALUES ($1, $2, $3)
            ON CONFLICT (persona_id)
            DO UPDATE SET
                display_name = EXCLUDED.display_name,
                email_address = EXCLUDED.email_address,
                updated_at = now()
            "#,
        )
        .bind(&persona_id)
        .bind(&self.display_name)
        .bind(&email)
        .execute(self.pool)
        .await?;

        // Create identity via upsert.
        identity_store
            .upsert(&persona_id, "email", &email, "testkit")
            .await?;

        // Create a default interaction context for the persona record.
        let persona = NewPersonaInteractionContext {
            interaction_context_id: format!("persona:{}", Uuid::new_v4()),
            source_persona_id: persona_id.clone(),
            name: self.display_name,
            context: Some("test".into()),
            default_tone: Some("neutral".into()),
            default_language: Some("en".into()),
            preferred_channel: Some(email),
        };
        persona_store.upsert(&persona).await?;

        Ok(persona_id)
    }
}
