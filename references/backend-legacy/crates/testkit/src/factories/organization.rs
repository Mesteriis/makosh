use makosh_hub_backend::domains::organizations::api::OrganizationStore;
use sqlx::postgres::PgPool;
use uuid::Uuid;

pub struct OrganizationFactory<'a> {
    pool: &'a PgPool,
    display_name: String,
    org_type: Option<String>,
}

impl<'a> OrganizationFactory<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self {
            pool,
            display_name: format!(
                "Test Org {}",
                Uuid::new_v4()
                    .to_string()
                    .chars()
                    .take(8)
                    .collect::<String>()
            ),
            org_type: Some("company".into()),
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = name.into();
        self
    }

    pub fn with_type(mut self, org_type: impl Into<String>) -> Self {
        self.org_type = Some(org_type.into());
        self
    }

    pub async fn create(
        self,
    ) -> Result<
        makosh_hub_backend::domains::organizations::api::Organization,
        makosh_hub_backend::domains::organizations::api::OrganizationError,
    > {
        let store = OrganizationStore::new(self.pool.clone());
        store
            .create(&self.display_name, self.org_type.as_deref())
            .await
    }
}
